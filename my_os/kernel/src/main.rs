#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code))]

mod arch;
mod drivers;
mod fs;
mod memory;
mod utils;

#[cfg(not(test))]
use core::{arch::global_asm, panic::PanicInfo};

#[cfg(not(test))]
global_asm!(
    r#"
    .section .text._start,"ax"
    .global _start
    .type _start,@function
_start:
    mov esp, offset boot_stack_top
    xor ebp, ebp
    push ebx
    push eax
    call kernel_main

1:
    cli
    hlt
    jmp 1b

    .section .bss.boot_stack,"aw",@nobits
    .align 16
boot_stack:
    .skip 16384
boot_stack_top:
"#
);

const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;
const MULTIBOOT_FLAGS: u32 = 0x00000003;
const MULTIBOOT_CHECKSUM: u32 =
    (0u32).wrapping_sub(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_FLAGS);

#[cfg(not(test))]
#[used]
#[unsafe(link_section = ".multiboot")]
static MULTIBOOT_HEADER: [u32; 3] = [
    MULTIBOOT_HEADER_MAGIC,
    MULTIBOOT_FLAGS,
    MULTIBOOT_CHECKSUM,
];

pub(crate) struct BootChecks {
    pub(crate) bootloader_ok: bool,
    pub(crate) multiboot_info_ok: bool,
    pub(crate) kernel_ok: bool,
}

impl BootChecks {
    const fn from_boot_state(multiboot_magic: u32, multiboot_info_addr: u32) -> Self {
        Self {
            bootloader_ok: multiboot_magic == MULTIBOOT_BOOTLOADER_MAGIC,
            multiboot_info_ok: multiboot_info_addr != 0,
            kernel_ok: false,
        }
    }

    pub(crate) const fn all_ok(&self) -> bool {
        self.bootloader_ok && self.multiboot_info_ok && self.kernel_ok
    }
}

fn kernel_init() {
    arch::init();
    memory::init();
    fs::init();
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(multiboot_magic: u32, multiboot_info_addr: u32) -> ! {
    drivers::init();

    let mut checks = BootChecks::from_boot_state(multiboot_magic, multiboot_info_addr);
    kernel_init();
    checks.kernel_ok = true;

    drivers::show_boot_report(&checks);
    fs::run_shell()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    drivers::show_panic();
    utils::halt()
}
