//! Implementation of the I2C protocol via the Arduino [Wire](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire) library
//! 
//! Implementation and most documentation taken from the official [Wire source](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire/src)

#![allow(non_snake_case, non_upper_case_globals, dead_code, non_camel_case_types)]

use crate::registers::{ Register, TWSR, TWCR, TWBR, TWAR, TWDR };
use crate::wiring::{ digital_write, Pin };
use crate::constants::CPU_FREQUENCY;
use crate::prelude::delay_micros;
use crate::volatile::Volatile;
use crate::timing::micros;

/// Length of master, TX, and RX buffers.
pub const TWI_BUFFER_LENGTH: usize = 32;

/// 
#[derive(Debug, Clone, Copy)]
pub struct ByteBuffer {
    /// Index of the buffer.
    pub index: usize,
    /// Length of the buffer.
    pub length: usize,
    /// Inner array containing the buffer data.
    pub inner: [u8; TWI_BUFFER_LENGTH],
}

impl ByteBuffer {
    /// Creates a new zeroed buffer.
    pub const fn new() -> ByteBuffer {
        ByteBuffer {
            index: 0,
            length: 0,
            inner: [0; TWI_BUFFER_LENGTH],
        }
    }

    /// Creates a new buffer from a single byte.
    pub fn single(byte: u8) -> ByteBuffer {
        let mut blank = ByteBuffer::new();
        blank.inner[0] = byte;
        blank
    }

    /// Resets the length and index to zero.
    /// Effectivly clears the buffer.
    pub fn reset(&mut self) {
        self.index = 0;
        self.length = 0;
    }
}

impl Iterator for ByteBuffer {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            // Reset for next iteration
            self.index = 0;
            return None;
        }

        Some(self.inner[self.index as usize])
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    /// Ready to transmit.
    READY,
    /// Controller reciever mode.
    CRX,
    /// Controller transmitter mode.
    CTX,
    /// Peripheral reciever mode.
    PRX,
    /// Peripheral tramitter mode.
    PTX,
}

const TWI_FREQ: u64 = 100_000;

/// SLA+R address
const TW_READ: u8 = 1;
/// SLA+W address
const TW_WRITE: u8 = 0;

/// 
const TW_STATUS_MASK: u8 = 0b1111_1000; // TWS7 | TWS6 | TWS5 | TWS4 | TWS3

/// TWI Status Flags
/// 
/// `TW_CT_xxx` : Controller transmitter
/// 
/// `TW_CR_xxx` : Controller receiver
///
/// `TW_PT_xxx` : Peripheral transmitter
///
/// `TW_PR_xxx` : Peripheral receiver
pub enum Flags {
    /// Start condition transmitted
    TW_START = 0x08,
    /// Repeated start condition transmitted
    TW_REP_START = 0x10,
    /// SLA+W transmitted, ACK received
    TW_CT_SLA_ACK = 0x18,
    /// SLA+W transmitted, NACK received
    TW_CT_SLA_NACK = 0x20,
    /// Data transmitted, ACK received
    TW_CT_DATA_ACK = 0x28,
    /// Data transmitted, NACK, received
    TW_CT_DATA_NACK = 0x30,
    /// Arbitration lost in SLA+W/R, data, or NACK.
    /// 
    /// TW_CT_ARB_LOST and TW_CR_ARB_LOST share this flag.
    TW_CT_CR_ARB_LOST = 0x38,
    /// SLA+R transmitted, ACK received
    TW_CR_SLA_ACK = 0x40,
    /// RLA+R transmitted, NACK received
    TW_CR_SLA_NACK = 0x48,
    /// Data recived, ACK returned
    TW_CR_DATA_ACK = 0x50,
    /// Data recived, NACK returned
    TW_CR_DATA_NACK = 0x58,
    /// SLA+R received, ACK returned
    TW_PT_SLA_ACK = 0xA8,
    /// Artibation lost in SLA+RW, SLA+R recived, ACK returned
    TW_PT_ARB_LOST_SLA_ACK = 0xB0,
    /// Data transmitted, ACK received
    TW_PT_DATA_ACK = 0xB8,
    /// Data transmitted, NACK received
    TW_PT_DATA_NACK = 0xC0,
    /// Last data byte transmitted, ACK received
    TW_PT_LAST_DATA = 0xC8,
    /// SLA+W recieved, ACK returned
    TW_PR_SLA_ACK = 0x60,
    /// Arbitration lost in SLA+RW, SLA+W received, ACK returned
    TW_PR_ARB_LOST_SLA_ACK = 0x68,
    /// General call received, ACK returned
    TW_PR_GCALL_ACK = 0x70,
    /// Arbitration lost in SLA+RW, general call received, ACK returned
    TW_PR_ARB_LOST_GCALL_ACK = 0x78,
    /// Data received, ACK returned
    TW_PR_DATA_ACK = 0x80,
    /// Data received, NACK returned
    TW_PR_DATA_NACK = 0x88,
    /// General call data received, ACK returned
    TW_PR_GCALL_DATA_ACK = 0x90,
    /// General call data received, NACK returned
    TW_PR_GCALL_DATA_NACK = 0x98,
    /// Stop or repeated start condition received while selected
    TW_PR_STOP = 0xA0,
    /// No state information available
    TW_NO_INFO = 0xF8,
    /// Illegal start or stop condition
    TW_BUS_ERROR = 0x00,
}

