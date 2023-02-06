use crate::CPU_FREQUENCY;
#[cfg(any(feature = "millis", feature = "delay"))]
use crate::volatile::Volatile;
use crate::registers::{ Register, TCNT1L, TCNT1H, TIFR0, TCCR0A, TCCR0B, TIMSK0, TCNT0 };

const MICROS: u64 = 100000;
const MILLIS: u64 = 1000;

pub fn read_timer1() -> u16 {
    let (high_byte, low_byte) = unsafe {
        // In order to read 16 bit registers on the ATmega328p
        // you need to read the low byte before the high byte
        let low_byte = TCNT1L::read();
        let high_byte = TCNT1H::read();
        (high_byte, low_byte)
    };
    ((high_byte as u16) << 8) | low_byte as u16 // Use both bytes to construct a u16
}

/// Sleep for the specified number of clock cycles.
/// Has a precision of 64 cycles.
pub fn delay_cycles(cycles: u64) {
    // Timer 0 prescaler is set to 64, which means the timer increments every 8 clock cycles
    let scaled_cycles = cycles/64;
    
    let individual = (scaled_cycles%256) as u8;
    let initial = unsafe{ TCNT0::read() };

    // Checks if (the value already in Timer 0 + the individual ticks needed to delay) are greater than the max value of Timer 0 (2^8)
    // The value in Timer 0 cannot be changed as this will offset millis()
    let (of_required, remaining) = if initial.checked_add(individual).is_none() {
        ((scaled_cycles/256)+1, individual-initial)
    } else {
        (scaled_cycles/256, individual)
    };

    // Disable Timer 0 OVF interrupt
    // This prevents TOV0 from being cleared automatically
    unsafe { TIMSK0::TOIE0.clear(); }
    
    // The TCNT1 counter is an 8 bit register, so we need to wait for overflow interrupts if the number of cycles is more than 256 (2^8)
    for _ in 0..of_required {
        unsafe { 
            while !TIFR0::TOV0.read_bit() {}
            // To clear a bit in the TIFR you must write it high
            TIFR0::write(0b0000_0001);
        }
        // Update SYSTICK since the Timer 0 overflow interrupt is captured by this loop
        #[cfg(feature = "millis")]
        SYSTICK.operate(|val| val + 1);
    }

    // Renable Timer 0 OVF interrupt
    // This allows systick to update
    unsafe { TIMSK0::TOIE0.set(); }

    // Wait for the remaining cycles
    unsafe { TCNT0::until(|val| val <= remaining) }
}

/// Sleep for a given number of microseconds.
/// Has a precision of 8μs.
pub fn delay_micros(us: u64) {
    delay_cycles(us*CPU_FREQUENCY/MICROS);
}

/// Sleep for a given number of milliseconds.
pub fn delay(ms: u64) {
    delay_cycles(ms*CPU_FREQUENCY/MILLIS);
}

#[cfg(feature = "millis")]
static SYSTICK: Volatile<u64> = Volatile::new(0);

/// The total milliseconds since system boot.
#[inline]
#[cfg(feature = "millis")]
pub fn millis() -> u64 {
    SYSTICK.read().wrapping_mul(64 * 256) / (CPU_FREQUENCY/MILLIS)
}

#[cfg(feature = "millis")]
#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_16"]
pub unsafe extern "avr-interrupt" fn TIMER0_OVF() {
    SYSTICK.operate(|val| val + 1);
}
