//! Libraries to match the official Arduino language such as Wire and TimeLib

pub mod time;
pub mod color;

#[cfg(any(feature = "twowire", doc))]
#[doc(cfg(feature = "twowire"))]
pub mod wire;
