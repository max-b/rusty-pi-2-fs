use std::collections::HashMap;
use std::{cmp, fmt, io};

use traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `n` _before_
    /// `partition.start` is made to physical sector `n`. Cached sectors before
    /// `partition.start` are the size of a physical sector. An access to a
    /// sector `n` at or after `partition.start` is made to the _logical_ sector
    /// `n - partition.start`. Cached sectors at or after `partition.start` are
    /// the size of a logical sector, `partition.sector_size`.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedDevice
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition,
        }
    }

    /// Maps a user's request for a sector `virt` to the physical sector and
    /// number of physical sectors required to access `virt`.
    fn virtual_to_physical(&self, virt: u64) -> (u64, u64) {
        if self.device.sector_size() == self.partition.sector_size {
            (virt, 1)
        } else if virt < self.partition.start {
            (virt, 1)
        } else {
            let factor = self.partition.sector_size / self.device.sector_size();
            let logical_offset = virt - self.partition.start;
            let physical_offset = logical_offset * factor;
            let physical_sector = self.partition.start + physical_offset;
            (physical_sector, factor)
        }
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        if self.cache.get(&sector).is_none() {
            let data_bytes = self.read_sector_from_disk(sector)?;
            self.cache.insert(
                sector,
                CacheEntry {
                    data: data_bytes,
                    dirty: true,
                },
            );
        }

        let cache = self.cache.get_mut(&sector).unwrap();
        cache.dirty = true;

        Ok(&mut cache.data[..])
    }

    fn read_sector_from_disk(&mut self, virt: u64) -> io::Result<Vec<u8>> {
        let (physical_sector, num_sectors) = self.virtual_to_physical(virt);
        let sector_size = self.partition.sector_size;
        let mut data = vec![0; (sector_size * num_sectors) as usize];
        for i in 0..num_sectors {
            let start = (i * self.device.sector_size()) as usize;
            self.device.read_sector(
                physical_sector + i,
                &mut data[start..start + self.device.sector_size() as usize],
            )?;
        }

        Ok(data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        if self.cache.get(&sector).is_none() {
            let data_bytes = self.read_sector_from_disk(sector)?;
            self.cache.insert(
                sector,
                CacheEntry {
                    data: data_bytes,
                    dirty: false,
                },
            );
        }

        // TODO: Is there a better way to get a reference to the above?
        Ok(&self.cache.get(&sector).as_ref().unwrap().data[..])
    }
}

impl BlockDevice for CachedDevice {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let amount_to_read = cmp::min(self.partition.sector_size as usize, buf.len());
        buf.copy_from_slice(&(self.get(n)?)[..amount_to_read]);
        Ok(amount_to_read)
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
