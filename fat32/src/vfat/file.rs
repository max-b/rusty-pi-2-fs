use std::cmp::{max, min};
use std::io::{self, SeekFrom};

use traits;
use vfat::{Cluster, Metadata, Shared, VFat};

#[derive(Debug)]
pub struct File {
    pub metadata: Metadata,
    pub start_cluster: Cluster,
    pub vfat: Shared<VFat>,
    pub offset: u32,
    data: Option<Vec<u8>>
}

impl File {
    pub fn new(metadata: Metadata, start_cluster: Cluster, vfat: Shared<VFat>) -> File {
        File {
            metadata,
            start_cluster,
            vfat,
            offset: 0u32,
            data: None,
        }
    }

    pub fn initialize(&mut self) -> io::Result<()> {
        match self.data {
            Some(_) => Ok(()),
            None => {
                let mut tmp_buf = Vec::new();
                self.vfat.borrow_mut().read_chain(self.start_cluster, &mut tmp_buf)?;
                self.data = Some(tmp_buf);
                Ok(())
            }
        }
    }
}

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_offset: i64 = match pos {
            SeekFrom::Start(offset) =>  offset as i64,
            SeekFrom::End(offset) => self.metadata.size as i64 - offset,
            SeekFrom::Current(offset) => self.offset as i64 + offset
        };

        if new_offset < 0 || new_offset > self.metadata.size as i64 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "seek is invalid"));
        }

        self.offset = new_offset as u32;

        Ok(self.offset as u64)
    }
}

impl traits::File for File {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!()
    }

    fn size(&self) -> u64 {
        self.metadata.size as u64
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

        if self.data.is_none() {
            self.initialize()?;
        }

        let num_bytes_to_read = min(buf.len(), (self.metadata.size - self.offset) as usize);

        println!("metadata: {:?}", self.metadata);
        // println!("data: {:#x?}", &self.data.as_ref().unwrap()[..100]);
        println!("buf.len(): {:#?}", buf.len());

        &buf[..num_bytes_to_read].copy_from_slice(&self.data.as_ref().unwrap()[self.offset as usize..self.offset as usize + num_bytes_to_read]);

        io::Seek::seek(self, SeekFrom::Current(num_bytes_to_read as i64))?;
        Ok(num_bytes_to_read)
    }
}
