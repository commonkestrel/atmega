//! Library for driving Adafruit NeoPixel addressable LEDs,
//! FLORA RGB Smart Pixels and compatible devices -- WS2811, WS2812, WS2812B,
//! SK6812, etc.
//! 
//! Adapted from the official [NeoPixel library](https://github.com/adafruit/Adafruit_NeoPixel) created by Adafruit

use crate::wiring::{ Pin, PinMode, pin_mode, digital_write };

pub struct Neopixel<'a, const LENGTH: usize> {
    begun: bool,
    pixels: &'a [u8],
    pin: Pin,
    length: u16,
    w_offset: u8,
    r_offset: u8,
}

impl<'a, const LENGTH: usize> Neopixel<'a, LENGTH> {
    pub fn update_length(&mut self, n: u16) {
        let num_bytes = n * if self.w_offset == self.r_offset {3} else {4};
        
        self.length = n;
    }
}
