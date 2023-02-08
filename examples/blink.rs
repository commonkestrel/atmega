#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

struct State {}

fn setup() -> State {
    pin_mode(Pin::D9, PinMode::OUTPUT);
    State {}
}

fn run(_state: State) {
    digital_toggle(Pin::D9);
    delay(1000);
}
