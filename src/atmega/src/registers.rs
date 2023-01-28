#![allow(non_camel_case_types)]
use core::{ ptr::{ write_volatile, read_volatile }, ops, cmp };

pub trait Register: Sized + Clone + Copy + Into<u8>
                    + ops::BitAnd<u8, Output=u8>
                    + ops::BitAndAssign<u8>
                    + ops::BitOr<u8, Output = u8>
                    + ops::BitOrAssign<u8>
                    + ops::BitXor<u8, Output = u8>
                    + ops::BitXorAssign<u8>
                    + cmp::PartialEq<u8>
                    + cmp::PartialOrd<u8>
 {
    const ADDR: *mut u8;

    #[inline(always)]
    unsafe fn read() -> u8 {
        read_volatile(Self::ADDR)
    }

    #[inline(always)]
    unsafe fn write(value: u8) {
        write_volatile(Self::ADDR, value) 
    }
    
    #[inline(always)]
    unsafe fn operate<F: Fn(u8) -> u8>(operator: F) {
        Self::write(operator(Self::read()))
    }

    fn bit(&self) -> u8 {
        Into::<u8>::into(*self)
    }

    unsafe fn read_bit(&self) -> bool {
        0 < Self::read() & (1 << self.bit())
    }

    unsafe fn set(&self) {
        Self::write(Self::read() | (1 << self.bit()))
    }

    unsafe fn clear(&self) {
        Self::write(Self::read() & !(1 << self.bit()))
    }

    unsafe fn toggle(&self) {
        Self::write(Self::read() ^ (1 << self.bit()))
    }

    unsafe fn set_value(&self, value: bool) {
        if value {
            self.set();
        } else {
            self.clear();
        }
    }

    unsafe fn until<F: Fn(u8) -> bool>(check: F) {
        loop {
            let value = Self::read();
            if check(value) {
                return;
            }
        }
    }
}

/// Initialize a type as a Register.
/// 
/// Syntax: `register!(<type>[<address>],);`
macro_rules! register {
    ($($t:ty[$addr:expr],)*) => {
        $(
            impl Into<u8> for $t {
                fn into(self) -> u8 {
                    self as u8
                }
            }
            impl ops::BitAnd<u8> for $t {
                type Output = u8;
                fn bitand(self, rhs: u8) -> Self::Output {
                    unsafe { Self::read() & rhs }
                }
            }
            impl ops::BitAndAssign<u8> for $t {
                fn bitand_assign(&mut self, rhs: u8) {
                    unsafe { Self::operate(|val| val & rhs); }
                }
            }
            impl ops::BitOr<u8> for $t {
                type Output = u8;
                fn bitor(self, rhs: u8) -> Self::Output {
                    unsafe { Self::read() | rhs }
                }
            }
            impl ops::BitOrAssign<u8> for $t {
                fn bitor_assign(&mut self, rhs: u8) {
                    unsafe { Self::operate(|val| val | rhs) }
                }
            }
            impl ops::BitXor<u8> for $t {
                type Output = u8;
                fn bitxor(self, rhs: u8) -> Self::Output {
                    unsafe { Self::read() ^ rhs }
                }
            }
            impl ops::BitXorAssign<u8> for $t {
                fn bitxor_assign(&mut self, rhs: u8) {
                    unsafe { Self::operate(|val| val ^ rhs) }
                }
            }
            impl cmp::PartialEq<u8> for $t {
                fn eq(&self, other: &u8) -> bool {
                    unsafe { Self::read() == *other }
                }
            }
            impl cmp::PartialOrd<u8> for $t {
                fn ge(&self, other: &u8) -> bool {
                    let val = unsafe { Self::read() };
                    val >= *other
                }
                fn gt(&self, other: &u8) -> bool {
                    let val = unsafe { Self::read() };
                    val > *other
                }
                fn le(&self, other: &u8) -> bool {
                    let val = unsafe { Self::read() };
                    val <= *other
                }
                fn lt(&self, other: &u8) -> bool {
                    let val = unsafe { Self::read() };
                    val < *other
                }
                fn partial_cmp(&self, other: &u8) -> Option<cmp::Ordering> {
                    let val = unsafe { Self::read() };
                    Some(val.cmp(other))
                }
            }
            impl Register for $t {
                const ADDR: *mut u8 = $addr as *mut u8;
            }
            
        )*
    };
}

