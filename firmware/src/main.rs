#![no_std]
#![no_main]

mod leds;

use cortex_m_rt::entry;
use panic_halt as _;
use snippet_protocol::state::State;
use stm32f3xx_hal::{delay::Delay, pac, prelude::*};

use leds::{pattern, Leds};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

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

    // Temporary: advance on a timer until the button is wired up in step C.
    let mut state = State::Default;
    loop {
        leds.show(pattern(state));
        delay.delay_ms(2000u16);
        state = state.next();
    }
}
