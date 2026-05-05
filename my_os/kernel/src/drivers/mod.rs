use crate::BootChecks;
use core::arch::asm;
use core::ptr::write_volatile;

const COM1: u16 = 0x3F8;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

// VGA Colors
pub const COLOR_BLACK: u8 = 0;
pub const COLOR_BLUE: u8 = 1;
pub const COLOR_GREEN: u8 = 2;
pub const COLOR_CYAN: u8 = 3;
pub const COLOR_RED: u8 = 4;
pub const COLOR_MAGENTA: u8 = 5;
pub const COLOR_BROWN: u8 = 6;
pub const COLOR_LIGHT_GREY: u8 = 7;
pub const COLOR_DARK_GREY: u8 = 8;
pub const COLOR_LIGHT_BLUE: u8 = 9;
pub const COLOR_LIGHT_GREEN: u8 = 10;
pub const COLOR_LIGHT_CYAN: u8 = 11;
pub const COLOR_LIGHT_RED: u8 = 12;
pub const COLOR_LIGHT_MAGENTA: u8 = 13;
pub const COLOR_YELLOW: u8 = 14;
pub const COLOR_WHITE: u8 = 15;

pub const fn make_color(fg: u8, bg: u8) -> u8 {
    fg | (bg << 4)
}

static mut CURRENT_COLOR: u8 = 0x0F; // White on Black
static mut VGA_CURSOR_ROW: usize = 0;
static mut VGA_CURSOR_COL: usize = 0;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
        CURRENT_COLOR = make_color(COLOR_WHITE, COLOR_BLACK);
    }
}

pub fn set_color(fg: u8, bg: u8) {
    unsafe {
        CURRENT_COLOR = make_color(fg, bg);
    }
}

pub fn write_str(message: &str) {
    write(message);
    vga_print(message);
}

pub fn write_str_color(message: &str, fg: u8) {
    let old_color = unsafe { CURRENT_COLOR };
    set_color(fg, COLOR_BLACK);
    write_str(message);
    unsafe { CURRENT_COLOR = old_color; }
}

pub fn vga_print(message: &str) {
    for byte in message.bytes() {
        if byte == b'\n' {
            vga_newline();
        } else if byte == b'\r' {
            unsafe { VGA_CURSOR_COL = 0; }
        } else {
            write_vga_char(byte);
        }
    }
}

fn vga_newline() {
    unsafe {
        VGA_CURSOR_COL = 0;
        VGA_CURSOR_ROW += 1;
        if VGA_CURSOR_ROW >= VGA_HEIGHT {
            scroll_vga();
            VGA_CURSOR_ROW = VGA_HEIGHT - 1;
        }
    }
}

pub fn vga_println(message: &str) {
    vga_print(message);
    vga_newline();
}

pub fn write_line(message: &str) {
    write_str(message);
    write("\r\n");
    vga_newline();
}

pub fn read_line(buffer: &mut [u8]) -> usize {
    let mut len = 0;
    while len < buffer.len() {
        let key = read_key();
        if key == b'\n' || key == b'\r' {
            write_line("");
            break;
        } else if key == 0x08 {
            // Backspace
            if len > 0 {
                len -= 1;
                // Serial backspace
                write_byte(0x08);
                write_byte(b' ');
                write_byte(0x08);
                // VGA backspace
                backspace_vga();
            }
        } else if key >= 32 && key <= 126 {
            if len < buffer.len() {
                buffer[len] = key;
                len += 1;
                let tmp = [key];
                let s = core::str::from_utf8(&tmp).unwrap_or("");
                write_str(s);
            }
        }
    }
    len
}

fn read_key() -> u8 {
    unsafe {
        loop {
            if inb(COM1 + 5) & 1 != 0 {
                return inb(COM1);
            }
            if inb(0x64) & 1 != 0 {
                let scancode = inb(0x60);
                if let Some(c) = scancode_to_char(scancode) {
                    return c;
                }
            }
        }
    }
}

