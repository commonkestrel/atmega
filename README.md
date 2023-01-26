# ATmega328p-rs

Just a small project for controlling an ATmega328p (the microcontroller in the Arduino Uno) with Rust.
This is nowhere near finished, I just had fun working at a lower level with the Philipp Oppermann [blog_os](https://os.phil-opp.com/) and figured this was a perfect project to work on.

Build with `cargo build --release`, and upload the resulting `.elf` file at `./avr-atmega328p/release/arduino-rs.elf` to a board or simulator.
If the dev build (`cargo build`) fails with `error: ran out of registers during register allocation`, try increasing the `opt-level` for `[profile.dev]` to 3.
Advice on flashing to a real chip can be founr [here](https://book.avr-rust.com/004-flashing-a-crate-to-chip.html)

The [ATmega328p Datasheet](ATmega328P_Datasheet) and the [ATmega328 Register Reference](https://arbaranwal.github.io/tutorial/2017/06/23/atmega328-register-reference.html) by [arbaranwal](https://github.com/arbaranwal) was a huge help for register addresses and functions.