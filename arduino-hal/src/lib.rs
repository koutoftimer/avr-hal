#![no_std]

#[cfg(not(feature = "board-selected"))]
compile_error!(
    "This crate requires you to specify your target Arduino board as a feature.

    Please select one of the following

    * arduino-uno
    "
);

#[cfg(feature = "mcu-atmega")]
pub use atmega_hal::entry;
#[cfg(feature = "mcu-atmega")]
pub use atmega_hal::pac;

#[cfg(feature = "board-selected")]
pub mod clock;
#[cfg(feature = "board-selected")]
pub use clock::default::DefaultClock;

#[cfg(feature = "board-selected")]
mod delay;
#[cfg(feature = "board-selected")]
pub use delay::{delay_ms, delay_us, Delay};

#[cfg(feature = "board-selected")]
pub mod port;
#[cfg(feature = "board-selected")]
pub use port::Pins;

#[cfg(feature = "board-selected")]
pub struct Peripherals {
    pub pins: Pins,
}

#[cfg(feature = "board-selected")]
impl Peripherals {
    fn new(dp: pac::Peripherals) -> Self {
        Self {
            #[cfg(feature = "atmega-hal")]
            pins: Pins::with_mcu_pins(atmega_hal::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD)),
        }
    }

    pub fn take() -> Option<Self> {
        pac::Peripherals::take().map(Self::new)
    }
}
