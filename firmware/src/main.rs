#![no_std]
#![no_main]

mod leds;

use cortex_m_rt::entry;
use panic_halt as _;
use snippet_protocol::{decode, device::Device, encode, framer::FrameReader, Request, Response, MAX_FRAME};
use cortex_m::asm::delay;
use stm32f3xx_hal::{
    pac,
    prelude::*,
    usb::{Peripheral, UsbBus},
};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use leds::{pattern, Leds};

/// How many times to poll USB while waiting for room to send a reply.
const SEND_ATTEMPTS: u32 = 100_000;

/// Write all of `data`, polling USB as we go. Gives up rather than hang if
/// the host has stopped reading.
fn send_all<B: usb_device::bus::UsbBus>(
    usb_dev: &mut UsbDevice<'_, B>,
    serial: &mut SerialPort<'_, B>,
    data: &[u8],
) {
    let mut sent = 0;
    for _ in 0..SEND_ATTEMPTS {
        usb_dev.poll(&mut [serial]);
        match serial.write(&data[sent..]) {
            Ok(n) => sent += n,
            Err(UsbError::WouldBlock) => {}
            Err(_) => return,
        }
        if sent == data.len() {
            return;
        }
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    // USB needs an exact 48 MHz clock, which comes from the 8 MHz crystal.
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .pclk2(24.MHz())
        .freeze(&mut flash.acr);
    assert!(clocks.usbclk_valid());

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut leds = Leds::new([
        gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe10.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe11.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe12.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe14.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
        gpioe.pe15.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade(),
    ]);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // The board pulls D+ up. Driving it low for ~10 ms makes the PC see a
    // disconnect, so it re-detects the device after every flash.
    let mut usb_dp = gpioa
        .pa12
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    usb_dp.set_low().ok();
    delay(clocks.sysclk().0 / 100);

    let usb_dm = gpioa
        .pa11
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    let usb_dp = usb_dp.into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm,
        pin_dp: usb_dp,
    };
    let usb_bus = UsbBus::new(usb);
    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Snippet")
        .product("Snippet display")
        .serial_number("0001")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut device = Device::new();
    let mut reader = FrameReader::new();
    leds.show(pattern(device.state));

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];
        let Ok(count) = serial.read(&mut buf) else {
            continue;
        };

        for &byte in &buf[..count] {
            let Some(frame) = reader.push(byte) else {
                continue;
            };

            // Bad bytes from the host get a Rejected reply, never a crash.
            let reply = match decode::<Request>(frame) {
                Ok(request) => device.handle(request),
                Err(_) => Response::Rejected,
            };
            leds.show(pattern(device.state));

            let mut out = [0u8; MAX_FRAME];
            if let Ok(encoded) = encode(&reply, &mut out) {
                send_all(&mut usb_dev, &mut serial, encoded);
            }
        }
    }
}
