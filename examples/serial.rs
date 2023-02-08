#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

struct State {}

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() -> State {
    pin_mode(Pin::D9, PinMode::OUTPUT);
    Serial::begin(9600);
    State {}
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn run(_state: &mut State) {
    if Serial::available() > 0 {
        digital_write(Pin::D9, HIGH);
    }
}