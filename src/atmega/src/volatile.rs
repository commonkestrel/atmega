use crate::interrupt;
use core::cell::UnsafeCell;
use core::ptr::{ write_volatile, read_volatile };

/// A dead simple safe(ish) mutable global variable.
/// Used for communicating to and from interrupts.
pub struct Volatile<T: Copy>(UnsafeCell<T>);

impl<T: Copy> Volatile<T> {
    pub const fn new(value: T) -> Volatile<T> {
        Volatile(UnsafeCell::new(value))
    }

    pub fn read(&self) -> T {
        interrupt::without(|| unsafe { read_volatile(self.0.get()) })
    }

    pub fn write(&self, value: T) {
        interrupt::without(|| unsafe { write_volatile(self.0.get(), value); });
    }

    /// Reads the value and writes the output of the operation.
    pub fn operate<F: Fn(T) -> T>(&self, operator: F) {
        interrupt::without(|| unsafe { write_volatile(self.0.get(), operator(read_volatile(self.0.get()))) });
    }
}

unsafe impl<T: Copy + Send> Send for Volatile<T> {}
unsafe impl<T: Copy + Send + Sync> Sync for Volatile<T> {}
