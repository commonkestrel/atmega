#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use atmega::prelude::*;

run!(setup, runner);

struct State {
    original_millis: u32,
}

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() -> State {
    //pin_mode(Pin::D7, PinMode::INPUT_PULLUP);
    //pin_mode(Pin::D8, PinMode::OUTPUT);
    pin_mode(Pin::D9, PinMode::OUTPUT);
    digital_write(Pin::D9, HIGH);
    State {
        original_millis: millis(),
    }
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn runner(state: &mut State) {
    //let button = !digital_read(Pin::D7);
    //digital_write(Pin::D8, button);

    //digital_toggle(Pin::D8);
    //delay(1000);

    if millis() > 1000 {
        digital_write(Pin::D9, LOW);
    }
}
