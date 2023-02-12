//! Implementation of the I2C protocol via the Arduino [Wire](https://github.com/arduino/ArduinoCore-avr/tree/master/libraries/Wire) library
#![allow(non_upper_case_globals, unused_must_use)]

use crate::volatile::Volatile;
use crate::buffer::Buffer;

mod util;
use util::TWI_BUFFER_LENGTH;

static rx_buffer: Volatile<Buffer<TWI_BUFFER_LENGTH>> = Volatile::new(Buffer::new());
static tx_buffer: Volatile<Buffer<TWI_BUFFER_LENGTH>> = Volatile::new(Buffer::new());
static tx_address: Volatile<u8> = Volatile::new(0);
static transmitting: Volatile<bool> = Volatile::new(false);

static user_on_receive: Volatile<Option<fn(usize)>> = Volatile::new(None);
static user_on_request: Volatile<Option<fn()>> = Volatile::new(None);

/// Initialize TWI interface 
pub fn begin() {
    rx_buffer.as_mut(|buf| buf.clear());
    tx_buffer.as_mut(|buf| buf.clear());

    util::twi_init();

    util::twi_attach_slave_tx_event(on_request_service);
    util::twi_attach_slave_rx_event(on_receive_service);
}

/// `begin` and set the TWI slave address
pub fn begin_addr(address: u8) {
    begin();
    util::set_address(address);
}

/// Disable the TWI interface
pub fn end() {
    util::twi_disable();
}

/// Sets the TWI clock frequency
pub fn set_clock(freq: u64) {
    util::set_frequency(freq);
}

/// Sets the TWI timeout
/// 
/// `timeout`: a timeout value in microseconds, if zero then timeout checking is disabled
/// 
/// `reset_with_timeout`: dictates whether the TWI interface should be automatically reset on timeout
/// 
/// This limits the maximum time to wait for the TWI hardware. If more time passes, the bus is assumed
/// to have locked up (e.g. due to noise-induced glitches or faulty slaves) and the transaction is aborted.
/// Optionally, thw TWI hardware is also reset, which can be required to allow subsequent transactions to 
/// succeed in some cases (in particular when noise has made the TWI hardware thinmk there is a second
/// master that has claimed the bus).
/// 
/// When a timeout is triggered, a flag is set that can be queried with `get_wire_timeout_flag()` and is cleared
/// when `clear_wire_timeout_flag()` or `set_wire_timeout_us()` is called.
/// 
/// Note that this timeout can also trigger while waiting for clock stretching or waiting for a second master 
/// to complete its tranaction. So make sure to adapt the timeout to accommodate for those cases if needed.
/// A typical timeout would be 25ms (which is the maximum clock stretching allowed by the SMBus protocol),
/// but (much) shorter values will usually also work.
/// 
/// In the future, a timeout will be enabled by default, so if you require the timeout to be disabled, it is 
/// recommenced that you disable it by default using `set_wire_timeout_us(0)`, even though that is currently
/// the default.
pub fn set_wire_timeout(timeout: u32, reset_with_timeout: bool) {
    util::twi_set_timeout_us(timeout, reset_with_timeout);
}

/// Returns `true` if timeout has occurred since the flag was last cleared.
pub fn get_wire_timeout_flag() {
    util::twi_manage_timeout_flag(false);
}

/// Clears the TWI timeout flag.
pub fn clear_wire_timeout_flag() {
    util::twi_manage_timeout_flag(true);
}

/// Request data from the given address after transmitting to the internal register address given.
pub fn iaddr_request_from(address: u8, quantity: u8, iaddress: u32, addr_size: u8, send_stop: bool) -> Result<(), ()> {
    if addr_size > 0 {
        begin_transmission(address);

        // Write internal register address - most significant byte first
        // The maximum size of internal address is 3 bytes
        for i in (0..addr_size.min(3)).rev() {
            write(((iaddress >> (i*8)) & 0xFF) as u8);
        }
        end_transmission(false);
    }

    request_from(address, quantity, send_stop)
}

/// Request data from the given address
pub fn request_from(address: u8, quantity: u8, send_stop: bool) -> Result<(), ()> {
    let clamped = quantity.min(TWI_BUFFER_LENGTH as u8);

    let read = util::read_from(address, clamped, send_stop)?;
    rx_buffer.as_mut(|buf| {
        buf.clear();
        for byte in read {
            buf.write(byte);
        }
    });

    Ok(())
}

