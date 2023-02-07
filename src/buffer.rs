const BUFFER_SIZE: usize = 64;

/// Simple implementation of a value buffer of length 64.
/// This is an adaptation of the USART_BUFFER from [johncobb/avr_328p_usart](https://github.com/johncobb/avr_328p_usart)
#[derive(Debug, Clone, Copy)]
pub struct Buffer {
    head: usize,
    tail: usize,
    buffer: [u8; BUFFER_SIZE],
}

impl Buffer {
    pub const fn new() -> Buffer {
        Buffer {
            head: 0,
            tail: 0,
            buffer: [0; BUFFER_SIZE],
        }
    }

    pub fn write(&mut self, value: u8) {
        let i = (self.head + 1) % BUFFER_SIZE;

        // if we should be storing the received character into the location
        // just before the tail (meaning that the head would advance to the
        // current location of the tail), we're about to overflow the buffer
        // and so we don't write the character or advance the head.
        if i != self.tail {
            self.buffer[self.head] = value;
            self.head = i;
        }
    }

    pub fn available(&self) -> u8 {
        ((BUFFER_SIZE + self.head - self.tail) % BUFFER_SIZE) as u8
    }

    pub fn read(&mut self) -> Option<u8> {
        // if the head isn't ahead of the tail, we don't have any characters
         if self.head == self.tail {
            return None;
         }

         let value = self.buffer[self.tail];
         self.tail = (self.tail + 1) % BUFFER_SIZE;
         Some(value)
    }

    pub fn clear(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.head = 0;
        self.tail = 0;
    }
}
