//! Library for driving Adafruit NeoPixel addressable LEDs,
//! FLORA RGB Smart Pixels and compatible devices -- WS2811, WS2812, WS2812B,
//! SK6812, etc.
//! 
//! Adapted from the official [NeoPixel library](https://github.com/adafruit/Adafruit_NeoPixel) created by Adafruit

use crate::libraries::color::Color;
use crate::wiring::{ Pin, PinMode, pin_mode, digital_write };

// RGB NeoPixel permutations; white and red offsets are always same
// Offset:        W          R          G          B
const NEO_RGB: u8 =  ((0 << 6) | (0 << 4) | (1 << 2) | (2)); ///< Transmit as R,G,B
const NEO_RBG: u8 =  ((0 << 6) | (0 << 4) | (2 << 2) | (1)); ///< Transmit as R,B,G
const NEO_GRB: u8 =  ((1 << 6) | (1 << 4) | (0 << 2) | (2)); ///< Transmit as G,R,B
const NEO_GBR: u8 =  ((2 << 6) | (2 << 4) | (0 << 2) | (1)); ///< Transmit as G,B,R
const NEO_BRG: u8 =  ((1 << 6) | (1 << 4) | (2 << 2) | (0)); ///< Transmit as B,R,G
const NEO_BGR: u8 =  ((2 << 6) | (2 << 4) | (1 << 2) | (0)); ///< Transmit as B,G,R

// RGBW NeoPixel permutations; all 4 offsets are distinct
// Offset:         W          R          G          B
const NEO_WRGB: u8 = ((0 << 6) | (1 << 4) | (2 << 2) | (3)); ///< Transmit as W,R,G,B
const NEO_WRBG: u8 = ((0 << 6) | (1 << 4) | (3 << 2) | (2)); ///< Transmit as W,R,B,G
const NEO_WGRB: u8 = ((0 << 6) | (2 << 4) | (1 << 2) | (3)); ///< Transmit as W,G,R,B
const NEO_WGBR: u8 = ((0 << 6) | (3 << 4) | (1 << 2) | (2)); ///< Transmit as W,G,B,R
const NEO_WBRG: u8 = ((0 << 6) | (2 << 4) | (3 << 2) | (1)); ///< Transmit as W,B,R,G
const NEO_WBGR: u8 = ((0 << 6) | (3 << 4) | (2 << 2) | (1)); ///< Transmit as W,B,G,R

const NEO_RWGB: u8 = ((1 << 6) | (0 << 4) | (2 << 2) | (3)); ///< Transmit as R,W,G,B
const NEO_RWBG: u8 = ((1 << 6) | (0 << 4) | (3 << 2) | (2)); ///< Transmit as R,W,B,G
const NEO_RGWB: u8 = ((2 << 6) | (0 << 4) | (1 << 2) | (3)); ///< Transmit as R,G,W,B
const NEO_RGBW: u8 = ((3 << 6) | (0 << 4) | (1 << 2) | (2)); ///< Transmit as R,G,B,W
const NEO_RBWG: u8 = ((2 << 6) | (0 << 4) | (3 << 2) | (1)); ///< Transmit as R,B,W,G
const NEO_RBGW: u8 = ((3 << 6) | (0 << 4) | (2 << 2) | (1)); ///< Transmit as R,B,G,W

const NEO_GWRB: u8 = ((1 << 6) | (2 << 4) | (0 << 2) | (3)); ///< Transmit as G,W,R,B
const NEO_GWBR: u8 = ((1 << 6) | (3 << 4) | (0 << 2) | (2)); ///< Transmit as G,W,B,R
const NEO_GRWB: u8 = ((2 << 6) | (1 << 4) | (0 << 2) | (3)); ///< Transmit as G,R,W,B
const NEO_GRBW: u8 = ((3 << 6) | (1 << 4) | (0 << 2) | (2)); ///< Transmit as G,R,B,W
const NEO_GBWR: u8 = ((2 << 6) | (3 << 4) | (0 << 2) | (1)); ///< Transmit as G,B,W,R
const NEO_GBRW: u8 = ((3 << 6) | (2 << 4) | (0 << 2) | (1)); ///< Transmit as G,B,R,W

const NEO_BWRG: u8 = ((1 << 6) | (2 << 4) | (3 << 2) | (0)); ///< Transmit as B,W,R,G
const NEO_BWGR: u8 = ((1 << 6) | (3 << 4) | (2 << 2) | (0)); ///< Transmit as B,W,G,R
const NEO_BRWG: u8 = ((2 << 6) | (1 << 4) | (3 << 2) | (0)); ///< Transmit as B,R,W,G
const NEO_BRGW: u8 = ((3 << 6) | (1 << 4) | (2 << 2) | (0)); ///< Transmit as B,R,G,W
const NEO_BGWR: u8 = ((2 << 6) | (3 << 4) | (1 << 2) | (0)); ///< Transmit as B,G,W,R
const NEO_BGRW: u8 = ((3 << 6) | (2 << 4) | (1 << 2) | (0)); ///< Transmit as B,G,R,W

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

/// 8-bit gamma-correction table.
const GAMMA_TABLE: [u8; 256] = [
    0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,   0,   1,   1,   1,   1,   1,   1,
    1,   1,   1,   1,   1,   1,   2,   2,   2,   2,   2,   2,   2,   2,   3,
    3,   3,   3,   3,   3,   4,   4,   4,   4,   5,   5,   5,   5,   5,   6,
    6,   6,   6,   7,   7,   7,   8,   8,   8,   9,   9,   9,   10,  10,  10,
    11,  11,  11,  12,  12,  13,  13,  13,  14,  14,  15,  15,  16,  16,  17,
    17,  18,  18,  19,  19,  20,  20,  21,  21,  22,  22,  23,  24,  24,  25,
    25,  26,  27,  27,  28,  29,  29,  30,  31,  31,  32,  33,  34,  34,  35,
    36,  37,  38,  38,  39,  40,  41,  42,  42,  43,  44,  45,  46,  47,  48,
    49,  50,  51,  52,  53,  54,  55,  56,  57,  58,  59,  60,  61,  62,  63,
    64,  65,  66,  68,  69,  70,  71,  72,  73,  75,  76,  77,  78,  80,  81,
    82,  84,  85,  86,  88,  89,  90,  92,  93,  94,  96,  97,  99,  100, 102,
    103, 105, 106, 108, 109, 111, 112, 114, 115, 117, 119, 120, 122, 124, 125,
    127, 129, 130, 132, 134, 136, 137, 139, 141, 143, 145, 146, 148, 150, 152,
    154, 156, 158, 160, 162, 164, 166, 168, 170, 172, 174, 176, 178, 180, 182,
    184, 186, 188, 191, 193, 195, 197, 199, 202, 204, 206, 209, 211, 213, 215,
    218, 220, 223, 225, 227, 230, 232, 235, 237, 240, 242, 245, 247, 250, 252, 255,
];

pub struct Neopixel<C, const LENGTH: usize> 
where C: Color + Copy
{
    begun: bool,
    pixels: [C; LENGTH],
    pin: Pin,
}

impl<C, const LENGTH: usize> Neopixel<C, LENGTH> 
where C: Color + Copy
{
    /// Creates a new instance of a Neopixel array.
    pub fn new(pin: Pin, initializer: C) -> Neopixel<C, LENGTH> {
        Neopixel {
            begun: false,
            pixels: [initializer; LENGTH],
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
