//! Allows easy interactions with important registers in the ATmega328p.

#![allow(non_camel_case_types)]
#![allow(missing_docs)]

use core::{ ptr::{ write_volatile, read_volatile }, ops, cmp };

use atmega_macros::Register;

/// Quick trait to allow for using zero and one in a generic way.
/// ONLY used in `Register`.
pub trait Integer: Sized + Clone + Copy + Into<Self> + Default
            + ops::BitAnd<Self, Output=Self>
            + ops::BitAndAssign<Self>
            + ops::BitOr<Self, Output = Self>
            + ops::BitOrAssign<Self>
            + ops::BitXor<Self, Output = Self>
            + ops::BitXorAssign<Self>
            + ops::Shl<Self, Output = Self>
            + ops::Shr<Self, Output = Self>
            + ops::Not<Output = Self>
            + cmp::PartialEq<Self>
            + cmp::PartialOrd<Self>
{
    const ZERO: Self;
    const ONE:  Self;
}

impl Integer for u8 {
    const ZERO: u8 = 0;
    const ONE:  u8 = 1;
}

impl Integer for u16 {
    const ZERO: u16 = 0;
    const ONE:  u16 = 1;
}

impl Integer for u32 {
    const ZERO: u32 = 0;
    const ONE:  u32 = 1;
}

impl Integer for u64 {
    const ZERO: u64 = 0;
    const ONE:  u64 = 1;
}

impl Integer for u128 {
    const ZERO: u128 = 0;
    const ONE:  u128 = 1;
}

/// Generic utilies for interacting with registers, like reading, writing, operating, etc...
/// Meant to be applied to an `enum` with the varients used for individual bits for generating a bit mask
pub trait Register<SIZE>: Sized + Clone + Copy + Into<SIZE>
where SIZE: Integer
{
    const READ: *mut SIZE;
    const WRITE: *mut SIZE;

    #[inline(always)]
    unsafe fn read() -> SIZE {
        read_volatile(Self::READ)
    }

    #[inline(always)]
    unsafe fn write(value: SIZE) {
        write_volatile(Self::WRITE, value) 
    }
    
    #[inline(always)]
    unsafe fn operate<F: Fn(SIZE) -> SIZE>(operator: F) {
        Self::write(operator(Self::read()))
    }

    #[inline(always)]
    fn bit(&self) -> SIZE {
        Into::<SIZE>::into(*self)
    }

    #[inline(always)]
    fn bv(&self) -> SIZE {
        SIZE::ONE << self.bit()
    }

    #[inline(always)]
    unsafe fn is_set(&self) -> bool {
        SIZE::default() < Self::read() & (SIZE::ONE << self.bit())
    }

    #[inline(always)]
    unsafe fn is_clear(&self) -> bool {
        SIZE::ONE == Self::read() & (SIZE::ONE << self.bit())
    }

    #[inline(always)]
    unsafe fn set(&self) {
        Self::write(Self::read() | (SIZE::ONE << self.bit()))
    }

    #[inline(always)]
    unsafe fn clear(&self) {
        Self::write(Self::read() & !(SIZE::ONE << self.bit()))
    }

    #[inline(always)]
    unsafe fn toggle(&self) {
        Self::write(Self::read() ^ (SIZE::ONE << self.bit()))
    }

    #[inline(always)]
    unsafe fn set_value(&self, value: bool) {
        if value {
            self.set();
        } else {
            self.clear();
        }
    }

    #[inline(always)]
    unsafe fn until<F: Fn(SIZE) -> bool>(check: F) {
        while !check(Self::read()) {}
    }
}

/// AVR Status Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x3F, write=0x5F, size=8)]
pub enum SREG {
    C = 0,
    Z = 1,
    N = 2,
    V = 3,
    S = 4,
    H = 5,
    T = 6,
    I = 7,
    None,
}

