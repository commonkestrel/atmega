//! Easy 8-bit operations

use core::ptr::{ read_volatile, write_volatile };

/// Flips the bit at the given offset.
/// Equivalent to a `not` operation.
#[inline(always)]
pub fn toggle(byte: u8, bit: u8) -> u8 {
    byte ^ (1 << bit)
}

/// Sets the bit at the given offset, changing to a `1`
#[inline(always)]
pub fn set(byte: u8, bit: u8) -> u8 {
    byte | (1 << bit)
}


/// Clears the bit at the given offset, changing to a `0`
#[inline(always)]
pub fn clear(byte: u8, bit: u8) -> u8 {
    byte & !(1 << bit)
}


/// Changes the bit at the given offset to the given value.
#[inline(always)]
pub fn set_value(byte: u8, bit: u8, value: bool) -> u8 {
    if value {
        set(byte, bit)
    } else {
        clear(byte, bit)
    }
}

#[inline(always)]
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
/// write_volatile(ADDR, 0b0000_1111);
/// registers::operate(ADDR, |val| !val);
/// assert_eq!(read_volatile(ADDR), 0b1111_0000);
/// ```
#[inline(always)]
pub unsafe fn operate<F: Fn(u8) -> u8>(address: *mut u8, operator: F) {
    let current = read_volatile(address);
    write_volatile(address, operator(current));
}

/// Converts Binary Coded Decimal (BCD) to decimal.
#[inline(always)]
pub fn from_bcd(num: u8) -> u8 {
    (num/16 * 10) + (num % 16)
}

pub fn from_dec(num: u8) -> u8 {
    (num/10 * 16) + (num % 10)
}
