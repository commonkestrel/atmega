#![allow(non_upper_case_globals)]
//!

use core::arch::asm;

use crate::volatile::Volatile;
use crate::wiring::{ self, Pin };
use crate::registers::{ Register, SPCR, EIMSK, SREG, SPDR, SPSR };
use crate::interrupts;
use crate::constants::CPU_FREQUENCY;
use crate::buffer::Buffer;
use crate::buf;

pub const MOSI: Pin = Pin::D11;
pub const MISO: Pin = Pin::D12;
pub const SCK:  Pin = Pin::D13;
pub const SS:   Pin = Pin::D10;

const SPI_MODE_MASK: u8 = 0x0C; // CPOL = bit 3, CPHA = bit 2 on SPCR
const SPI_CLOCK_MASK: u8 = 0x03; // SPR1 = bit 1, SPR0 = bit 0 on SPCR
const SPI_2XCLOCK_MASK: u8 = 0x01;  // SPI2X = bit 0 on SPSR

static initialized: Volatile<usize> = Volatile::new(0);
static interrupt_mode: Volatile<InterruptMode> = Volatile::new(InterruptMode::Mode0);
static interrupt_mask: Volatile<u8> = Volatile::new(0);

static interrupt_save: Volatile<u8> = Volatile::new(0);

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

/// Initialize the SPI registers and pins.
pub fn begin() {
    // Interrupts are disabled in Volatile.as_mut()
    initialized.as_mut(|init| {
        let register = wiring::Registers::from(SS).ddrx();
        if !unsafe { register.read() } {
            wiring::digital_write(SS, true);
        }

        wiring::pin_mode(SS, wiring::PinMode::OUTPUT);

        unsafe {
            SPCR::MSTR.set();
            SPCR::SPE.set();
        }

        wiring::pin_mode(SCK, wiring::PinMode::OUTPUT);
        wiring::pin_mode(MOSI, wiring::PinMode::OUTPUT);

        *init += 1;
    });
}

pub fn end() {
    // Interrupts are disabled inside Volatile.as_mut()
    initialized.as_mut(|init| { 
        if *init > 0 { // Protect from a scheduler and prevent transaction_begin
            // Decrease the reference counter
            *init += 1; 
        } else { // If there are no more references disable SPI
            unsafe { SPCR::SPE.clear() };
            interrupt_mode.write(InterruptMode::Mode0);
        }
    })
}

enum Interrupt {
    INT0,
    INT1,
    None,
}

fn using_interrupt(interrupt: Interrupt) {
    interrupt_mask.as_mut(|mask| {
        use Interrupt::*;
        *mask |= match interrupt {
            INT0 => 0x01,
            INT1 => 0x02,
            None => {
                interrupt_mode.write(InterruptMode::Mode2);
                0x00
            },
        };

        interrupt_mode.as_mut(|mode| {
            if *mode == InterruptMode::Mode0 {
                *mode = InterruptMode::Mode1;
            } 
        });
    });
}

