use crate::CPU_FREQUENCY;
use crate::registers::{ UBRR0H, UBRR0L, UCSR0A, UCSR0B, UCSR0C, UDR0, Register };
use core::fmt::Write;

#[cfg(any(feature = "serial-buffer", doc))]
#[doc(cfg(feature = "serial-buffer"))]
use crate::buffer::Buffer;
#[cfg(any(feature = "serial-buffer", doc))]
#[doc(cfg(feature = "serial-buffer"))]
use crate::volatile::Volatile;

#[cfg(any(feature = "serial-buffer", doc))]
#[doc(cfg(feature = "serial-buffer"))]
static USART_BUFFER: Volatile<Buffer> = Volatile::new(Buffer::new());

pub struct Serial {}

impl Serial {
    pub const fn new() -> Self {
        Serial {}
    }

    /// Initialize serial at the given baud rate
    pub fn begin(baud: u32) {
        let ubrr = ((CPU_FREQUENCY / (16*baud) as u64)-1) as u16;
        unsafe {
            // Write baud rate to UBRR
            UBRR0H::write(((ubrr >> 8) & 0x0F) as u8);
            UBRR0L::write((ubrr & 0xFF) as u8);

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
        }
    }

    pub fn transmit_ready() -> bool {
        unsafe { UCSR0A::UDRE0.read_bit() }
    }

    /// Transmits byte over serial.
    /// Blocking
    pub fn transmit(byte: u8) {
        while !Self::transmit_ready() {}
        unsafe { UDR0::write(byte) };
    }

    #[cfg(any(not(feature = "serial-buffer"), doc))]
    #[doc(cfg(not(feature = "serial-buffer")))]
    pub fn recieve_ready() -> bool {
        unsafe { UCSR0A::RXC0.read_bit() }
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
    pub fn available() -> u8 {
        USART_BUFFER.read().available()
    }

    /// Read the byte at the front of the USART buffer
    #[cfg(any(feature = "serial-buffer", doc))]
    #[doc(cfg(feature = "serial-buffer"))]
    pub fn read() -> Option<u8> {
        USART_BUFFER.read().read()
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            Self::transmit(c as u8);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    Serial::new().write_fmt(args).unwrap();
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