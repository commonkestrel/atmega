//! Simple implementation of a byte buffer of length 64.
//! 
//! This is an adaptation of the `USART_BUFFER` from [`avr_328p_usart`](https://github.com/johncobb/avr_328p_usart)

/// Byte buffer of length 64
#[derive(Debug, Clone, Copy)]
pub struct Buffer<const SIZE: usize> {
    head: usize,
    tail: usize,
    buffer: [u8; SIZE],
}

impl<const SIZE: usize> Buffer<SIZE> {
    /// Creates a new buffer set to all 0s
    pub const fn new() -> Buffer<SIZE> {
        Buffer {
            head: 0,
            tail: 0,
            buffer: [0; SIZE],
        }
    }

    /// Writes a byte to the head of the buffer.
    /// Does not do anything if the buffer is full.
    pub fn write(&mut self, value: u8) {
        let i = (self.head + 1) % SIZE;

        // if we should be storing the received character into the location
        // just before the tail (meaning that the head would advance to the
        // current location of the tail), we're about to overflow the buffer
        // and so we don't write the character or advance the head.
        if i != self.tail {
            self.buffer[self.head] = value;
            self.head = i;
        }
    }

    /// Returns the total bytes stored in the buffer.
    pub fn len(&self) -> u8 {
        ((SIZE + self.head - self.tail) % SIZE) as u8
    }

    /// Reads the byte at the front of the buffer.
    /// Returns `None` if there is no data stored in the buffer.
    pub fn read(&mut self) -> Option<u8> {
        // if the head isn't ahead of the tail, we don't have any characters
         if self.head == self.tail {
            return None;
         }

         let value = self.buffer[self.tail];
         self.tail = (self.tail + 1) % SIZE;
         Some(value)
    }

    /// Sets all bytes in the buffer to 0.
    pub fn clear(&mut self) {
        self.buffer = [0; SIZE];
        self.head = 0;
        self.tail = 0;
    }
}
