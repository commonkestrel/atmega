#![no_std]
#![feature(lang_items, asm_experimental_arch, abi_avr_interrupt, error_in_core, doc_cfg)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod wiring;
pub mod registers;
pub mod prelude;
pub mod timer;
pub mod volatile;
pub mod interrupt;
pub mod constants;
pub mod serial;
pub mod bits;
pub mod buffer;

#[cfg(any(feature = "interrupt-macro", doc))]
#[doc(cfg(feature = "interrupt-macro"))]
pub use atmega_macros::interrupt;

#[doc(hidden)]
pub fn init() {
    wiring::init();
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
            $crate::init();
            
            let mut state = $setup();
            loop{ $loop(&mut state) }
        }
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {}
}

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
