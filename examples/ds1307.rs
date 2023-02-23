#![no_std]
#![no_main]

use atmega::prelude::*;
use atmega::libraries::wire;
use atmega::drivers::ds1307;

run!(setup, run);

fn setup() {
    Serial::begin(9600);
    wire::begin();
}

fn run() {
    println!("hi {:?}", ds1307::read());
    delay(1000);
}
