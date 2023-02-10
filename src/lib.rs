//! A fast, easy, recognizable interface for the ATmega328p

#![no_std]
#![feature(lang_items, asm_experimental_arch, abi_avr_interrupt, error_in_core, doc_cfg)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub mod wiring;
pub mod registers;
pub mod prelude;
pub mod timers;
pub mod volatile;
pub mod interrupts;
pub mod constants;
pub mod serial;
pub mod bits;
pub mod buffer;
pub mod drivers;

#[cfg(any(feature = "interrupt-macro", doc))]
#[doc(cfg(feature = "interrupt-macro"))]
pub use atmega_macros::interrupt;

#[doc(hidden)]
pub fn _init() {
    wiring::_init();
}

/// Takes two arguments, `setup()` and `run()`.
/// `setup` is the same as `setup` in the Arduino language.
/// `run` is the same as `loop` in the Arduino language.
/// 
#[macro_export]
macro_rules! run {
    ($setup: ident, $loop: ident) => {
        #[no_mangle]
        pub extern "C" fn main() -> ! {
            $crate::_init();
            
            let mut state = $setup();
            loop{ $loop(&mut state) }
        }
    }
}

/// Panic handler.
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {}
}

/// Defines the exception handling personality.
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
