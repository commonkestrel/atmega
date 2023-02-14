//! Re-exports of important traits, types, macros, and functions used with atmega. Meant to be glob imported.

pub use crate::serial::Serial;
pub use crate::registers::Register;
pub use crate::{ run, print, println };
pub use crate::timing::{ delay, delay_micros };
pub use crate::wiring::{ Pin, PinMode, HIGH, LOW, pin_mode, digital_read, digital_write, digital_toggle, analog_read, analog_write };

#[cfg(any(feature = "millis", doc))]
#[doc(cfg(feature = "millis"))]
pub use crate::timing::millis;
