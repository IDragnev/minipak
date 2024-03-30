use super::prelude::*;

/// A program header (loader view, segment mapped into memory)
#[derive(Derivative, DekuRead, DekuWrite, Clone)]
#[derivative(Debug)]
pub struct ProgramHeader {
    pub r#type: SegmentType,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub flags: u32,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub offset: u64,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub vaddr: u64,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub paddr: u64,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub file_size: u64,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub mem_size: u64,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub align: u64,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Copy, PartialEq)]
#[deku(type = "u32")]
pub enum SegmentType {
    #[deku(id = "0x0")]
    Null,
    #[deku(id = "0x1")]
    Load,
    #[deku(id = "0x2")]
    Dynamic,
    #[deku(id = "0x3")]
    Interp,
    #[deku(id = "0x7")]
    Tls,
    #[deku(id = "0x6474e551")]
    GnuStack,
    #[deku(id_pat = "_")]
    Other(u32),
}

impl ProgramHeader {
    pub const SIZE: u16 = 56;

    pub const EXECUTE: u32 = 1;
    pub const WRITE: u32 = 2;
    pub const READ: u32 = 4;

    pub fn file_range(&self) -> core::ops::Range<u64> {
        self.offset..(self.offset  + self.file_size)
    }

    pub fn mem_range(&self) -> core::ops::Range<u64> {
        self.vaddr..(self.vaddr  + self.mem_size)
    }
}