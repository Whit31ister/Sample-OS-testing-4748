pub fn halt() -> ! {
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        core::hint::spin_loop();
    }
}
