use core::arch::asm;
use crate::registers::{ SREG, Register };

/// This is a copy of the private `Interrupt` enum in `atmega_macros`
/// 
/// This is not used anywhere, but contains all recognized interrupt function names for the `#[interrupt]` attribute
#[allow(non_camel_case_types)]
pub enum Interrupt {
    RESET        = 0,
    INT0         = 1,
    INT1         = 2,
    PCINT0       = 3,
    PCINT1       = 4,
    PCINT2       = 5,
    WDT          = 6,
    TIMER2_COMPA = 7,
    TIMER2_COMPB = 8,
    TIMER2_OVF   = 9,
    TIMER1_CAPT  = 10,
    TIMER1_COMPA = 11,
    TIMER1_COMPB = 12,
    TIMER1_OVF   = 13,
    TIMER0_COMPA = 14,
    TIMER0_COMPB = 15,
    TIMER0_OVF   = 16,
    SPI_STC      = 17,
    USART_RX     = 18,
    USART_UDRE   = 19,
    USART_TX     = 20,
    ADC          = 21,
    EE_READY     = 22,
    ANALOG_COMP  = 23,
    TWI          = 24,
    SPM_READY    = 25,
}

/// Enables global interrupts
#[inline(always)]
pub fn enable() {
    unsafe { asm!("sei"); }
}

/// Disables global interrupts
#[inline(always)]
pub fn disable() {
    unsafe { asm!("cli"); }
}

/// Runs a function with interrupts disabled
pub fn without<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    disable();
    let r = f();
    enable();
    r
}

/// Checks if global interrupts are enabled
pub fn enabled() -> bool {
    // Reads the Global Interrupt Enable bit in the AVR Status Register
    unsafe { SREG::I.read_bit() }
}