/// ADC Control and Status Register A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x7A, size=8)]
pub enum ADCSRA {
    ADPS0 = 0,
    ADPS1 = 1,
    ADPS2 = 2,
    ADIE  = 3,
    ADIF  = 4,
    ADATE = 5,
    ADSC  = 6,
    ADEN  = 7,
    None,
}

/// ADC Control and Status Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x7B, size=8)]
pub enum ADCSRB {
    ADTS0 = 0,
    ADTS1 = 1,
    ADTS2 = 2,
    ACME  = 6,
    None,
}

/// ADC Multiplexer Selection Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x7C, size=8)]
pub enum ADMUX {
    MUX0  = 0,
    MUX1  = 1,
    MUX2  = 2,
    MUX3  = 3,
    ADLAR = 5,
    REFS0 = 6,
    REFS1 = 7,
    None,
}

/// Port B Input Pins Address
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x03, write=0x23, size=8)]
pub enum PINB {
    PINB0 = 0,
    PINB1 = 1,
    PINB2 = 2,
    PINB3 = 3,
    PINB4 = 4,
    PINB5 = 5,
    PINB6 = 6,
    PINB7 = 7,
    None,
}

/// Port B Data Direction Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x04, write=0x24, size=8)]
pub enum DDRB {
    DDRB0 = 0,
    DDRB1 = 1,
    DDRB2 = 2,
    DDRB3 = 3,
    DDRB4 = 4,
    DDRB5 = 5,
    DDRB6 = 6,
    DDRB7 = 7,
    None,
}

/// Port B Data Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x05, write=0x25, size=8)]
pub enum PORTB {
    PORTB0 = 0,
    PORTB1 = 1,
    PORTB2 = 2,
    PORTB3 = 3,
    PORTB4 = 4,
    PORTB5 = 5,
    PORTB6 = 6,
    PORTB7 = 7,
    None,
}

/// Port C Input Pins Address
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x08, write=0x28, size=8)]
pub enum PINC {
    PINC0 = 0,
    PINC1 = 1,
    PINC2 = 2,
    PINC3 = 3,
    PINC4 = 4,
    PINC5 = 5,
    PINC6 = 6,
    None,
}

/// Port C Data Direction Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x07, write=0x27, size=8)]
pub enum DDRC {
    DDRC0 = 0,
    DDRC1 = 1,
    DDRC2 = 2,
    DDRC3 = 3,
    DDRC4 = 4,
    DDRC5 = 5,
    DDRC6 = 6,
    None,
}

/// Port C Data Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x06, write=0x26, size=8)]
pub enum PORTC {
    PORTC0 = 0,
    PORTC1 = 1,
    PORTC2 = 2,
    PORTC3 = 3,
    PORTC4 = 4,
    PORTC5 = 5,
    PORTC6 = 6,
    None,
}

/// Port D Input Pins Address
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x09, write=0x29, size=8)]
pub enum PIND {
    PIND0 = 0,
    PIND1 = 1,
    PIND2 = 2,
    PIND3 = 3,
    PIND4 = 4,
    PIND5 = 5,
    PIND6 = 6,
    PIND7 = 7,
    None,
}

/// Port D Data Direction Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x0A, write=0x2A, size=8)]
pub enum DDRD {
    DDRD0 = 0,
    DDRD1 = 1,
    DDRD2 = 2,
    DDRD3 = 3,
    DDRD4 = 4,
    DDRD5 = 5,
    DDRD6 = 6,
    DDRD7 = 7,
    None,
}

/// Port D Data Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x0B, write=0x2B, size=8)]
pub enum PORTD {
    PORTD0 = 0,
    PORTD1 = 1,
    PORTD2 = 2,
    PORTD3 = 3,
    PORTD4 = 4,
    PORTD5 = 5,
    PORTD6 = 6,
    PORTD7 = 7,
    None,
}

/// MCU Control Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x35, write=0x55, size=8)]
pub enum MCUCR {
    IVCE  = 0,
    IVSEL = 1,
    PUD   = 4,
    BODSE = 5,
    BODS  = 6,
    None,
}