/// Begin transmitting to the given slave address.
pub fn begin_transmission(address: u8) {
    // Indicate that we are transmitting
    transmitting.write(true);
    // Set address of targeted slave
    tx_address.write(address);
    // Reset tx_buffer
    tx_buffer.as_mut(|buf| buf.clear());
}

/// Originally, `end transmission` was an `fn()` function.
/// It has been modified to take one parameter indicating
/// whether or not a STOP should be performed on the bus.
/// Calling `end_transmission(false)` allows a sketch to
/// perform a repeated start.
/// 
/// WARNING: Nothing in the library keeps track of whether
/// the bus tenure has been properly ended with a STOP. It
/// is very possible to leave the bus in a hung state if
/// no call to `end_transmission(true)` is made. Some I2C
/// devices will behave oddly if they do not see a STOP.
pub fn end_transmission(send_stop: bool) -> Result<(), util::WriteError> {
    // Transmit buffer (blocking)
    let ret = util::write_to(tx_address.read(), tx_buffer.read(), true, send_stop);
    // Reset tx buffer
    tx_buffer.as_mut(|buf| buf.clear());
    // Indicate that we are done transmitting
    transmitting.write(false);

    ret
}

/// Must be called in `slave tx event callback` or after `begin_transmission(address)`
pub fn write(data: u8) -> Result<(), ()> {
    if transmitting.read() {
    // In master transmitter mode
        // Don't bother if buffer is full
        if tx_buffer.read().is_full() {
            return Err(());
        }
        // put byte in tx buffer
        tx_buffer.as_mut(|buf| buf.write(data));
    } else {
    // In slave send mode
        // Reply to master
        util::twi_transmit(Buffer::<1>::from_slice(&[data]));
    }

    Ok(())
}

/// Must be called in `slave tx event callback` or after `begin_transmission(address)`
pub fn write_all<const SIZE: usize>(data: Buffer<SIZE>) {
    if transmitting.read() {
    // In master transmitter mode
        for byte in data {
            write(byte);
        }
    } else {
    // In slave send mode
        // Reply to master
        util::twi_transmit(data);
    }
}

/// The number of bytes available in the rx buffer.
/// 
/// Must be called in `slave rx event callback` or after `request_from(address, num_bytes)`
pub fn available() -> usize {
    rx_buffer.read().len()
}

/// Reads the byte at the front of the rx buffer if there is any data available;.
pub fn read() -> Option<u8> {
    rx_buffer.read().read()
}

/// Must be called in `slave_rx_event_callback()`
/// or after `request_from(address, num_bytes)`
pub fn peek() -> Option<u8> {
    if rx_buffer.read().is_empty() {
        return None;
    }

    Some(rx_buffer.read()[0])
}

/// `flush()` is unimplemented in the official library, 
/// and has been marked as 'won't fix' in [issue #253](https://github.com/arduino/ArduinoCore-avr/issues/253).
/// Added for parity with the official `Wire` library.
pub fn flush() {
     // XXX: unimplemented
}

fn on_receive_service(bytes_in: Buffer<TWI_BUFFER_LENGTH>) {
    // don't bother if rx buffer is in use by a master request_from() op
    // I know this drops data, but it allows for slight supidity
    // meaning, they may not have read all the master request_from() data yet
    if !rx_buffer.read().is_empty() {
        return;
    }

    if let Some(callback) = user_on_receive.read() {
        // Copy twi rx buffer into local read buffewr
        // This enables new reads to happen in parallel
        for byte in bytes_in {
            rx_buffer.as_mut(|buf| buf.write(byte));
        }
        callback(bytes_in.len());
    }
}

fn on_request_service() {
    // don't bother if user hasn't registered a callback
    if let Some(callback) = user_on_request.read() {
        // Reset tx buffer
        // !!! This will kill any pending pre-master send_to() activity
        tx_buffer.as_mut(|buf| buf.clear());

        callback();
    }
}

/// Sets the callback for when data is received.
/// 
/// The number of bytes received is passed as input.
pub fn on_receive(callback: fn(num_bytes: usize)) {
    user_on_receive.write(Some(callback));
}

/// Sets the callback for when data is requested.
pub fn on_request(callback: fn()) {
    user_on_request.write(Some(callback));
}
