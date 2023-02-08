//! A dead simple safe(ish) way to communicate to and from interrupts with global statics.

use crate::interrupt;
use core::cell::UnsafeCell;
use core::ptr::{ write_volatile, read_volatile };

/// A dead simple safe(ish) mutable global variable.
/// Used for communicating to and from interrupts.
/// 
/// ## Safety
/// This is safe since the atmega328p is a strictly single threaded processor.
pub struct Volatile<T: Copy>(UnsafeCell<T>);

impl<T: Copy> Volatile<T> {
    pub const fn new(value: T) -> Volatile<T> {
        Volatile(UnsafeCell::new(value))
    }
    
    /// Reads the stored data
    pub fn read(&self) -> T {
        interrupt::without(|| unsafe { read_volatile(self.0.get()) })
    }
    
    /// Overwrites the stored data
    pub fn write(&self, value: T) {
        interrupt::without(|| unsafe { write_volatile(self.0.get(), value); });
    }

    /// Reads the value and writes the output of the operation.
    pub fn operate<F: Fn(T) -> T>(&self, operator: F) {
        interrupt::without(|| unsafe { write_volatile(self.0.get(), operator(read_volatile(self.0.get()))) });
    }
    
    /// Passes the data of type `T` and passes it into the given function as `&mut T`
    /// Allows the changing of the inner data without reading and overwriting all contents.
    pub fn as_mut<F, R>(&self, operation: F) -> R
    where F: Fn(&mut T) -> R
    {
        interrupt::without(|| {
            unsafe { operation(&mut *self.0.get()) }
        })
    }
}

unsafe impl<T: Copy + Send> Send for Volatile<T> {}
unsafe impl<T: Copy + Send + Sync> Sync for Volatile<T> {}
