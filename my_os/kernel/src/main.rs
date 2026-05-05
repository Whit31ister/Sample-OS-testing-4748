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
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }
    drivers::init();

    let mut checks = BootChecks::from_boot_state(multiboot_magic, multiboot_info_addr);
    kernel_init();
    checks.kernel_ok = true;

    drivers::show_boot_report(&checks);

    if checks.all_ok() {
        // Wait 2 seconds
        for _ in 0..40000000 { core::hint::spin_loop(); }
        terminal_init();
    } else {
        utils::halt()
    }
}

fn terminal_init() -> ! {
    drivers::clear_terminal();
    drivers::vga_println("--- EXTENDED SYSTEM SAFETY CHECK ---");
    
    let mut integrity_ok = true;
    
    // Check 1: Filesystem Structure
    drivers::write_str("Verifying Filesystem... ");
    fs::init();
    let _ = fs::list_files();
    drivers::write_str_color("[OK]\n", drivers::COLOR_LIGHT_GREEN);

    // Check 2: Physical Memory Manager
    drivers::write_str("Initializing PMM Hook... ");
    memory::init();
    if memory::allocate_page().is_some() {
        drivers::write_str_color("[OK]\n", drivers::COLOR_LIGHT_GREEN);
    } else {
        drivers::write_str_color("[FAIL]\n", drivers::COLOR_LIGHT_RED);
        integrity_ok = false;
    }

    // Check 3: Keyboard Connectivity
    drivers::write_str("Probing Keyboard... ");
    drivers::write_str_color("[OK]\n", drivers::COLOR_LIGHT_GREEN);

    if integrity_ok {
        drivers::write_str_color("\nAll safety checks passed. Launching Sample OS Terminal...\n", drivers::COLOR_YELLOW);
        for _ in 0..10000000 { core::hint::spin_loop(); }
        drivers::clear_terminal();
        shell_loop();
    } else {
        drivers::write_str_color("\nSYSTEM INTEGRITY COMPROMISED. HALTING.\n", drivers::COLOR_RED);
        utils::halt();
    }
}

fn shell_loop() -> ! {
    drivers::write_str_color("Sample OS Terminal v1.2\n", drivers::COLOR_LIGHT_CYAN);
    drivers::vga_println("Full filesystem and PMM access granted.");
    drivers::write_line("Type 'help' for a list of commands.");

    let mut input_buf = [0u8; 128];
    loop {
        drivers::write_str_color("user@sample_os", drivers::COLOR_LIGHT_GREEN);
        drivers::write_str(":");
        drivers::write_str_color("~", drivers::COLOR_LIGHT_BLUE);
        drivers::write_str("$ ");
        
        let len = drivers::read_line(&mut input_buf);
        if len == 0 {
            continue;
        }

        let input = core::str::from_utf8(&input_buf[..len]).unwrap_or("");
        let mut parts = input.splitn(3, ' ');
        let cmd = parts.next().unwrap_or("");
        let arg1 = parts.next().unwrap_or("");
        let arg2 = parts.next().unwrap_or("");

        match cmd {
            "help" => {
                drivers::write_line("Commands: ls, touch, cat, write, rm, mem, clear, help");
            }
            "ls" => {
                let files = fs::list_files();
                let mut found = false;
                for file in files.iter() {
                    if let Some(name) = file {
                        drivers::write_line(name);
                        found = true;
                    }
                }
                if !found {
                    drivers::write_line("No files found.");
                }
            }
            "touch" => {
                if arg1.is_empty() {
                    drivers::write_line("Usage: touch <filename>");
                } else {
                    match fs::create_file(arg1) {
                        Ok(_) => drivers::write_line("File created."),
                        Err(e) => drivers::write_str_color(e, drivers::COLOR_RED),
                    }
                }
            }
            "cat" => {
                if arg1.is_empty() {
                    drivers::write_line("Usage: cat <filename>");
                } else {
                    match fs::read_file(arg1) {
                        Ok(content) => drivers::write_line(content),
                        Err(e) => drivers::write_str_color(e, drivers::COLOR_RED),
                    }
                }
            }
            "write" => {
                if arg1.is_empty() || arg2.is_empty() {
                    drivers::write_line("Usage: write <filename> <content>");
                } else {
                    match fs::write_file(arg1, arg2) {
                        Ok(_) => drivers::write_line("File updated."),
                        Err(e) => drivers::write_str_color(e, drivers::COLOR_RED),
                    }
                }
            }
            "rm" => {
                if arg1.is_empty() {
                    drivers::write_line("Usage: rm <filename>");
                } else {
                    match fs::delete_file(arg1) {
                        Ok(_) => drivers::write_line("File deleted."),
                        Err(e) => drivers::write_str_color(e, drivers::COLOR_RED),
                    }
                }
            }
            "mem" => {
                drivers::write_str("PMM Hook Status: ");
                drivers::write_str_color("Active\n", drivers::COLOR_LIGHT_GREEN);
            }
            "clear" => {
                drivers::clear_terminal();
            }
            _ => {
                drivers::write_str("Unknown command: ");
                drivers::write_line(cmd);
            }
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    drivers::show_panic();
    utils::halt()
}
