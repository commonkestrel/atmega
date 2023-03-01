//! Library for driving Adafruit NeoPixel addressable LEDs,
//! FLORA RGB Smart Pixels and compatible devices -- WS2811, WS2812, WS2812B,
//! SK6812, etc.
//! 
//! Adapted from the official [NeoPixel library](https://github.com/adafruit/Adafruit_NeoPixel) created by Adafruit

use crate::libraries::color::Color;
use crate::wiring::{ Pin, PinMode, pin_mode, digital_write };

/// The order of primary colors in the NeoPixel data stream can vary among
/// device types, manufacturers and even different revisions of the same
/// item.  The third parameter to the Adafruit_NeoPixel constructor encodes
/// the per-pixel byte offsets of the red, green and blue primaries (plus
/// white, if present) in the data stream -- the following #defines provide
/// an easier-to-use named version for each permutation. e.g. NEO_GRB
/// indicates a NeoPixel-compatible device expecting three bytes per pixel,
/// with the first byte transmitted containing the green value, second
/// containing red and third containing blue. The in-memory representation
/// of a chain of NeoPixels is the same as the data-stream order; no
/// re-ordering of bytes is required when issuing data to the chain.
/// Most of these values won't exist in real-world devices, but it's done
/// this way so we're ready for it (also, if using the WS2811 driver IC,
/// one might have their pixels set up in any weird permutation).
pub enum Order {
    /// Transmit as R,G,B
    RGB  
    /// Transmit as R,B,G
    RBG  
    /// Transmit as G,R,B
    GRB  
    /// Transmit as G,B,R
    GBR  
    /// Transmit as B,R,G
    BRG  
    /// Transmit as B,G,R
    BGR  
    /// Transmit as W,R,G,B
    WRGB  
    /// Transmit as W,R,G,B
    WRBG  
    /// Transmit as W,G,R,B
    WGRB  
    /// Transmit as W,G,B,R
    WGBR  
    /// Transmit as W,B,R,G
    WBRG  
    /// Transmit as W,B,G,R
    WBGR  
    /// Transmit as R,W,G,B
    RWGB  
    /// Transmit as R,W,B,G
    RWBG  
    /// Transmit as R,G,W,B
    RGWB  
    /// Transmit as R,G,B,W
    RGBW  
    /// Transmit as R,B,W,G
    RBWG  
    /// Transmit as R,B,G,W
    RBGW  
    /// Transmit as G,W,R,B 
    GWRB  
    /// Transmit as G,W,B,R
    GWBR  
    /// Transmit as G,R,W,B
    GRWB  
    /// Transmit as G,R,B,W
    GRBW  
    /// Transmit as G,R,W,R
    GBWR  
    /// Transmit as G,B,R,W
    GBRW  
    /// Transmit as B,W,R,G
    BWRG  
    /// Transmit as B,W,G,R
    BWGR  
    /// Transmit as B,R,W,G
    BRWG  
    /// Transmit as B,R,G,W
    BRGW  
    /// Transmit as B,G,W,R
    BGWR  
    /// Transmit as B,G,R,W
    BGRW  
}

