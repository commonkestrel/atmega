#![no_std]
#![no_main]

use atmega::prelude::*;

#[atmega::entry]
fn main() -> ! {
    pin_mode(LED_BUILTIN, PinMode::Output);
    loop {
        digital_toggle(LED_BUILTIN);
        delay(1000);
    }
}