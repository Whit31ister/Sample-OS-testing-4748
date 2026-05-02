#![no_std]
#![no_main]

mod arch;
mod drivers;
mod fs;
mod memory;
mod utils;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    arch::init();
    memory::init();
    drivers::init();
    fs::init();

    utils::halt()
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    utils::halt()
}