#[derive(Clone, Copy)]
pub enum SREG {
    C = 0,
    Z = 1,
    N = 2,
    V = 3,
    S = 4,
    H = 5,
    T = 6,
    I = 7,
}

/// ADC Control and Status Register A
#[derive(Clone, Copy)]
pub enum ADCSRA {
    ADPS0 = 0,
    ADPS1 = 1,
    ADPS2 = 2,
    ADIE  = 3,
    ADIF  = 4,
    ADATE = 5,
    ADSC  = 6,
    ADEN  = 7,
}

/// ADC Control and Status Register B
#[derive(Clone, Copy)]
pub enum ADCSRB {
    ADTS0 = 0,
    ADTS1 = 1,
    ADTS2 = 2,
    ACME  = 6,
}

/// ADC Multiplexer Selection Register
#[derive(Clone, Copy)]
pub enum ADMUX {
    MUX0  = 0,
    MUX1  = 1,
    MUX2  = 2,
    MUX3  = 3,
    ADLAR = 5,
    REFS0 = 6,
    REFS1 = 7,
}

/// Port B Input Pins Address
#[derive(Clone, Copy)]
pub enum PINB {
    PINB0 = 0,
    PINB1 = 1,
    PINB2 = 2,
    PINB3 = 3,
    PINB4 = 4,
    PINB5 = 5,
    PINB6 = 6,
    PINB7 = 7,
}

/// Port B Data Direction Register
#[derive(Clone, Copy)]
pub enum DDRB {
    DDRB0 = 0,
    DDRB1 = 1,
    DDRB2 = 2,
    DDRB3 = 3,
    DDRB4 = 4,
    DDRB5 = 5,
    DDRB6 = 6,
    DDRB7 = 7,
}

/// Port B Data Register
#[derive(Clone, Copy)]
pub enum PORTB {
    PORTB0 = 0,
    PORTB1 = 1,
    PORTB2 = 2,
    PORTB3 = 3,
    PORTB4 = 4,
    PORTB5 = 5,
    PORTB6 = 6,
    PORTB7 = 7,
}

/// Port C Input Pins Address
#[derive(Clone, Copy)]
pub enum PINC {
    PINC0 = 0,
    PINC1 = 1,
    PINC2 = 2,
    PINC3 = 3,
    PINC4 = 4,
    PINC5 = 5,
    PINC6 = 6,
}

/// Port C Data Direction Register
#[derive(Clone, Copy)]
pub enum DDRC {
    DDRC0 = 0,
    DDRC1 = 1,
    DDRC2 = 2,
    DDRC3 = 3,
    DDRC4 = 4,
    DDRC5 = 5,
    DDRC6 = 6,
}

/// Port C Data Register
#[derive(Clone, Copy)]
pub enum PORTC {
    PORTC0 = 0,
    PORTC1 = 1,
    PORTC2 = 2,
    PORTC3 = 3,
    PORTC4 = 4,
    PORTC5 = 5,
    PORTC6 = 6,
}

/// Port D Input Pins Address
#[derive(Clone, Copy)]
pub enum PIND {
    PIND0 = 0,
    PIND1 = 1,
    PIND2 = 2,
    PIND3 = 3,
    PIND4 = 4,
    PIND5 = 5,
    PIND6 = 6,
    PIND7 = 7,
}

/// Port D Data Direction Register
#[derive(Clone, Copy)]
pub enum DDRD {
    DDRD0 = 0,
    DDRD1 = 1,
    DDRD2 = 2,
    DDRD3 = 3,
    DDRD4 = 4,
    DDRD5 = 5,
    DDRD6 = 6,
    DDRD7 = 7,
}

/// Port D Data Register
#[derive(Clone, Copy)]
pub enum PORTD {
    PORTD0 = 0,
    PORTD1 = 1,
    PORTD2 = 2,
    PORTD3 = 3,
    PORTD4 = 4,
    PORTD5 = 5,
    PORTD6 = 6,
    PORTD7 = 7,
}

