#![no_std]
#![feature(lang_items, asm_experimental_arch)]

pub mod pins;
pub mod registers;
pub mod prelude;
pub mod timing;

use core::panic::PanicInfo;

#[macro_export]
macro_rules! run {
    ($setup: ident, $loop: ident) => {
        #[no_mangle]
        pub extern "C" fn main() -> ! {
            $setup();
            loop{ $loop() }
        }
    }
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
