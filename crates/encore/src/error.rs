#[derive(Debug)]
pub enum EncoreError {
    /// mmap fixed address provided is not aligned to 0x1000: {0}
    MmapMemUnaligned(u64),
    /// mmap file offset provided is not aligned to 0x1000: {0}
    MmapFileUnaligned(u64),
    /// mmap syscall failed
    MmapFailed,
}