use fms_core::{Fms, FmsError};

use crate::pmm::PhysicalMemory;
use crate::virtio_blk::VirtioBlkDevice;

pub fn fs_mount_over_virtio<P: PhysicalMemory>(
    _fs: &mut Option<Fms>,
    _blk: &mut VirtioBlkDevice,
    _pmm: &mut P,
) -> Result<(), FmsError> {
    Err(FmsError::Io)
}

pub fn fs_format_over_virtio<P: PhysicalMemory>(
    _fs: &mut Option<Fms>,
    _blk: &mut VirtioBlkDevice,
    _pmm: &mut P,
    _total_blocks: u32,
) -> Result<(), FmsError> {
    Err(FmsError::Io)
}