/// Power Reduction Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x64, size=8)]
pub enum PRR {
    PRADC    = 0,
    PRUSART0 = 1,
    PRSPI0   = 2,
    PRTIM1   = 3,
    PRTIM0   = 5,
    PRTIM2   = 6,
    PRTWI0   = 7,
    None,
}

/// General TC Control Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x23, write=0x43, size=8)]
pub enum GTCCR {
    PSRSYNC = 0,
    PSRASY  = 1,
    TSM     = 7,
    None,
}

/// TC0 Interrupt Flag Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x15, write=0x35, size=8)]
pub enum TIFR0 {
    TOV0  = 0,
    OCF0A = 1,
    OCF0B = 2,
    None,
}

/// TC0 Control Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x24, write=0x44, size=8)]
pub enum TCCR0A {
    WGM00  = 0,
    WGM01  = 1,
    COM0B0 = 4,
    COM0B1 = 5,
    COM0A0 = 6,
    COM0A1 = 7,
    None,
}

/// TC0 Control Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x25, write=0x45, size=8)]
pub enum TCCR0B {
    CS00  = 0,
    CS01  = 1,
    CS02  = 2,
    WGM02 = 3,
    FOC0B = 6,
    FOC0A = 7,
    None,
}

/// Counter value register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x26, write=0x46, size=8)]
pub enum TCNT0 {
    TCNT00 = 0,
    TCNT01 = 1,
    TCNT02 = 2,
    TCNT03 = 3,
    TCNT04 = 4,
    TCNT05 = 5,
    TCNT06 = 6,
    TCNT07 = 7,
    None,
}

/// Timer/Counter1 Interrupt Flag Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x16, write=0x36, size=8)]
pub enum TIFR1 {
    TOV1  = 0,
    OCF1A = 1,
    OCF1B = 2,
    ICF1  = 5,
    None,
}

/// Timer/Counter1 Interrupt Mask Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x6F, size=8)]
pub enum TIMSK1 {
    TOIE1  = 0,
    OCIE1A = 1,
    OCIE1B = 2,
    ICIE1  = 5,
    None,
}

/// Timer/Counter1 Control Register A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x80, size=8)]
pub enum TCCR1A {
    WGM10  = 0,
    WGM11  = 1,
    COM1B0 = 4,
    COM1B1 = 5,
    COM1A0 = 6,
    COM1A1 = 7,
    None,
}

/// Timer/Counter1 Control Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x81, size=8)]
pub enum TCCR1B {
    CS10  = 0,
    CS11  = 1,
    CS12  = 2,
    WGM12 = 3,
    WGM13 = 4,
    ICES1 = 6,
    ICNC1 = 7,
    None,
}

/// Timer/Counter2 Control Register A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xB0, size=8)]
pub enum TCCR2A {
    WGM20  = 0,
    WGM21  = 1,
    COM2B0 = 4,
    COM2B1 = 5,
    COM2A0 = 6,
    COM2A1 = 7,
    None,
}

/// Timer/Counter2 Control Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xB1, size=8)]
pub enum TCCR2B {
    CS20  = 0,
    CS21  = 1,
    CS22  = 2,
    WGM22 = 3,
    FOC2B = 6,
    FOC2A = 7,
    None,
}

/// Timer / Counter1
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x84, size=16)]
pub enum TCNT1L {
    TCNT1L0 = 0,
    TCNT1L1 = 1,
    TCNT1L2 = 2,
    TCNT1L3 = 3,
    TCNT1L4 = 4,
    TCNT1L5 = 5,
    TCNT1L6 = 6,
    TCNT1L7 = 7,
    TCNT1H0 = 8,
    TCNT1H1 = 9,
    TCNT1H2 = 10,
    TCNT1H3 = 11,
    TCNT1H4 = 12,
    TCNT1H5 = 13,
    TCNT1H6 = 14,
    TCNT1H7 = 15,
    None,
}

