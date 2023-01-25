use core::ptr::read_volatile;
use crate::registers::{ self, DDRB, PORTB, PINB, DDRC, PORTC, PINC, DDRD, PORTD, PIND, Address };

#[derive(Debug, Clone)]
pub enum Pin {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
    D12,
    D13,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
}

impl From<Pin> for Port {
    fn from(value: Pin) -> Port {
        match value {
            Pin::D0  => Port::D(0),
            Pin::D1  => Port::D(1),
            Pin::D2  => Port::D(2),
            Pin::D3  => Port::D(3),
            Pin::D4  => Port::D(4),
            Pin::D5  => Port::D(5),
            Pin::D6  => Port::D(6),
            Pin::D7  => Port::D(7),
            Pin::D8  => Port::B(0),
            Pin::D9  => Port::B(1),
            Pin::D10 => Port::B(2),
            Pin::D11 => Port::B(3),
            Pin::D12 => Port::B(4),
            Pin::D13 => Port::B(5),
            Pin::A0  => Port::C(0),
            Pin::A1  => Port::C(1),
            Pin::A2  => Port::C(2),
            Pin::A3  => Port::C(3),
            Pin::A4  => Port::C(4),
            Pin::A5  => Port::C(5),
        }
    }
}

#[derive(Debug, Clone)]
enum Port {
    B(u8),
    C(u8),
    D(u8),
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum PinMode {
    INPUT,
    INPUT_PULLUP,
    OUTPUT,
}

pub const HIGH: bool = true;
pub const LOW: bool = false;

pub fn pin_mode(pin: Pin, value: PinMode) {
    let port: Port = pin.clone().into();
    let (bit, address) = match port {
        Port::B(bit) => (bit, DDRB),
        Port::C(bit) => (bit, DDRC),
        Port::D(bit) => (bit, DDRD),
    };
    match value {
        PinMode::INPUT => unsafe { registers::operate(address, |x| registers::clear(x, bit)); },
        PinMode::OUTPUT => unsafe { registers::operate(address, |x| registers::set(x, bit)); },
        PinMode::INPUT_PULLUP => {
            unsafe { registers::operate(address, |x| registers::clear(x, bit)) };
            digital_write(pin, HIGH);
        },
    }
}

pub fn digital_write(pin: Pin, value: bool) {
    let port: Port = pin.into();
    let (bit, address) = match port {
        Port::B(bit) => (bit, PORTB),
        Port::C(bit) => (bit, PORTC),
        Port::D(bit) => (bit, PORTD),
    };

    unsafe { registers::operate(address, |x| registers::set_value(x, bit, value)); }
}

pub fn digital_read(pin: Pin) -> bool {
    let port: Port = pin.into();
    let (bit, address) = match port {
        Port::B(bit) => (bit, PINB::address()),
        Port::C(bit) => (bit, PINC),
        Port::D(bit) => (bit, PIND),
    };
    let value = unsafe { read_volatile(address) };
    registers::read(value, bit)
}

pub fn digital_toggle(pin: Pin) {
    let port: Port = pin.into();
    let (bit, address) = match port {
        Port::B(bit) => (bit, PORTB),
        Port::C(bit) => (bit, PORTC),
        Port::D(bit) => (bit, PORTD),
    };

    unsafe { registers::operate(address, |x| registers::toggle(x, bit)); }
}
