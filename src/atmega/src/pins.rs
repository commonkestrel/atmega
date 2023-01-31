use crate::registers::{ Register, PINx, DDRx, PORTx, PINB, DDRB, PORTB, PINC, DDRC, PORTC, PIND, DDRD, PORTD, ADMUX, ADCSRA, ADCL, ADCH };

#[derive(Debug, Clone, Copy)]
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

impl Pin {
    fn is_digital(&self) -> bool {
        // When converted to a number Pins 0-13 are digital, 14-19 are analog
        (*self as u8) <= 13
    }
}

impl From<u8> for Pin {
    fn from(value: u8) -> Self {
        match value {
            0  => Self::D0,
            1  => Self::D1,
            2  => Self::D2,
            3  => Self::D3,
            4  => Self::D4,
            5  => Self::D5,
            6  => Self::D6,
            7  => Self::D7,
            8  => Self::D8,
            9  => Self::D9,
            10 => Self::D10,
            11 => Self::D11,
            12 => Self::D12,
            13 => Self::D13,
            14 => Self::A0,
            15 => Self::A1,
            16 => Self::A2,
            17 => Self::A3,
            18 => Self::A4,
            19 => Self::A5,
            _ => unreachable!(),
        }
    }
}

impl core::fmt::Display for Pin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone)]
enum Registers {
    B(u8),
    C(u8),
    D(u8),
}

impl From<Pin> for Registers {
    fn from(value: Pin) -> Registers {
        match value {
            Pin::D0  => Registers::D(0),
            Pin::D1  => Registers::D(1),
            Pin::D2  => Registers::D(2),
            Pin::D3  => Registers::D(3),
            Pin::D4  => Registers::D(4),
            Pin::D5  => Registers::D(5),
            Pin::D6  => Registers::D(6),
            Pin::D7  => Registers::D(7),
            Pin::D8  => Registers::B(0),
            Pin::D9  => Registers::B(1),
            Pin::D10 => Registers::B(2),
            Pin::D11 => Registers::B(3),
            Pin::D12 => Registers::B(4),
            Pin::D13 => Registers::B(5),
            Pin::A0  => Registers::C(0),
            Pin::A1  => Registers::C(1),
            Pin::A2  => Registers::C(2),
            Pin::A3  => Registers::C(3),
            Pin::A4  => Registers::C(4),
            Pin::A5  => Registers::C(5),
        }
    }
}

impl Registers {
    fn pinx(&self) -> PINx {
        match self {
            Self::B(offset) => {
                match offset {
                    0 => PINx::B(PINB::PINB0),
                    1 => PINx::B(PINB::PINB1),
                    2 => PINx::B(PINB::PINB2),
                    3 => PINx::B(PINB::PINB3),
                    4 => PINx::B(PINB::PINB4),
                    5 => PINx::B(PINB::PINB5),
                    6 => PINx::B(PINB::PINB6),
                    7 => PINx::B(PINB::PINB7),
                    _ => unreachable!(),
                }
            },
            Self::C(offset) => {
                match offset {
                    0 => PINx::C(PINC::PINC0),
                    1 => PINx::C(PINC::PINC1),
                    2 => PINx::C(PINC::PINC2),
                    3 => PINx::C(PINC::PINC3),
                    4 => PINx::C(PINC::PINC4),
                    5 => PINx::C(PINC::PINC5),
                    6 => PINx::C(PINC::PINC6),
                    _ => unreachable!(),
                }
            },
            Self::D(offset) => {
                match offset {
                    0 => PINx::D(PIND::PIND0),
                    1 => PINx::D(PIND::PIND1),
                    2 => PINx::D(PIND::PIND2),
                    3 => PINx::D(PIND::PIND3),
                    4 => PINx::D(PIND::PIND4),
                    5 => PINx::D(PIND::PIND5),
                    6 => PINx::D(PIND::PIND6),
                    7 => PINx::D(PIND::PIND7),
                    _ => unreachable!(),
                }
            },
        }
    }

    fn ddrx(&self) -> DDRx {
        match self {
            Self::B(offset) => {
                match offset {
                    0 => DDRx::B(DDRB::DDRB0),
                    1 => DDRx::B(DDRB::DDRB1),
                    2 => DDRx::B(DDRB::DDRB2),
                    3 => DDRx::B(DDRB::DDRB3),
                    4 => DDRx::B(DDRB::DDRB4),
                    5 => DDRx::B(DDRB::DDRB5),
                    6 => DDRx::B(DDRB::DDRB6),
                    7 => DDRx::B(DDRB::DDRB7),
                    _ => unreachable!(),
                }
            },
            Self::C(offset) => {
                match offset {
                    0 => DDRx::C(DDRC::DDRC0),
                    1 => DDRx::C(DDRC::DDRC1),
                    2 => DDRx::C(DDRC::DDRC2),
                    3 => DDRx::C(DDRC::DDRC3),
                    4 => DDRx::C(DDRC::DDRC4),
                    5 => DDRx::C(DDRC::DDRC5),
                    6 => DDRx::C(DDRC::DDRC6),
                    _ => unreachable!(),
                }
            },
            Self::D(offset) => {
                match offset {
                    0 => DDRx::D(DDRD::DDRD0),
                    1 => DDRx::D(DDRD::DDRD1),
                    2 => DDRx::D(DDRD::DDRD2),
                    3 => DDRx::D(DDRD::DDRD3),
                    4 => DDRx::D(DDRD::DDRD4),
                    5 => DDRx::D(DDRD::DDRD5),
                    6 => DDRx::D(DDRD::DDRD6),
                    7 => DDRx::D(DDRD::DDRD7),
                    _ => unreachable!(),
                }
            },
        }
    }

