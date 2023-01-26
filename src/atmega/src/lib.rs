#![no_std]
#![feature(lang_items)]

pub mod pins;
pub mod registers;
pub mod prelude;

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
