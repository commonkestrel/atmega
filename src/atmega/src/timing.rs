use crate::registers::{ self, Register };
use core::ptr::{ read_volatile, write_volatile };

const CPU_FREQUENCY: &'static str = core::env!("AVR_CPU_FREQUENCY_HZ");
const TCNT1: *mut u16 = 0x84 as *mut u16;

pub fn timer_init() {
    unsafe { registers::TCCR1B::operate(|val| (val & 0b1111_1010) | 0b0000_0010); }
}

pub fn read_timer() -> u16 {
    unsafe { read_volatile(TCNT1) }
}

pub fn delay_cycles(cycles: u64) {

}

fn u8_to_u16(a: u8, b: u8) -> u16 {
    let mut combined: u16 = 0;
    
    for bit in 0..8 {
        if registers::read(a, bit) {
            combined += 1;
        }
        combined = combined << 1;
    }
    for bit in 0..8 {
        if registers::read(b, bit) {
            combined += 1;
        }
        combined = combined << 1;
}

    combined
}