fn not_using_interrupt(interrupt: Interrupt) {
    if interrupt_mode.as_deref(|mode| *mode == InterruptMode::Mode2) {
        return;
    }
    interrupt_mask.as_mut(|mask| {
        use Interrupt::*;
        *mask &= !match interrupt {
            INT0 => 0x01,
            INT1 => 0x02,
            None => 0x00,
        };

        if *mask == 0 {
            interrupt_mode.write(InterruptMode::Mode0);
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

impl DataMode {
    pub fn mask(self) -> u8 {
        use DataMode::*;
        match self {
            Mode0 => 0x00,
            Mode1 => 0x04,
            Mode2 => 0x08,
            Mode3 => 0x0C,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BitOrder {
    LSBFirst,
    MSBFirst,
}

pub struct SPISettings {
    spcr: u8,
    spsr: u8,
}

impl SPISettings {


    #[inline]
    pub fn new(clock: u32, bit_order: BitOrder, data_mode: DataMode) -> Self {
        // Clock settings are defined as follows. Note that this shows SPI2X
        // inverted, so the bits form increasing numbers. Also note that
        // fosc/64 appears twice
        // SPR1 SPR0 ~SPI2X Freq
        //   0    0     0   fosc/2
        //   0    0     1   fosc/4
        //   0    1     0   fosc/8
        //   0    1     1   fosc/16
        //   1    0     0   fosc/32
        //   1    0     1   fosc/64
        //   1    1     0   fosc/64
        //   1    1     1   fosc/128

        // We find the fastest clock that is less than or equal to the
        // given clock rate. The clock divider that results in clock_setting
        // is 2 ^^ (clock_div + 1). If nothing is slow enough, we'll use the
        // slowest (128 == 2 ^^ 7, so clock_div = 6).
        let clock_div = if clock >= CPU_FREQUENCY as u32 / 2 {
            0x00
        } else if clock >= CPU_FREQUENCY as u32 / 4 {
            0x01
        } else if clock >= CPU_FREQUENCY as u32 / 8 {
            0x02
        } else if clock >= CPU_FREQUENCY as u32 / 16 {
            0x03
        } else if clock >= CPU_FREQUENCY as u32 / 32 {
            0x04
        } else if clock >= CPU_FREQUENCY as u32 / 64 {
            0x05
        } else {
            // Compensate for the duplicate fosc/64
            0x07
        } ^ 0x01; // Invert the SPI2X bit.

        let spcr = SPCR::SPE.bv() | SPCR::MSTR.bv() | if bit_order == BitOrder::LSBFirst { SPCR::DORD.bv() } else { 0 } | 
                    (data_mode.mask() & SPI_MODE_MASK) | ((clock_div >> 1) & SPI_CLOCK_MASK);
        let spsr = clock_div | SPI_2XCLOCK_MASK;

        SPISettings {
            spcr, spsr,
        }
    }
}

impl Default for SPISettings {
    #[inline(always)]
    fn default() -> Self {
        Self::new(4_000_000, BitOrder::LSBFirst, DataMode::Mode0)
    }
}

/// Before using [`transfer()`] or asserting chip select pins,
/// this function is used to gain exclusive access to the SPI bus
/// and configure the correct settings.
pub fn begin_transaction(settings: SPISettings) {
    let interrupts::Status(sreg) = interrupts::disable();

    interrupt_save.as_mut(|save| {
        if interrupt_mode.as_deref(|mode| *mode == InterruptMode::Mode0) {
            unsafe {
                *save = EIMSK::read();
                EIMSK::operate(|eimsk| eimsk & !interrupt_mask.read());
                SREG::write(sreg);
            }
        } else {
            *save = sreg;
        }
    });
}

/// Write to the SPI bus (MOSI pin) and also recieve (MISO pin)
#[inline(always)]
pub fn transfer(data: u8) -> u8 {
    unsafe {
        SPDR::write(data);
        /*
         * The following NOP introduces a small delay that can prevent the wat
         * loop from iterating when running at the maximum speed. This gives
         * about 10% more speed, even if it seems counter-intuitive. At lower
         * speeds it is unnoticed.
         */
        asm!("nop");
        while SPSR::SPIF.is_clear() {}
        SPDR::read()
    }
}

/// Write 16-bit integer to the SPI bus (MOSI pin) and 
/// also recieve 16-bit integer (MISO pin)
#[inline(always)]
pub fn transfer16(data: u16) -> u16 {
    let in_lsb = data as u8;
    let in_msb = (data >> 8) as u8;
    let (out_lsb, out_msb) = unsafe {
        if SPCR::DORD.is_clear() {
            SPDR::write(in_msb);
            asm!("nop"); // See transfer(u8) function
            while SPSR::SPIF.is_clear() {}
            let out_msb = SPDR::read();

            SPDR::write(in_lsb);
            asm!("nop");
            while SPSR::SPIF.is_clear() {}
            let out_lsb = SPDR::read();

            (out_lsb, out_msb)
        } else {
            SPDR::write(in_lsb);
            asm!("nop");
            while SPSR::SPIF.is_clear() {}
            let out_lsb = SPDR::read();

            SPDR::write(in_msb);
            asm!("nop");
            while SPSR::SPIF.is_clear() {}
            let out_msb = SPDR::read();

            (out_lsb, out_msb)
        }
    };

    out_lsb as u16 & ((out_msb as u16) << 8)
}

/// Writes the contents of a [`Buffer`] to the SPI bus.
/// Returns the recieved contents in a [`Buffer`] of the same `SIZE`.
pub fn transfer_all<const SIZE: usize>(buf: Buffer<u8, SIZE>) -> Buffer<u8, SIZE> {
    let mut out = buf![];

    for byte in buf {
        unsafe {
            SPDR::write(byte);
            while SPSR::SPIF.is_clear() {}
            out.write(SPDR::read());
        }
    }

    out
}

/// After performing a group of transfers and releasing the chip select
/// signal, this function allows others to access the SPI bus.
pub fn end_transaction() {
    interrupt_mode.as_deref(|mode| {
        if *mode != InterruptMode::Mode0 {
            let sreg = interrupts::disable();
            if *mode == InterruptMode::Mode1 {
                unsafe {
                    EIMSK::write(interrupt_save.read());
                    interrupts::restore(sreg);
                }
            } else {
                unsafe { SREG::write(interrupt_save.read()); }
            }
        }
    });
}


