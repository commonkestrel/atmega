//! Implementation of the I2C protocol via the Arduino [Wire](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire) library
//! 
//! Implementation and some documentation taken from the official [source code](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire/src)
#![allow(non_snake_case, non_upper_case_globals)]

use crate::registers::{ Register, TWSR, TWCR, TWBR, TWAR, TWDR };
use crate::wiring::{ digital_write, Pin };
use crate::constants::CPU_FREQUENCY;
use crate::util::delay::_delay_us;
use crate::buffer::Buffer;
use crate::time::micros;

#[derive(PartialEq)]
enum State {
    READY,
    MRX,
    MTX,
    SRC,
    STX,
}

const TWI_FREQ: u64 = 100_000;

/// SLA+R address
const TW_READ: u8 = 1;
/// SLA+W address
const TW_WRITE: u8 = 0;

// TW_MT_xxx : master transmitter
// TW_MR_xxx : master receiver
// TW_ST_xxx : slave transmitter
// TW_SR_xxx : slave receiver

const TW_STATUS_MASK: u8 = 0b1111_1000; // TWS7 | TWS6 | TWS5 | TWS4 | TWS3
/// Start contidion transmitted
const TW_START: u8 = 0x08;
/// Repeated start condition transmitted
const TW_REP_START: u8 = 0x10;
/// SLA+W transmitted, ACK received
const TW_MT_SLA_ACK: u8 = 0x18;
/// SLA+W transmitted, NACK received
const TW_MT_SLA_NACK: u8 = 0x20;
/// Data transmitted, ACK received
const TW_MT_DATA_ACK: u8 = 0x28;
/// Data transmitted, NACK, received
const TW_MT_DATA_NACK: u8 = 0x30;
/// Arbitration lost in SLA+W or data
const TW_MT_ARB_LOST: u8 = 0x38;
/// Arbitration lost in SLA+R or NACK
const TW_MR_ARB_LOST: u8 = 0x38;
/// SLA+R transmitted, ACK received
const TW_MR_SLA_ACK: u8 = 0x40;
/// RLA+R transmitted, NACK received
const TW_MR_SLA_NACK: u8 = 0x48;
/// Data recived, ACK returned
const TW_MR_DATA_ACK: u8 = 0x50;
/// Data recived, NACK returned
const TW_MR_DATA_NACK: u8 = 0x58;
/// SLA+R received, ACK returned
const TW_ST_SLA_ACK: u8 = 0xA8;
/// Artibation lost in SLA+RW, SLA+R recived, ACK returned
const TW_ST_ARB_LOST_SLA_ACK: u8 = 0xB0;
/// Data transmitted, ACK received
const TW_ST_DATA_ACK: u8 = 0xB8;
/// Data transmitted, NACK received
const TW_ST_DATA_NACK: u8 = 0xC0;
/// Last data byte transmitted, ACK received
const TW_ST_LAST_DATA: u8 = 0xC8;
/// SLA+W recieved, ACK returned
const TW_SR_SLA_ACK: u8 = 0x60;
/// Arbitration lost in SLA+RW, SLA+W received, ACK returned
const TW_SR_ARB_LOST_SLA_ACK: u8 = 0x68;
/// General call received, ACK returned
const TW_SR_GCALL_ACK: u8 = 0x70;
/// Arbitration lost in SLA+RW, general call received, ACK returned
const TW_SR_ARB_LOST_GCALL_ACK: u8 = 0x78;
/// Data received, ACK returned
const TW_SR_DATA_ACK: u8 = 0x80;
/// Data received, NACK returned
const TW_SR_DATA_NACK: u8 = 0x88;
/// General call data received, ACK returned
const TW_SR_GCALL_DATA_ACK: u8 = 0x90;
/// General call data received, NACK returned
const TW_SR_GCALL_DATA_NACK: u8 = 0x98;
/// Stop or repeated start condition received while selected
const TW_SR_STOP: u8 = 0xA0;
/// No state information available
const TW_NO_INFO: u8 = 0xF8;
/// Illegal start or stop condition
const TW_BUS_ERROR: u8 = 0x00;

pub const TWI_BUFFER_LENGTH: usize = 32;

static mut twi_state: State = State::READY;
static mut twi_slarw: u8 = 0;
static mut twi_send_stop: bool = true;    // should the transaction end with a stop
static mut twi_in_rep_start: bool = false; // in the middle of a repeated start

// twi_timeout_us > 0 prevents the code from getting stuck in various while loops here
// if twi_timeout_us == 0 then timeout checking is disabled (the previous Wire lib behavior)
// at some point in the future, the default twi_timeout_us value could become 25000
// and twi_do_reset_on_timeout could become true
// to conform to the SMBus standard
// http://smbus.org/specs/SMBus_3_1_20180319.pdf
const TWI_TIMEOUT_US: u32 = 0;
static mut twi_timed_out_flag:bool = false;       // a timeout has been seen
static mut twi_do_reset_on_timeout: bool = false; // reset the TWI registers on timeout