/// MCU Control Register
#[derive(Clone, Copy)]
pub enum MCUCR {
    IVCE  = 0,
    IVSEL = 1,
    PUD   = 4,
    BODSE = 5,
    BODS  = 6,
}

/// Power Reduction Register
#[derive(Clone, Copy)]
pub enum PRR {
    PRADC    = 0,
    PRUSART0 = 1,
    PRSPI0   = 2,
    PRTIM1   = 3,
    PRTIM0   = 5,
    PRTIM2   = 6,
    PRTWI0   = 7,
}

/// General TC Control Register
#[derive(Clone, Copy)]
pub enum GRCCR {
    PSRSYNC = 0,
    PSRASY  = 1,
    TSM     = 7,
}

/// TC0 Interrupt Flag Register
#[derive(Clone, Copy)]
pub enum TIFR0 {
    TOV0  = 0,
    OCF0A = 1,
    OCF0B = 2,
}

/// TC0 Control Register B
#[derive(Clone, Copy)]
pub enum TCCR0A {
    WGM00  = 0,
    WGM01  = 1,
    COM0B0 = 4,
    COM0B1 = 5,
    COM0A0 = 6,
    COM0A1 = 7,
}

/// TC0 Control Register B
#[derive(Clone, Copy)]
pub enum TCCR0B {
    CS00  = 0,
    CS01  = 1,
    CS02  = 2,
    WGM02 = 3,
    FOC0B = 6,
    FOC0A = 7,
}

/// Counter value register
#[derive(Clone, Copy)]
pub enum TCNT0 {
    TCNT00 = 0,
    TCNT01 = 1,
    TCNT02 = 2,
    TCNT03 = 3,
    TCNT04 = 4,
    TCNT05 = 5,
    TCNT06 = 6,
    TCNT07 = 7,
}

#[derive(Clone, Copy)]
pub enum TIFR1 {
    TOV1  = 0,
    OCF1A = 1,
    OCF1B = 2,
    ICF1  = 5,
}

#[derive(Clone, Copy)]
pub enum TIMSK1 {
    TOIE1  = 0,
    OCIE1A = 1,
    OCIE1B = 2,
    ICIE1  = 5,
}

#[derive(Clone, Copy)]
pub enum TCCR1A {
    WGM10  = 0,
    WGM11  = 1,
    COM1B0 = 4,
    COM1B1 = 5,
    COM1A0 = 6,
    COM1A1 = 7,
}

#[derive(Clone, Copy)]
pub enum TCCR1B {
    CS10  = 0,
    CS11  = 1,
    CS12  = 2,
    WGM12 = 3,
    WGM13 = 4,
    ICES1 = 6,
    ICNC1 = 7,
}

#[derive(Clone, Copy)]
pub enum TCNT1L {
    TCNT1L0 = 0,
    TCNT1L1 = 1,
    TCNT1L2 = 2,
    TCNT1L3 = 3,
    TCNT1L4 = 4,
    TCNT1L5 = 5,
    TCNT1L6 = 6,
    TCNT1L7 = 7,
}

#[derive(Clone, Copy)]
pub enum TCNT1H {
    TCNT1H0 = 0,
    TCNT1H1 = 1,
    TCNT1H2 = 2,
    TCNT1H3 = 3,
    TCNT1H4 = 4,
    TCNT1H5 = 5,
    TCNT1H6 = 6,
    TCNT1H7 = 7,
}

/// Output Compare Register A
#[derive(Clone, Copy)]
pub enum OCR0A {
    OCR0A0 = 0,
    OCR0A1 = 1,
    OCR0A2 = 2,
    OCR0A3 = 3,
    OCR0A4 = 4,
    OCR0A5 = 5,
    OCR0A6 = 6,
    OCR0A7 = 7,
}

/// Output Compare Register B
#[derive(Clone, Copy)]
pub enum OCR0B {
    OCR0B0 = 0,
    OCR0B1 = 1,
    OCR0B2 = 2,
    OCR0B3 = 3,
    OCR0B4 = 4,
    OCR0B5 = 5,
    OCR0B6 = 6,
    OCR0B7 = 7,
}

/// Interrupt Mask Register
#[derive(Clone, Copy)]
pub enum TIMSK0 {
    TOIE0 = 0,
    OCIEA = 1,
    OCIEB = 2,
}

