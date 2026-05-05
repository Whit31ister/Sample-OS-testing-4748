#![no_std]

use core::cmp::min;

pub const FMS_MAGIC: u32 = 0x3146_5346;
pub const FMS_VERSION: u32 = 1;
pub const FMS_BLOCK_SIZE: usize = 512;
pub const FMS_MAX_FILES: usize = 64;
pub const FMS_FILENAME_MAX: usize = 32;
pub const FMS_FILE_TABLE_BLOCKS: usize = 8;
pub const FMS_DEFAULT_TOTAL_BLOCKS: u32 = 4096;

const SUPERBLOCK_FIELDS: usize = 9;
const FILE_ENTRY_BYTES: usize = 1 + FMS_FILENAME_MAX + 4 + 4 + 4;
const TABLE_BYTES: usize = FMS_FILE_TABLE_BLOCKS * FMS_BLOCK_SIZE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FmsError {
    Io,
    BadFs,
    NotFound,
    Exists,
    NoSpace,
    BadName,
    TooLarge,
}

pub trait BlockDevice {
    fn read_block(&mut self, block_number: u32, out: &mut [u8; FMS_BLOCK_SIZE]) -> Result<(), FmsError>;
    fn write_block(&mut self, block_number: u32, data: &[u8; FMS_BLOCK_SIZE]) -> Result<(), FmsError>;
}

#[derive(Clone, Copy, Debug)]
pub struct FmsSuperblock {
    pub magic: u32,
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u32,
    pub file_table_start: u32,
    pub file_table_blocks: u32,
    pub data_start: u32,
    pub max_files: u32,
    pub file_count: u32,
}

impl FmsSuperblock {
    pub const fn new(total_blocks: u32) -> Self {
        Self {
            magic: FMS_MAGIC,
            version: FMS_VERSION,
            block_size: FMS_BLOCK_SIZE as u32,
            total_blocks,
            file_table_start: 1,
            file_table_blocks: FMS_FILE_TABLE_BLOCKS as u32,
            data_start: (1 + FMS_FILE_TABLE_BLOCKS) as u32,
            max_files: FMS_MAX_FILES as u32,
            file_count: 0,
        }
    }

    fn encode_into(&self, block: &mut [u8; FMS_BLOCK_SIZE]) {
        block.fill(0);
        write_u32(block, 0, self.magic);
        write_u32(block, 4, self.version);
        write_u32(block, 8, self.block_size);
        write_u32(block, 12, self.total_blocks);
        write_u32(block, 16, self.file_table_start);
        write_u32(block, 20, self.file_table_blocks);
        write_u32(block, 24, self.data_start);
        write_u32(block, 28, self.max_files);
        write_u32(block, 32, self.file_count);
    }

    fn decode_from(block: &[u8; FMS_BLOCK_SIZE]) -> Self {
        Self {
            magic: read_u32(block, 0),
            version: read_u32(block, 4),
            block_size: read_u32(block, 8),
            total_blocks: read_u32(block, 12),
            file_table_start: read_u32(block, 16),
            file_table_blocks: read_u32(block, 20),
            data_start: read_u32(block, 24),
            max_files: read_u32(block, 28),
            file_count: read_u32(block, 32),
        }
    }

