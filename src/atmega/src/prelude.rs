pub use crate::{ pins::{ Pin, PinMode, HIGH, LOW, pin_mode, digital_read, digital_write, digital_toggle } };
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