fn blank_transmit() {}
static mut twi_on_slave_transmit: fn() = blank_transmit;

fn blank_receive(buf: Buffer<TWI_BUFFER_LENGTH>) {}
static mut twi_on_slave_receive: fn(Buffer<TWI_BUFFER_LENGTH>) = blank_receive;

static mut twi_master_buffer: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();
static mut twi_tx_buffer: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();
static mut twi_rx_buffer: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();

static mut twi_error: u8 = 0xFF;

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

pub fn read_from(address: u8, length: u8, send_stop: bool) -> Option<Buffer<TWI_BUFFER_LENGTH>> {
    // Ensure data will fit into buffer
    if TWI_BUFFER_LENGTH < length as usize {
        return None;
    }

    let start_micros = micros();

    unsafe {
        while twi_state != State::READY  {
            if TWI_TIMEOUT_US > 0 && (micros() - start_micros) > TWI_TIMEOUT_US as u64 {
                handle_timeout(twi_do_reset_on_timeout);
                return None;
            }
        }

        twi_state = State::MRX;
        twi_send_stop = send_stop;

        twi_error = 0xFF;

        twi_master_buffer.clear();

        twi_slarw = TW_READ | (address << 1);

        if twi_in_rep_start {
            twi_in_rep_start = false;
            let start_micros = micros();

            while TWCR::TWWC.read_bit() {
                TWDR::write(twi_slarw);
                if TWI_TIMEOUT_US > 0 && (micros() - start_micros) > TWI_TIMEOUT_US as u64 {
                    handle_timeout(twi_do_reset_on_timeout);
                    return None;
                }
            }
            // enable INTs, but not START
            TWCR::TWINT.set();
            TWCR::TWEA.set();
            TWCR::TWEN.set();
            TWCR::TWIE.set();
        } else {
            // Sent start condition
            TWCR::TWINT.set();
            TWCR::TWEA.set();
            TWCR::TWEN.set();
            TWCR::TWIE.set();
            TWCR::TWSTA.set();
        }
        
        let start_micros = micros();
        while twi_state == State::MRX {
            if TWI_TIMEOUT_US > 0 && (micros() - start_micros) > TWI_TIMEOUT_US as u64 {
                handle_timeout(twi_do_reset_on_timeout);
                return None;
            }
        }
        
        let len = twi_master_buffer.len().min(length as usize);

        let mut ret: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();
        for i in 0..len {
            ret.write(twi_master_buffer[i]);
        }

        Some(ret)
    }
}

pub fn twi_attach_slave_rx_event(function: fn(Buffer<TWI_BUFFER_LENGTH>)) {
    unsafe { twi_on_slave_receive = function };
}

pub fn twi_attach_slave_tx_event(function: fn()) {
    unsafe { twi_on_slave_transmit = function };
}

pub fn twi_reply(ack: bool) {
    if ack {
        unsafe { TWCR::write( TWCR::TWEN.bit() | TWCR::TWIE.bit() | TWCR::TWINT.bit() | TWCR::TWEA.bit() ) }
    } else {
        unsafe { TWCR::write( TWCR::TWEN.bit() | TWCR::TWIE.bit() | TWCR::TWINT.bit() ) }
    }
}

pub fn twi_stop() {
    // Send stop condition
    unsafe { TWCR::write( TWCR::TWEN.bit() | TWCR::TWIE.bit() | TWCR::TWINT.bit() | TWCR::TWEA.bit()  | TWCR::TWSTO.bit() ) }

    // Wait for stop condition to be executed on bus
    // TWINT is not set after a stop condition!
    // We can't use micros() from an ISR, since micros relies on interrutps, so approximate the timeout with cycle-counted delays
    const US_PER_LOOP: u32 = 8;
    let mut counter = (TWI_TIMEOUT_US + US_PER_LOOP - 1) / US_PER_LOOP; // Round up
    while unsafe { TWCR::TWSTO.read_bit() } {
        if TWI_TIMEOUT_US > 0 {
            if counter > 0 {
                _delay_us(US_PER_LOOP as u64);
                counter -= 1;
            } else {
                handle_timeout(unsafe { twi_do_reset_on_timeout });
                return
            }
        }
    }

    unsafe { twi_state = State::READY; }
}

pub fn handle_timeout(reset: bool) {
    unsafe {
        twi_timed_out_flag = true;
        
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

#[doc(hidden)]
#[inline(always)]
#[allow(non_snake_case)]
#[export_name = "__vector_24"]
pub unsafe extern "avr-interrupt" fn TWI() {
    match TWSR::read() & TW_STATUS_MASK {
        TW_BUS_ERROR => {
            twi_error = TW_BUS_ERROR;
            twi_stop();
        }
        _ => {}
    }
}
