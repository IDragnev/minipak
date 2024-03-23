#![no_std]

extern crate alloc;

pub use deku;
use deku::prelude::*;
use encore::prelude::*;

mod manifest;
pub use manifest::*;

mod writer;
pub use writer::*;

mod format;
pub use format::*;

#[derive(displaydoc::Display, Debug)]
pub enum PixieError {
    /// `{0}`
    Deku(DekuError),
    /// `{0}`
    Encore(EncoreError),
}

impl From<DekuError> for PixieError {
    fn from(e: DekuError) -> Self {
        Self::Deku(e)
    }
}

impl From<EncoreError> for PixieError {
    fn from(e: EncoreError) -> Self {
        Self::Encore(e)
    }
}

pub struct Object<'a> {
    header: ObjectHeader,
    slice: &'a [u8],
}

impl<'a> Object<'a> {
    /// Read an ELF object from a given slice
    pub fn new(slice: &'a [u8]) -> Result<Self, PixieError> {
        let input = (slice, 0);
        let (_, header) = ObjectHeader::from_bytes(input)?;

        Ok(Self { slice, header })
    }

    /// Returns the ELF object header
    pub fn header(&self) -> &ObjectHeader {
        &self.header
    }

    /// Returns the full slice
    pub fn slice(&self) -> &[u8] {
        &self.slice
    }
}