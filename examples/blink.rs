#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

fn setup() {
    pin_mode(LED_BUILTIN, PinMode::Output);
}

fn run() {
    digital_toggle(LED_BUILTIN);
    delay_millis(1000);
}
