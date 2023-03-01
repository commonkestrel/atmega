//! Utilities for working with color for RGB LEDs

pub trait Color {
    /// Red, Green, and Blue values of the color from 0-255
    fn rgb(&self) -> (u8, u8, u8);
    /// Percentage values of the red, green, and blue parts from 0.0-1.0
    fn rgbf(&self) -> (f32, f32, f32);
    /// Hue, saturation, and value of the color from 0-255
    fn hsv(&self) -> (u8, u8, u8);
    /// Percentage values of the hue, saturation, and value of the color from 0.0-1.0
    fn hsvf(&self) -> (f32, f32, f32);
}