# atmega

A fast, easy, recognizable interface for the ATmega328p.
Provides many of the features of the Arduino language already used, with the benifit of Rust's memory safety and language features.
Not quite finished, but this works well enough to make simple programs.

### Why should you use this?
If you are looking for a higher level, easy to use interface with the ATmega328p, this is the crate for you.
This crate aims to provide easy, high-level interfaces to low-level operations without abstracting too much of the control logic away.

However, if you want a lower-level interface to dozens of microcontrollers including the ATmega328p, [`ruduino`](https://github.com/avr-rust/ruduino) or [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) are probably better options.

## Setup
Rust has a wonderful toolchain for AVR targets, but we need to set it up first.
An easy way to set up an `atmega` project is [`cargo-generate`](https://github.com/cargo-generate/cargo-generate)
A [`cargo-generate`](https://github.com/cargo-generate/cargo-generate) template is provided at [atmega-template](https://github.com/commonkestrel/atmega-template)
Assuming you have nothing installed, run the following commands:
```sh
cargo install cargo-generate
cargo generate --git https://github.com/commonkestrel/atmega-template --name $PROJECT_NAME
cd $PROJECT_NAME
cargo override set nightly
rustup component add rust-src
cargo install ravedude
```
This should install [cargo-generate](https://github.com/cargo-generate/cargo-generate) and create a new project from the [template](https://github.com/commonkestrel/atmega-template). 
AVR Rust is only available in the nightly version, so we need to set the toolchain in our project directory to nightly.
After this we install [`ravedude`](https://github.com/Rahix/avr-hal/tree/main/ravedude), a Rust wrapper around avrdude that provides easy access to the target's serial console, similar to the Arduino IDE serial port.

## Examples
Let's start with a classic: blinking an LED
[`atmega-template`](https://github.com/commonkestrel/atmega-template) includes a small blink example, but we're going to recreate it here.
This program should go in `src/main.rs`, like usual.

To start add this:
```rust
#![no_std]
#![no_main]
```
I know, it already looks scary, but stay with me.
All this does is disable the Rust standard library and tell the compiler we don't have a main function.
*Why do we need to do that?*, you may ask.
The Rust standard library links to many things that AVR targets just don't have, like native `C` libraries and an allocator.
Disabling the standard library allows us to compile to targets without this.
Many of Rust's features are disabled this way, like `Vec`, `String`, `Mutex`, `format!()`, etc.
Many Rust standard library features are available through the `core` crate, however, 
