#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use atmega::prelude::*;

run!(setup, run);

struct State {
    prev_millis: u64,
}

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() -> State {
    Serial::begin(57600);
    pin_mode(Pin::D9, PinMode::OUTPUT);
    State { prev_millis: millis() }
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn run(state: &mut State) {
    let ms = millis();
    if ms - state.prev_millis >= 2000 {
        digital_toggle(Pin::D9);
        state.prev_millis = ms;
        println!("{}ms, {}", ms, digital_read(Pin::D9));
    }
}
