//! Functions and macros that allow for easy serial communication via the ATmega328p USART
//! 
//! # Examples
//! ```no_run
//! use atmega::serial::*;
//! 
//! Serial::begin(9600);
//! println!("hello world!");
//! ```
//! Initializes the USART to a baud rate of 9600 and transmits "hello world"

use crate::constants::CPU_FREQUENCY;
use crate::registers::{ UBRR0, UCSR0A, UCSR0B, UCSR0C, UDR0, Register };
#[cfg(feature = "serial-print")]
use core::fmt::Write;

#[cfg(any(feature = "serial-buffer", doc))]
#[doc(cfg(feature = "serial-buffer"))]
use crate::buffer::Buffer;
#[cfg(any(feature = "serial-buffer", doc))]
#[doc(cfg(feature = "serial-buffer"))]
use crate::volatile::Volatile;

#[cfg(any(feature = "serial-buffer", doc))]
#[doc(cfg(feature = "serial-buffer"))]
static USART_BUFFER: Volatile<Buffer<u8, 32>> = Volatile::new(Buffer::new());

/// Easy interface with the USART with `core::fmt::Write` implemented.
pub struct Serial;

impl Serial {
    /// Initialize serial at the given baud rate
    pub fn begin(baud: u32) {
        let ubrr = ((CPU_FREQUENCY / (16*baud) as u64)-1) as u16;
        unsafe {
            // Write baud rate to UBRR
            UBRR0::write(ubrr);

            // Set async
            UCSR0C::UMSEL00.clear();

            // Set single stop bit
            UCSR0C::USBS0.clear();

            // Set parity disabled
            UCSR0C::UPM00.clear();
            UCSR0C::UPM01.clear();

            // Eight bit data bit
            UCSR0C::UCSZ00.set();
            UCSR0C::UCSZ01.set();

            // Enable Reciever and Transmitter
            UCSR0B::RXEN0.set();
            UCSR0B::TXEN0.set();

            // Enable USART_RX interrupt
            #[cfg(feature = "serial-buffer")]
            UCSR0B::RXCIE0.set();
        }
    }

    /// Checks if the USART is ready to transmit the next byte.
    pub fn _transmit_ready() -> bool {
        unsafe { UCSR0A::UDRE0.is_set() }
    }

    /// Transmits byte over serial.
    /// Blocking
    pub fn transmit(byte: u8) {
        while !Self::_transmit_ready() {}
        unsafe { UDR0::write(byte) };
    }

    /// Checks if the USART has a byte to read.
    #[cfg(any(not(feature = "serial-buffer"), doc))]
    #[doc(cfg(not(feature = "serial-buffer")))]
    pub fn recieve_ready() -> bool {
        unsafe { UCSR0A::RXC0.is_set() }
    }

    /// Waits for a byte to be recieved over serial.
    /// Blocking, use `try_serial()` for a non-blocking version.
    #[cfg(any(not(feature = "serial-buffer"), doc))]
    #[doc(cfg(not(feature = "serial-buffer")))]
    pub fn recieve() -> u8 {
        while !Self::recieve_ready() {}
        unsafe { UDR0::read() }
    }

    /// Returns recieved data if there is any available.
    #[cfg(any(not(feature = "serial-buffer"), doc))]
    #[doc(cfg(not(feature = "serial-buffer")))]
    pub fn try_recieve() -> Option<u8> {
        if Self::recieve_ready() {
            Some(unsafe { UDR0::read() })
        } else {
            None
        }
    }

    /// The total bytes stored in the USART buffer
    #[cfg(any(feature = "serial-buffer", doc))]
    #[doc(cfg(feature = "serial-buffer"))]
    pub fn len() -> u8 {
        USART_BUFFER.read().len() as u8
    }

    /// Read the byte at the front of the USART buffer
    #[cfg(any(feature = "serial-buffer", doc))]
    #[doc(cfg(feature = "serial-buffer"))]
    pub fn read() -> Option<u8> {
        USART_BUFFER.read().read()
    }
}

#[cfg(any(feature = "serial-print", doc))]
#[doc(cfg(feature = "serial-print"))]
impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            Self::transmit(c as u8);
        }
        Ok(())
    }
}


/// Prints to the serial output.
/// `Serial::begin()` must have been called previously or the program will freeze.
/// Uses the same syntax as [`std::print`](https://doc.rust-lang.org/std/macro.print.html). See [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html) for more info.
/// 
/// # Example
/// ```no_run
/// Serial::begin(9600);
/// let var = 42;
/// println!("{} ", var);
/// ```
#[macro_export]
#[cfg(any(feature = "serial-print", doc))]
#[doc(cfg(feature = "serial-print"))]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

/// Prints to the serial output, with a newline.
/// `Serial::begin()` must have been called previously or the program will freeze.
/// Uses the same syntax as [`std::println`](https://doc.rust-lang.org/std/macro.println.html). See [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html) for more info.
/// 
/// # Example
/// ```no_run
/// Serial::begin(9600);
/// let var = 42;
/// println!("var = {}", var);
/// ```
#[macro_export]
#[cfg(any(feature = "serial-print", doc))]
#[doc(cfg(feature = "serial-print"))]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
#[allow(unused_must_use)]
#[cfg(any(feature = "serial-print", doc))]
#[doc(cfg(feature = "serial-print"))]
pub fn _print(args: ::core::fmt::Arguments) {
    // Calling unwrap adds about 300 bytes, which is not necessary with no reason to panic
    (Serial{}).write_fmt(args);
}

#[cfg(feature = "serial-buffer")]
#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_18"]
pub unsafe extern "avr-interrupt" fn USART_RX() {
    crate::wiring::digital_write(crate::wiring::Pin::D9, true);
    USART_BUFFER.operate(|mut buf| { buf.write(UDR0::read()); buf });
}