    fn portx(&self) -> PORTx {
        match self {
            Self::B(offset) => {
                match offset {
                    0 => PORTx::B(PORTB::PORTB0),
                    1 => PORTx::B(PORTB::PORTB1),
                    2 => PORTx::B(PORTB::PORTB2),
                    3 => PORTx::B(PORTB::PORTB3),
                    4 => PORTx::B(PORTB::PORTB4),
                    5 => PORTx::B(PORTB::PORTB5),
                    6 => PORTx::B(PORTB::PORTB6),
                    7 => PORTx::B(PORTB::PORTB7),
                    _ => unreachable!(),
                }
            },
            Self::C(offset) => {
                match offset {
                    0 => PORTx::C(PORTC::PORTC0),
                    1 => PORTx::C(PORTC::PORTC1),
                    2 => PORTx::C(PORTC::PORTC2),
                    3 => PORTx::C(PORTC::PORTC3),
                    4 => PORTx::C(PORTC::PORTC4),
                    5 => PORTx::C(PORTC::PORTC5),
                    6 => PORTx::C(PORTC::PORTC6),
                    _ => unreachable!(),
                }
            },
            Self::D(offset) => {
                match offset {
                    0 => PORTx::D(PORTD::PORTD0),
                    1 => PORTx::D(PORTD::PORTD1),
                    2 => PORTx::D(PORTD::PORTD2),
                    3 => PORTx::D(PORTD::PORTD3),
                    4 => PORTx::D(PORTD::PORTD4),
                    5 => PORTx::D(PORTD::PORTD5),
                    6 => PORTx::D(PORTD::PORTD6),
                    7 => PORTx::D(PORTD::PORTD7),
                    _ => unreachable!(),
                }
            },
        }
    }
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
    let register = Registers::from(pin.clone()).ddrx();
    match value {
        PinMode::INPUT => unsafe { 
            register.clear();
            digital_write(pin, LOW);
        },
        PinMode::OUTPUT => unsafe { register.set(); },
        PinMode::INPUT_PULLUP => {
            unsafe { register.clear(); }
            digital_write(pin, HIGH);
        },
    }
}

pub fn digital_write(pin: Pin, value: bool) {
    let register = Registers::from(pin).portx();
    unsafe { register.set_value(value); }
}

pub fn digital_read(pin: Pin) -> bool {
    let register = Registers::from(pin).pinx();
    unsafe { register.read() }
}

pub fn digital_toggle(pin: Pin) {
    let register = Registers::from(pin).portx();
    unsafe { register.toggle(); }
}

/// Returns the state of the given analog pin
/// Values are from 0-1023
/// A digital pin will return 0 if LOW or 1023 if HIGH
pub fn analog_read(pin: Pin) -> u16 {
    if pin.is_digital() {
        let value = digital_read(pin);
        return if value { 1023 } else { 0 };
    }

    // Get MUX address
    #[allow(non_snake_case)]
    let (MUX2, MUX1, MUX0) = match pin {
        Pin::A0 => (false, false, false),
        Pin::A1 => (false, false, true),
        Pin::A2 => (false, true,  false),
        Pin::A3 => (false, true,  true),
        Pin::A4 => (true,  false, false),
        Pin::A5 => (true,  false, true),
        _ => unreachable!(),
    };

    
    unsafe {
        // Set Analog Channel Selection Bits to address to the given analog pin
        ADMUX::MUX0.set_value(MUX0);
        ADMUX::MUX1.set_value(MUX1);
        ADMUX::MUX2.set_value(MUX2);
        ADMUX::MUX3.set_value(false);
        
        // Starts the analog to digital conversion
        ADCSRA::ADSC.set();

        // ADSC is automatically zeroed when the conversion finishes
        while ADCSRA::ADSC.read_bit() {}

        // Sets the presentation so that the lower 8 bits are stored in ADCL
        ADMUX::ADLAR.clear();
        
        let low_bits = ADCL::read();
        let high_bits = ADCH::read();

        // Conbines low and high bits into single u16
        (low_bits as u16) | (high_bits as u16)
    }
}

/// Sets the given analog pin to the given value between 0-1023
pub fn analog_write(pin: Pin, value: u16) {
    if pin.is_digital() {
        // Rounds value if pin is digital
        digital_write(pin, value >= 512);
        return;
    }

    todo!()
} 
