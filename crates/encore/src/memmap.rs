use crate::error::EncoreError;

pub struct MmapOptions {
    len: u64,
}

impl MmapOptions {
    /// Create a new set of mmmap options
    pub fn new(len: u64) -> Self {
        Self { len }
    }

    /// Create a memory mapping
    pub fn map(&mut self) -> Result<u64, EncoreError> {
        todo!()
    }
}