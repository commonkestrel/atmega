//! Utilities for reading and controlling time

use core::arch::asm;
use crate::constants::CPU_FREQUENCY;
use crate::registers::{ Register, TCNT0 };

#[cfg(feature = "millis")]
use crate::volatile::Volatile;

/// Milliseconds in a second
pub const MILLIS: u64 = 1_000;
/// Microseconds in a second
pub const MICROS: u64 = 1_000_000;

/// Delay loop using a 16 bit counter, so upto 65536 iterations are possible.
/// (The value 65536 would have to passed as 0)
/// The loop executes four CPU cycles per iteration,
/// not including the overhead the compiler requires to setup the counter register pair.
/// 
/// Thus, at a CPU speed of 1MHZ, delays of up to about 262.1 
/// milliseconds can be achieved
#[inline(always)]
pub fn _delay_loop(count: u16) {
    unsafe {
        asm!(
            "1: sbiw {0}, 1",
            "brne 1b",
            inout(reg_iw) count => _,
        );
    }
}

/// Delay the specified CPU cycles using _delay_loop()
/// Has a precision of 4 cycles.
#[inline(always)]
pub fn delay_cycles(cycles: u64) {
    // Each iteration in _delay_loop() takes 4 clock cycles
    let loops = cycles/4;

    let outer = loops / 65536;
    let last = (loops % 65536 + 1) as u16;

    for _ in 0..outer {
        // The value 65536 has to be passed to _delay_loop() as 0
        _delay_loop(0);
    }

    _delay_loop(last);
}

/// Delay the specified number of microseconds
/// On boards with a clock speed of less than 4MHz the precision will be less than 1us
#[inline(always)]
pub fn delay_micros(us: u64) {
    delay_cycles(us * (CPU_FREQUENCY/MICROS));
}

/// Delay the specified number of milliseconds
#[inline(always)]
pub fn delay(ms: u64) {
    delay_cycles(ms * (CPU_FREQUENCY/MILLIS));
}

#[cfg(feature = "millis")]
static SYSTICK: Volatile<u64> = Volatile::new(0);

/// The total milliseconds since system boot.
#[inline]
#[cfg(any(feature = "millis", doc))]
#[doc(cfg(feature = "millis"))]
pub fn millis() -> u64 {
    SYSTICK.read().wrapping_mul(64 * 256) / (CPU_FREQUENCY/MILLIS)
}

/// The number of microseconds that have passed since system boot.
/// Has a precision of 4us on a 16MHz chip.
#[inline]
#[cfg(any(feature = "millis", doc))]
#[doc(cfg(feature = "millis"))]
pub fn micros() -> u64 {
    let timer = unsafe { TCNT0::read() };
    (SYSTICK.read().wrapping_mul(64 * 256) / (CPU_FREQUENCY/MICROS)) + (timer as u64 * 4)
}

#[cfg(feature = "millis")]
#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_16"]
pub unsafe extern "avr-interrupt" fn TIMER0_OVF() {
    SYSTICK.operate(|val| val + 1);
}
