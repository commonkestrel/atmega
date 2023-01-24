#![no_std]
#![no_main]
#![feature(lang_items)]

use core::panic::PanicInfo;
use core::ptr::{ write_volatile, read_volatile };

const PINB: *mut u8 = 0x23 as *mut u8;
const DDRB: *mut u8 = 0x23 as *mut u8;
const PORTB: *mut u8 = 0x23 as *mut u8;

const PINC: *mut u8 = 0x23 as *mut u8;
const DDRC: *mut u8 = 0x23 as *mut u8;
const PORTC: *mut u8 = 0x23 as *mut u8;

const PIND: *mut u8 = 0x23 as *mut u8;
const DDRD: *mut u8 = 0x23 as *mut u8;
const PORTD: *mut u8 = 0x23 as *mut u8;

const HIGH: bool = true;
const LOW: bool = false;

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}

fn digital_write(pin: u8, value: bool) {
    unsafe {

    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
