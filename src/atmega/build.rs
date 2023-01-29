use std::{ env, path::Path, fs::File, io::Write };

const CPU_FREQUENCY: Option<&str> = option_env!("AVR_CPU_FREQUENCY_HZ");



fn main() {
    let out_dir = env::var("OUT_DIR").expect("Environment variable OUT_DIR not defined");
    let dest_path = Path::new(&out_dir).join("constants.rs");
    let mut f = File::create(&dest_path).expect("Could not create file");

    let frequency: u64 = match CPU_FREQUENCY {
        Some(frequency) => match frequency.parse() {
            Ok(freq) => freq,
            Err(_) => {
                println!("Unable to parse AVR_CPU_FREQUENCY_HZ into u64, defaulting to 16MHz");
                16_000_000
            }
        },
        None => 16_000_000,
    };
    write!(&mut f, "const CPU_FREQUENCY: u64 = {};", frequency).expect("Failed to write file");
    println!("cargo:rerun-if-env-changed=AVR_CPU_FREQUENCY_HZ");
}