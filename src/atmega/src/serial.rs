use crate::CPU_FREQUENCY;
use crate::registers::{ UBRR0H, UBRR0L, UCSR0A, UCSR0B, UCSR0C, UDR0, Register };
use core::hint::spin_loop;

pub struct Serial {}

impl Serial {
    pub fn begin(baud:u32) {
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

    pub fn ready() -> bool {
        unsafe { UCSR0A::UDRE0.read_bit() }
    }

    pub fn transmit(byte: u8) {
        while !Self::ready() {
            spin_loop();
        }
        unsafe { UDR0::write(byte) };
    }
}

/*
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
    use core::fmt::Write;
    Serial{}.write_fmt(args).unwrap();
}
*/