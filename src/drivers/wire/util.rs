//! Implementation of the I2C protocol via the Arduino [Wire](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire) library
//! 
//! Implementation and some documentation taken from the official [source code](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire/src)

use core::ptr::{ read_volatile, write_volatile };
use crate::registers::{ Register, TWSR };
use crate::buffer::Buffer;

enum State {
    Ready,
    Mrx,
    Mtx,
    Src,
    Stx
}

const TWI_FREQ: u32 = 100000;

const TWI_BUFFER_LENGTH: usize = 32;

static mut twi_state: State = State::Ready;
static mut twi_slarw: u8 = 0;
static mut twi_send_stop: bool = true;    // should the transaction end with a stop
static mut twi_in_rep_start: bool = false; // in the middle of a repeated start

// twi_timeout_us > 0 prevents the code from getting stuck in various while loops here
// if twi_timeout_us == 0 then timeout checking is disabled (the previous Wire lib behavior)
// at some point in the future, the default twi_timeout_us value could become 25000
// and twi_do_reset_on_timeout could become true
// to conform to the SMBus standard
// http://smbus.org/specs/SMBus_3_1_20180319.pdf
const twi_timeout_us: u32 = 0;
static mut twi_timed_out_flag:bool = false;       // a timeout has been seen
static mut twi_do_reset_on_timeout: bool = false; // reset the TWI registers on timeout

static mut twi_master_buffer: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();
static mut twi_tx_buffer: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();
static mut twi_rx_buffer: Buffer<TWI_BUFFER_LENGTH> = Buffer::new();

static mut twi_error: u8 = 0;

/// Readies twi pins and sets twi bitrate
pub fn twi_init() {
    unsafe {
        TWSR::TWPS0.clear();
        TWSR::TWPS1.clear();
    }
}
