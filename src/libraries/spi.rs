//!

use crate::volatile::Volatile;

static initialized: Volatile<usize> = Volatile::new(0);
static interrupt_mode: Volatile<InterruptMode> = Volatile::new(InterruptMode::Mode0);

/// Defines the clock polarity and phase.
/// Only used in `interrupt_mode`
/// 
/// ### CPOL
/// `CPOL`= 0: The leading edge is a rising edge, and the trailing edge is a falling edge.
/// 
/// `CPOL`= 1: The leading edge is a falling edge, and the trailing edge is a rising edge.
/// 
/// ### CPHA
/// `CPHA`= 0: Half a cycle with the clock idle, followed by a half cycle with the clock asserted.
/// 
/// `CPHA`= 1: Half a cycle with the clock asserted, followed by a half cycle with the clock idle.
/// 
/// More information on [Wikipedia](https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Clock_polarity_and_phase)
#[derive(PartialEq, Clone, Copy)]
enum InterruptMode {
    /// `CPOL`= 0, `CPHA`= 0
    Mode0 = 0,
    /// `CPOL`= 0, `CPHA`= 1
    Mode1 = 1,
    /// `CPOL`= 1, `CPHA`= 0
    Mode2 = 2,
    /// `CPOL`= 1, `CPHA`= 1
    Mode3 = 3,
}

pub fn begin() {
    
}