impl Flags {
    fn from_flag(flag: u8) -> Option<Flags> {
        use Flags::*;
        match flag & TW_STATUS_MASK {
            0x00 => Some(TW_BUS_ERROR),
            0x08 => Some(TW_START),
            0x10 => Some(TW_REP_START),
            0x18 => Some(TW_CT_SLA_ACK),
            0x20 => Some(TW_CT_SLA_NACK),
            0x28 => Some(TW_CT_DATA_ACK),
            0x30 => Some(TW_CT_DATA_NACK),
            0x38 => Some(TW_CT_CR_ARB_LOST),
            0x40 => Some(TW_CR_SLA_ACK),
            0x48 => Some(TW_CR_SLA_NACK),
            0x50 => Some(TW_CR_DATA_ACK),
            0x58 => Some(TW_CR_DATA_NACK),
            0x60 => Some(TW_PR_SLA_ACK),
            0x68 => Some(TW_PR_ARB_LOST_SLA_ACK),
            0x70 => Some(TW_PR_GCALL_ACK),
            0x78 => Some(TW_PR_ARB_LOST_GCALL_ACK),
            0x80 => Some(TW_PR_DATA_ACK),
            0x88 => Some(TW_PR_DATA_NACK),
            0x90 => Some(TW_PR_GCALL_DATA_ACK),
            0x98 => Some(TW_PR_GCALL_DATA_NACK),
            0xA0 => Some(TW_PR_STOP),
            0xA8 => Some(TW_PT_SLA_ACK),
            0xB0 => Some(TW_PT_ARB_LOST_SLA_ACK),
            0xB8 => Some(TW_PT_DATA_ACK),
            0xC0 => Some(TW_PT_DATA_NACK),
            0xC8 => Some(TW_PT_LAST_DATA),
            0xF8 => Some(TW_NO_INFO),
            _ => None,
        }
    }
}



static twi_state: Volatile<State> = Volatile::new(State::READY);
static twi_slarw: Volatile<u8> = Volatile::new(0);
static twi_send_stop: Volatile<bool> = Volatile::new(true);     // should the transaction end with a stop
static twi_in_rep_start: Volatile<bool> = Volatile::new(false); // in the middle of a repeated start

// twi_timeout_us > 0 prevents the code from getting stuck in various while loops here
// if twi_timeout_us == 0 then timeout checking is disabled (the previous Wire lib behavior)
// at some point in the future, the default twi_timeout_us value could become 25000
// and twi_do_reset_on_timeout could become true
// to conform to the SMBus standard
// http://smbus.org/specs/SMBus_3_1_20180319.pdf
static twi_timeout_us: Volatile<u32> = Volatile::new(0);
static twi_timed_out_flag: Volatile<bool> = Volatile::new(false);       // a timeout has been seen
static twi_do_reset_on_timeout: Volatile<bool> = Volatile::new(false); // reset the TWI registers on timeout

fn blank_transmit() {}
static twi_on_peripheral_transmit: Volatile<fn()> = Volatile::new(blank_transmit);

fn blank_receive(_bytes: ByteBuffer, _length: usize) {}
static twi_on_peripheral_receive: Volatile<fn(ByteBuffer, usize)> = Volatile::new(blank_receive);

static twi_master_buffer: Volatile<ByteBuffer> = Volatile::new(ByteBuffer::new());
static twi_tx_buffer: Volatile<ByteBuffer> = Volatile::new(ByteBuffer::new());
static twi_rx_buffer: Volatile<ByteBuffer> = Volatile::new(ByteBuffer::new());

