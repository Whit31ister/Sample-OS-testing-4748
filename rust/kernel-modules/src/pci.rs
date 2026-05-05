use crate::io::{in32, out32};

pub const PCI_CONFIG_ADDRESS_PORT: u16 = 0xCF8;
pub const PCI_CONFIG_DATA_PORT: u16 = 0xCFC;

pub const VIRTIO_PCI_VENDOR_ID: u16 = 0x1AF4;
pub const VIRTIO_PCI_DEVICE_MIN: u16 = 0x1000;
pub const VIRTIO_PCI_DEVICE_MAX: u16 = 0x107F;

#[derive(Clone, Copy, Debug, Default)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub bar0: u32,
    pub interrupt_line: u8,
}

pub fn pci_init() {}

pub fn pci_config_read32(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    let address = make_config_address(bus, slot, function, offset);
    out32(PCI_CONFIG_ADDRESS_PORT, address);
    in32(PCI_CONFIG_DATA_PORT)
}

pub fn pci_config_write32(bus: u8, slot: u8, function: u8, offset: u8, value: u32) {
    let address = make_config_address(bus, slot, function, offset);
    out32(PCI_CONFIG_ADDRESS_PORT, address);
    out32(PCI_CONFIG_DATA_PORT, value);
}

pub fn pci_find_virtio_device() -> Option<PciDeviceInfo> {
    for bus in 0u16..=255 {
        for slot in 0u8..32 {
            let id_reg = pci_config_read32(bus as u8, slot, 0, 0x00);
            let vendor_id = (id_reg & 0xFFFF) as u16;
            let device_id = (id_reg >> 16) as u16;
            if vendor_id == 0xFFFF {
                continue;
            }

            if vendor_id == VIRTIO_PCI_VENDOR_ID
                && (VIRTIO_PCI_DEVICE_MIN..=VIRTIO_PCI_DEVICE_MAX).contains(&device_id)
            {
                let bar0 = pci_config_read32(bus as u8, slot, 0, 0x10);
                let irq = pci_config_read32(bus as u8, slot, 0, 0x3C) as u8;
                return Some(PciDeviceInfo {
                    bus: bus as u8,
                    slot,
                    function: 0,
                    vendor_id,
                    device_id,
                    bar0,
                    interrupt_line: irq,
                });
            }
        }
    }
    None
}

fn make_config_address(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    (1 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC)
}
