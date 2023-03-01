//! Utilities for working with color for RGB LEDs

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

pub trait Color: Clone + Copy {
    fn from_rgb(red: u8, green: u8, blue: u8) -> Self;
    fn from_rgbw(red: u8, green: u8, blue: u8, white: u8) -> Self;

    /// Red, green, and blue values of the color from 0-255.
    fn rgb(&self) -> (u8, u8, u8);
    /// Red, green, blue, and white values of he color from 0-255.
    fn rgbw(&self) -> (u8, u8, u8, u8);
}

#[derive(Clone, Copy)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color for RGB {
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> RGB {
        RGB { red, green, blue }
    }

    pub fn from_rgbw(red: u8, green: u8, blue: u8, white: u8) -> RGB {
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);

    }
}

#[derive(Clone, Copy)]
pub struct RGBW {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub white: u8,
}

impl Color for RGBW {
    fn from_rgb(red: u8, green: u8, blue: u8) -> RGBW {
        RGBW {
            red, green, blue,
            white: red.min(green).min(blue),
        }
    }

    fn from_rgbw(red: u8, green: u8, blue: u8, white: u8) -> RGBW {
        RGBW { red, green, blue, white }
    }

    fn rgb(&self) -> (u8, u8, u8) {

    }

    fn rgbw(&self) -> (u8, u8, u8, u8) {

    }
}

impl RGBW {
    /// Most NeoPixel blue LEDs are not perfect, and output a bit of white.
    /// This function helps to correct this.
    pub fn blue_correct(&self) -> RGBW {
        RGBW {
            white: self.white - self.blue/5
            ..self
        }
    }
}

#[derive(Clone, Copy)]
pub struct HSI {
    pub hue: u8,
    pub saturation: u8,
    pub intensity: u8,
}