static twi_error: Volatile<u8> = Volatile::new(0xFF);

/// Readies twi pins and sets twi bitrate
pub fn twi_init() {
    // Activate internal pullups for TWI
    digital_write(Pin::SDA, true);
    digital_write(Pin::SCL, true);
    
    unsafe {
        // Initialize TWI prescaler and bit rate
        TWSR::TWPS0.clear();
        TWSR::TWPS1.clear();
        TWBR::write((((CPU_FREQUENCY / TWI_FREQ) - 16) / 2) as u8);
        
        // Enable TWI module, acks, and TWI interrupt
        TWCR::TWEN.set();
        TWCR::TWIE.set();
        TWCR::TWEA.set();
    }
}

pub fn twi_disable() {
    unsafe {
        // Disable TWI module, acks, and TWI interrupt
        TWCR::TWEN.clear();
        TWCR::TWIE.clear();
        TWCR::TWEA.clear();
    }

    // Disable internal pullups for TWI
    digital_write(Pin::SDA, false);
    digital_write(Pin::SCL, false);
}

pub fn set_address(address: u8) {
    unsafe { TWAR::write(address << 1) }
}

pub fn set_frequency(frequency: u64) {
    unsafe { TWBR::write((((CPU_FREQUENCY / frequency) - 16)/2) as u8); }
}

/// Error from `read_from()`
pub enum ReadError {
    /// Requested length is too large to fit in the buffer.
    TooLarge,
    /// Request timed out.
    Timeout,
}

/// Attempts to become TWI bus controller and read a
/// series of bytes from a device on the bus.
/// 
/// `address` is a 7-bit I2C device address.
pub fn read_from(address: u8, length: usize, send_stop: bool) -> Result<ByteBuffer, ReadError> {
    // Ensure data will fit into buffer
    if TWI_BUFFER_LENGTH < length as usize {
        return Err(ReadError::TooLarge);
    }

    let start_micros = micros();
    while twi_state.read() != State::READY  {
        if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
            twi_handle_timeout(twi_do_reset_on_timeout.read());
            return Err(ReadError::Timeout);
        }
    }
    
    twi_state.write(State::CRX);
    twi_send_stop.write(send_stop);
    // Reset error state (0xFF.. no error occurred)
    twi_error.write(0xFF);

    twi_master_buffer.as_mut(|buf| {
        buf.index = 0;
        buf.length = length-1; // This is not intuitive, read on...
        // On receive, the previously configured ACK/NACK setting is transmitted in
        // response to the received byte before the interrupt is signalled. 
        // Therefore we must actually set NACK when the _next_ to last byte is
        // received, causing that NACK to be sent in response to receiving the last
        // expected byte of data.
    });

    twi_slarw.write(TW_READ | (address << 1));

    unsafe {
        if twi_in_rep_start.read() {
            // If we're in the repeated start state, then we've already sent the start,
            // (we hope), and the TWI statemachine is just waiting for the address byte.
            // We need to remove ourselves from the repeated start state before we enable interrupts,
            // since the ISR is ASYNC, and we could get confused if we hit the ISR before cleaning
            // up. Also, don't enable the START interrupt. There may be one pending from the
            // repeated start that we sent ourselves, and that would really confuse things.
            twi_in_rep_start.write(false); // Remember, we're dealing with an ASYNC ISR

            let start_micros = micros();
            let timeout_us = twi_timeout_us.read();

            while TWCR::TWWC.read_bit() {
                TWDR::write(twi_slarw.read());
                if timeout_us > 0 && (micros() - start_micros) > timeout_us as u64 {
                    twi_handle_timeout(twi_do_reset_on_timeout.read());
                    return Err(ReadError::Timeout);
                }
            }
            // enable INTs, but not START
            TWCR::TWINT.set();
            TWCR::TWEA.set();
            TWCR::TWEN.set();
            TWCR::TWIE.set();
            TWCR::TWSTA.clear();
        } else {
            // Sent start condition
            TWCR::TWINT.set();
            TWCR::TWEA.set();
            TWCR::TWEN.set();
            TWCR::TWIE.set();
            TWCR::TWSTA.set();
        }
        
        let start_micros = micros();
        while twi_state.read() == State::CRX {
            if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
                twi_handle_timeout(twi_do_reset_on_timeout.read());
                return Err(ReadError::Timeout);
            }
        }
        
        twi_master_buffer.as_mut(|buf| {
            let mut data: ByteBuffer = ByteBuffer::new();
            for i in 0..length as usize {
                data.inner[i] = buf.inner[i];
            }

            Ok(data)
        })
    }
}

