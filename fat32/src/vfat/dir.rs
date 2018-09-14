use std::borrow::Cow;
use std::char::decode_utf16;
use std::ffi::OsStr;
use std::{cmp, fmt, io, mem};

use traits;
use util::VecExt;
use vfat::{Attributes, Date, Metadata, Time, Timestamp};
use vfat::{Cluster, Entry, File, Shared, VFat};

const BYTES_IN_ENTRY: usize = 32;

pub struct Dir {
    pub metadata: Metadata,
    pub start_cluster: Cluster,
    pub vfat: Shared<VFat>,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatRegularDirEntry {
    filename: [u8; 8],
    extension: [u8; 3],
    attributes: Attributes,
    _reserved: u8,
    created_cs: u8,
    created: Timestamp,
    accessed: Date,
    cluster_hi: u16,
    last_modified: Timestamp,
    cluster_lo: u16,
    size: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatLfnDirEntry {
    seq_no: u8,
    pub chars1: [u8; 10],
    attributes: Attributes,
    dirtype: u8,
    checksum: u8,
    pub chars2: [u8; 12],
    _r: [u8; 2],
    pub chars3: [u8; 4],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatUnknownDirEntry {
    pub _bytes: [u8; 32],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl Dir {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        for entry in traits::Dir::entries(self)? {
            let name = match name.as_ref().to_str() {
                None => { return Err(io::Error::new(io::ErrorKind::InvalidInput, "name not valid utf8")) },
                Some(name) => name
            };

            if traits::Entry::name(&entry).eq_ignore_ascii_case(name) {
                return Ok(entry);
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Entry not found"))
    }
}

pub struct DirIter {
    vfat: Shared<VFat>,
    dir_entries: Vec<VFatDirEntry>,
}

impl DirIter {
    fn new(dir: &Dir) -> io::Result<DirIter> {
        let mut vfat = dir.vfat.borrow_mut();
        let mut dir_entries: Vec<VFatDirEntry> = Vec::new();
        let mut buf: Vec<u8> = Vec::new();
        let mut static_buf = [0; BYTES_IN_ENTRY];
        vfat.read_chain(dir.start_cluster, &mut buf)?;
        for entry in buf.chunks(BYTES_IN_ENTRY) {
            static_buf.copy_from_slice(entry);
            unsafe {
                dir_entries.push(mem::transmute(static_buf));
            }
        }
        dir_entries.reverse();

        Ok(DirIter {
            vfat: dir.vfat.clone(),
            dir_entries,
        })
    }
}

impl Iterator for DirIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        println!("== Next ==");
        if self.dir_entries.is_empty() {
            return None;
        }
        
        let mut next = self.dir_entries.pop().unwrap();
        let mut unknown = unsafe { next.unknown };
        while unknown._bytes[0] == 0 || unknown._bytes[0] == 0x0E5 {
            if unknown._bytes[0] == 0x0E5 {
                next = match self.dir_entries.pop() {
                    Some(val) => val,
                    None => { return None; }
                };
                unknown = unsafe { next.unknown };
            } else {
                return None;
            }
        }

        if unknown._bytes[0] == 0x00 {
            return None;
        } else if unknown._bytes[0] == 0xE5 {
            return self.next();
        }

        let mut name = String::new();
        let mut name_bytes = Vec::new();
        let mut is_lfn = false;

        println!("unknown = {:#x?}", unknown);

        while unknown._bytes[11] == 0xF {
            let lfn = unsafe { next.long_filename };

            println!("lfn = {:#x?}", lfn);

            if lfn.seq_no != 0xE5 {

                is_lfn = true;
                let mut tmp_buf = Vec::new();
                tmp_buf.extend_from_slice(&lfn.chars1);
                tmp_buf.extend_from_slice(&lfn.chars2);
                tmp_buf.extend_from_slice(&lfn.chars3);

                tmp_buf.reverse();
                name_bytes.extend_from_slice(&tmp_buf);
            }

            next = self.dir_entries.pop().unwrap();
            unknown = unsafe { next.unknown };
        }

        name_bytes.reverse();

        let reg = unsafe { next.regular };

        println!("reg = {:#x?}", reg);
        if is_lfn {
            let mut chars: Vec<u16> = name_bytes
                .iter()
                .skip(1)
                .step_by(2)
                .zip(name_bytes.iter().step_by(2))
                .map(|(first, second)| ((*first as u16) << 8) | (*second as u16))
                .collect();

            let end = match chars.iter().position(|n| *n == 0 || *n == 0xFF) {
                Some(n) => n,
                None => chars.len(),
            };

            println!("chars = {:x?}", chars);
            println!("end = {}", end);

            name.push_str(
                &decode_utf16(&mut chars[..end].iter().cloned())
                    .map(|r| r.unwrap_or('_'))
                    .collect::<String>(),
            );
        } else {
            let end = match reg.filename.iter().position(|n| *n == 0 || *n == 0x20) {
                Some(n) => n,
                None => reg.filename.len(),
            };

            name.push_str(&String::from_utf8_lossy(&reg.filename[..end]));
            match reg.extension.iter().position(|b| *b == 0x00 || *b == 0x20) {
                Some(pos) => {
                    if pos > 0 {
                        name.push_str(".");
                        name.push_str(&String::from_utf8_lossy(&reg.extension[..pos]));
                    }
                },
                None => {
                    name.push_str(".");
                    name.push_str(&String::from_utf8_lossy(&reg.extension[..]));
                }
            }
        }

        println!("name = {:?}", name);

        let start_cluster = ((reg.cluster_hi as u32) << 16) | (reg.cluster_lo as u32);
        let metadata = Metadata {
            name,
            size: reg.size,
            attributes: reg.attributes,
            created: reg.created,
            accessed: reg.accessed,
            last_modified: reg.last_modified,
        };

        println!("metadata: {:#x?}", metadata);

        if reg.attributes.0 & 0x10 != 0 {
            Some(Entry::Dir(Dir {
                metadata,
                start_cluster: Cluster::from(start_cluster),
                vfat: self.vfat.clone(),
            }))
        } else {
            Some(Entry::File(File::new(
                metadata,
                Cluster::from(start_cluster),
                self.vfat.clone(),
            )))
        }
    }
}

impl traits::Dir for Dir {
    type Entry = Entry;
    type Iter = DirIter;

    fn entries(&self) -> io::Result<Self::Iter> {
        DirIter::new(&self)
    }
}

impl fmt::Debug for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Dir")
            .field("name", &self.metadata.name)
            .field("last_modified", &self.metadata.last_modified)
            .field("created", &self.metadata.created)
            .field("accessed", &self.metadata.accessed)
            .finish()
    }
}