    fn is_valid(&self) -> bool {
        self.magic == FMS_MAGIC
            && self.version == FMS_VERSION
            && self.block_size == FMS_BLOCK_SIZE as u32
            && self.file_table_blocks == FMS_FILE_TABLE_BLOCKS as u32
            && self.max_files == FMS_MAX_FILES as u32
            && self.total_blocks > self.data_start
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FmsFileEntry {
    pub used: bool,
    pub name: [u8; FMS_FILENAME_MAX],
    pub start_block: u32,
    pub block_count: u32,
    pub size: u32,
}

impl FmsFileEntry {
    pub const fn empty() -> Self {
        Self {
            used: false,
            name: [0; FMS_FILENAME_MAX],
            start_block: 0,
            block_count: 0,
            size: 0,
        }
    }

    pub fn set_name(&mut self, input: &str) {
        self.name.fill(0);
        let bytes = input.as_bytes();
        let len = min(bytes.len(), FMS_FILENAME_MAX.saturating_sub(1));
        self.name[..len].copy_from_slice(&bytes[..len]);
    }

    pub fn name_len(&self) -> usize {
        self.name.iter().position(|b| *b == 0).unwrap_or(FMS_FILENAME_MAX)
    }

    pub fn name_bytes(&self) -> &[u8] {
        &self.name[..self.name_len()]
    }
}

pub struct Fms {
    pub superblock: FmsSuperblock,
    mounted: bool,
}

impl Fms {
    pub fn format(device: &mut impl BlockDevice, total_blocks: u32) -> Result<Self, FmsError> {
        if total_blocks <= (1 + FMS_FILE_TABLE_BLOCKS) as u32 {
            return Err(FmsError::BadFs);
        }

        let zero = [0u8; FMS_BLOCK_SIZE];
        for block in 0..total_blocks {
            device.write_block(block, &zero)?;
        }

        let fs = Self {
            superblock: FmsSuperblock::new(total_blocks),
            mounted: true,
        };
        fs.save_superblock(device)?;
        Ok(fs)
    }

    pub fn mount(device: &mut impl BlockDevice) -> Result<Self, FmsError> {
        let mut block = [0u8; FMS_BLOCK_SIZE];
        device.read_block(0, &mut block)?;
        let superblock = FmsSuperblock::decode_from(&block);
        if !superblock.is_valid() {
            return Err(FmsError::BadFs);
        }
        Ok(Self {
            superblock,
            mounted: true,
        })
    }

    pub fn unmount(&mut self) {
        self.mounted = false;
    }

    pub fn list(
        &mut self,
        device: &mut impl BlockDevice,
        out_entries: &mut [FmsFileEntry; FMS_MAX_FILES],
        out_count: &mut usize,
    ) -> Result<(), FmsError> {
        let table = self.load_table(device)?;
        let mut count = 0usize;
        for entry in table.iter() {
            if entry.used {
                out_entries[count] = *entry;
                count += 1;
            }
        }
        *out_count = count;
        Ok(())
    }

    pub fn stat(&mut self, device: &mut impl BlockDevice, name: &str) -> Result<FmsFileEntry, FmsError> {
        validate_name(name)?;
        let table = self.load_table(device)?;
        let idx = find_entry(&table, name).ok_or(FmsError::NotFound)?;
        Ok(table[idx])
    }

    pub fn delete(&mut self, device: &mut impl BlockDevice, name: &str) -> Result<(), FmsError> {
        validate_name(name)?;
        let mut table = self.load_table(device)?;
        let idx = find_entry(&table, name).ok_or(FmsError::NotFound)?;
        table[idx] = FmsFileEntry::empty();
        self.superblock.file_count = self.superblock.file_count.saturating_sub(1);
        self.save_table(device, &table)?;
        self.save_superblock(device)
    }

    pub fn write_file(
        &mut self,
        device: &mut impl BlockDevice,
        name: &str,
        data: &[u8],
    ) -> Result<(), FmsError> {
        validate_name(name)?;
        let mut table = self.load_table(device)?;
        let needed_blocks = bytes_to_blocks(data.len());

        let existing = find_entry(&table, name);
        let replacing = existing.is_some();
        let entry_index = if let Some(idx) = existing {
            table[idx] = FmsFileEntry::empty();
            idx
        } else {
            find_free_entry(&table).ok_or(FmsError::NoSpace)?
        };

        let start_block = if needed_blocks == 0 {
            self.superblock.data_start
        } else {
            find_contiguous_space(&self.superblock, &table, needed_blocks as u32).ok_or(FmsError::NoSpace)?
        };

        for i in 0..needed_blocks {
            let mut block = [0u8; FMS_BLOCK_SIZE];
            let offset = i * FMS_BLOCK_SIZE;
            let end = min(offset + FMS_BLOCK_SIZE, data.len());
            if offset < end {
                block[..(end - offset)].copy_from_slice(&data[offset..end]);
            }
            device.write_block(start_block + i as u32, &block)?;
        }

        let mut entry = FmsFileEntry::empty();
        entry.used = true;
        entry.set_name(name);
        entry.start_block = start_block;
        entry.block_count = needed_blocks as u32;
        entry.size = data.len() as u32;
        table[entry_index] = entry;

        if !replacing {
            self.superblock.file_count = self.superblock.file_count.saturating_add(1);
        }

        self.save_table(device, &table)?;
        self.save_superblock(device)
    }

    pub fn read_file(
        &mut self,
        device: &mut impl BlockDevice,
        name: &str,
        out: &mut [u8],
        out_read: &mut usize,
    ) -> Result<(), FmsError> {
        let entry = self.stat(device, name)?;
        if out.len() < entry.size as usize {
            return Err(FmsError::TooLarge);
        }

        let mut copied = 0usize;
        for i in 0..entry.block_count {
            let mut block = [0u8; FMS_BLOCK_SIZE];
            device.read_block(entry.start_block + i, &mut block)?;
            let remaining = entry.size as usize - copied;
            let to_copy = min(remaining, FMS_BLOCK_SIZE);
            out[copied..copied + to_copy].copy_from_slice(&block[..to_copy]);
            copied += to_copy;
        }

        *out_read = copied;
        Ok(())
    }

    fn save_superblock(&self, device: &mut impl BlockDevice) -> Result<(), FmsError> {
        if !self.mounted {
            return Err(FmsError::BadFs);
        }
        let mut block = [0u8; FMS_BLOCK_SIZE];
        self.superblock.encode_into(&mut block);
        device.write_block(0, &block)
    }

    fn load_table(&self, device: &mut impl BlockDevice) -> Result<[FmsFileEntry; FMS_MAX_FILES], FmsError> {
        if !self.mounted {
            return Err(FmsError::BadFs);
        }

        let mut table_bytes = [0u8; TABLE_BYTES];
        for i in 0..FMS_FILE_TABLE_BLOCKS {
            let mut block = [0u8; FMS_BLOCK_SIZE];
            device.read_block(self.superblock.file_table_start + i as u32, &mut block)?;
            let off = i * FMS_BLOCK_SIZE;
            table_bytes[off..off + FMS_BLOCK_SIZE].copy_from_slice(&block);
        }

        Ok(decode_table(&table_bytes))
    }

    fn save_table(&self, device: &mut impl BlockDevice, entries: &[FmsFileEntry; FMS_MAX_FILES]) -> Result<(), FmsError> {
        if !self.mounted {
            return Err(FmsError::BadFs);
        }

        let table_bytes = encode_table(entries);
        for i in 0..FMS_FILE_TABLE_BLOCKS {
            let mut block = [0u8; FMS_BLOCK_SIZE];
            let off = i * FMS_BLOCK_SIZE;
            block.copy_from_slice(&table_bytes[off..off + FMS_BLOCK_SIZE]);
            device.write_block(self.superblock.file_table_start + i as u32, &block)?;
        }
        Ok(())
    }
}

pub fn fms_strerror(err: FmsError) -> &'static str {
    match err {
        FmsError::Io => "io error",
        FmsError::BadFs => "invalid filesystem",
        FmsError::NotFound => "file not found",
        FmsError::Exists => "file already exists",
        FmsError::NoSpace => "no space left",
        FmsError::BadName => "invalid filename",
        FmsError::TooLarge => "buffer too small",
    }
}

fn write_u32(dst: &mut [u8], offset: usize, value: u32) {
    dst[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(src: &[u8], offset: usize) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&src[offset..offset + 4]);
    u32::from_le_bytes(bytes)
}

fn validate_name(name: &str) -> Result<(), FmsError> {
    if name.is_empty() || name.len() >= FMS_FILENAME_MAX || name.as_bytes().contains(&b'/') {
        return Err(FmsError::BadName);
    }
    Ok(())
}

fn bytes_to_blocks(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        (size + FMS_BLOCK_SIZE - 1) / FMS_BLOCK_SIZE
    }
}

