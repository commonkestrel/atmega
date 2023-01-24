# Arduino-rs

Just a small project for controlling an arduino in Rust.
This is nowhere near finished, I just had fun creating an os with the Philipp Oppermann [blog_os](https://os.phil-opp.com/) blog, and figured this was a perfect project to work on.
Build with `cargo build --release`, and upload the resulting `.elf` file at `./avr-atmega328p/release/arduino-rs.elf` to a board or simulator.