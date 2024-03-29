//! Utilities for controlling global system interrupts

use core::arch::asm;
use crate::registers::{ SREG, Register };

/// This is a copy of the private `Interrupt` enum in `atmega_macros`
/// 
/// This is not used anywhere, but contains all recognized interrupt function names for the `#[interrupt]` attribute
#[allow(non_camel_case_types)]
pub enum Interrupt {
    /// External pin, power-on reset, brown-out reset and watchdog system reset
    RESET        = 0,
    /// External interrupt reqeest 0
    INT0         = 1,
    /// External interrupt request 1
    INT1         = 2,
    /// Pin change interrupt request 0
    PCINT0       = 3,
    /// Pin change interrupt request 1
    PCINT1       = 4,
    /// Pin change interrupt request 2
    PCINT2       = 5,
    /// Watchdog time-out interrupt
    WDT          = 6,
    /// Timer/Counter2 compare match A
    TIMER2_COMPA = 7,
    /// Timer/Counter2 compare match B
    TIMER2_COMPB = 8,
    /// Timer/Counter2 overflow
    TIMER2_OVF   = 9,
    /// Timer/Counter1 capture event
    TIMER1_CAPT  = 10,
    /// Timer/Counter1 compare match A
    TIMER1_COMPA = 11,
    /// Timer/Counter1 compare match B
    TIMER1_COMPB = 12,
    /// Timer/Counter1 overflow
    TIMER1_OVF   = 13,
    /// Timer/Counter0 compare match A
    TIMER0_COMPA = 14,
    /// Timer/Counter0 compare match B
    TIMER0_COMPB = 15,
    /// Timer/Counter0 overflow
    TIMER0_OVF   = 16,
    /// SPI serial transfer complete
    SPI_STC      = 17,
    /// USART Rx complete
    USART_RX     = 18,
    /// USART data register empty
    USART_UDRE   = 19,
    /// USART Tx complete
    USART_TX     = 20,
    /// ADC conversion complete
    ADC          = 21,
    /// EEPROM ready
    EE_READY     = 22,
    /// Analog comparator
    ANALOG_COMP  = 23,
    /// 2-wire serial interface
    TWI          = 24,
    /// Store program memory ready
    SPM_READY    = 25,
}

/// Enables global interrupts
#[inline(always)]
pub fn enable() {
    unsafe { asm!("sei"); }
}

/// Status of the Status Register
pub struct Status(u8);

/// Disables global interrupts
#[inline(always)]
pub fn disable() -> Status {
    let sreg: u8;

    unsafe {
        asm!(    
            "in {0}, 0x3F",
            "cli",
            out(reg) sreg,
            options(nostack),
        )
    }

    Status(sreg)
}

/// The state to set interrupts after running a function in `without()`.
pub enum State {
    /// Forces interrutps on.
    ForceOn,
    /// Forces interrupts off.
    ForceOff,
    /// Restores state from before execution.
    Restore,
}

/// Runs a function with interrupts disabled, setting interupts to the given state after.
pub fn without<F, R>(after: State, f: F) -> R
where
    F: FnOnce() -> R,
{
    let state = disable();
    let r = f();
    match after {
        State::ForceOn => { enable(); },
        State::ForceOff => { disable(); },
        State::Restore => unsafe { restore(state) },
    }
    r
}

/// 
pub unsafe fn restore(Status(sreg): Status) {
    unsafe {
        asm!(
            "out 0x3F, {0}",
            in(reg) sreg,
            options(nostack),
        )
    }
}

/// Checks if global interrupts are enabled
pub fn enabled() -> bool {
    // Reads the Global Interrupt Enable bit in the AVR Status Register
    unsafe { SREG::I.read_bit() }
}