fn find_entry(entries: &[FmsFileEntry; FMS_MAX_FILES], name: &str) -> Option<usize> {
    let target = name.as_bytes();
    entries.iter().position(|entry| entry.used && entry.name_bytes() == target)
}

fn find_free_entry(entries: &[FmsFileEntry; FMS_MAX_FILES]) -> Option<usize> {
    entries.iter().position(|entry| !entry.used)
}

fn find_contiguous_space(sb: &FmsSuperblock, entries: &[FmsFileEntry; FMS_MAX_FILES], needed_blocks: u32) -> Option<u32> {
    if needed_blocks == 0 {
        return Some(sb.data_start);
    }
    if needed_blocks > (sb.total_blocks - sb.data_start) {
        return None;
    }

    let last_start = sb.total_blocks - needed_blocks;
    let mut candidate = sb.data_start;
    while candidate <= last_start {
        let candidate_end = candidate + needed_blocks;
        let mut overlaps = false;

        for entry in entries.iter() {
            if !entry.used || entry.block_count == 0 {
                continue;
            }
            let entry_start = entry.start_block;
            let entry_end = entry_start + entry.block_count;
            if candidate < entry_end && candidate_end > entry_start {
                overlaps = true;
                candidate = entry_end;
                break;
            }
        }

        if !overlaps {
            return Some(candidate);
        }
    }

    None
}

fn decode_table(bytes: &[u8; TABLE_BYTES]) -> [FmsFileEntry; FMS_MAX_FILES] {
    let mut out = [FmsFileEntry::empty(); FMS_MAX_FILES];
    for (i, entry) in out.iter_mut().enumerate() {
        let off = i * FILE_ENTRY_BYTES;
        entry.used = bytes[off] != 0;
        entry.name.copy_from_slice(&bytes[off + 1..off + 1 + FMS_FILENAME_MAX]);
        entry.start_block = read_u32(bytes, off + 1 + FMS_FILENAME_MAX);
        entry.block_count = read_u32(bytes, off + 1 + FMS_FILENAME_MAX + 4);
        entry.size = read_u32(bytes, off + 1 + FMS_FILENAME_MAX + 8);
    }
    out
}

fn encode_table(entries: &[FmsFileEntry; FMS_MAX_FILES]) -> [u8; TABLE_BYTES] {
    let mut out = [0u8; TABLE_BYTES];
    for (i, entry) in entries.iter().enumerate() {
        let off = i * FILE_ENTRY_BYTES;
        out[off] = if entry.used { 1 } else { 0 };
        out[off + 1..off + 1 + FMS_FILENAME_MAX].copy_from_slice(&entry.name);
        write_u32(&mut out, off + 1 + FMS_FILENAME_MAX, entry.start_block);
        write_u32(&mut out, off + 1 + FMS_FILENAME_MAX + 4, entry.block_count);
        write_u32(&mut out, off + 1 + FMS_FILENAME_MAX + 8, entry.size);
    }
    out
}

#[allow(dead_code)]
const _: usize = SUPERBLOCK_FIELDS;
