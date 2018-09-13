use std::cmp::min;
use std::io;
use std::mem::size_of;
use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};
use mbr::MasterBootRecord;
use traits::{BlockDevice, FileSystem};
use vfat::{BiosParameterBlock, CachedDevice, Partition};
use vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Shared, Status};

const FAT_ENTRY_SIZE: u16 = 4;

#[derive(Debug)]
pub struct VFat {
    device: CachedDevice,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_dir_cluster: Cluster,
}

impl VFat {
    pub fn from<T>(mut device: T) -> Result<Shared<VFat>, Error>
    where
        T: BlockDevice + 'static,
    {
        let mbr = MasterBootRecord::from(&mut device)?;
        let bpb_offset = mbr.get_fat_partition_offset();
        if bpb_offset.is_none() {
            return Err(Error::NotFound);
        }

        let bpb = BiosParameterBlock::from(&mut device, bpb_offset.unwrap() as u64)?;
        Ok(Shared::new(VFat {
            device: CachedDevice::new(
                device,
                Partition {
                    start: (bpb_offset.unwrap() + 1) as u64,
                    sector_size: bpb.bytes_per_sector as u64,
                },
            ),
            bytes_per_sector: bpb.bytes_per_sector as u16,
            sectors_per_cluster: bpb.sectors_per_cluster,
            sectors_per_fat: bpb.sectors_per_fat as u32,
            fat_start_sector: bpb.reserved_sectors as u64,
            data_start_sector: (bpb.sectors_per_fat as u64) * (bpb.num_fats as u64)
                + (bpb.reserved_sectors as u64),
            root_dir_cluster: Cluster::from(bpb.root_cluster_num),
        }))
    }

    /// A method to read from an offset of a cluster into a buffer
    fn read_cluster(
        &mut self,
        cluster: Cluster,
        // offset: usize, TODO: WAT?
        buf: &mut [u8],
    ) -> io::Result<usize> {
        let start_read_sector =
            self.data_start_sector as u64 + cluster.0 as u64 * self.sectors_per_cluster as u64;
        self.device.read_sector(start_read_sector, &mut buf[..])
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut cluster_cursor = start;
        let mut bytes_read = 0usize;

        loop {
            let fat_entry = self.fat_entry(cluster_cursor)?;
            cluster_cursor = match fat_entry.status() {
                Status::Data(next) => {
                    buf.resize_default(
                        buf.len()
                            + self.bytes_per_sector as usize * self.sectors_per_cluster as usize,
                    );
                    bytes_read += self.read_cluster(cluster_cursor, &mut buf[bytes_read..])?;
                    next
                }
                Status::Eoc(_) => {
                    buf.resize_default(
                        buf.len()
                            + self.bytes_per_sector as usize * self.sectors_per_cluster as usize,
                    );
                    bytes_read += self.read_cluster(cluster_cursor, &mut buf[bytes_read..])?;

                    return Ok(bytes_read);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Fat entry is Free/Reserved/Bad",
                    ));
                }
            }
        }
    }

    /// A method to return a reference to a `FatEntry` for a cluster where the
    /// reference points directly into a cached sector.
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<FatEntry> {
        let entries_per_sector = (self.bytes_per_sector / FAT_ENTRY_SIZE) as u32;
        // index of the sector that contains this cluster. e.g. if there are
        // 10 fat entries per sector and we want sector 12, this should be 1
        let fat_sector_index = cluster.0 / entries_per_sector;
        // index of the entry within the given sector, e.g. if we have the
        // sector with entries 10-20 and we want sectore 12, this should be 2
        let fat_entry_index = cluster.0 % entries_per_sector;

        let fat_entries = self.device.get(fat_sector_index.into())?;
        let idx = (fat_entry_index * FAT_ENTRY_SIZE as u32) as usize;
        let raw_fat_entry = LittleEndian::read_u32(&fat_entries[idx..idx + 4]);
        Ok(FatEntry(raw_fat_entry))
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = ::traits::Dummy;
    type Dir = ::traits::Dummy;
    type Entry = ::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }

    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only file system")
    }

    fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
    where
        P: AsRef<Path>,
    {
        unimplemented!("read only file system")
    }

    fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        unimplemented!("read only file system")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only file system")
    }
}
