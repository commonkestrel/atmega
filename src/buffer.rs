//! Simple implementation of a byte buffer with a variable length.
//! 
//! This is an adaptation of the `USART_BUFFER` from [`avr_328p_usart`](https://github.com/johncobb/avr_328p_usart)

use core::mem::MaybeUninit;

/// Byte buffer of variable length.
/// Default length is 32.
#[derive(Debug, Clone, Copy)]
pub struct Buffer<T: Copy, const SIZE: usize = 32> {
    head: usize,
    tail: usize,
    buffer: [MaybeUninit<T>; SIZE],
}

impl<T: Copy, const SIZE: usize> Buffer<T, SIZE> {
    /// Maximum size of the buffer
    pub const MAX_SIZE: usize = SIZE;

    /// Creates a new buffer set to all 0s
    #[inline(always)]
    pub const fn new() -> Self {
        Buffer {
            head: 0,
            tail: 0,
            buffer: MaybeUninit::uninit_array(),
        }
    }

    /// Creates a blank buffer and writes the contents of the passed slice into the buffer.
    /// 
    /// # Panics
    /// Will panic if the length of the slice is larger than the maximum size of the buffer.
    pub fn copy_from_slice(data: &[T]) -> Self {
        if data.len() > SIZE {
            panic!("Slice larger than Buffer MAX_SIZE");
        }
        let mut new = Self::new();
        for byte in data {
            new.write(*byte);
        }

        new
    }

    /// Writes a byte to the head of the buffer.
    /// Does not do anything if the buffer is full.
    pub fn write(&mut self, value: T) {
        let i = (self.head + 1) % SIZE;

        // if we should be storing the received character into the location
        // just before the tail (meaning that the head would advance to the
        // current location of the tail), we're about to overflow the buffer
        // and so we don't write the character or advance the head.
        if i != self.tail {
            self.buffer[self.head].write(value);
            self.head = i;
        }
    }

    /// Returns the total bytes stored in the buffer.
    pub fn len(&self) -> usize {
        (SIZE + self.head - self.tail) % SIZE
    }

    /// Returns the available space left in the buffer
    /// before writes are ignored.
    pub fn available(&self) -> usize {
        SIZE-1 - self.len()
    }

    /// Reads the byte at the front of the buffer.
    /// Returns `None` if there is no data stored in the buffer.
    pub fn read(&mut self) -> Option<T> {
        // if the head isn't ahead of the tail, we don't have any characters
         if self.head == self.tail {
            return None;
         }

         let value = unsafe { self.buffer[self.tail].assume_init() };
         self.tail = (self.tail + 1) % SIZE;
         Some(value)
    }

    /// Returns `true` if the buffer contains no bytes.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    /// Returns `true` if the buffer is at it's maximum capacity, meaning any further writes will be ignored.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len() >= Self::MAX_SIZE
    }

    /// Clears the buffer
    #[inline(always)]
    pub fn clear(&mut self) {
        self.head = self.tail;
    }
}

impl<T: Copy, const SIZE: usize> core::ops::Index<usize> for Buffer<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("Index out of range! Index of {} into Buffer of length {}.", index, self.len());
        }
        let i = self.tail.wrapping_add(index) % SIZE;
        unsafe { self.buffer[i].assume_init_ref() }
    }
}

impl<T: Copy, const SIZE: usize> core::ops::IndexMut<usize> for Buffer<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len() {
            panic!("Index out of range! Index of {} into Buffer of length {}.", index, self.len());
        }
        let i = self.tail.wrapping_add(index) % SIZE;
        unsafe { self.buffer[i].assume_init_mut() }
    }
}

impl<T: Copy, const SIZE: usize> Iterator for Buffer<T, SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.read()
    }
}
