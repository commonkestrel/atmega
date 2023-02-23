#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

fn setup() {
    pin_mode(Pin::D9, PinMode::OUTPUT);
}

fn run() {
    digital_toggle(Pin::D9);
    delay(1000);
}