/// Timer 0 Output Compare Register A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x27, write=0x47, size=8)]
pub enum OCR0A {
    OCR0A0 = 0,
    OCR0A1 = 1,
    OCR0A2 = 2,
    OCR0A3 = 3,
    OCR0A4 = 4,
    OCR0A5 = 5,
    OCR0A6 = 6,
    OCR0A7 = 7,
    None,
}

/// Timer 0 Output Compare Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x28, write=0x48, size=8)]
pub enum OCR0B {
    OCR0B0 = 0,
    OCR0B1 = 1,
    OCR0B2 = 2,
    OCR0B3 = 3,
    OCR0B4 = 4,
    OCR0B5 = 5,
    OCR0B6 = 6,
    OCR0B7 = 7,
    None,
}

/// Timer 1 Output Compare Register A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x88, size=16)]
pub enum OCR1A {
    OCR0A0  = 0,
    OCR0A1  = 1,
    OCR0A2  = 2,
    OCR0A3  = 3,
    OCR0A4  = 4,
    OCR0A5  = 5,
    OCR0A6  = 6,
    OCR0A7  = 7,
    OCR0B8  = 8,
    OCR0B9  = 9,
    OCR0B10 = 10,
    OCR0B11 = 11,
    OCR0B12 = 12,
    OCR0B13 = 13,
    OCR0B14 = 14,
    OCR0B15 = 15,
    None,
}

/// Timer 1 Output Compare Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x8A, size=16)]
pub enum OCR1B {
    OCR0A0  = 0,
    OCR0A1  = 1,
    OCR0A2  = 2,
    OCR0A3  = 3,
    OCR0A4  = 4,
    OCR0A5  = 5,
    OCR0A6  = 6,
    OCR0A7  = 7,
    OCR0B8  = 8,
    OCR0B9  = 9,
    OCR0B10 = 10,
    OCR0B11 = 11,
    OCR0B12 = 12,
    OCR0B13 = 13,
    OCR0B14 = 14,
    OCR0B15 = 15,
    None,
}

/// Timer 2 Output Compare Register A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xB3, size=8)]
pub enum OCR2A {
    OCR2A0 = 0,
    OCR2A1 = 1,
    OCR2A2 = 2,
    OCR2A3 = 3,
    OCR2A4 = 4,
    OCR2A5 = 5,
    OCR2A6 = 6,
    OCR2A7 = 7,
    None,
}

/// Timer 2 Output Compare Register B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xB4, size=8)]
pub enum OCR2B {
    OCR2B0 = 0,
    OCR2B1 = 1,
    OCR2B2 = 2,
    OCR2B3 = 3,
    OCR2B4 = 4,
    OCR2B5 = 5,
    OCR2B6 = 6,
    OCR2B7 = 7,
    None,
}

/// Timer 0 Interrupt Mask Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x6E, size=8)]
pub enum TIMSK0 {
    TOIE0 = 0,
    OCIEA = 1,
    OCIEB = 2,
    None,
}

/// USART Baud Rate Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xC4, size=16)]
pub enum UBRR0 {
    UBRR00  = 0,
    UBRR01  = 1,
    UBRR02  = 2,
    UBRR03  = 3,
    UBRR04  = 4,
    UBRR05  = 5,
    UBRR06  = 6,
    UBRR07  = 7,
    UBRR08  = 8,
    UBRR09  = 9,
    UBRR010 = 10,
    UBRR011 = 11,
    None,
}

/// USART Control and Status Register 0 A
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xC0, size=8)]
pub enum UCSR0A {
    MPCM0 = 0,
    U2X0  = 1,
    UPE0  = 2,
    DOR0  = 3,
    FE0   = 4,
    UDRE0 = 5,
    TXC0  = 6,
    RXC0  = 7,
    None,
}

