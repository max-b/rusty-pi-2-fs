use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

/// A structure containing a date and time.
#[repr(C, packed)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub time: Time,
    pub date: Date,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub name: String,
    pub size: u32,
    pub attributes: Attributes,
    pub created: Timestamp,
    pub accessed: Date,
    pub last_modified: Timestamp,
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        1980 + ((self.date.0 >> 9) & 0b1111111) as usize
    }
    fn month(&self) -> u8 {
        ((self.date.0 >> 5) & 0xF) as u8
    }

    fn day(&self) -> u8 {
        (self.date.0 & 0b11111) as u8
    }

    fn hour(&self) -> u8 {
        ((self.time.0 >> 11) & 0b11111) as u8
    }

    fn minute(&self) -> u8 {
        ((self.time.0 >> 5) & 0b111111) as u8
    }

    fn second(&self) -> u8 {
        2 * (self.time.0 & 0b11111) as u8
    }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attributes.0 & 0x01 != 0
    }

    fn hidden(&self) -> bool {
        self.attributes.0 & 0x02 != 0
    }

    fn created(&self) -> Self::Timestamp {
        self.created
    }

    fn accessed(&self) -> Self::Timestamp {
        Timestamp {
            date: self.accessed,
            time: Time(0),
        }
    }

    fn modified(&self) -> Self::Timestamp {
        self.last_modified
    }
}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
