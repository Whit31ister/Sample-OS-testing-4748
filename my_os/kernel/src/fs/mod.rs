#![allow(static_mut_refs)]

use crate::{drivers, memory};
use fms_core::{BlockDevice, Fms, FmsError, FmsFileEntry, FMS_BLOCK_SIZE, FMS_MAX_FILES};

const RAM_DISK_BLOCKS: u32 = 1024;
const RAM_DISK_BYTES: usize = RAM_DISK_BLOCKS as usize * FMS_BLOCK_SIZE;
const RAM_DISK_PAGES: usize = RAM_DISK_BYTES / 4096;
const INPUT_BUFFER_SIZE: usize = 256;

static mut RAM_DISK_BASE: *mut u8 = core::ptr::null_mut();
static mut FS_STATE: Option<Fms> = None;

struct RamDisk;

impl BlockDevice for RamDisk {
    fn read_block(&mut self, block_number: u32, out: &mut [u8; FMS_BLOCK_SIZE]) -> Result<(), FmsError> {
        if block_number >= RAM_DISK_BLOCKS || unsafe { RAM_DISK_BASE.is_null() } {
            return Err(FmsError::Io);
        }

        let start = block_number as usize * FMS_BLOCK_SIZE;
        let end = start + FMS_BLOCK_SIZE;
        unsafe {
            let disk = core::slice::from_raw_parts(RAM_DISK_BASE, RAM_DISK_BYTES);
            out.copy_from_slice(&disk[start..end]);
        }
        Ok(())
    }

    fn write_block(&mut self, block_number: u32, data: &[u8; FMS_BLOCK_SIZE]) -> Result<(), FmsError> {
        if block_number >= RAM_DISK_BLOCKS || unsafe { RAM_DISK_BASE.is_null() } {
            return Err(FmsError::Io);
        }

        let start = block_number as usize * FMS_BLOCK_SIZE;
        let end = start + FMS_BLOCK_SIZE;
        unsafe {
            let disk = core::slice::from_raw_parts_mut(RAM_DISK_BASE, RAM_DISK_BYTES);
            disk[start..end].copy_from_slice(data);
        }
        Ok(())
    }
}

pub fn init() {
    let base = memory::alloc_pages(RAM_DISK_PAGES);
    if base.is_null() {
        drivers::write_line("filesystem init failed: out of PMM pages");
        return;
    }

    unsafe {
        RAM_DISK_BASE = base;
        core::ptr::write_bytes(RAM_DISK_BASE, 0, RAM_DISK_BYTES);
    }

    let mut disk = RamDisk;
    let fs = match Fms::format(&mut disk, RAM_DISK_BLOCKS) {
        Ok(fs) => fs,
        Err(_) => return,
    };

    unsafe {
        FS_STATE = Some(fs);
    }
}

pub fn run_shell() -> ! {
    drivers::write_line("Entering filesystem shell.");
    drivers::write_line("Type 'help' for commands.");

    let mut input = [0u8; INPUT_BUFFER_SIZE];
    loop {
        drivers::prompt();
        let len = drivers::read_line(&mut input);
        if len == 0 {
            continue;
        }

        handle_command(&input[..len]);
    }
}

fn handle_command(line: &[u8]) {
    let command = match core::str::from_utf8(line) {
        Ok(text) => text.trim(),
        Err(_) => {
            drivers::write_line("invalid input encoding");
            return;
        }
    };

    if command.is_empty() {
        return;
    }

    if command == "help" {
        show_help();
        return;
    }
    if command == "mkfs" {
        cmd_mkfs();
        return;
    }
    if command == "files" {
        cmd_files();
        return;
    }
    if command == "pmm" {
        cmd_pmm();
        return;
    }
    if command == "halt" {
        drivers::write_line("halting");
        crate::utils::halt();
    }

    if let Some(name) = command.strip_prefix("open ") {
        cmd_open(name.trim());
        return;
    }
    if let Some(name) = command.strip_prefix("delete ") {
        cmd_delete(name.trim());
        return;
    }
    if let Some(name) = command.strip_prefix("info ") {
        cmd_info(name.trim());
        return;
    }
    if let Some(rest) = command.strip_prefix("save ") {
        cmd_save(rest);
        return;
    }

    drivers::write_line("unknown command");
}

fn show_help() {
    drivers::write_line("commands:");
    drivers::write_line("  help");
    drivers::write_line("  mkfs");
    drivers::write_line("  files");
    drivers::write_line("  save <name> <text>");
    drivers::write_line("  open <name>");
    drivers::write_line("  info <name>");
    drivers::write_line("  delete <name>");
    drivers::write_line("  pmm");
    drivers::write_line("  halt");
}

fn cmd_mkfs() {
    let mut disk = RamDisk;
    match Fms::format(&mut disk, RAM_DISK_BLOCKS) {
        Ok(fs) => unsafe {
            FS_STATE = Some(fs);
            drivers::write_line("filesystem formatted");
        },
        Err(err) => {
            drivers::write("mkfs failed: ");
            drivers::write_line(fms_err(err));
        }
    }
}

