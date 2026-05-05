use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

use crate::io::{in16, in32, in8, out16, out32, out8};
use crate::pci::{pci_find_virtio_device, pci_init, PciDeviceInfo};
use crate::pmm::PhysicalMemory;

pub const VIRTIO_BLK_SECTOR_SIZE: usize = 512;
pub const VIRTIO_BLK_QUEUE_SIZE: u16 = 8;

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 0x01;
const VIRTIO_STATUS_DRIVER: u8 = 0x02;
const VIRTIO_STATUS_DRIVER_OK: u8 = 0x04;
const VIRTIO_STATUS_FEATURES_OK: u8 = 0x08;
const VIRTIO_STATUS_FAILED: u8 = 0x80;

const VIRTIO_DESC_F_NEXT: u16 = 0x01;
const VIRTIO_DESC_F_WRITE: u16 = 0x02;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;
const VIRTIO_BLK_S_OK: u8 = 0;

const VIRTIO_PCI_DEVICE_FEATURES: u16 = 0x00;
const VIRTIO_PCI_GUEST_FEATURES: u16 = 0x04;
const VIRTIO_PCI_QUEUE_ADDRESS: u16 = 0x08;
const VIRTIO_PCI_QUEUE_SIZE: u16 = 0x0C;
const VIRTIO_PCI_QUEUE_SELECT: u16 = 0x0E;
const VIRTIO_PCI_QUEUE_NOTIFY: u16 = 0x10;
const VIRTIO_PCI_DEVICE_STATUS: u16 = 0x12;
const VIRTIO_PCI_ISR_STATUS: u16 = 0x13;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; VIRTIO_BLK_QUEUE_SIZE as usize],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtqUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtqUsedElem; VIRTIO_BLK_QUEUE_SIZE as usize],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtioBlkReqHeader {
    pub req_type: u32,
    pub reserved: u32,
    pub sector: u64,
}

pub struct VirtioBlkDevice {
    pub pci: PciDeviceInfo,
    pub io_base: u16,
    pub irq_line: u8,
    pub queue_size: u16,
    pub last_used_idx: u16,
    pub desc: *mut VirtqDesc,
    pub avail: *mut VirtqAvail,
    pub used: *mut VirtqUsed,
    pub req_header: *mut VirtioBlkReqHeader,
    pub req_status: *mut u8,
    pub req_header_phys: u64,
    pub req_status_phys: u64,
}

impl Default for VirtioBlkDevice {
    fn default() -> Self {
        Self {
            pci: PciDeviceInfo::default(),
            io_base: 0,
            irq_line: 0,
            queue_size: 0,
            last_used_idx: 0,
            desc: core::ptr::null_mut(),
            avail: core::ptr::null_mut(),
            used: core::ptr::null_mut(),
            req_header: core::ptr::null_mut(),
            req_status: core::ptr::null_mut(),
            req_header_phys: 0,
            req_status_phys: 0,
        }
    }
}

impl VirtioBlkDevice {
    pub fn init(&mut self, pmm: &mut impl PhysicalMemory) -> bool {
        *self = Self::default();
        pci_init();
        self.pci = match pci_find_virtio_device() {
            Some(dev) => dev,
            None => return false,
        };

        if (self.pci.bar0 & 0x1) == 0 {
            return false;
        }
        self.io_base = (self.pci.bar0 & !0x3) as u16;
        self.irq_line = self.pci.interrupt_line;

        write8(self.io_base, VIRTIO_PCI_DEVICE_STATUS, 0);
        write8(self.io_base, VIRTIO_PCI_DEVICE_STATUS, VIRTIO_STATUS_ACKNOWLEDGE);
        write8(
            self.io_base,
            VIRTIO_PCI_DEVICE_STATUS,
            VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER,
        );

        let _features = read32(self.io_base, VIRTIO_PCI_DEVICE_FEATURES);
        let guest_features = 0u32;
        write32(self.io_base, VIRTIO_PCI_GUEST_FEATURES, guest_features);

        write8(
            self.io_base,
            VIRTIO_PCI_DEVICE_STATUS,
            VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK,
        );
        if (read8(self.io_base, VIRTIO_PCI_DEVICE_STATUS) & VIRTIO_STATUS_FEATURES_OK) == 0 {
            write8(self.io_base, VIRTIO_PCI_DEVICE_STATUS, VIRTIO_STATUS_FAILED);
            return false;
        }

        if !self.setup_queue(pmm) {
            write8(self.io_base, VIRTIO_PCI_DEVICE_STATUS, VIRTIO_STATUS_FAILED);
            return false;
        }

        write8(
            self.io_base,
            VIRTIO_PCI_DEVICE_STATUS,
            VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK | VIRTIO_STATUS_DRIVER_OK,
        );

        let _ = read8(self.io_base, VIRTIO_PCI_ISR_STATUS);
        true
    }

    pub fn read_block(&mut self, pmm: &impl PhysicalMemory, sector: u64, buffer: *mut u8) -> i32 {
        self.submit_rw(pmm, VIRTIO_BLK_T_IN, sector, buffer)
    }

    pub fn write_block(&mut self, pmm: &impl PhysicalMemory, sector: u64, buffer: *const u8) -> i32 {
        self.submit_rw(pmm, VIRTIO_BLK_T_OUT, sector, buffer as *mut u8)
    }

