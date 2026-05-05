use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use fms_core::{
    BlockDevice, Fms, FmsError, FmsFileEntry, FMS_BLOCK_SIZE, FMS_DEFAULT_TOTAL_BLOCKS, FMS_MAX_FILES,
};

struct HostDisk {
    file: File,
    total_blocks: u32,
}

impl HostDisk {
    fn create(path: &str, total_blocks: u32) -> Result<Self, FmsError> {
        let mut file = File::create(path).map_err(|_| FmsError::Io)?;
        let zero = [0u8; FMS_BLOCK_SIZE];
        for _ in 0..total_blocks {
            file.write_all(&zero).map_err(|_| FmsError::Io)?;
        }
        file.flush().map_err(|_| FmsError::Io)?;
        Ok(Self { file, total_blocks })
    }

    fn open(path: &str) -> Result<Self, FmsError> {
        let mut file = File::options()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|_| FmsError::Io)?;
        let size = file.seek(SeekFrom::End(0)).map_err(|_| FmsError::Io)?;
        if size == 0 || size % FMS_BLOCK_SIZE as u64 != 0 {
            return Err(FmsError::BadFs);
        }

        Ok(Self {
            file,
            total_blocks: (size / FMS_BLOCK_SIZE as u64) as u32,
        })
    }
}

impl BlockDevice for HostDisk {
    fn read_block(&mut self, block_number: u32, out: &mut [u8; FMS_BLOCK_SIZE]) -> Result<(), FmsError> {
        if block_number >= self.total_blocks {
            return Err(FmsError::Io);
        }
        self.file
            .seek(SeekFrom::Start(block_number as u64 * FMS_BLOCK_SIZE as u64))
            .map_err(|_| FmsError::Io)?;
        self.file.read_exact(out).map_err(|_| FmsError::Io)
    }

    fn write_block(&mut self, block_number: u32, data: &[u8; FMS_BLOCK_SIZE]) -> Result<(), FmsError> {
        if block_number >= self.total_blocks {
            return Err(FmsError::Io);
        }
        self.file
            .seek(SeekFrom::Start(block_number as u64 * FMS_BLOCK_SIZE as u64))
            .map_err(|_| FmsError::Io)?;
        self.file.write_all(data).map_err(|_| FmsError::Io)?;
        self.file.flush().map_err(|_| FmsError::Io)
    }
}

fn usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} mkfs <disk.img> [total_blocks]");
    eprintln!("  {program} files <disk.img>");
    eprintln!("  {program} save <disk.img> <host_file> <fs_name>");
    eprintln!("  {program} open <disk.img> <fs_name> [host_file]");
    eprintln!("  {program} delete <disk.img> <fs_name>");
    eprintln!("  {program} info <disk.img> <fs_name>");
}

fn parse_blocks(s: &str) -> Result<u32, FmsError> {
    s.parse::<u32>().map_err(|_| FmsError::BadFs)
}

fn print_error(err: FmsError) {
    let msg = match err {
        FmsError::Io => "io error",
        FmsError::BadFs => "invalid filesystem",
        FmsError::NotFound => "file not found",
        FmsError::Exists => "file already exists",
        FmsError::NoSpace => "no space left",
        FmsError::BadName => "invalid filename",
        FmsError::TooLarge => "buffer too small",
    };
    eprintln!("{msg}");
}

fn list_files(fs: &mut Fms, disk: &mut HostDisk) -> Result<(), FmsError> {
    let mut entries = [FmsFileEntry::empty(); FMS_MAX_FILES];
    let mut count = 0usize;
    fs.list(disk, &mut entries, &mut count)?;
    println!("{:<32} {:>10} {:>12} {:>12}", "name", "size", "start_block", "blocks");
    for entry in entries.iter().take(count) {
        let name = std::str::from_utf8(entry.name_bytes()).unwrap_or("<bad utf8>");
        println!(
            "{:<32} {:>10} {:>12} {:>12}",
            name, entry.size, entry.start_block, entry.block_count
        );
    }
    Ok(())
}

fn run() -> Result<(), FmsError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage(&args[0]);
        return Err(FmsError::BadFs);
    }

    match args[1].as_str() {
        "mkfs" => {
            if args.len() != 3 && args.len() != 4 {
                usage(&args[0]);
                return Err(FmsError::BadFs);
            }
            let total_blocks = if args.len() == 4 {
                parse_blocks(&args[3])?
            } else {
                FMS_DEFAULT_TOTAL_BLOCKS
            };
            let mut disk = HostDisk::create(&args[2], total_blocks)?;
            let _ = Fms::format(&mut disk, total_blocks)?;
            println!(
                "formatted {} with {} blocks ({} bytes each)",
                args[2], total_blocks, FMS_BLOCK_SIZE
            );
            Ok(())
        }
        "files" => {
            if args.len() != 3 {
                usage(&args[0]);
                return Err(FmsError::BadFs);
            }
            let mut disk = HostDisk::open(&args[2])?;
            let mut fs = Fms::mount(&mut disk)?;
            list_files(&mut fs, &mut disk)
        }
        "save" => {
            if args.len() != 5 {
                usage(&args[0]);
                return Err(FmsError::BadFs);
            }
            let data = std::fs::read(&args[3]).map_err(|_| FmsError::Io)?;
            let mut disk = HostDisk::open(&args[2])?;
            let mut fs = Fms::mount(&mut disk)?;
            fs.write_file(&mut disk, &args[4], &data)
        }
        "open" => {
            if args.len() != 4 && args.len() != 5 {
                usage(&args[0]);
                return Err(FmsError::BadFs);
            }
            let mut disk = HostDisk::open(&args[2])?;
            let mut fs = Fms::mount(&mut disk)?;
            let entry = fs.stat(&mut disk, &args[3])?;
            let mut buf = vec![0u8; entry.size as usize];
            let mut read = 0usize;
            fs.read_file(&mut disk, &args[3], &mut buf, &mut read)?;
            if args.len() == 5 {
                std::fs::write(&args[4], &buf[..read]).map_err(|_| FmsError::Io)?;
            } else {
                std::io::stdout()
                    .write_all(&buf[..read])
                    .map_err(|_| FmsError::Io)?;
            }
            Ok(())
        }
        "delete" => {
            if args.len() != 4 {
                usage(&args[0]);
                return Err(FmsError::BadFs);
            }
            let mut disk = HostDisk::open(&args[2])?;
            let mut fs = Fms::mount(&mut disk)?;
            fs.delete(&mut disk, &args[3])
        }
        "info" => {
            if args.len() != 4 {
                usage(&args[0]);
                return Err(FmsError::BadFs);
            }
            let mut disk = HostDisk::open(&args[2])?;
            let mut fs = Fms::mount(&mut disk)?;
            let entry = fs.stat(&mut disk, &args[3])?;
            let name = std::str::from_utf8(entry.name_bytes()).unwrap_or("<bad utf8>");
            println!("name: {name}");
            println!("size: {}", entry.size);
            println!("start_block: {}", entry.start_block);
            println!("blocks: {}", entry.block_count);
            Ok(())
        }
        _ => {
            usage(&args[0]);
            Err(FmsError::BadFs)
        }
    }
}

fn main() {
    if let Err(err) = run() {
        print_error(err);
        std::process::exit(1);
    }
}
