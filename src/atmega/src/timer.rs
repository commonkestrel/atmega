// Used to import environment variables as values other than &'static str
// The CPU_FREQUENCY constant is imported this way
include!(concat!(env!("OUT_DIR"), "/constants.rs")); 

use core::arch::asm;
use crate::mutex::Mutex;
use crate::registers::{ Register, TCNT1L, TCNT1H, TCCR1B, TIFR1, TCCR0A, TCCR0B, TIMSK0, OCR0A };

const MICROS: u64 = 100000;
const MILLIS: u64 = 1000;

static SYSTICK: Mutex<u32> = Mutex::new(0);

/* struct Mutex<T> {
    data: T,
    mut_lock: AtomicBool,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Mutex {
            data,
            mut_lock: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {

    }
} */

pub enum Prescale {
    P1 = 1,
    P8 = 2,
    P64 = 3,
    P256 = 4,
    P1024 = 5,
}

fn timer1_init(prescale: Prescale) {
    unsafe {
        // Disable TIMER1 OVF interrupt
        // This prevents TOV1 from being cleared automatically
        // TIMSK1::TOIE1.clear();

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
    let (high_byte, low_byte) = unsafe {
        // In order to read 16 bit registers on the ATmega328p
        // you need to read the low byte before the high byte
        let low_byte = TCNT1L::read();
        let high_byte = TCNT1H::read();
        (high_byte, low_byte)
    };
    ((high_byte as u16) << 8) | low_byte as u16 // Use both bytes to construct a u16
}

/// Sleep for the specified number of clock cycles
/// Has a precision of 8 cycles
pub fn delay_cycles(cycles: u64) {
    // Set the timer prescale at 8, since at 16MHz this means the timer increments twice every μs
    timer1_init(Prescale::P8);
    
    // Timer is prescaled by 8, which means every 8 ticks the timer increments
    let scaled_cycles = cycles/8; 
    
    // The TCNT1 counter is a 16 bit register, so we need to wait for overflow interrupts if the number of cycles is more than the u16 max
    let of_required = scaled_cycles/core::u16::MAX as u64;

    for _ in 0..of_required {
        unsafe { 
            while TIFR1::read() & 0b0000_0001 == 0 {
                core::hint::spin_loop();
            } 
            TIFR1::write(0b0000_0001);
        }
    }

    let remaining = (scaled_cycles%core::u16::MAX as u64) as u16;

    while read_timer1() < remaining {
        core::hint::spin_loop();
    }
}

/// Sleep for a given number of microseconds (1/1,000,000 of a second)
pub fn delay_micros(us: u64) {
    delay_cycles(us*CPU_FREQUENCY/MICROS);
}

/// Sleep for a given number of milliseconds (1/1,000 of a second)
pub fn delay(ms: u64) {
    delay_cycles(ms*CPU_FREQUENCY/MILLIS);
}

pub fn begin_systick() {
    let mut guard = SYSTICK.lock();
    *guard = 0;
    guard.unlock();
    unsafe {        
        // Write maximum of 250 to Output Compare for CTC mode in order to evenly divide into millis
        OCR0A::write(249);

        TCCR0A::WGM01.set();
        TCCR0A::COM0A0.set();

        // Set prescale for TIMER0 to 64
        TCCR0B::CS00.set();
        TCCR0B::CS01.set();
        TCCR0B::CS02.clear();

        TIMSK0::OCIEA.set();

        asm!("sei");
    }
}

#[inline]
pub fn millis() -> u32 {
    let guard = SYSTICK.lock();
    *guard
}

#[export_name = "__vector_14"]
pub unsafe extern "avr-interrupt" fn TIMER0_COMPA() {
    let mut guard = SYSTICK.lock();
    *guard += 1;
}
