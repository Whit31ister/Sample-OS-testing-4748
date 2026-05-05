use fms_core::{BlockDevice, Fms, FmsError};

use crate::pmm::PhysicalMemory;
use crate::virtio_blk::VirtioBlkDevice;

pub struct FsVirtioBridge<'a, P: PhysicalMemory> {
    pub blk: &'a mut VirtioBlkDevice,
    pub pmm: &'a P,
}

impl<'a, P: PhysicalMemory> BlockDevice for FsVirtioBridge<'a, P> {
    fn read_block(&mut self, block_number: u32, out: &mut [u8; fms_core::FMS_BLOCK_SIZE]) -> Result<(), FmsError> {
        if self.blk.read_block(self.pmm, block_number as u64, out.as_mut_ptr()) == 0 {
            Ok(())
        } else {
            Err(FmsError::Io)
        }
    }

    fn write_block(&mut self, block_number: u32, data: &[u8; fms_core::FMS_BLOCK_SIZE]) -> Result<(), FmsError> {
        if self.blk.write_block(self.pmm, block_number as u64, data.as_ptr()) == 0 {
            Ok(())
        } else {
            Err(FmsError::Io)
        }
    }
}

pub fn fs_mount_over_virtio<P: PhysicalMemory>(fs: &mut Option<Fms>, blk: &mut VirtioBlkDevice, pmm: &mut P) -> Result<(), FmsError> {
    let mut bridge = FsVirtioBridge {
        blk,
        pmm: pmm,
    };
    let mounted = Fms::mount(&mut bridge)?;
    *fs = Some(mounted);
    Ok(())
}

pub fn fs_format_over_virtio<P: PhysicalMemory>(
    fs: &mut Option<Fms>,
    blk: &mut VirtioBlkDevice,
    pmm: &mut P,
    total_blocks: u32,
) -> Result<(), FmsError> {
    let mut bridge = FsVirtioBridge {
        blk,
        pmm: pmm,
    };
    let formatted = Fms::format(&mut bridge, total_blocks)?;
    *fs = Some(formatted);
    Ok(())
}
