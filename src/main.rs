#![no_std]
#![no_main]
#![feature(lang_items)]

use atmega::prelude::*;

run!(setup, runner);

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() {
    pin_mode(Pin::D7, PinMode::INPUT_PULLUP);
    pin_mode(Pin::D8, PinMode::OUTPUT);
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn runner() {
    let button = digital_read(Pin::D7);
    digital_write(Pin::D8, button);
}