register!(
    SREG[0x3F],
    ADCSRA[0x7A], 
    ADCSRB[0x7B], 
    ADMUX[0x7C], 
    PINB[0x23], 
    DDRB[0x24], 
    PORTB[0x25],
    PINC[0x26],
    DDRC[0x27],
    PORTC[0x28],
    PIND[0x29],
    DDRD[0x2A],
    PORTD[0x2B],
    MCUCR[0x55],
    PRR[0x64],
    GRCCR[0x43],
    TIFR0[0x35],
    TCCR0A[0x44],
    TCCR0B[0x45],
    TCNT0[0x46],
    TIFR1[0x36],
    TIMSK1[0x6F],
    TCCR1A[0x80],
    TCCR1B[0x81],
    TCNT1L[0x84],
    TCNT1H[0x85],
    OCR0A[0x47],
    OCR0B[0x48],
    TIMSK0[0x6E],
);

/// Port B maps to pins `D13`-`D8`
/// 
/// Port C maps to pins `A6`-`A0`
/// 
/// Port D maps to pins `D7`-`D0`
pub enum PinReg<B: Register, C: Register, D: Register> {
    B(B),
    C(C),
    D(D),
}

/// Reads the state of Input Pins on port `x`
pub type PINx = PinReg<PINB, PINC, PIND>;
/// Initialises the pins on port `x` as either inputs or outputs
pub type DDRx = PinReg<DDRB, DDRC, DDRD>;
/// Defines the state of Output pins on port `x`
pub type PORTx = PinReg<PORTB, PORTC, PORTD>;

impl<B: Register, C: Register, D: Register> PinReg<B, C, D> {
    pub unsafe fn set(&self) {
        match self {
            Self::B(bit) => bit.set(),
            Self::C(bit) => bit.set(),
            Self::D(bit) => bit.set(),
        }
    }

    pub unsafe fn clear(&self) {
        match self {
            Self::B(bit) => bit.clear(),
            Self::C(bit) => bit.clear(),
            Self::D(bit) => bit.clear(),
        }
    }

    pub unsafe fn toggle(&self) {
        match self {
            Self::B(bit) => bit.toggle(),
            Self::C(bit) => bit.toggle(),
            Self::D(bit) => bit.toggle(),
        }
    }

    pub unsafe fn read(&self) -> bool {
        match self {
            Self::B(bit) => bit.read_bit(),
            Self::C(bit) => bit.read_bit(),
            Self::D(bit) => bit.read_bit(),
        }
    }

    pub unsafe fn set_value(&self, value: bool) {
        match self {
            Self::B(bit) => bit.set_value(value),
            Self::C(bit) => bit.set_value(value),
            Self::D(bit) => bit.set_value(value),
        }
    }
}

// 8-Bit operators

/// Flips the bit at the given offset.
/// Equivalent to a `not` operation.
pub fn toggle(byte: u8, bit: u8) -> u8 {
    byte ^ (1 << bit)
}

/// Sets the bit at the given offset, changing to a `1`
pub fn set(byte: u8, bit: u8) -> u8 {
    byte | (1 << bit)
}

/// Clears the bit at the given offset, changing to a `0`
pub fn clear(byte: u8, bit: u8) -> u8 {
    byte & !(1 << bit)
}

/// Changes the bit at the given offset to the given value.
pub fn set_value(byte: u8, bit: u8, value: bool) -> u8 {
    if value {
        set(byte, bit)
    } else {
        clear(byte, bit)
    }
}

/// Reads the bit at the given offset.
pub fn read(byte: u8, bit: u8) -> bool {
    let isolated = byte & (1 << bit);
    isolated != 0
}

/// Reads the byte at the given address, performs the given operation on the value, then writes the output back to the address.
/// 
/// # Example
/// ```
/// const ADDR: *mut u8 = 0x23 as *mut u8;
/// write_volatile(ADDR, 0b0011_0011);
/// registers::operate(ADDR, |val| !val);
/// assert_eq!(read_volatile(ADDR), 0b1100_1100);
/// ```
/// 
pub unsafe fn operate<F: Fn(u8) -> u8>(address: *mut u8, operator: F) {
    let current = read_volatile(address);
    write_volatile(address, operator(current));
}
