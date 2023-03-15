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
    let time = ds1307::read();
    println!("hi");
    match time {
        Ok(t) => println!("{}", t),
        Err(e) => println!("{:?}", e),
    }
}
