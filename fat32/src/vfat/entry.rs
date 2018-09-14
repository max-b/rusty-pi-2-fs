use traits;
use vfat::{Dir, File, Metadata};

#[derive(Debug)]
pub enum Entry {
    File(File),
    Dir(Dir),
}

impl traits::Entry for Entry {
    type File = File;
    type Dir = Dir;
    type Metadata = Metadata;

    /// The name of the file or directory corresponding to this entry.
    fn name(&self) -> &str {
        &self.metadata().name
    }

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata {
        match self {
            Entry::Dir(dir) => &dir.metadata,
            Entry::File(file) => &file.metadata,
        }
    }

    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&Self::File> {
        match self {
            Entry::File(file) => Some(file),
            Entry::Dir(_) => None,
        }
    }

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            Entry::Dir(dir) => Some(dir),
            Entry::File(_) => None,
        }
    }

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<Self::File> {
        match self {
            Entry::File(file) => Some(file),
            Entry::Dir(_) => None,
        }
    }

    /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Self::Dir> {
        match self {
            Entry::Dir(dir) => Some(dir),
            Entry::File(_) => None,
        }
    }
}
