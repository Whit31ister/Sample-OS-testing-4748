#![allow(static_mut_refs)]

use core::ptr::null_mut;

const PAGE_SIZE: usize = 4096;
const PMM_PAGES: usize = 256;
const PMM_BYTES: usize = PAGE_SIZE * PMM_PAGES;

static mut PMM_ARENA: [u8; PMM_BYTES] = [0; PMM_BYTES];
static mut PMM_NEXT_PAGE: usize = 0;

pub fn init() {
    unsafe {
        PMM_NEXT_PAGE = 0;
    }
}

pub fn alloc_pages(page_count: usize) -> *mut u8 {
    if page_count == 0 {
        return null_mut();
    }

    unsafe {
        let start = PMM_NEXT_PAGE;
        let end = start.saturating_add(page_count);
        if end > PMM_PAGES {
            return null_mut();
        }

        PMM_NEXT_PAGE = end;
        PMM_ARENA.as_mut_ptr().add(start * PAGE_SIZE)
    }
}

pub fn virt_to_phys(virtual_addr: *const u8) -> usize {
    virtual_addr as usize
}

pub fn pages_used() -> usize {
    unsafe { PMM_NEXT_PAGE }
}

pub const fn page_size() -> usize {
    PAGE_SIZE
}
