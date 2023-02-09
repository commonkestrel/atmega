#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

struct State {
    analog: u8,
    direction: bool, // true for up, false for down
}

fn setup() -> State {
    pin_mode(Pin::D9, PinMode::OUTPUT);
    State { analog: 0, direction: true }
}

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

    delay(10);
}
