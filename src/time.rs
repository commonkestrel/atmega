//! Utilities for reading and controlling time

use crate::constants::CPU_FREQUENCY;
use crate::registers::{ Register, TCNT1L, TCNT1H, TIFR0, TIMSK0, TCNT0 };
use crate::util::delay::{ _delay_cycles, _delay_us, _delay_ms, MILLIS, MICROS };

#[cfg(feature = "millis")]
use crate::volatile::Volatile;

/// Sleep for a given number of CPU cycles
/// Has a precision of 4 cycles
pub fn delay_cycles(cycles: u64) {
    _delay_cycles(cycles);
}

/// Sleep for a given number of microseconds.
pub fn delay_micros(us: u64) {
    _delay_us(us);
}

/// Sleep for a given number of milliseconds.
pub fn delay(ms: u64) {
    _delay_ms(ms);
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
