#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

const LED_BUILTIN: Pin = Pin::D13;

fn setup() {
    pin_mode(LED_BUILTIN, PinMode::OUTPUT);
}

fn run() {
    digital_toggle(LED_BUILTIN);
    delay(1000);
}
