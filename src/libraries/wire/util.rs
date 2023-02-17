//! Implementation of the I2C protocol via the Arduino [Wire](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire) library
//! 
//! Implementation and some documentation taken from the official [source code](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire/src)
#![allow(non_snake_case, non_upper_case_globals, dead_code, non_camel_case_types)]

use crate::registers::{ Register, TWSR, TWCR, TWBR, TWAR, TWDR };
use crate::wiring::{ digital_write, Pin };
use crate::constants::CPU_FREQUENCY;
use crate::prelude::delay_micros;
use crate::volatile::Volatile;
use crate::buffer::Buffer;
use crate::timing::micros;

#[derive(Clone, Copy, PartialEq)]
enum State {
    READY,
    MRX,
    MTX,
    SRX,
    STX,
}

const TWI_FREQ: u64 = 100_000;

/// SLA+R address
const TW_READ: u8 = 1;
/// SLA+W address
const TW_WRITE: u8 = 0;

const TW_PTATUS_MASK: u8 = 0b1111_1000; // TWS7 | TWS6 | TWS5 | TWS4 | TWS3
/// Start contidion transmitted

// TW_CT_xxx : Controller transmitter
// TW_CR_xxx : Controller receiver
// TW_PT_xxx : Peripheral transmitter
// TW_PR_xxx : Peripheral receiver

pub enum Flags {
    /// Start condition transmitted
    TW_PTART = 0x08,
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
    /// Arbitration lost in SLA+W/R, data, or NACK
    /// TW_CT_ARB_LOST and TW_CR_ARB_LOST share this flag.
    TW_CT_MR_ARB_LOST = 0x38,
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
    fn from_flag(flag: u8) -> Flags {
        use Flags::*;
        crate::println!("{:X}, {}", flag & TW_PTATUS_MASK, (flag & TW_PTATUS_MASK) == 0x08);
        match flag & TW_PTATUS_MASK {
            0x00 => TW_BUS_ERROR,
            0x08 => TW_PTART,
            0x10 => TW_REP_START,
            0x18 => TW_CT_SLA_ACK,
            0x20 => TW_CT_SLA_NACK,
            0x28 => TW_CT_DATA_ACK,
            0x30 => TW_CT_DATA_NACK,
            0x38 => TW_CT_MR_ARB_LOST,
            0x40 => TW_CR_SLA_ACK,
            0x48 => TW_CR_SLA_NACK,
            0x50 => TW_CR_DATA_ACK,
            0x58 => TW_CR_DATA_NACK,
            0x60 => TW_PR_SLA_ACK,
            0x68 => TW_PR_ARB_LOST_SLA_ACK,
            0x70 => TW_PR_GCALL_ACK,
            0x78 => TW_PR_ARB_LOST_GCALL_ACK,
            0x80 => TW_PR_DATA_ACK,
            0x88 => TW_PR_DATA_NACK,
            0x90 => TW_PR_GCALL_DATA_ACK,
            0x98 => TW_PR_GCALL_DATA_NACK,
            0xA0 => TW_PR_STOP,
            0xA8 => TW_PT_SLA_ACK,
            0xB0 => TW_PT_ARB_LOST_SLA_ACK,
            0xB8 => TW_PT_DATA_ACK,
            0xC0 => TW_PT_DATA_NACK,
            0xC8 => TW_PT_LAST_DATA,
            0xF8 => TW_NO_INFO,
            _ => {unreachable!()},
        }
    }
}

pub const TWI_BUFFER_LENGTH: usize = 32;

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

fn blank_receive(_buf: Buffer<TWI_BUFFER_LENGTH>) {}
static twi_on_peripheral_receive: Volatile<fn(Buffer<TWI_BUFFER_LENGTH>)> = Volatile::new(blank_receive);

static twi_master_buffer: Volatile<Buffer<TWI_BUFFER_LENGTH>> = Volatile::new(Buffer::new());
static twi_tx_buffer: Volatile<Buffer<TWI_BUFFER_LENGTH>> = Volatile::new(Buffer::new());
static twi_rx_buffer: Volatile<Buffer<TWI_BUFFER_LENGTH>> = Volatile::new(Buffer::new());

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

pub fn read_from(address: u8, length: u8, send_stop: bool) -> Result<Buffer<TWI_BUFFER_LENGTH>, ()> {
    // Ensure data will fit into buffer
    if TWI_BUFFER_LENGTH < length as usize {
        return Err(());
    }

    let start_micros = micros();

    unsafe {
        while twi_state.read() != State::READY  {
            if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
                twi_handle_timeout(twi_do_reset_on_timeout.read());
                return Err(());
            }
        }

        twi_state.write(State::MRX);
        twi_send_stop.write(send_stop);

        twi_error.write(0xFF);

        twi_master_buffer.as_mut(|buf| buf.clear());

        twi_slarw.write(TW_READ | (address << 1));

        if twi_in_rep_start.read() {
            twi_in_rep_start.write(false);
            let start_micros = micros();

            while TWCR::TWWC.read_bit() {
                TWDR::write(twi_slarw.read());
                if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
                    twi_handle_timeout(twi_do_reset_on_timeout.read());
                    return Err(());
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
        while twi_state.read() == State::MRX {
            if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
                twi_handle_timeout(twi_do_reset_on_timeout.read());
                return Err(());
            }
        }
        
        let len = twi_master_buffer.read().len().min(length as usize);

        let mut ret: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();
        for i in 0..len {
            ret.write(twi_master_buffer.read()[i]);
        }

        Ok(ret)
    }
}

