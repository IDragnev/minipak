use alloc::string::String;

#[derive(Debug)]
pub enum EncoreError {
    /// mmap fixed address provided is not aligned to 0x1000: {0}
    MmapMemUnaligned(u64),
    /// mmap file offset provided is not aligned to 0x1000: {0}
    MmapFileUnaligned(u64),
    /// mmap syscall failed
    MmapFailed,
    /// Could not open file `0`
    Open(String),
    /// Could not write to file `0`
    Write(String),
    /// Could not statfile `0`
    Stat(String),
}