    fn setup_queue(&mut self, pmm: &mut impl PhysicalMemory) -> bool {
        write16(self.io_base, VIRTIO_PCI_QUEUE_SELECT, 0);
        let hw_qsize = read16(self.io_base, VIRTIO_PCI_QUEUE_SIZE);
        if hw_qsize == 0 {
            return false;
        }
        self.queue_size = hw_qsize.min(VIRTIO_BLK_QUEUE_SIZE);

        let desc_bytes = size_of::<VirtqDesc>() * self.queue_size as usize;
        let avail_bytes = 4 + (2 * self.queue_size as usize);
        let used_offset = align_up(desc_bytes + avail_bytes, 4096);
        let used_bytes = 4 + (size_of::<VirtqUsedElem>() * self.queue_size as usize);
        let total_bytes = used_offset + used_bytes;
        let page_count = (total_bytes + 4095) / 4096;

        let queue_virt = match pmm.alloc_pages(page_count) {
            Some(p) => p,
            None => return false,
        };
        let queue_phys = pmm.virt_to_phys(queue_virt) as u64;
        if (queue_phys & 0xFFF) != 0 {
            return false;
        }

        unsafe {
            core::ptr::write_bytes(queue_virt, 0, page_count * 4096);
        }

        self.desc = queue_virt as *mut VirtqDesc;
        self.avail = unsafe { queue_virt.add(desc_bytes) } as *mut VirtqAvail;
        self.used = unsafe { queue_virt.add(used_offset) } as *mut VirtqUsed;
        self.last_used_idx = 0;

        let req_page = match pmm.alloc_pages(1) {
            Some(p) => p,
            None => return false,
        };
        unsafe {
            core::ptr::write_bytes(req_page, 0, 4096);
        }
        self.req_header = req_page as *mut VirtioBlkReqHeader;
        self.req_header_phys = pmm.virt_to_phys(req_page) as u64;
        self.req_status = unsafe { req_page.add(64) };
        self.req_status_phys = self.req_header_phys + 64;

        write32(self.io_base, VIRTIO_PCI_QUEUE_ADDRESS, (queue_phys >> 12) as u32);
        true
    }

    fn submit_rw(&mut self, pmm: &impl PhysicalMemory, req_type: u32, sector: u64, buffer: *mut u8) -> i32 {
        if self.desc.is_null() || self.avail.is_null() || self.used.is_null() || self.req_header.is_null() {
            return -1;
        }

        let buffer_phys = pmm.virt_to_phys(buffer) as u64;
        unsafe {
            (*self.req_header).req_type = req_type;
            (*self.req_header).reserved = 0;
            (*self.req_header).sector = sector;
            *self.req_status = 0xFF;

            (*self.desc.add(0)).addr = self.req_header_phys;
            (*self.desc.add(0)).len = size_of::<VirtioBlkReqHeader>() as u32;
            (*self.desc.add(0)).flags = VIRTIO_DESC_F_NEXT;
            (*self.desc.add(0)).next = 1;

            (*self.desc.add(1)).addr = buffer_phys;
            (*self.desc.add(1)).len = VIRTIO_BLK_SECTOR_SIZE as u32;
            (*self.desc.add(1)).flags = VIRTIO_DESC_F_NEXT | if req_type == VIRTIO_BLK_T_IN { VIRTIO_DESC_F_WRITE } else { 0 };
            (*self.desc.add(1)).next = 2;

            (*self.desc.add(2)).addr = self.req_status_phys;
            (*self.desc.add(2)).len = 1;
            (*self.desc.add(2)).flags = VIRTIO_DESC_F_WRITE;
            (*self.desc.add(2)).next = 0;

            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

            let avail_idx = (*self.avail).idx;
            let slot = avail_idx % self.queue_size;
            (*self.avail).ring[slot as usize] = 0;
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            (*self.avail).idx = avail_idx.wrapping_add(1);
        }

        write16(self.io_base, VIRTIO_PCI_QUEUE_NOTIFY, 0);

        unsafe {
            while self.last_used_idx == read_volatile(&(*self.used).idx) {}
            self.last_used_idx = self.last_used_idx.wrapping_add(1);
            if read_volatile(self.req_status) != VIRTIO_BLK_S_OK {
                return -1;
            }
        }
        0
    }
}

fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[inline]
fn read8(base: u16, off: u16) -> u8 {
    in8(base.wrapping_add(off))
}
#[inline]
fn read16(base: u16, off: u16) -> u16 {
    in16(base.wrapping_add(off))
}
#[inline]
fn read32(base: u16, off: u16) -> u32 {
    in32(base.wrapping_add(off))
}
#[inline]
fn write8(base: u16, off: u16, value: u8) {
    out8(base.wrapping_add(off), value)
}
#[inline]
fn write16(base: u16, off: u16, value: u16) {
    out16(base.wrapping_add(off), value)
}
#[inline]
fn write32(base: u16, off: u16, value: u32) {
    out32(base.wrapping_add(off), value)
}

#[allow(dead_code)]
#[inline]
fn mmio_write<T>(ptr: *mut T, value: T) {
    unsafe { write_volatile(ptr, value) }
}
