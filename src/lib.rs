#![no_std]

extern crate alloc;

pub mod delay;
pub mod errors;
pub mod gpio;
pub mod i2c;
pub mod rmt;
pub mod serial;
pub mod tasks;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        esp_idf_sys::abort();
        core::hint::unreachable_unchecked();
    }
}
