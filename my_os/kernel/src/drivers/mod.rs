use crate::BootChecks;
use core::arch::asm;
use core::ptr::write_volatile;

const COM1: u16 = 0x3F8;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_DEFAULT_COLOR: u8 = 0x0F;
const VGA_INFO_COLOR: u8 = 0x07;
const VGA_SUCCESS_COLOR: u8 = 0x0A;
const VGA_ERROR_COLOR: u8 = 0x0C;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
}

pub fn write_line(message: &str) {
    write(message);
    write("\r\n");
}

pub fn show_boot_report(report: &BootChecks) {
    clear_screen();
    write_screen_line(0, "Sample OS", VGA_DEFAULT_COLOR);
    write_screen_line(2, "Boot checks:", VGA_INFO_COLOR);
    write_check_line(4, "Bootloader handoff", report.bootloader_ok);
    write_check_line(5, "Multiboot info", report.multiboot_info_ok);
    write_check_line(6, "Kernel init", report.kernel_ok);

    if report.all_ok() {
        write_screen_line(8, "OS boot successful", VGA_SUCCESS_COLOR);
    } else {
        write_screen_line(8, "OS boot failed", VGA_ERROR_COLOR);
    }

    write_line("Sample OS boot report:");
    write_serial_check_line("Bootloader handoff", report.bootloader_ok);
    write_serial_check_line("Multiboot info", report.multiboot_info_ok);
    write_serial_check_line("Kernel init", report.kernel_ok);
    if report.all_ok() {
        write_line("OS boot successful");
    } else {
        write_line("OS boot failed");
    }
}

pub fn show_panic() {
    clear_screen();
    write_screen_line(0, "Sample OS", VGA_DEFAULT_COLOR);
    write_screen_line(2, "Kernel panic", VGA_ERROR_COLOR);
    write_line("Kernel panic");
}

fn write(message: &str) {
    for byte in message.bytes() {
        write_byte(byte);
    }
}

fn clear_screen() {
    for row in 0..VGA_HEIGHT {
        clear_row(row, VGA_DEFAULT_COLOR);
    }
}

fn clear_row(row: usize, color: u8) {
    for column in 0..VGA_WIDTH {
        write_cell(row, column, b' ', color);
    }
}

fn write_screen_line(row: usize, message: &str, color: u8) {
    clear_row(row, color);
    write_screen_text_at(row, 0, message, color);
}

fn write_screen_text_at(row: usize, column_offset: usize, message: &str, color: u8) {
    for (column, byte) in message
        .bytes()
        .take(VGA_WIDTH.saturating_sub(column_offset))
        .enumerate()
    {
        write_cell(row, column + column_offset, byte, color);
    }
}

fn write_check_line(row: usize, label: &str, ok: bool) {
    let color = if ok {
        VGA_SUCCESS_COLOR
    } else {
        VGA_ERROR_COLOR
    };
    let status = if ok { "[ok]" } else { "[fail]" };

    write_screen_line(row, label, color);
    write_screen_text_at(row, 24, status, color);
}

fn write_serial_check_line(label: &str, ok: bool) {
    write(label);
    write(": ");
    if ok {
        write_line("ok");
    } else {
        write_line("fail");
    }
}

fn write_cell(row: usize, column: usize, byte: u8, color: u8) {
    let offset = (row * VGA_WIDTH + column) * 2;
    unsafe {
        write_volatile(VGA_BUFFER.add(offset), byte);
        write_volatile(VGA_BUFFER.add(offset + 1), color);
    }
}

fn write_byte(byte: u8) {
    unsafe {
        while inb(COM1 + 5) & 0x20 == 0 {}
        outb(COM1, byte);
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}
