use snippet_protocol::state::State;
use stm32f3xx_hal::gpio::{Output, PXx, PushPull};
use stm32f3xx_hal::prelude::*;

/// The 8 LEDs on the board (PE8..PE15), standing in for the real display.
pub struct Leds {
    pins: [PXx<Output<PushPull>>; 8],
}

impl Leds {
    pub fn new(pins: [PXx<Output<PushPull>>; 8]) -> Self {
        Leds { pins }
    }

    /// Bit 0 of `mask` drives the first LED, bit 7 the last.
    pub fn show(&mut self, mask: u8) {
        for (i, pin) in self.pins.iter_mut().enumerate() {
            if mask & (1 << i) != 0 {
                pin.set_high().ok();
            } else {
                pin.set_low().ok();
            }
        }
    }
}

/// Which LEDs to light for each state: a bar that grows with the state number.
pub fn pattern(state: State) -> u8 {
    match state {
        State::Default => 0b0000_0001,
        State::Clock => 0b0000_0011,
        State::Media => 0b0000_0111,
        State::Upcoming => 0b0000_1111,
        State::Dayview => 0b0001_1111,
        State::System => 0b0011_1111,
        State::ToDo => 0b0111_1111,
        State::Cycle => 0b1111_1111,
    }
}
