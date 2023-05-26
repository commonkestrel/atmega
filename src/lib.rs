//! A fast, easy, recognizable interface for the ATmega328p

#![no_std]
#![feature(lang_items, 
    asm_experimental_arch, 
    abi_avr_interrupt, 
    error_in_core, 
    doc_cfg, 
    exclusive_range_pattern, 
    maybe_uninit_uninit_array, 
    const_maybe_uninit_uninit_array, 
    core_intrinsics,
    derive_const,
    const_trait_impl,
    const_discriminant,
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(overflowing_literals, arithmetic_overflow)]
#![warn(missing_docs)]

#[cfg(any(feature = "alloc", doc))]
#[doc(cfg(feature = "alloc"))]
pub mod allocator;
pub mod bits;
pub mod buffer;
pub mod constants;
pub mod drivers;
pub mod interrupts;
pub mod libraries;
pub mod prelude;
pub mod progmem;
pub mod registers;
pub mod serial;
pub mod timing;
pub mod volatile;
pub mod wiring;
pub mod math;
//pub mod hardwareserial;
pub use atmega_macros::{ entry, interrupt };

#[doc(hidden)]
pub fn _init() {
    wiring::_init();
}

/// Takes two arguments, `setup()` and `run()`.
/// `setup()` is the same as `setup()` in the Arduino language.
/// `run()` is the same as `loop()` in the Arduino language.
/// 
/// There are two ways to use this macro:
/// ## Stateless
/// The most basic way is the stateless way, where you just simply pass
/// a `setup()` function and a `run()` function:
/// 
/// ```
/// run!(setup, run);
/// 
/// fn setup() {}
/// fn run() {}
/// ```
/// 
/// This just runs `setup()` once, and then `run()` in a loop.
/// 
/// ## Stateful
/// Rust makes mutable global variables very difficult to prevent data races,
/// but this can make microcontroller programs more difficult than usual.
/// 
/// This crate attempts to fix this with the ability to pass a mutable state
/// into your loop.
/// 
/// This is done like so:
/// 
/// ```
/// run!(setup, run, State);
/// 
/// struct State {
///     x: usize,
/// }
/// 
/// fn setup() -> State {
///     State{ x: 0 }
/// }
/// 
/// fn run(state: &mut State) {
///     state.x += 1;
/// }
/// ```
/// 
/// This allows you to modify values between iterations without worrying about
/// global variables and data races.
#[macro_export]
macro_rules! run {
    ($setup: ident, $run: ident) => {
        #[no_mangle]
        pub extern "C" fn main() -> ! {
            $crate::_init();
            
            $setup();
            loop{ $run() }
        }
    };
    ($setup: ident, $run: ident, $state: tt) => {
        #[no_mangle]
        pub extern "C" fn main() -> ! {
            $crate::_init();
            
            let mut state: $state = $setup();
            loop{ $run(&mut state) }
        }
    };
}

/// Panic handler.
#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Defines the exception handling personality.
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
