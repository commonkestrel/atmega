//! Bindings for avr-libc math functions

pub fn sin(rads: f64) -> f64 {
    unsafe { bindings::sin(rads) }
}

mod bindings {
    extern "C" {
        pub fn atan(rads: f64) -> f64;
        pub fn sin(rads: f64) -> f64;
        pub fn cos(rads: f64) -> f64;
        pub fn tan(rads: f64) -> f64;
    }
}
