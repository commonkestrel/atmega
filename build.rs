use std::{ env, path::Path, fs::File, io::Write, time };

const CPU_FREQUENCY: Option<&str> = option_env!("AVR_CPU_FREQUENCY_HZ");

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Environment variable OUT_DIR not defined");
    let dest_path = Path::new(&out_dir).join("constants.rs");
    let mut f = File::create(&dest_path).expect("Could not create \"constants.rs\"");

    let frequency: u64 = match CPU_FREQUENCY {
        Some(frequency) => match frequency.parse() {
            Ok(freq) => freq,
            Err(_) => {
                println!("cargo:warning=Unable to parse AVR_CPU_FREQUENCY_HZ into u64, defaulting to 16MHz");
                16_000_000
            }
        },
        None => {
            println!("cargo:warning=AVR_CPU_FREQUENCY_HZ not found, defaulting to 16MHz");
            16_000_000
        },
    };
    write!(&mut f, "/// Reflects contents of AVR_CPU_FREQUENCY_HZ environment variable (defaults to 16MHz)\npub const CPU_FREQUENCY: u64 = {};", frequency).expect("Failed to write to OUT_DIR/constants.rs");
    let now = time::SystemTime::now();
    let unix = now.duration_since(time::UNIX_EPOCH).expect("System time set before Unix Epoch");
    write!(&mut f, "/// The unix timestamp at compile time.\npub const TIME: u64 = {};", unix.as_secs()).expect("Failed to write to OUT_DIR/constants.rs");
    println!("cargo:rerun-if-env-changed=AVR_CPU_FREQUENCY_HZ");
}