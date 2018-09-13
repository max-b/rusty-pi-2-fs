use std::{fmt, io};

use traits::BlockDevice;
use byteorder::{ByteOrder, LittleEndian};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CHS {
    head: u8,
    // sector: bits 0..5,
    // cylinder: bits 6..16,
    sector_starting_cylinder: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PartitionEntry {
    pub boot_indicator_flag: u8,
    pub starting_chs: CHS,
    pub partition_type: u8,
    pub ending_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    pub mbr_bootstrap: [u8; 436],
    pub disk_id: [u8; 10],
    pub partition_table_entries: [PartitionEntry; 4],
    pub bootsector_signature: [u8; 2],
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: &mut T) -> Result<MasterBootRecord, Error> {
        let mut mbr_sector = vec![0u8; device.sector_size() as usize];
        
        if let Err(err) = device.read_sector(0, &mut mbr_sector[..]) {
            return Err(Error::Io(err));
        }

        if &mbr_sector[device.sector_size() as usize - 2..] != &[0x55, 0xaa] {
            return Err(Error::BadSignature);
        }

        let mut mbr_bootstrap: [u8; 436] = [0; 436];
        mbr_bootstrap.copy_from_slice(&mbr_sector[0..436]);
        let mut disk_id: [u8; 10] = [0; 10];
        disk_id.copy_from_slice(&mbr_sector[436..446]);

        let mut partition_table_entries: [PartitionEntry; 4] = [Default::default(); 4];
        for (i, partition_entry_bytes) in (&mbr_sector[446..device.sector_size() as usize - 2]).chunks(16).enumerate() {
            if ![0x00, 0x80].contains(&partition_entry_bytes[0]) {
                return Err(Error::UnknownBootIndicator(i as u8));
            }

            let partition_entry = PartitionEntry {
                boot_indicator_flag: partition_entry_bytes[0],
                starting_chs: CHS {
                    head: partition_entry_bytes[1],
                    sector_starting_cylinder: LittleEndian::read_u16(&partition_entry_bytes[2..4]),
                },
                partition_type: partition_entry_bytes[4],
                ending_chs: CHS {
                    head: partition_entry_bytes[5],
                    sector_starting_cylinder: LittleEndian::read_u16(&partition_entry_bytes[6..8]),
                },
                relative_sector: LittleEndian::read_u32(&partition_entry_bytes[8..12]),
                total_sectors: LittleEndian::read_u32(&partition_entry_bytes[12..16]),
            };

            partition_table_entries[i] = partition_entry;
        }

        let mut bootsector_signature: [u8; 2] = [0; 2];
        bootsector_signature.copy_from_slice(&mbr_sector[device.sector_size() as usize - 2..]);

        Ok(MasterBootRecord {
            mbr_bootstrap,
            disk_id,
            partition_table_entries,
            bootsector_signature
        })
    }

    pub fn get_fat_partition_offset(&self) -> Option<u32> {
        for partition in self.partition_table_entries.iter() {
            if partition.partition_type == 0x0b || partition.partition_type == 0x0c {
                return Some(partition.relative_sector);
            }
        }
        None
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("disk_id", &self.disk_id)
            .field("partition_table_entries", &self.partition_table_entries)
            .field("bootsector_signature", &self.bootsector_signature)
            .finish()
    }
}
