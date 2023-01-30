#![no_std]
#![no_main]

use atmega::prelude::*;
use core::fmt::Write;

run!(setup, run);

struct State {
    prev_millis: u32,
}

/// Called once.
/// Used to initialize pins and peripherals.
/// Equivalent to the `setup` function in the Arduino language.
fn setup() -> State {
    Serial::begin(57600);
    pin_mode(Pin::D9, PinMode::OUTPUT);
    State { prev_millis: millis() }
}

/// Called in a loop indefinitly.
/// Equivalent to the `loop` function in the Arduino language.
fn run(state: &mut State) {
    let ms = millis();
    if ms - state.prev_millis > 1000 {
        digital_toggle(Pin::D9);
        state.prev_millis = ms;
        // Error: args.len()C:\Users\Jett\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\fmt\mod.rsunsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`called `Option::unwrap()` on a `None` valueC:\Users\Jett\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\char\convert.rsC:\Users\Jett\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\str\iter.rsC:\Users\Jett\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\str\validations.rsErrorattempt to add with overflowC:\Users\Jett\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\iter\traits\accum.rsunsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`attempt to add with overflowunsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`C:\Users\Jett\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rust
        Serial::new().write_str("                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     ").unwrap();
    }
}