fn cmd_files() {
    let mut disk = RamDisk;
    let Some(fs) = (unsafe { FS_STATE.as_mut() }) else {
        drivers::write_line("filesystem not initialized");
        return;
    };

    let mut entries = [FmsFileEntry::empty(); FMS_MAX_FILES];
    let mut count = 0usize;
    match fs.list(&mut disk, &mut entries, &mut count) {
        Ok(()) => {
            if count == 0 {
                drivers::write_line("(no files)");
                return;
            }
            for entry in entries.iter().take(count) {
                drivers::write_bytes(entry.name_bytes());
                drivers::write("  size=");
                drivers::write_u32(entry.size);
                drivers::write("  start=");
                drivers::write_u32(entry.start_block);
                drivers::write("  blocks=");
                drivers::write_u32(entry.block_count);
                drivers::write_line("");
            }
        }
        Err(err) => {
            drivers::write("files failed: ");
            drivers::write_line(fms_err(err));
        }
    }
}

fn cmd_save(rest: &str) {
    let Some(space) = rest.find(' ') else {
        drivers::write_line("usage: save <name> <text>");
        return;
    };
    let name = rest[..space].trim();
    let data = rest[space + 1..].as_bytes();
    if name.is_empty() || data.is_empty() {
        drivers::write_line("usage: save <name> <text>");
        return;
    }

    let mut disk = RamDisk;
    let Some(fs) = (unsafe { FS_STATE.as_mut() }) else {
        drivers::write_line("filesystem not initialized");
        return;
    };

    match fs.write_file(&mut disk, name, data) {
        Ok(()) => drivers::write_line("saved"),
        Err(err) => {
            drivers::write("save failed: ");
            drivers::write_line(fms_err(err));
        }
    }
}

fn cmd_open(name: &str) {
    if name.is_empty() {
        drivers::write_line("usage: open <name>");
        return;
    }

    let mut disk = RamDisk;
    let Some(fs) = (unsafe { FS_STATE.as_mut() }) else {
        drivers::write_line("filesystem not initialized");
        return;
    };

    let stat = match fs.stat(&mut disk, name) {
        Ok(entry) => entry,
        Err(err) => {
            drivers::write("open failed: ");
            drivers::write_line(fms_err(err));
            return;
        }
    };

    if stat.size as usize > INPUT_BUFFER_SIZE {
        drivers::write_line("file too large for shell output buffer");
        return;
    }

    let mut out = [0u8; INPUT_BUFFER_SIZE];
    let mut read = 0usize;
    match fs.read_file(&mut disk, name, &mut out, &mut read) {
        Ok(()) => {
            drivers::write_bytes(&out[..read]);
            drivers::write_line("");
        }
        Err(err) => {
            drivers::write("read failed: ");
            drivers::write_line(fms_err(err));
        }
    }
}

fn cmd_delete(name: &str) {
    if name.is_empty() {
        drivers::write_line("usage: delete <name>");
        return;
    }

    let mut disk = RamDisk;
    let Some(fs) = (unsafe { FS_STATE.as_mut() }) else {
        drivers::write_line("filesystem not initialized");
        return;
    };

    match fs.delete(&mut disk, name) {
        Ok(()) => drivers::write_line("deleted"),
        Err(err) => {
            drivers::write("delete failed: ");
            drivers::write_line(fms_err(err));
        }
    }
}

fn cmd_info(name: &str) {
    if name.is_empty() {
        drivers::write_line("usage: info <name>");
        return;
    }

    let mut disk = RamDisk;
    let Some(fs) = (unsafe { FS_STATE.as_mut() }) else {
        drivers::write_line("filesystem not initialized");
        return;
    };

    match fs.stat(&mut disk, name) {
        Ok(entry) => {
            drivers::write("name: ");
            drivers::write_bytes(entry.name_bytes());
            drivers::write_line("");
            drivers::write("size: ");
            drivers::write_u32(entry.size);
            drivers::write_line("");
            drivers::write("start_block: ");
            drivers::write_u32(entry.start_block);
            drivers::write_line("");
            drivers::write("blocks: ");
            drivers::write_u32(entry.block_count);
            drivers::write_line("");
        }
        Err(err) => {
            drivers::write("info failed: ");
            drivers::write_line(fms_err(err));
        }
    }
}

fn cmd_pmm() {
    drivers::write("page_size: ");
    drivers::write_usize(memory::page_size());
    drivers::write_line("");
    drivers::write("pages_used: ");
    drivers::write_usize(memory::pages_used());
    drivers::write_line("");
    drivers::write("ram_disk_pages: ");
    drivers::write_usize(RAM_DISK_PAGES);
    drivers::write_line("");
    drivers::write("ram_disk_ptr: ");
    drivers::write_u32(unsafe { memory::virt_to_phys(RAM_DISK_BASE) as u32 });
    drivers::write_line("");
}

fn fms_err(err: FmsError) -> &'static str {
    match err {
        FmsError::Io => "io error",
        FmsError::BadFs => "invalid filesystem",
        FmsError::NotFound => "not found",
        FmsError::Exists => "exists",
        FmsError::NoSpace => "no space",
        FmsError::BadName => "bad name",
        FmsError::TooLarge => "too large",
    }
}
