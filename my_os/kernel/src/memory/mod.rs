const PAGE_SIZE: usize = 4096;
const MEMORY_POOL_SIZE: usize = 1024 * 1024; // 1MB for demo

static mut MEMORY_POOL: [u8; MEMORY_POOL_SIZE] = [0; MEMORY_POOL_SIZE];
static mut NEXT_FREE_PAGE: usize = 0;

pub fn init() {
    unsafe {
        NEXT_FREE_PAGE = 0;
    }
}

pub fn allocate_page() -> Option<*mut u8> {
    unsafe {
        if NEXT_FREE_PAGE + PAGE_SIZE <= MEMORY_POOL_SIZE {
            let addr = MEMORY_POOL.as_mut_ptr().add(NEXT_FREE_PAGE);
            NEXT_FREE_PAGE += PAGE_SIZE;
            Some(addr)
        } else {
            None
        }
    }
}

pub fn used_memory() -> usize {
    unsafe { NEXT_FREE_PAGE }
}

pub fn total_memory() -> usize {
    MEMORY_POOL_SIZE
}
