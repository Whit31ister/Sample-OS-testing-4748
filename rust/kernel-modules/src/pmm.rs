pub trait PhysicalMemory {
    fn alloc_pages(&mut self, page_count: usize) -> Option<*mut u8>;
    fn virt_to_phys(&self, virtual_addr: *const u8) -> usize;
}
