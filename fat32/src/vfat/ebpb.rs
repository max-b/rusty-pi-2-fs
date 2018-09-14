use std::fmt;

use byteorder::{ByteOrder, LittleEndian};
use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    pub assembly_block: [u8; 3],
    pub oem_id: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub max_dir_entries: u16,
    pub total_logical_sectors_small: u16,
    pub fat_id: u8,
    pub _sectors_per_fat16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub num_hidden_sectors: u32,
    pub total_logical_sectors_large: u32,
    pub sectors_per_fat: u32,
    pub flags: u16,
    pub fat_version: [u8; 2],
    pub root_cluster_num: u32,
    pub fs_info_sector_num: u16,
    pub backup_boot_sector_num: u16,
    pub _r: [u8; 12],
    pub drive_num: u8,
    pub nt_flags: u8,
    pub signature: u8,
    pub volume_id: u32,
    pub volume_label_string: [u8; 11], // TODO: replace with string?
    pub system_id_string: [u8; 8],
    pub boot_code: [u8; 420],
    pub bootable_partition_signature: [u8; 2],
}

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(device: &mut T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut sector_bytes = vec![0u8; device.sector_size() as usize];
        if let Err(err) = device.read_sector(sector, &mut sector_bytes[..]) {
            return Err(Error::Io(err));
        }

        if &sector_bytes[device.sector_size() as usize - 2..] != &[0x55, 0xaa] {
            return Err(Error::BadSignature);
        }

        let mut assembly_block: [u8; 3] = [0; 3];
        assembly_block.copy_from_slice(&sector_bytes[0..3]);

        let mut oem_id: [u8; 8] = [0; 8];
        oem_id.copy_from_slice(&sector_bytes[3..11]);

        let mut fat_version: [u8; 2] = [0; 2];
        fat_version.copy_from_slice(&sector_bytes[42..44]);

        let mut volume_label_string: [u8; 11] = [0; 11];
        volume_label_string.copy_from_slice(&sector_bytes[71..82]);

        let mut system_id_string: [u8; 8] = [0; 8];
        system_id_string.copy_from_slice(&sector_bytes[82..90]);

        let mut boot_code: [u8; 420] = [0; 420];
        boot_code.copy_from_slice(&sector_bytes[90..510]);

        let mut bootable_partition_signature: [u8; 2] = [0; 2];
        bootable_partition_signature.copy_from_slice(&sector_bytes[510..512]);

        Ok(BiosParameterBlock {
            assembly_block,
            oem_id,
            bytes_per_sector: LittleEndian::read_u16(&sector_bytes[11..13]),
            sectors_per_cluster: sector_bytes[13],
            reserved_sectors: LittleEndian::read_u16(&sector_bytes[14..16]),
            num_fats: sector_bytes[16],
            max_dir_entries: LittleEndian::read_u16(&sector_bytes[17..19]),
            total_logical_sectors_small: LittleEndian::read_u16(&sector_bytes[19..21]),
            fat_id: sector_bytes[21],
            _sectors_per_fat16: 0u16,
            sectors_per_track: LittleEndian::read_u16(&sector_bytes[24..26]),
            num_heads: LittleEndian::read_u16(&sector_bytes[26..28]),
            num_hidden_sectors: LittleEndian::read_u32(&sector_bytes[28..32]),
            total_logical_sectors_large: LittleEndian::read_u32(&sector_bytes[32..36]),
            sectors_per_fat: LittleEndian::read_u32(&sector_bytes[36..40]),
            flags: LittleEndian::read_u16(&sector_bytes[40..42]),
            fat_version,
            root_cluster_num: LittleEndian::read_u32(&sector_bytes[44..48]),
            fs_info_sector_num: LittleEndian::read_u16(&sector_bytes[48..50]),
            backup_boot_sector_num: LittleEndian::read_u16(&sector_bytes[50..52]),
            _r: [0; 12],
            drive_num: sector_bytes[64],
            nt_flags: sector_bytes[65],
            signature: sector_bytes[66],
            volume_id: LittleEndian::read_u32(&sector_bytes[67..71]),
            volume_label_string,
            system_id_string,
            boot_code,
            bootable_partition_signature,
        })
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("oem_id", &self.oem_id)
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("reserved_sectors", &self.reserved_sectors)
            .field("sectors_per_fat", &self.sectors_per_fat)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("num_fats", &self.num_fats)
            .field("root_cluster_num", &self.root_cluster_num)
            .field("drive_num", &self.drive_num)
            .field("volume_id", &self.volume_id)
            .field("signature", &self.signature)
            .field("signature", &self.signature)
            .field(
                "volume_label_string",
                &std::str::from_utf8(&self.volume_label_string).unwrap(),
            ).field(
                "bootable_partition_signature",
                &self.bootable_partition_signature,
            ).finish()
    }
}