pub enum WriteError {
    /// The length of the data passed is larger than the TX master buffer.
    TooLarge,
    /// Address send, NACK received
    SlaNack = 2,
    /// Data send, NACK received
    DataNack,
    /// Other TWI error
    Other,
    /// Timed out
    Timeout,
}

pub fn write_to(address: u8, data: ByteBuffer, length: usize, wait: bool, send_stop: bool) -> Result<(), WriteError> {
    if TWI_BUFFER_LENGTH < length {
        return Err(WriteError::TooLarge);
    }

    // Wait until TWI is ready, become controller transmitter
    let start_micros = micros();
    while twi_state.read() != State::READY {
        if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
            twi_handle_timeout(twi_do_reset_on_timeout.read());
            return Err(WriteError::Timeout);
        }
    }
    twi_state.write(State::CTX);
    twi_send_stop.write(send_stop);
    // Reset error state (0xFF.. no error occured)
    twi_error.write(0xFF);

    twi_master_buffer.as_mut(|buf| {
        buf.index = 0;
        buf.length = length;
    });

    twi_master_buffer.as_mut(|buf| {
        for i in 0..length {
            buf.inner[i] = data.inner[i];
        } 
    });

    // Build sla+w, peripheral device address + w bit
    twi_slarw.write(TW_WRITE | (address << 1));

    // If we're in a repeated start, then we've already sent the START in the ISR.
    // Don't do it again.
    use TWCR::*;
    if twi_in_rep_start.read() {
        twi_in_rep_start.write(false);
        
        let start_micros = micros();
        unsafe {
            while TWCR::TWWC.read_bit() {
                TWDR::write(twi_slarw.read());
                if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
                    twi_handle_timeout(twi_do_reset_on_timeout.read());
                    return Err(WriteError::Timeout);
                }
            }
            // Enable INTs, but not START
            TWCR::write( TWINT.bv() | TWEA.bv() | TWEN.bv() | TWIE.bv() )
        }
    } else {
        crate::println!("z");

        // Send start condition
        unsafe { TWCR::write( TWINT.bv() | TWEA.bv() | TWEN.bv() | TWIE.bv() | TWSTA.bv() ); }
    }

    // Wait for write operation to complete
    let start_micros = micros();
    while wait && twi_state.read() == State::CTX {
        if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
            twi_handle_timeout(twi_do_reset_on_timeout.read());
            return Err(WriteError::Timeout);
        }
    }

    match twi_error.read() {
        0xFF => Ok(()),
        0x20 => Err(WriteError::SlaNack),
        0x30 => Err(WriteError::DataNack),
        _ => Err(WriteError::Other)
    }
}

/// Possible errors during transmission.
pub enum TransmitError {
    /// Length too long for TX buffer.
    TooLarge,
    /// Not peripheral transmitter.
    NotPTX,
}

/// Fills peripheral tx buffer with data.
/// Must be called in peripheral TX event callback.
pub fn twi_transmit(data: ByteBuffer, length: usize) -> Result<(), TransmitError> {
    // Ensure data will fit into buffer
    if TWI_BUFFER_LENGTH < twi_tx_buffer.as_deref(|buf| buf.length)+length {
        return Err(TransmitError::TooLarge);
    }

    // Ensure we are currently as peripheral transmitter
    if twi_state.read() != State::PTX {
        return Err(TransmitError::NotPTX);
    }

    // Copy data into tx buffer
    twi_tx_buffer.as_mut(|buf| {
        for i in 0..length {
            buf.inner[buf.length+i] = data.inner[i];
        }
        buf.length += length;
    });

    Ok(())
}

pub fn twi_attach_peripheral_rx_event(callback: fn(ByteBuffer, usize)) {
    twi_on_peripheral_receive.write(callback);
}

pub fn twi_attach_peripheral_tx_event(callback: fn()) {
    twi_on_peripheral_transmit.write(callback);
}

pub fn twi_reply(ack: bool) {
    use TWCR::*;
    if ack {
        unsafe { TWCR::write( TWEN.bv() | TWIE.bv() | TWINT.bv() | TWEA.bv() ) }
    } else {
        unsafe { TWCR::write( TWEN.bv() | TWIE.bv() | TWINT.bv() ) }
    }
}

