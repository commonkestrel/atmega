#![no_std]
#![no_main]
#![feature(lang_items)]

use atmega::prelude::*;

#[no_mangle]
pub extern "C" fn main() -> ! {
    pin_mode(Pin::D7, PinMode::INPUT_PULLUP);
    pin_mode(Pin::D8, PinMode::OUTPUT);
    loop {
        let button = digital_read(Pin::D7);
        digital_write(Pin::D8, button);
    }
}

