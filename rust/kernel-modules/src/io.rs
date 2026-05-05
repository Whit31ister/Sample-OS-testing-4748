#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn out8(port: u16, value: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags))
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn out16(port: u16, value: u16) {
    unsafe {
        core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nostack, preserves_flags))
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn out32(port: u16, value: u32) {
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nostack, preserves_flags))
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn in8(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nostack, preserves_flags))
    }
    value
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn in16(port: u16) -> u16 {
    let value: u16;
    unsafe {
        core::arch::asm!("in ax, dx", out("ax") value, in("dx") port, options(nostack, preserves_flags))
    }
    value
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn in32(port: u16) -> u32 {
    let value: u32;
    unsafe {
        core::arch::asm!("in eax, dx", out("eax") value, in("dx") port, options(nostack, preserves_flags))
    }
    value
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
pub fn out8(_port: u16, _value: u8) {}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
pub fn out16(_port: u16, _value: u16) {}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
pub fn out32(_port: u16, _value: u32) {}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
pub fn in8(_port: u16) -> u8 {
    0
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
pub fn in16(_port: u16) -> u16 {
    0
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
pub fn in32(_port: u16) -> u32 {
    0
}