pub fn twi_stop() {
    // Send stop condition
    use TWCR::*;
    unsafe { TWCR::write( TWEN.bv() | TWIE.bv() | TWINT.bv() | TWEA.bv()  | TWSTO.bv() ) }

    // Wait for stop condition to be executed on bus
    // TWINT is not set after a stop condition!
    // We can't use micros() from an ISR, since micros relies on interrutps, so approximate the timeout with cycle-counted delays
    const US_PER_LOOP: u32 = 8;
    let mut counter = (twi_timeout_us.read() + US_PER_LOOP - 1) / US_PER_LOOP; // Round up
    while unsafe { TWCR::TWSTO.read_bit() } {
        if twi_timeout_us.read() > 0 {
            if counter > 0 {
                delay_micros(US_PER_LOOP as u64);
                counter -= 1;
            } else {
                twi_handle_timeout(twi_do_reset_on_timeout.read());
                return
            }
        }
    }

    twi_state.write(State::READY);
}

pub fn twi_release_bus() {
    use TWCR::*;
    unsafe { TWCR::write( TWEN.bv() | TWIE.bv() | TWEA.bv() | TWINT.bv() ) };
}

pub fn twi_set_timeout_us(timeout: u32, reset_with_timeout: bool) {
    twi_timed_out_flag.write(false);
    twi_timeout_us.write(timeout);
    twi_do_reset_on_timeout.write(reset_with_timeout);
}

pub fn twi_handle_timeout(reset: bool) {
    unsafe {
        twi_timed_out_flag.write(true);
        
        if reset {
            let previous_TWBR = TWBR::read();
            let previous_TWAR = TWAR::read();

            twi_disable();
            twi_init();

            TWBR::write(previous_TWBR);
            TWAR::write(previous_TWAR);
        }
    }
}

pub fn twi_manage_timeout_flag(clear_flag: bool) -> bool {
    let flag = twi_timed_out_flag.read() ;
    if clear_flag {
        twi_timed_out_flag.write(false);
    }
    flag
}

