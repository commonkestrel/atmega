use crate::registers::{ self, Register, TCNT1L, TCNT1H, TCCR1B, TIFR1 };
use core::ptr::{ read_volatile, write_volatile };

const CPU_FREQUENCY: &'static str = core::env!("AVR_CPU_FREQUENCY_HZ");
const MICROS: u64 = 100000;
const MILLIS: u64 = 1000

enum Prescale {
    P1 = 1,
    P8 = 2,
    P64 = 3,
    P256 = 4,
    P1024 = 5,
}

pub fn timer1_init(prescale: Prescale) {
    unsafe {
        // Sets prescale to the given scale
        // and the other bits (ICNC1, ICNES and WGM bits) to 0
        TCCR1B::write(prescale as u8);

        // Set timer to 0
        // In order to write to 16 bit registers on the ATmega328p
        // you need to write the high byte before the low byte
        TCNT1H::write(0b0000_0000);
        TCNT1L::write(0b0000_0000);
    }
}

pub fn read_timer1() -> u16 {
    unsafe {
        // In order to read 16 bit registers on the ATmega328p
        // you need to read the low byte before the high byte
        let low_byte = TCNT1L::read();
        let high_byte = TCNT1H::read();
    }
    (high_byte as u16 << 8) | low_byte // Use both bytes to construct a u16
}

fn delay_cycles(cycles: u64) {
    // Set the timer prescale at 8, since at 16MHz this means the timer increments twice every μs
    timer1_init(prescale::P8);
    
    // Timer is prescaled by 8, which means every 8 ticks the timer increments
    let scaled_cycles = cycles/8; 
    
    // The TCNT1 counter is a 16 bit register,
    // so we need to for overflow interrupts if the number of cycles is more than the u16 max
    let of_required = scaled_cycles/core::u16::MAX as u64; 

    for _ in 0..of_required {
        while !TIFR1::TOV1::read_bit() {}
        unsafe { TIFR1::write(0b0000_0001); }
    }
    let remaining = scaled_cycles%core::u16::MAX as u64;
    while read_timer1() < remaining {}
}

pub fn delay_micros(us: u64) {
    delay_cycles(CPU_FREQUENCY/MICROS);
}

pub fn delay(ms: u64) {
    delay_cycles(CPU_FREQUENCY/MILLIS);
}
