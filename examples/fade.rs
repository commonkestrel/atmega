#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use atmega::prelude::*;

run!(setup, run);

struct State {
    analog: u8,
    direction: bool, // true for up, false for down
}

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() -> State {
    Serial::begin(9600);

    pin_mode(Pin::D9, PinMode::OUTPUT);

    State { analog: 0, direction: true }
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn run(state: &mut State) {
    analog_write(Pin::D9, state.analog);
    if state.analog == 0 {
        state.direction = true;
    } else if state.analog == 255 {
        state.direction = false;
    }

    if state.direction {
        state.analog += 1;
    } else {
        state.analog -= 1;
    }
    println!("{}", millis());
    delay(10);
}