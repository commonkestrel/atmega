//! Easy serial communication via the ATmega328p USART
//! 
//! # Examples
//! ```no_run
//! use atmega::serial::*;
//! 
//! Serial::begin(9600);
//! println!("hello world!");
//! ```
//! Initializes the USART to a baud rate of 9600 and transmits "hello world"
//! 
//! Implementation and documentation ported from the official 
//! [`HardwareSerial`](https://github.com/arduino/ArduinoCore-avr/blob/master/cores/arduino/HardwareSerial.cpp) in the arduino core.

use crate::bv;
use crate::buffer::Buffer;
use crate::constants::CPU_FREQUENCY;
use crate::interrupts::{ self, State };
use crate::registers::{ Register, UDR0, UCSR0A, UCSR0B, UBRR0H, UBRR0L, SREG };

const BUFFER_LENGTH: usize = 64;
static mut RX_BUFFER: Buffer<u8, BUFFER_LENGTH> = Buffer::new();
static mut TX_BUFFER: Buffer<u8, BUFFER_LENGTH> = Buffer::new();

static mut WRITTEN: bool = false;

pub struct Serial;

impl Serial {
    unsafe fn _tx_udr_empty_irq() {
        // If interrupts are enabled, there must be more data in the output buffer.
        // Use if let to avoid unnecessary unwraps.
        if let Some(next) = TX_BUFFER.read() {
            // Send the next byte.
            UDR0::write(next);
        }

        // Clear the TXC bit -- "can be cleared by writing a one to its bit location".
        // This makes sure flush() won't return until the bytes acutally got written.
        // Other r/w bits are preserved, and zeros written to the rest.
        UCSR0A::operate(|ucsra| ucsra & bv!(UCSR0A::U2X0) | bv!(UCSR0A::MPCM0) | bv!(UCSR0A::TXC0));

        WRITTEN = false;

        if TX_BUFFER.is_empty() {
            // Buffer empty, so disable interrupts
            UCSR0B::UDRIE0.clear();
        }
    }

    unsafe fn _rx_complete_irq() {
        if UCSR0A::UPE0.is_clear() {
            let c = UDR0::read();
            RX_BUFFER.write(c);
        } else {
            // Parity error, read byte and discard.
            UDR0::read();
        }
    }

    /// Enables transmission by setting baud rate and 
    /// enabling tx/rx.
    pub fn begin(baud: u64) {
        // Try u2x mode first
        let mut baud_setting = (CPU_FREQUENCY / 4 / baud - 1) / 2;
        unsafe { UCSR0A::write(bv!(UCSR0A::U2X0)); }

        // Hardcoded exception for 57600 for compatibility with the bootloader
        // shipped with the Duemilanove and previous boards and the firmware
        // on the 8U2 on the Uno and Mega 2560. Also, The baud_setting cannot
        // be > 4095, so switch back to non-u2x mode if the baud rate is too
        // low.
        if ((CPU_FREQUENCY == 16_000_000) && (baud == 57600)) || (baud_setting > 4095) {
            unsafe { UCSR0A::write(0); }
            baud_setting = (CPU_FREQUENCY / 8 / baud - 1) / 2;
        }

        // Assingn the baud_setting, a.k.a. ubbr (USART Baud Rate Register)
        unsafe {
            UBRR0H::write((baud_setting as u8) >> 8_u8);
            UBRR0L::write(baud_setting as u8);
        }

        // Set the data bits, parity, and stop bits.
        unsafe {
            UCSR0B::RXEN0.set();
            UCSR0B::TXEN0.set();
            UCSR0B::RXCIE0.set();
            UCSR0B::UDRIE0.clear();
        }
    }

    pub fn end() {
        // Wait for transmission of outgoing data
        Self::flush();

        unsafe {
            UCSR0B::RXEN0.clear();
            UCSR0B::TXEN0.clear();
            UCSR0B::RXCIE0.clear();
            UCSR0B::UDRIE0.clear();
        
            // Clear any recieved data.
            RX_BUFFER.clear();
        }
    }

    pub fn available() -> usize {
        unsafe { RX_BUFFER.len() }
    }

    pub fn peek() -> Option<u8> {
        unsafe {
            if RX_BUFFER.is_empty() {
                return None;
            }

            Some(RX_BUFFER[0])
        }
    }

    pub fn read() -> Option<u8> {
        unsafe { RX_BUFFER.read() }
    }

    pub fn tx_available() -> bool {
        unsafe { !TX_BUFFER.is_full() }
    }

    pub fn flush() {
        // If we have never written a byte, no need to flush. This special
        // case is needed since there is no way to force the TXC (transmit
        // complete) bit to 1 during initialization
        if unsafe {!WRITTEN} {return};

        unsafe {
            while UCSR0B::UDRIE0.is_set() || UCSR0A::TXC0.is_clear() {
                if SREG::I.is_clear() && UCSR0B::UDRIE0.is_set() && UCSR0A::UDRE0.is_set()  {
                    // Interrupts are globally disabled, but the DR empty
                    // interrupt should be enabled, so poll the DR empty
                    // flag to prevent deadlock.
                    Self::_tx_udr_empty_irq();
                }
            }
        }
    }

    pub fn write(c: u8) {
        unsafe {
            WRITTEN = true;

            // If the buffer and the data register is empty, just write the byte
            // to the data register and be done. This shortcut helps
            // significantlyu improve the effective datarate at high
            // (> 500kBit/s) bitrates, where interrupt overhead becomes a slowdown.
            if TX_BUFFER.is_empty() && UCSR0A::UDRE0.is_set() {
                //If TXC is cleared before writing UDR and the previous byte
                // completes before writing to UDR, TXC will be set but a byte
                // is still being transmitted causing flush() to return too soon.
                // So writing UDR must happen first.
                // Writing UDR and clearing TC must be done atomically, otherwise
                // interrupts might delay the TXC clear so the byte written to UDR
                // is transmitted (setting TXC) before clearing TXC. Then TXC will
                // be cleared when no bytes are left, causing flush() to hang.
                interrupts::without(State::Restore, || {
                    UDR0::write(c);

                    UCSR0A::operate(|val| (val & bv!(UCSR0A::U2X0) | bv!(UCSR0A::MPCM0)) | bv!(UCSR0A::TXC0));
                });

                return;
            }

            // If the output buffer is full, there's nothing for it to do
            // other than to wait for the interrupt handler to empty it a bit.
            while TX_BUFFER.is_full() {
                // Interrupts are disabled, so we'll have to poll the data
                // register empty flag ourselves. If it is set, pretend an
                // interrupt has happened and call the handler to free up
                // space for us.
                if SREG::I.is_clear() && UCSR0A::UDRE0.is_set() {
                    Self::_tx_udr_empty_irq();
                }
            }

            interrupts::without(State::Restore, || {
                TX_BUFFER.write(c);
                UCSR0B::UDRIE0.set();
            });
        }
    }
}

#[cfg(feature = "serial-buffer")]
#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_18"]
pub unsafe extern "avr-interrupt" fn USART_RX() {
    use crate::wiring;
    wiring::digital_write(wiring::Pin::D9, true);
    Serial::_rx_complete_irq();
}

#[cfg(feature = "serial-buffer")]
#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_19"]
pub unsafe extern "avr-interrupt" fn USART_UDRE() {
    Serial::_tx_udr_empty_irq();
}