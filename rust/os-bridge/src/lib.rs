#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};

use kernel_modules::virtio_blk::VirtioBlkDevice;

static RUST_ENTERED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn rust_kernel_entry() {
    /*
     * This function is called from assembly start.
     * Keep it minimal for now: prove Rust code is linked and callable.
     */
    RUST_ENTERED.store(true, Ordering::SeqCst);

    /*
     * Touch the VirtIO type so this crate validates cross-crate integration.
     * Full init needs real PMM and hardware setup from your kernel.
     */
    let _device = VirtioBlkDevice::default();
}
