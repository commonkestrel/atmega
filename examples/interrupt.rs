#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use atmega::prelude::*;

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}

#[atmega::interrupt]
unsafe fn TIMER1_OVF() {
    digital_toggle(Pin::D9);
}
