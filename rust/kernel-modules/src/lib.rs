#![no_std]

pub mod fs_glue;
pub mod io;
pub mod pci;
pub mod pmm;
pub mod virtio_blk;

#[no_mangle]
pub extern "C" fn rust_kernel_entry() {
    // This is where your VirtIO and FS logic starts!
    loop {}
}