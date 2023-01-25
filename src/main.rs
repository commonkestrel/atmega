#![no_std]
#![no_main]
#![feature(lang_items)]

use atmega::prelude::*;

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {
        digital_toggle(Pin::D10);
    }
}