fn scancode_to_char(scancode: u8) -> Option<u8> {
    match scancode {
        0x1E => Some(b'a'), 0x30 => Some(b'b'), 0x2E => Some(b'c'), 0x20 => Some(b'd'),
        0x12 => Some(b'e'), 0x21 => Some(b'f'), 0x22 => Some(b'g'), 0x23 => Some(b'h'),
        0x17 => Some(b'i'), 0x24 => Some(b'j'), 0x25 => Some(b'k'), 0x26 => Some(b'l'),
        0x32 => Some(b'm'), 0x31 => Some(b'n'), 0x18 => Some(b'o'), 0x19 => Some(b'p'),
        0x10 => Some(b'q'), 0x13 => Some(b'r'), 0x1F => Some(b's'), 0x14 => Some(b't'),
        0x16 => Some(b'u'), 0x2F => Some(b'v'), 0x11 => Some(b'w'), 0x2D => Some(b'x'),
        0x15 => Some(b'y'), 0x2C => Some(b'z'),
        0x02 => Some(b'1'), 0x03 => Some(b'2'), 0x04 => Some(b'3'), 0x05 => Some(b'4'),
        0x06 => Some(b'5'), 0x07 => Some(b'6'), 0x08 => Some(b'7'), 0x09 => Some(b'8'),
        0x0A => Some(b'9'), 0x0B => Some(b'0'),
        0x39 => Some(b' '), 0x1C => Some(b'\n'), 0x0E => Some(0x08),
        0x33 => Some(b','), 0x34 => Some(b'.'), 0x35 => Some(b'/'), 0x27 => Some(b';'),
        0x28 => Some(b'\''), 0x1A => Some(b'['), 0x1B => Some(b']'), 0x2B => Some(b'\\'),
        0x0C => Some(b'-'), 0x0D => Some(b'='),
        _ => None,
    }
}

fn write_vga_char(c: u8) {
    unsafe {
        if VGA_CURSOR_COL >= VGA_WIDTH {
            vga_newline();
        }
        write_cell(VGA_CURSOR_ROW, VGA_CURSOR_COL, c, CURRENT_COLOR);
        VGA_CURSOR_COL += 1;
    }
}

fn backspace_vga() {
    unsafe {
        if VGA_CURSOR_COL > 0 {
            VGA_CURSOR_COL -= 1;
            write_cell(VGA_CURSOR_ROW, VGA_CURSOR_COL, b' ', CURRENT_COLOR);
        } else if VGA_CURSOR_ROW > 0 {
            VGA_CURSOR_ROW -= 1;
            VGA_CURSOR_COL = VGA_WIDTH - 1;
            write_cell(VGA_CURSOR_ROW, VGA_CURSOR_COL, b' ', CURRENT_COLOR);
        }
    }
}

fn scroll_vga() {
    unsafe {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let src_offset = (row * VGA_WIDTH + col) * 2;
                let dst_offset = ((row - 1) * VGA_WIDTH + col) * 2;
                let byte = *VGA_BUFFER.add(src_offset);
                let color = *VGA_BUFFER.add(src_offset + 1);
                write_volatile(VGA_BUFFER.add(dst_offset), byte);
                write_volatile(VGA_BUFFER.add(dst_offset + 1), color);
            }
        }
        clear_row(VGA_HEIGHT - 1, CURRENT_COLOR);
    }
}

pub fn clear_terminal() {
    unsafe {
        for row in 0..VGA_HEIGHT {
            clear_row(row, CURRENT_COLOR);
        }
        VGA_CURSOR_ROW = 0;
        VGA_CURSOR_COL = 0;
    }
}

fn clear_row(row: usize, color: u8) {
    for column in 0..VGA_WIDTH {
        write_cell(row, column, b' ', color);
    }
}

pub fn show_boot_report(report: &BootChecks) {
    clear_terminal();
    write_screen_line(0, "Sample OS", make_color(COLOR_LIGHT_CYAN, COLOR_BLACK));
    write_screen_line(2, "Boot checks:", make_color(COLOR_WHITE, COLOR_BLACK));
    write_check_line(4, "Bootloader handoff", report.bootloader_ok);
    write_check_line(5, "Multiboot info", report.multiboot_info_ok);
    write_check_line(6, "Kernel init", report.kernel_ok);

    if report.all_ok() {
        write_screen_line(8, "OS boot successful", make_color(COLOR_LIGHT_GREEN, COLOR_BLACK));
    } else {
        write_screen_line(8, "OS boot failed", make_color(COLOR_LIGHT_RED, COLOR_BLACK));
    }

    write_line("Sample OS boot report:");
    write_serial_check_line("Bootloader handoff", report.bootloader_ok);
    write_serial_check_line("Multiboot info", report.multiboot_info_ok);
    write_serial_check_line("Kernel init", report.kernel_ok);
}

pub fn show_panic() {
    set_color(COLOR_WHITE, COLOR_RED);
    clear_terminal();
    vga_println("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    vga_println("!                                KERNEL PANIC                                 !");
    vga_println("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
}

fn write(message: &str) {
    for byte in message.bytes() {
        write_byte(byte);
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
    let color = if ok { COLOR_GREEN } else { COLOR_RED };
    let status = if ok { "[ok]" } else { "[fail]" };
    write_screen_text_at(row, 0, label, make_color(COLOR_WHITE, COLOR_BLACK));
    write_screen_text_at(row, 24, status, make_color(color, COLOR_BLACK));
}

fn write_serial_check_line(label: &str, ok: bool) {
    write(label);
    write(": ");
    if ok {
        write("\x1b[32mok\x1b[0m\r\n");
    } else {
        write("\x1b[31mfail\x1b[0m\r\n");
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
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}