impl Order {
    /// Bits 5,4 of this value are the offset (0-3) from the first byte of a
    /// pixel to the location of the red color byte.  Bits 3,2 are the green
    /// offset and 1,0 are the blue offset.  If it is an RGBW-type device
    /// (supporting a white primary in addition to R,G,B), bits 7,6 are the
    /// offset to the white byte...otherwise, bits 7,6 are set to the same value
    /// as 5,4 (red) to indicate an RGB (not RGBW) device.
    /// i.e. binary representation:
    /// 0bWWRRGGBB for RGBW devices
    /// 0bRRRRGGBB for RGB
    fn byte(&self) {
        use Order::*;
        match self {
            // RGB NeoPixel permutations; white and red offsets are always same
            // Offset:   W          R          G          B
            NEO_RGB => ((0 << 6) | (0 << 4) | (1 << 2) | (2)); ///< Transmit as R,G,B
            NEO_RBG => ((0 << 6) | (0 << 4) | (2 << 2) | (1)); ///< Transmit as R,B,G
            NEO_GRB => ((1 << 6) | (1 << 4) | (0 << 2) | (2)); ///< Transmit as G,R,B
            NEO_GBR => ((2 << 6) | (2 << 4) | (0 << 2) | (1)); ///< Transmit as G,B,R
            NEO_BRG => ((1 << 6) | (1 << 4) | (2 << 2) | (0)); ///< Transmit as B,R,G
            NEO_BGR => ((2 << 6) | (2 << 4) | (1 << 2) | (0)); ///< Transmit as B,G,R

            // RGBW NeoPixel permutations; all 4 offsets are distinct
            // Offset:    W          R          G          B
            NEO_WRGB => ((0 << 6) | (1 << 4) | (2 << 2) | (3)); ///< Transmit as W,R,G,B
            NEO_WRBG => ((0 << 6) | (1 << 4) | (3 << 2) | (2)); ///< Transmit as W,R,B,G
            NEO_WGRB => ((0 << 6) | (2 << 4) | (1 << 2) | (3)); ///< Transmit as W,G,R,B
            NEO_WGBR => ((0 << 6) | (3 << 4) | (1 << 2) | (2)); ///< Transmit as W,G,B,R
            NEO_WBRG => ((0 << 6) | (2 << 4) | (3 << 2) | (1)); ///< Transmit as W,B,R,G
            NEO_WBGR => ((0 << 6) | (3 << 4) | (2 << 2) | (1)); ///< Transmit as W,B,G,R

            NEO_RWGB => ((1 << 6) | (0 << 4) | (2 << 2) | (3)); ///< Transmit as R,W,G,B
            NEO_RWBG => ((1 << 6) | (0 << 4) | (3 << 2) | (2)); ///< Transmit as R,W,B,G
            NEO_RGWB => ((2 << 6) | (0 << 4) | (1 << 2) | (3)); ///< Transmit as R,G,W,B
            NEO_RGBW => ((3 << 6) | (0 << 4) | (1 << 2) | (2)); ///< Transmit as R,G,B,W
            NEO_RBWG => ((2 << 6) | (0 << 4) | (3 << 2) | (1)); ///< Transmit as R,B,W,G
            NEO_RBGW => ((3 << 6) | (0 << 4) | (2 << 2) | (1)); ///< Transmit as R,B,G,W

            NEO_GWRB => ((1 << 6) | (2 << 4) | (0 << 2) | (3)); ///< Transmit as G,W,R,B
            NEO_GWBR => ((1 << 6) | (3 << 4) | (0 << 2) | (2)); ///< Transmit as G,W,B,R
            NEO_GRWB => ((2 << 6) | (1 << 4) | (0 << 2) | (3)); ///< Transmit as G,R,W,B
            NEO_GRBW => ((3 << 6) | (1 << 4) | (0 << 2) | (2)); ///< Transmit as G,R,B,W
            NEO_GBWR => ((2 << 6) | (3 << 4) | (0 << 2) | (1)); ///< Transmit as G,B,W,R
            NEO_GBRW => ((3 << 6) | (2 << 4) | (0 << 2) | (1)); ///< Transmit as G,B,R,W

            NEO_BWRG => ((1 << 6) | (2 << 4) | (3 << 2) | (0)); ///< Transmit as B,W,R,G
            NEO_BWGR => ((1 << 6) | (3 << 4) | (2 << 2) | (0)); ///< Transmit as B,W,G,R
            NEO_BRWG => ((2 << 6) | (1 << 4) | (3 << 2) | (0)); ///< Transmit as B,R,W,G
            NEO_BRGW => ((3 << 6) | (1 << 4) | (2 << 2) | (0)); ///< Transmit as B,R,G,W
            NEO_BGWR => ((2 << 6) | (3 << 4) | (1 << 2) | (0)); ///< Transmit as B,G,W,R
            NEO_BGRW => ((3 << 6) | (2 << 4) | (1 << 2) | (0)); ///< Transmit as B,G,R,W
        }
    }
}

/// 8-bit unsigned sine wave table (0-255).
const SINE_TABLE: [u8; 256]  = [
    128, 131, 134, 137, 140, 143, 146, 149, 152, 155, 158, 162, 165, 167, 170,
    173, 176, 179, 182, 185, 188, 190, 193, 196, 198, 201, 203, 206, 208, 211,
    213, 215, 218, 220, 222, 224, 226, 228, 230, 232, 234, 235, 237, 238, 240,
    241, 243, 244, 245, 246, 248, 249, 250, 250, 251, 252, 253, 253, 254, 254,
    254, 255, 255, 255, 255, 255, 255, 255, 254, 254, 254, 253, 253, 252, 251,
    250, 250, 249, 248, 246, 245, 244, 243, 241, 240, 238, 237, 235, 234, 232,
    230, 228, 226, 224, 222, 220, 218, 215, 213, 211, 208, 206, 203, 201, 198,
    196, 193, 190, 188, 185, 182, 179, 176, 173, 170, 167, 165, 162, 158, 155,
    152, 149, 146, 143, 140, 137, 134, 131, 128, 124, 121, 118, 115, 112, 109,
    106, 103, 100, 97,  93,  90,  88,  85,  82,  79,  76,  73,  70,  67,  65,
    62,  59,  57,  54,  52,  49,  47,  44,  42,  40,  37,  35,  33,  31,  29,
    27,  25,  23,  21,  20,  18,  17,  15,  14,  12,  11,  10,  9,   7,   6,
    5,   5,   4,   3,   2,   2,   1,   1,   1,   0,   0,   0,   0,   0,   0,
    0,   1,   1,   1,   2,   2,   3,   4,   5,   5,   6,   7,   9,   10,  11,
    12,  14,  15,  17,  18,  20,  21,  23,  25,  27,  29,  31,  33,  35,  37,
    40,  42,  44,  47,  49,  52,  54,  57,  59,  62,  65,  67,  70,  73,  76,
    79,  82,  85,  88,  90,  93,  97,  100, 103, 106, 109, 112, 115, 118, 121, 124,
];

pub struct Neopixel<C, const LENGTH: usize> 
where C: Color + Copy
{
    begun: bool,
    pixels: [C; LENGTH],
    order: Order,
    pin: Pin,
}

impl<C, const LENGTH: usize> Neopixel<C, LENGTH> 
where C: Color + Copy
{
    /// Creates a new instance of a Neopixel array.
    pub fn new(pin: Pin, order: Order, initializer: C) -> Neopixel<C, LENGTH> {
        Neopixel {
            begun: false,
            pixels: [initializer; LENGTH],
            order
            pin
        }
    }

    pub fn begin(&mut self) {
        if self.pin != Pin::D0 {
            pin_mode(self.pin, PinMode::OUTPUT);
            digital_write(self.pin, false);
        }
        self.begun = true;
    }
}
