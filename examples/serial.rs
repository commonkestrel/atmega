#![no_std]
#![no_main]

use atmega::prelude::*;

run!(setup, run);

fn setup() {
    pin_mode(Pin::D9, PinMode::Output);
    Serial::begin(9600);
}

fn run() {
    if let Some(byte) = Serial::try_recieve() {
        println!("{}", byte as char);
        digital_write(Pin::D9, HIGH);
    }
}
