#![no_std]
#![no_main]
#![feature(lang_items)]

pub mod pin;
pub mod bits;
pub mod prelude;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