/// USART Control and Status Register 0 B
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xC1, size=8)]
pub enum UCSR0B {
    TXB80  = 0,
    RXB80  = 1,
    UCSZ02 = 2,
    TXEN0  = 3,
    RXEN0  = 4,
    UDRIE0 = 5,
    TXCIE0 = 6,
    RXCIE0 = 7,
    None,
}

/// USART Control and Status Register 0 C
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xC2, size=8)]
pub enum UCSR0C {
    UCPOL0  = 0,
    UCSZ00  = 1,
    UCSZ01  = 2,
    USBS0   = 3,
    UPM00   = 4,
    UPM01   = 5,
    UMSEL00 = 6,
    UMSEL01 = 7,
    None,
}

/// USART I/O Data Register 0
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xC6, size=8)]
pub enum UDR0 {
    UDR00 = 0,
    UDR01 = 1,
    UDR02 = 2,
    UDR03 = 3,
    UDR04 = 4,
    UDR05 = 5,
    UDR06 = 6,
    UDR07 = 7,
    None,
}

/// ADC Data Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0x77, size=16)]
pub enum ADC {
    None,
}

/// TWI Status Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xB9, size=8)]
pub enum TWSR {
    TWPS0 = 0,
    TWPS1 = 1,
    TWS3 = 3,
    TWS4 = 4,
    TWS5 = 5,
    TWS6 = 6,
    TWS7 = 7,
    None,
}

/// TWI Data Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xBB, size=8)]
pub enum TWDR {
    TWD0 = 0,
    TWD1 = 1,
    TWD2 = 2,
    TWD3 = 3,
    TWD4 = 4,
    TWD5 = 5,
    TWD6 = 6,
    TWD7 = 7,
    None,
}

/// TWI Control Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xBC, size=8)]
pub enum TWCR {
    TWIE  = 0,
    TWEN  = 2,
    TWWC  = 3,
    TWSTO = 4,
    TWSTA = 5,
    TWEA  = 6,
    TWINT = 7,
    None,
}

/// TWI Bit Rate Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xB8, size=8)]
pub enum TWBR {
    TWBR0 = 0,
    TWBR1 = 1,
    TWBR2 = 2,
    TWBR3 = 3,
    TWBR4 = 4,
    TWBR5 = 5,
    TWBR6 = 6,
    TWBR7 = 7,
    None,
}

/// TWI (Peripheral) Address Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(addr=0xBA, size=8)]
pub enum TWAR {
    TWGCE = 0,
    TWA0  = 1,
    TWA1  = 2,
    TWA2  = 3,
    TWA3  = 4,
    TWA4  = 5,
    TWA5  = 6,
    TWA6  = 7,
    None,
}

/// External Interrupt Mask Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x1D, write=0x3D, size=8)]
pub enum EIMSK {
    INT0 = 0,
    INT1 = 1,
    None,
}

/// SPI Control Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x2C, write=0x4C, size=8)]
pub enum SPCR {
    SPR0 = 0,
    SPR1 = 1,
    CPHA = 2,
    CPOL = 3,
    MSTR = 4,
    DORD = 5,
    SPE  = 6,
    SPIE = 7,
    None,
}

/// SPI Data Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x2E, write=0x4E, size=8)]
pub enum SPDR {
    LSB = 0,
    MSB = 7,
    None,
}

/// SPI Status Register
#[derive(Clone, Copy, PartialEq, Register)]
#[register(read=0x2E, write=0x4E, size=8)]
pub enum SPSR {
    SPI2X = 0,
    WCOL  = 6,
    SPIF  = 7,
    None,
}

/// Port B maps to pins `D13`-`D8`,
/// Port C maps to pins `A6`-`A0`,
/// Port D maps to pins `D7`-`D0`
pub enum PinReg<B: Register<u8>, C: Register<u8>, D: Register<u8>> {
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

impl<B: Register<u8>, C: Register<u8>, D: Register<u8>> PinReg<B, C, D> {
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
            Self::B(bit) => bit.is_set(),
            Self::C(bit) => bit.is_set(),
            Self::D(bit) => bit.is_set(),
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
