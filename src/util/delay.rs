//! Utilities to delay small time periods.

use core::arch::asm;
use crate::constants::CPU_FREQUENCY;

const MILLIS: u64 = 1_000;
const MICROS: u64 = 1_000_000;

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
pub fn _delay_cycles(count: u64) {
    // Each iteration in _delay_loop() takes 4 clock cycles
    let loops = count/4;

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
pub fn _delay_us(count: u64) {
    _delay_cycles(count / (CPU_FREQUENCY/MICROS));
}

/// Delay the specified number of milliseconds
#[inline(always)]
pub fn _delay_ms(count: u64) {
    _delay_cycles(count / (CPU_FREQUENCY/MILLIS));
}