pub enum WriteError {
    /// Address send, NACK received
    SlaNack = 2,
    /// Data send, NACK received
    DataNack,
    /// Other TWI error
    Other,
    /// Timed out
    Timeout,
}

pub fn write_to(address: u8, data: Buffer<TWI_BUFFER_LENGTH>, wait: bool, send_stop: bool) -> Result<(), WriteError> {
    let start_micros = micros();
    while twi_state.read() != State::READY {
        if twi_timeout_us.read() > 0 && (micros() - start_micros) > twi_timeout_us.read() as u64 {
            twi_handle_timeout(twi_do_reset_on_timeout.read());
            return Err(WriteError::Timeout);
        }
    }

    twi_state.write(State::MTX);
    twi_send_stop.write(send_stop);
    // Reset error state 0xFF.. no error occured)
    twi_error.write(0xFF);

    twi_master_buffer.as_mut(|buf| buf.clear());

    for byte in data {
        twi_master_buffer.as_mut(|buf| buf.write(byte));
    }

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
        // Send start condition
        unsafe { TWCR::write( TWINT.bv() | TWEA.bv() | TWEN.bv() | TWIE.bv() | TWSTA.bv() ); }
    }

    // Wait for write operation to complete
    let start_micros = micros();
    while wait && twi_state.read() == State::MTX {
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

pub enum TransmitStatus {
    TooLarge,
    NotSTX,
    Ok,
}

/// Fills peripheral tx buffer with data.
pub fn twi_transmit<const SIZE: usize>(data: Buffer<SIZE>) -> TransmitStatus {
    // Ensure data will fit into buffer
    let tx_len = twi_tx_buffer.read().len();
    if TWI_BUFFER_LENGTH < (tx_len + data.len()) {
        return TransmitStatus::TooLarge;
    }

    // Ensure we are currently as peripheral transmitter
    if twi_state.read() != State::STX {
        return TransmitStatus::NotSTX;
    }

    // Copy data into tx buffer
    for byte in data {
        twi_tx_buffer.as_mut(|buf| buf.write(byte));
    }

    TransmitStatus::Ok
}

pub fn twi_attach_peripheral_rx_event(callback: fn(Buffer<TWI_BUFFER_LENGTH>)) {
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
    match Flags::from_flag(TWSR::read()) {
        TW_REP_START => {
            TWDR::write(twi_slarw.read());
            twi_reply(true);
        },
        TW_CT_DATA_ACK => {
            // If there is data to send, send it, otherwise stop
            if let Some(data) = twi_master_buffer.as_mut(|buf| buf.read()) {
                TWDR::write(data);
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
        TW_CT_MR_ARB_LOST => { // Lost bus arbitration
            twi_error.write(TW_CT_MR_ARB_LOST as u8);
            twi_release_bus();
        },

        // Master Receiver
        TW_CR_DATA_ACK => { // Data received, ACK sent
            // Put byte into buffer
            twi_master_buffer.as_mut(|buf| buf.write(TWDR::read()));
        },
        TW_CR_SLA_ACK => { // Address sent, ACK reeceived
            // ACK if more bytes are expected, otherwise NACK
            twi_reply(twi_master_buffer.read().len() > 0)
        },
        TW_CR_DATA_NACK => { // Data received, NACK sent
            twi_master_buffer.as_mut(|buf| buf.write(TWDR::read()));
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

        // Slave Receiver
        TW_PR_ARB_LOST_GCALL_ACK => { // Lost arbitration, returned ACK
            // Enter peripheral receiver mode
            twi_state.write(State::SRX);
            //Indicate that rx buffer can be overwritten and ACK
            twi_rx_buffer.as_mut(|buf| buf.clear());
            twi_reply(true);
        },
        TW_PR_GCALL_DATA_ACK => { // Data received generallty, returned ACK
            // If there is still room in the rx buffer
            if !twi_rx_buffer.read().is_full() {
                // Put byte in buffer and ACK
                twi_rx_buffer.as_mut(|buf| buf.write(TWDR::read()));
                twi_reply(true);
            } else {
                // otherwise NACK
                twi_reply(false);
            }
        },
        TW_PR_STOP => { // Stop or repeated start condition received
            // ACK future responses and leave peripheral receiver state
            twi_release_bus();
            //Put a null char after data if there's room
            twi_rx_buffer.as_mut(|buf| buf.write('\0' as u8));
            // Callback to the user defined callback
            twi_on_peripheral_receive.read()(twi_rx_buffer.read());
            // Since we submit rx buffer to "wire" library, we can reset it
            twi_rx_buffer.as_mut(|buf| buf.clear());
        },
        TW_PR_GCALL_DATA_NACK => { // Data received generally, returned NACK
            twi_reply(false);
        },

        // Slave Transmitter
        TW_PT_ARB_LOST_SLA_ACK => { // Arbitration lost, returned ACK
            // Enter peripheral transmitter mode
            twi_state.write(State::STX);
            // Ready the tx buffer for iteration
            twi_tx_buffer.as_mut(|buf| buf.clear());
            // Request for tx buffer to be filled
            // Note: User must call twi_transmit(bytes) to do this
            twi_on_peripheral_transmit.read()();
        },
        TW_PT_DATA_ACK => { // Byte sent, ACK returned
            // Copy data to output register
            if let Some(byte) = twi_tx_buffer.as_mut(|buf| buf.read()) {
                TWDR::write(byte);
            }
            //If there is more to send, ACK, otherwise NACK
            twi_reply(!twi_tx_buffer.read().is_empty());
        },
        TW_PT_LAST_DATA => { // Received ACK, but we are done already!
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
