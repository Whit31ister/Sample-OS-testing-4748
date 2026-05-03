#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code))]

mod arch;
mod drivers;
mod fs;
mod memory;
mod utils;

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
const MULTIBOOT_MAGIC: u32 = 0x1BADB002;
#[cfg(not(test))]
const MULTIBOOT_FLAGS: u32 = 0x00000003;
#[cfg(not(test))]
const MULTIBOOT_CHECKSUM: u32 = (0u32).wrapping_sub(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS);

#[cfg(not(test))]
#[used]
#[unsafe(link_section = ".multiboot")]
static MULTIBOOT_HEADER: [u32; 3] = [
    MULTIBOOT_MAGIC,
    MULTIBOOT_FLAGS,
    MULTIBOOT_CHECKSUM,
];

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    kernel_init();
    utils::halt()
}

fn kernel_init() {
    arch::init();
    memory::init();
    drivers::init();
    fs::init();
    drivers::write_line("Sample OS kernel initialized");
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    utils::halt()
}
