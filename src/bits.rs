//! Easy 8-bit operations

use core::ptr::{ read_volatile, write_volatile };

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