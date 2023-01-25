# ATmega328p-rs

Just a small project for controlling an ATmega328p (the microcontroller in the Arduino Uno) in Rust.
This is nowhere near finished, I just had fun working at a lower level with the Philipp Oppermann [blog_os](https://os.phil-opp.com/) and figured this was a perfect project to work on.

Build with `cargo build --release`, and upload the resulting `.elf` file at `./avr-atmega328p/release/arduino-rs.elf` to a board or simulator.
If the dev build (`cargo build`) fails with `error: ran out of registers during register allocation`, try increasing the `opt-level` for `[profile.dev]` to 3.