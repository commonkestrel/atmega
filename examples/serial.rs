#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

struct State;

fn setup() -> State {
    pin_mode(Pin::D9, PinMode::OUTPUT);
    Serial::begin(9600);

    State
}

fn run(_state: &mut State) {
    if let Some(byte) = Serial::try_recieve() {
        println!("{}", byte as char);
        digital_write(Pin::D9, HIGH);
    }
}
