use crate::{
    error::EncoreError,
    syscall::{
        self,
        MmapFlags,
        MmapProt,
        FileDescriptor,
    },
};

pub struct MmapOptions {
    prot: MmapProt,
    flags: MmapFlags,
    len: u64,
    file: Option<FileOpts>,
    at: Option<u64>,
}

#[derive(Default, Clone)]
pub struct FileOpts {
    pub fd: FileDescriptor,
    pub offset: u64,
}

impl MmapOptions {
    /// Create a new set of mmmap options
    pub fn new(len: u64) -> Self {
        Self {
            prot: MmapProt::READ | MmapProt::WRITE,
            flags: MmapFlags::ANONYMOUS | MmapFlags::PRIVATE,
            len,
            file: None,
            at: None,
        }
    }

    pub fn file(&mut self, file: FileOpts) -> &mut Self {
        self.file = Some(file);
        self
    }

    pub fn prot(&mut self, prot: MmapProt) -> &mut Self {
        self.prot = prot;
        self
    }

    /// Set flags. Note that `ANONYMOUS` and `PRIVATE` are the default,
    /// and this overwrites them. If `at` is set, `FIXED` is also used.
    pub fn flags(&mut self, flags: MmapFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    /// Specify a fixed address for this mapping (sets the `FIXED` flag)
    pub fn at(&mut self, at: u64) -> &mut Self {
        self.at = Some(at);
        self
    }

    /// Create a memory mapping
    pub fn map(&mut self) -> Result<u64, EncoreError> {
        let mut flags = self.flags;

        if let Some(at) = &self.at {
            if !is_aligned(*at) {
                return Err(EncoreError::MmapMemUnaligned(*at));
            }
            flags.insert(MmapFlags::FIXED);
        }

        if let Some(file) = &self.file {
            if !is_aligned(file.offset) {
                return Err(EncoreError::MmapFileUnaligned(file.offset));
            }
            flags.remove(MmapFlags::ANONYMOUS);
        }

        let file = self.file.clone().unwrap_or_default();
        let addr = self.at.unwrap_or_default();

        let res = unsafe {
            syscall::mmap(addr, self.len, self.prot, flags, file.fd, file.offset)
        };
        if res as i64 == -1 {
            return Err(EncoreError::MmapFailed);
        }
        Ok(res)
    }
}

fn is_aligned(x: u64) -> bool {
    x & 0xFFF == 0
}