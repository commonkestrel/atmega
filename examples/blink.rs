#![no_std]
#![no_main]

use atmega::prelude::*;

const LED_BUILTIN: Pin = Pin::D13;

run!(setup, run);

fn setup() {
    pin_mode(LED_BUILTIN, PinMode::OUTPUT);
}

fn run() {
    digital_toggle(LED_BUILTIN);
    delay(1000);
}
