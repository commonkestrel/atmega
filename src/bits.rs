use core::ptr::{ write_volatile, read_volatile };

pub const ADCSRA: *mut u8 = 0x7A as *mut u8;
pub const ADCSRB: *mut u8 = 0x7B as *mut u8;
pub const ADMUX:  *mut u8 = 0x7C as *mut u8;

pub const PINB:   *mut u8 = 0x23 as *mut u8;
pub const DDRB:   *mut u8 = 0x24 as *mut u8;
pub const PORTB:  *mut u8 = 0x25 as *mut u8;
pub const PINC:   *mut u8 = 0x26 as *mut u8;
pub const DDRC:   *mut u8 = 0x27 as *mut u8;
pub const PORTC:  *mut u8 = 0x28 as *mut u8;
pub const PIND:   *mut u8 = 0x29 as *mut u8;
pub const DDRD:   *mut u8 = 0x2A as *mut u8;
pub const PORTD:  *mut u8 = 0x2B as *mut u8;

pub const MCUCR:  *mut u8 = 0x55 as *mut u8;
pub const PRR:    *mut u8 = 0x64 as *mut u8;
pub const GRCCR:  *mut u8 = 0x43 as *mut u8;

pub const TIFR0:  *mut u8 = 0x35 as *mut u8;
pub const TCCR0A: *mut u8 = 0x44 as *mut u8;
pub const TCCR0B: *mut u8 = 0x45 as *mut u8;
pub const TCNT0:  *mut u8 = 0x46 as *mut u8;

pub const OCR0A:  *mut u8 = 0x47 as *mut u8;
pub const OCR0B:  *mut u8 = 0x48 as *mut u8;

pub const TIMSK0: *mut u8 = 0x6E as *mut u8;


pub fn toggle(original: u8, bit: u8) -> u8 {
    original ^ (1 << bit)
}

pub fn set(original: u8, bit: u8) -> u8 {
    original | (1 << bit)
}

pub fn clear(original: u8, bit: u8) -> u8 {
    original & (1 << bit)
}

pub fn set_value(original: u8, bit: u8, value: bool) -> u8 {
    if value {
        set(original, bit)
    } else {
        clear(original, bit)
    }
}

pub fn read(original: u8, bit: u8) -> bool {
    let isolated = original & (1 << bit);
    isolated != 0
}

pub unsafe fn operate<F: Fn(u8) -> u8>(address: *mut u8, operator: F) {
    let current = read_volatile(address);
    write_volatile(address, operator(current));
}
