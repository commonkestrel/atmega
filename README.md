# atmega

A fast, easy, recognizable interface for the ATmega328p.
Provides many of the features of the Arduino language already used, with the benifit of Rust's memory safety and language features.
Not quite finished, but this works well enough to make simple programs.

### Why should you use this?
If you are looking for a high(ish) level, easy to use interface with the ATmega328p, this is the crate for you.
This crate aims to provide easy, high-level interfaces to low-level operations without abstracting too much of the control logic away.
It's main goal is to emulate the official Arduino language as closely as possible.

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
Let's start with a classic: blinking an LED: 

[`atmega-template`](https://github.com/commonkestrel/atmega-template) includes a small blink example, which we're going to recreate here.
This program should go in `src/main.rs`, like usual.

To start add this:
```rust
#![no_std]
#![no_main]
```
I know, it already looks scary, but stay with me.
All this does is tell the compiler to link to [`libcore`](https://doc.rust-lang.org/core/) instead of [`libstd`](https://doc.rust-lang.org/std/).
We need to do this because [`libstd`](https://doc.rust-lang.org/std/) requires certain C dependencies as well as an allocator, which AVR targets do not have.

However, this does mean that certain data types, functions, and macros like `Vec`, `String`, and `format!` will be unavailable, since these require an allocator.

After this, import the atmega prelude:
```rust
use atmega::prelude::*;
```
The atmega prelude includes important functions and macros, like `digital_write()`, `pin_mode()`, `delay()`, just like the functions in the Arduino language.

Next we need to add a `setup` function and initialize the pin the LED will be connected to:
```rust
fn setup() {
    pin_mode(Pin::D9, PinMode::OUTPUT);
}
```
In this example we are using pin D9 for the LED, and we initialize it to output.

Now we need a loop. Since the `loop` keyword is already taken, `run()` is used instead.
```rust
fn run() {

}
```
This function will be called indefinetly in a loop, just like the `loop()` function in the Arduino language.

Now we need to actually blink the LED.
Add this inside `run()`:
```rust
digital_toggle(Pin::D9);
delay(1000);
```
This toggles the output of pin D9 (which we initialized as output earlier), then delays 1000 milliseconds, or 1 second.

Finally, add this near the top of your file: 
```rust
run!(setup, run);
```
This is a utility macro and takes care of exporting a `main` function, initializing timers and registers, and running your code.

After that, your `main.rs` should look like this: 
```rust
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
```

If you created your project with [cargo-generate](https://github.com/cargo-generate/cargo-generate), connect your chip to a USB port run `cargo run`.
You should see you LED start to flash!