#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_24"]
pub unsafe extern "avr-interrupt" fn TWI() {
    use Flags::*;
    if let Some(status) = Flags::from_flag(TWSR::read()) {
        // Handle fallthroughs first.
        // Were handled with __attribute__ ((fallthrough)); in original library.
        match status {
            TW_CR_DATA_ACK => {// Data received, ACK sent
                // Put byte into buffer
                twi_master_buffer.as_mut(|buf| {
                    buf.inner[buf.index];
                    buf.index += 1;
                });
            },
            TW_PT_SLA_ACK | TW_PT_ARB_LOST_SLA_ACK => {
                // Enter peripheral tranmitter mode
                twi_state.write(State::PTX);
                // Clear the tx buffer to ready it for writes.
                twi_tx_buffer.as_mut(|buf| buf.reset());
                // Request for tx_buffer to be filled.
                // Note: User must call twi_transmit() to do this.
                twi_on_peripheral_transmit.read()();
                // If they didn't change buffer & length, initialize it.
                twi_tx_buffer.as_mut(|buf| {
                    if buf.length == 0 {
                        buf.length = 1;
                        buf.inner[0] = 0x00;
                    }
                });
            },
            _ => {},
        }

        match status {
            TW_START | TW_REP_START => {
                TWDR::write(twi_slarw.read());
                twi_reply(true);
            },
            TW_CT_SLA_ACK | TW_CT_DATA_ACK => {
                // If there is data to send, send it, otherwise stop
                if twi_master_buffer.as_deref(|buf| buf.index < buf.length) {
                    twi_master_buffer.as_mut(|buf| {
                        TWDR::write(buf.inner[buf.index]);
                        buf.index += 1;
                    });
                    twi_reply(true);
                } else {
                    if twi_send_stop.read() {
                        twi_stop();
                    } else {
                        twi_in_rep_start.write(true); // We're going send the START
                        // Don't enable the interrupt. We'll generate the start, but we
                        // avoid handling the interrupt until we're in the next transaction,
                        // at the point where we would normally issue the start.
                        use TWCR::*;
                        TWCR::write( TWINT.bv() | TWSTA.bv() | TWEN.bv() );
                        twi_state.write(State::READY);
                    }
                }
            },
            TW_CT_SLA_NACK => { // Address send, NACK received
                twi_error.write(TW_CT_SLA_NACK as u8);
                twi_stop();
            },
            TW_CT_DATA_NACK => { // Data send, NACK received
                twi_error.write(TW_CT_DATA_NACK as u8);
                twi_stop();
            },
            TW_CT_CR_ARB_LOST => { // Lost bus arbitration
                twi_error.write(TW_CT_CR_ARB_LOST as u8);
                twi_release_bus();
            },

            // Controller Receiver
            TW_CR_DATA_ACK | TW_CR_SLA_ACK => { // Address/data sent, ACK received
                // ACK if more bytes are expected, otherwise NACK
                twi_reply(twi_master_buffer.as_deref(|buf| buf.index < buf.length))
            },
            TW_CR_DATA_NACK => { // Data received, NACK sent
                twi_master_buffer.as_mut(|buf| {
                    buf.inner[buf.index] = TWDR::read();
                    buf.index += 1;
                });
                if twi_send_stop.read() {
                    twi_stop();
                } else {
                    twi_in_rep_start.write(true); // We're going send the START
                    // Don't enable the interrupt. We'll generate the start, but we
                    // avoid handling the interrupt until we're in the next transaction,
                    // at the point where we would normally issue the start.
                    use TWCR::*;
                    TWCR::write( TWINT.bv() | TWSTA.bv() | TWEN.bv() );
                    twi_state.write(State::READY);
                }
            },
            TW_CR_SLA_NACK => { // Address sent, NACK received
                twi_stop();
            },
            // TW_CR_ARB_LOST handled by TW_CT_ARB_LOST arm

            // Peripheral Receiver
            TW_PR_SLA_ACK | TW_PR_GCALL_ACK | TW_PR_ARB_LOST_SLA_ACK | TW_PR_ARB_LOST_GCALL_ACK => {
                // Enter peripheral receiver mode
                twi_state.write(State::PRX);
                //Indicate that rx buffer can be overwritten and ACK
                twi_rx_buffer.as_mut(|buf| buf.reset());
                twi_reply(true);
            },
            TW_PR_DATA_ACK | TW_PR_GCALL_DATA_ACK => { // Data received generallty, returned ACK
                // If there is still room in the rx buffer
                let available = twi_rx_buffer.as_mut(|buf| {
                    let available = buf.index < TWI_BUFFER_LENGTH;
                    if available {
                        buf.inner[buf.index] = TWDR::read();
                    }
                    available
                });
                twi_reply(available);
            },
            TW_PR_STOP => { // Stop or repeated start condition received
                // ACK future responses and leave peripheral receiver state
                twi_release_bus();
                //Put a null char after data if there's room
                twi_rx_buffer.as_mut(|buf| {
                    if buf.index < TWI_BUFFER_LENGTH {
                        buf.inner[buf.index] = 0x00;
                    }
                    // Callback to user defined callback.
                    twi_on_peripheral_receive.read()(buf.clone(), buf.index);
                    // Since we submit rx buffer to Wire we can reset it.
                    buf.index = 0;
                });
            },
            TW_PR_DATA_NACK | TW_PR_GCALL_DATA_NACK => { // Data received generally, returned NACK
                twi_reply(false);
            },

            // Peripheral Transmitter
            TW_PT_SLA_ACK | TW_PT_ARB_LOST_SLA_ACK => { // Arbitration lost, returned ACK
                // Enter peripheral transmitter mode
                twi_state.write(State::PTX);
                // Ready the tx buffer for iteration
                twi_tx_buffer.as_mut(|buf| buf.reset());
                // Request for tx buffer to be filled
                // Note: User must call twi_transmit(bytes, length) to do this
                twi_on_peripheral_transmit.read()();
                // If they didn't change buffer & length, initialize it.
                twi_tx_buffer.as_mut(|buf| {
                    if buf.length == 0 {
                        buf.length = 1;
                        buf.inner[0] = 0x00;
                    }
                });
            },
            TW_PT_DATA_ACK => { // Byte sent, ACK returned
                twi_tx_buffer.as_mut(|buf| {
                    // Copy data to output register
                    TWDR::write(buf.inner[buf.index]);
                    buf.index += 1;
                    //If there is more to send, ACK, otherwise NACK
                    twi_reply(buf.index < buf.length);
                });
            },
            TW_PT_DATA_NACK | TW_PT_LAST_DATA => { // Recieved NACK indicating that we are done, or ACK after we are done.
                // ACK future responses
                twi_reply(true);
                // Leave peripheral receiver state
                twi_state.write(State::READY);
            },
            TW_BUS_ERROR => {
                twi_error.write(TW_BUS_ERROR as u8);
                twi_stop();
            }
            _ => {}
        }
    }
}
