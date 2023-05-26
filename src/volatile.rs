//! A dead simple safe(ish) way to communicate to and from interrupts with global statics.

use crate::interrupts::{ self, State };
use core::cell::UnsafeCell;
use core::ptr::{ write_volatile, read_volatile };

/// A dead simple safe(ish) mutable global variable.
/// Used for communicating to and from interrupts.
/// 
/// ## Safety
/// The ATmega328p is a strictly single-threaded processor. 
/// Interrupts are disabled during all operations.
pub struct Volatile<T: Copy>(UnsafeCell<T>);

impl<T: Copy> Volatile<T> {
    /// Creates a new `Volatile` that contains the given data.
    #[inline(always)]
    pub const fn new(value: T) -> Volatile<T> {
        Volatile(UnsafeCell::new(value))
    }
    
    /// Reads the stored data.
    #[inline(always)]
    pub fn read(&self) -> T {
        interrupts::without(State::Restore, || unsafe { read_volatile(self.0.get()) })
    }
    
    /// Overwrites the stored data.
    #[inline(always)]
    pub fn write(&self, value: T) {
        interrupts::without(State::Restore, || unsafe { write_volatile(self.0.get(), value); });
    }

    /// Passes the current value into the function and stores the return value.
    pub fn operate<F: FnMut(T) -> T>(&self, mut operator: F) {
        interrupts::without(State::Restore, || unsafe { write_volatile(self.0.get(), operator(read_volatile(self.0.get()))) });
    }

    /// Consumes the wrapper and returns the data contained
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }

    /// Passes the data of type `T` and passes it into the given function as `&T`.
    /// Allows for reading of inner data without copying or cloning the inner data to a new area in memory.
    /// 
    /// Disables interrupts during call.
    pub fn as_deref<F, R>(&self, mut operation: F) -> R
    where F: FnMut(&T) -> R
    {
        interrupts::without(State::Restore, || {
            unsafe { operation(&*self.0.get()) }
        })
    }
    
    /// Passes the data of type `T` and passes it into the given function as `&mut T`.
    /// Allows the changing of the inner data without reading and overwriting all contents.
    /// 
    /// Disables interrupts during call.
    pub fn as_mut<F, R>(&self, mut operation: F) -> R
    where F: FnMut(&mut T) -> R
    {
        interrupts::without(State::Restore, || {
            unsafe { operation(&mut *self.0.get()) }
        })
    }
}

impl<T: Copy + Default> Default for Volatile<T> {
    fn default() -> Self {
        Volatile::new(T::default())
    }
}

unsafe impl<T: Copy + Send> Send for Volatile<T> {}
unsafe impl<T: Copy + Send + Sync> Sync for Volatile<T> {}
