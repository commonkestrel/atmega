//! Library for driving Adafruit NeoPixel addressable LEDs,
//! FLORA RGB Smart Pixels and compatible devices -- WS2811, WS2812, WS2812B,
//! SK6812, etc.
//! 
//! Adapted from the official [NeoPixel library](https://github.com/adafruit/Adafruit_NeoPixel) created by Adafruit

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
pub enum Format {
    /// Transmit as R,G,B
    RGB,
    /// Transmit as R,B,G
    RBG,
    /// Transmit as G,R,B
    GRB,
    /// Transmit as G,B,R
    GBR,
    /// Transmit as B,R,G
    BRG,
    /// Transmit as B,G,R
    BGR,
    /// Transmit as W,R,G,B
    WRGB,
    /// Transmit as W,R,G,B
    WRBG,
    /// Transmit as W,G,R,B
    WGRB,
    /// Transmit as W,G,B,R
    WGBR,
    /// Transmit as W,B,R,G
    WBRG,
    /// Transmit as W,B,G,R
    WBGR,
    /// Transmit as R,W,G,B
    RWGB,
    /// Transmit as R,W,B,G
    RWBG,
    /// Transmit as R,G,W,B
    RGWB,
    /// Transmit as R,G,B,W
    RGBW,
    /// Transmit as R,B,W,G
    RBWG,
    /// Transmit as R,B,G,W
    RBGW,
    /// Transmit as G,W,R,B 
    GWRB,
    /// Transmit as G,W,B,R
    GWBR,
    /// Transmit as G,R,W,B
    GRWB,
    /// Transmit as G,R,B,W
    GRBW,
    /// Transmit as G,R,W,R
    GBWR,
    /// Transmit as G,B,R,W
    GBRW,
    /// Transmit as B,W,R,G
    BWRG,
    /// Transmit as B,W,G,R
    BWGR,
    /// Transmit as B,R,W,G
    BRWG,
    /// Transmit as B,R,G,W
    BRGW,
    /// Transmit as B,G,W,R
    BGWR,
    /// Transmit as B,G,R,W
    BGRW,
}

impl Format {
    /// Bits 5,4 of this value are the offset (0-3) from the first byte of a
    /// pixel to the location of the red color byte.  Bits 3,2 are the green
    /// offset and 1,0 are the blue offset.  If it is an RGBW-type device
    /// (supporting a white primary in addition to R,G,B), bits 7,6 are the
    /// offset to the white byte...otherwise, bits 7,6 are set to the same value
    /// as 5,4 (red) to indicate an RGB (not RGBW) device.
    /// i.e. binary representation:
    /// 0bWWRRGGBB for RGBW devices
    /// 0bRRRRGGBB for RGB
    const fn format(&self) -> u8 {
        use Format::*;
        match self {
            // RGB NeoPixel permutations; white and red offsets are always same
            // Offset:  W          R          G          B
            RGB => (0 << 6) | (0 << 4) | (1 << 2) | (2),
            RBG => (0 << 6) | (0 << 4) | (2 << 2) | (1),
            GRB => (1 << 6) | (1 << 4) | (0 << 2) | (2),
            GBR => (2 << 6) | (2 << 4) | (0 << 2) | (1),
            BRG => (1 << 6) | (1 << 4) | (2 << 2) | (0),
            BGR => (2 << 6) | (2 << 4) | (1 << 2) | (0),

            // RGBW NeoPixel permutations; all 4 offsets are distinct
            // Offset:   W          R          G          B
            WRGB => (0 << 6) | (1 << 4) | (2 << 2) | (3),
            WRBG => (0 << 6) | (1 << 4) | (3 << 2) | (2),
            WGRB => (0 << 6) | (2 << 4) | (1 << 2) | (3),
            WGBR => (0 << 6) | (3 << 4) | (1 << 2) | (2),
            WBRG => (0 << 6) | (2 << 4) | (3 << 2) | (1),
            WBGR => (0 << 6) | (3 << 4) | (2 << 2) | (1),

            RWGB => (1 << 6) | (0 << 4) | (2 << 2) | (3),
            RWBG => (1 << 6) | (0 << 4) | (3 << 2) | (2),
            RGWB => (2 << 6) | (0 << 4) | (1 << 2) | (3),
            RGBW => (3 << 6) | (0 << 4) | (1 << 2) | (2),
            RBWG => (2 << 6) | (0 << 4) | (3 << 2) | (1),
            RBGW => (3 << 6) | (0 << 4) | (2 << 2) | (1),

            GWRB => (1 << 6) | (2 << 4) | (0 << 2) | (3),
            GWBR => (1 << 6) | (3 << 4) | (0 << 2) | (2),
            GRWB => (2 << 6) | (1 << 4) | (0 << 2) | (3),
            GRBW => (3 << 6) | (1 << 4) | (0 << 2) | (2),
            GBWR => (2 << 6) | (3 << 4) | (0 << 2) | (1),
            GBRW => (3 << 6) | (2 << 4) | (0 << 2) | (1),

            BWRG => (1 << 6) | (2 << 4) | (3 << 2) | (0),
            BWGR => (1 << 6) | (3 << 4) | (2 << 2) | (0),
            BRWG => (2 << 6) | (1 << 4) | (3 << 2) | (0),
            BRGW => (3 << 6) | (1 << 4) | (2 << 2) | (0),
            BGWR => (2 << 6) | (3 << 4) | (1 << 2) | (0),
            BGRW => (3 << 6) | (2 << 4) | (1 << 2) | (0),
        }
    }

    const fn is_rgb(&self) -> bool {
        use Format::*;
        match self {
            RGB => true,
            RBG => true,
            GRB => true,
            GBR => true,
            BRG => true,
            BGR => true,
            _ => false,
        }
    }
}

use crate::progmem;

progmem! {
    /// 8-bit gamma-correction table.
    static progmem GAMMA_TABLE: [u8; 256] = [
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
}

progmem! {
    /// 8-bit unsigned sine wave table (0-255).
    static progmem SINE_TABLE: [u8; 256]  = [
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
}

/// Stores state and methods for interacting with 
/// Adafruit NeoPixels and compatible devices.
pub struct Neopixel<const LENGTH: usize> {
    /// Whether the `begin()` method has been called on this instance.
    begun: bool,
    /// The state of each pixel.
    pixels: [u32; LENGTH],
    /// The transmission format.
    format: Format,
    /// The signal pin connected to the array.
    pin: Pin,
}

impl<const LENGTH: usize> Neopixel<LENGTH> {
    /// Creates a new instance of a Neopixel array.
    /// Call the `begin()` method before use.
    pub fn new(pin: Pin, format: Format) -> Neopixel<LENGTH> {
        Neopixel {
            begun: false,
            pixels: [0; LENGTH],
            format,
            pin
        }
    }

    /// Configure the NeoPixel pin for output.
    pub fn begin(&mut self) {
        if self.pin != Pin::D0 {
            pin_mode(self.pin, PinMode::OUTPUT);
            digital_write(self.pin, false);
        }
        self.begun = true;
    }
}
