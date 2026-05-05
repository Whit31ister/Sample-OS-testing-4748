use crate::pci::{pci_find_virtio_device, pci_init, PciDeviceInfo};
use crate::pmm::PhysicalMemory;

#[derive(Clone, Copy, Debug, Default)]
pub struct VirtioBlkDevice {
    pub pci: PciDeviceInfo,
    pub io_base: u16,
    pub irq_line: u8,
}

impl VirtioBlkDevice {
    pub fn init(&mut self, _pmm: &mut impl PhysicalMemory) -> bool {
        pci_init();
        let Some(dev) = pci_find_virtio_device() else {
            return false;
        };
        self.pci = dev;

        if (dev.bar0 & 0x1) == 0 {
            return false;
        }

        self.io_base = (dev.bar0 & !0x3) as u16;
        self.irq_line = dev.interrupt_line;
        true
    }

    pub fn read_block(&mut self, _pmm: &impl PhysicalMemory, _sector: u64, _buffer: *mut u8) -> i32 {
        -1
    }

    pub fn write_block(&mut self, _pmm: &impl PhysicalMemory, _sector: u64, _buffer: *const u8) -> i32 {
        -1
    }
}
