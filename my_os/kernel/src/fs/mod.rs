const MAX_FILES: usize = 32;
const MAX_FILE_SIZE: usize = 2048;
const MAX_FILENAME_SIZE: usize = 32;

#[derive(Copy, Clone)]
struct FileEntry {
    name: [u8; MAX_FILENAME_SIZE],
    data: [u8; MAX_FILE_SIZE],
    size: usize,
    used: bool,
}

impl FileEntry {
    const fn new() -> Self {
        Self {
            name: [0; MAX_FILENAME_SIZE],
            data: [0; MAX_FILE_SIZE],
            size: 0,
            used: false,
        }
    }
}

static mut FILESYSTEM: [FileEntry; MAX_FILES] = [FileEntry::new(); MAX_FILES];

pub fn init() {
    // Filesystem is statically initialized.
}

pub fn create_file(name: &str) -> Result<(), &'static str> {
    if name.len() >= MAX_FILENAME_SIZE {
        return Err("Filename too long");
    }

    unsafe {
        // Check if file already exists
        for i in 0..MAX_FILES {
            if FILESYSTEM[i].used && get_name(i) == name {
                return Err("File already exists");
            }
        }

        // Find empty slot
        for i in 0..MAX_FILES {
            if !FILESYSTEM[i].used {
                FILESYSTEM[i].used = true;
                FILESYSTEM[i].size = 0;
                let bytes = name.as_bytes();
                FILESYSTEM[i].name[..bytes.len()].copy_from_slice(bytes);
                for j in bytes.len()..MAX_FILENAME_SIZE {
                    FILESYSTEM[i].name[j] = 0;
                }
                return Ok(());
            }
        }
    }
    Err("No space left on filesystem")
}

pub fn delete_file(name: &str) -> Result<(), &'static str> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILESYSTEM[i].used && get_name(i) == name {
                FILESYSTEM[i].used = false;
                return Ok(());
            }
        }
    }
    Err("File not found")
}

pub fn write_file(name: &str, content: &str) -> Result<(), &'static str> {
    if content.len() > MAX_FILE_SIZE {
        return Err("Content too large");
    }

    unsafe {
        for i in 0..MAX_FILES {
            if FILESYSTEM[i].used && get_name(i) == name {
                let bytes = content.as_bytes();
                FILESYSTEM[i].data[..bytes.len()].copy_from_slice(bytes);
                FILESYSTEM[i].size = bytes.len();
                return Ok(());
            }
        }
    }
    Err("File not found")
}

pub fn read_file(name: &str) -> Result<&'static str, &'static str> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILESYSTEM[i].used && get_name(i) == name {
                let size = FILESYSTEM[i].size;
                let slice = &FILESYSTEM[i].data[..size];
                return core::str::from_utf8(slice).map_err(|_| "Invalid UTF-8");
            }
        }
    }
    Err("File not found")
}

pub fn list_files() -> [Option<&'static str>; MAX_FILES] {
    let mut names = [None; MAX_FILES];
    unsafe {
        for i in 0..MAX_FILES {
            if FILESYSTEM[i].used {
                names[i] = Some(get_name(i));
            }
        }
    }
    names
}

fn get_name(index: usize) -> &'static str {
    unsafe {
        let name_bytes = &FILESYSTEM[index].name;
        let mut len = 0;
        while len < MAX_FILENAME_SIZE && name_bytes[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&name_bytes[..len]).unwrap_or("")
    }
}
