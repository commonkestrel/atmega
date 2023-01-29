#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

struct State {
    prev_millis: u32,
}

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() -> State {
    pin_mode(Pin::D9, PinMode::OUTPUT);
    State {
        prev_millis: millis(),
    }
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn run(state: &mut State) {
    if millis() - state.prev_millis > 1000 {
        digital_toggle(Pin::D9);
        state.prev_millis = millis();
    }
}
