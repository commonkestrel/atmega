//! Utilities for interacting with pins

// Some documentation taken from https://github.com/arduino/ArduinoCore-avr/blob/master/cores/arduino/wiring.c

use crate::registers::*;

/// Initializes timers for PWM
pub fn _init() {
    unsafe {
        // this needs to be called before setup() or some functions won't work there
        crate::interrupts::enable();

        // timer 0 is also used for fast hardware pwm
        // (using phase-correct PWM would mean that timer 0 overflowed half as often
        // resulting in different millis() behavior)
        TCCR0A::WGM01.set();
        TCCR0A::WGM00.set();
        
        // set timer 0 prescale factor to 64
        TCCR0B::CS01.set();
        TCCR0B::CS00.set();

        // enable timer 0 overflow interrupt 
        TIMSK0::TOIE0.set();

        // timers 1 and 2 are used for phase-correct hardware pwm
        // this is better for motors as it ensures an even waveform
        // note, however, that fast pwm mode can achieve a frequency of up
        // 8 MHz (with a 16 MHz clock) at 50% duty cycle

        // set timer 1 prescale factor to 64
        TCCR1B::write(0);
        TCCR1B::CS11.set();

        // put timer1 in 8-bit phase correct pwm mode
        TCCR1A::WGM10.set();

        // set timer 2 prescale factor to 64
        TCCR2B::CS22.set();

        // configure timer 2 for phase correct pwm (8-bit)
        TCCR2A::WGM20.set();
        
        // set a2d prescaler so we are inside the desired 50-200 KHz range
        let adp = match crate::constants::CPU_FREQUENCY {
            16_000_000.. => (true,  true,  true),  // 16 MHz / 128 = 125 KHz
            8_000_000..  => (false, true,  true),  // 8 MHz / 64 = 125 KHz
            4_000_000..  => (true,  false, true),  // 4 MHz / 32 = 125 KHz
            2_000_000..  => (false, false, true),  // 2 MHz / 16 = 125 KHz
            1_000_000..  => (true,  true,  false), // 1 MHz / 8 = 125 KHz
            _            => (true,  false, false), // 128 KHz / 2 = 64 KHz -> This is the closest you can get, the prescaler is 2
        };

        ADCSRA::ADPS0.set_value(adp.0);
        ADCSRA::ADPS1.set_value(adp.1);
        ADCSRA::ADPS2.set_value(adp.2);

        // enable a2d conversions
        ADCSRA::ADEN.set();

        // the bootloader connects pins 0 and 1 to the USART
        // disconnect them here so they can be used as normal digital i/o
        // they will be reconnected in Serial::begin()
        UCSR0B::write(0);
    }
}

#[allow(missing_docs)]
#[derive(Debug, PartialEq, Clone, Copy)]
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
    SDA,
    SCL,
}

impl Pin {
    fn is_digital(&self) -> bool {
        // When converted to a number Pins 0-13 are digital, 14-19 are analog
        (*self as u8) <= 13
    }

    fn pwm(&self) -> Option<Timer> {
        // Pins 3, 5, 6, 8, 10, and 11 are PWM pins
        match self {
            Self::D6  => Some(Timer::TIMER0A),
            Self::D5  => Some(Timer::TIMER0B),
            Self::D9  => Some(Timer::TIMER1A),
            Self::D10 => Some(Timer::TIMER1B),
            Self::D11 => Some(Timer::TIMER2A),
            Self::D3  => Some(Timer::TIMER2B),
            _ => None
        }
    }
}

enum Timer {
    TIMER0A,
    TIMER0B,
    TIMER1A,
    TIMER1B,
    TIMER2A,
    TIMER2B,
}

impl Timer {
    /// Connect PWM to pin on timer
    fn connect_pwm(&self) {
        use Timer::*;
        unsafe {
            match self {
                TIMER0A => { TCCR0A::COM0A1.set(); },
                TIMER0B => { TCCR0A::COM0B1.set(); },
                TIMER1A => { TCCR1A::COM1A1.set(); },
                TIMER1B => { TCCR1A::COM1B1.set(); },
                TIMER2A => { TCCR2A::COM2A1.set(); },
                TIMER2B => { TCCR2A::COM2B1.set(); },
            }
        }
    }

    fn set_ocr(&self, value: u8) {
        use Timer::*;
        unsafe {
            match self {
                TIMER0A => { OCR0A::write(value); },
                TIMER0B => { OCR0B::write(value); },
                TIMER1A => {
                    OCR1AH::write(0);
                    OCR1AL::write(value);
                },
                TIMER1B => {
                    OCR1BH::write(0);
                    OCR1BL::write(value);
                }
                TIMER2A => { OCR2A::write(value); },
                TIMER2B => { OCR2B::write(value); },
            };
        }
    }
}

impl core::fmt::Display for Pin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
pub enum PinMode {
    INPUT,
    INPUT_PULLUP,
    OUTPUT,
}

/// A high value, usually 5V
pub const HIGH: bool = true;
/// A low value, usually 0V
pub const LOW: bool = false;

/// The registers and offset of a pin.
/// This applies to `PORTx`, `PINx`, and `DDRx`.
#[derive(Debug, Clone)]
pub enum Registers {
    /// B registers: `PORTB`, `PINB`, and `DDRB`
    B(u8),
    /// C registers: `PORTC`, `PINC`, and `DDRC`
    C(u8),
    /// D registers: `PORTD`, `PIND`, and `DDRD`
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
            Pin::SDA => Registers::C(4),
            Pin::SCL => Registers::C(5),
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

/// Sets the mode of the given pin to the given value.
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

/// Sets the given pin to HIGH if `true`, LOW if `false`
pub fn digital_write(pin: Pin, value: bool) {
    let register = Registers::from(pin).portx();
    unsafe { register.set_value(value); }
}

/// Reads the voltage of the given pin, returning `true` if it is above 3V on a 5V chip or above 2V on a 3.3V chip.
pub fn digital_read(pin: Pin) -> bool {
    let register = Registers::from(pin).pinx();
    unsafe { register.read() }
}

/// Toggles the output at the given pin, equivalent to a not (`!`) operation
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
        #[cfg(not(feature = "twowire"))]
        Pin::A4 => (true,  false, false),
        #[cfg(not(feature = "twowire"))]
        Pin::A5 => (true,  false, true),
        #[cfg(feature = "twowire")]
        Pin::SDA => (true, false, false),
        #[cfg(feature = "twowire")]
        Pin::SCL => (true, false, true),
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
        while ADCSRA::ADSC.is_set() {}

        // Sets the presentation so that the lower 8 bits are stored in ADCL
        ADMUX::ADLAR.clear();
        
        let low_bits = ADCL::read();
        let high_bits = ADCH::read();

        // Conbines low and high bits into single u16
        (low_bits as u16) | (high_bits as u16)
    }
}

/// Sets the given PWM pin to the given value between 0-255.
/// If the given pin does not have PWM this will call `digital_write` instead.
pub fn analog_write(pin: Pin, value: u8) {
    pin_mode(pin, PinMode::OUTPUT);
    if value == 0 {
        digital_write(pin, LOW);
    } else if value == 255 {
        digital_write(pin, HIGH);
    }

    let pwm = pin.pwm();
    if let Some(timer) = pwm {
        timer.connect_pwm();  // connect pwm to pin
        timer.set_ocr(value); // set pwm duty
    } else {
        // Round to high or low if the pin does not have PWM
        digital_write(pin, value >= 128)
    }
} 
