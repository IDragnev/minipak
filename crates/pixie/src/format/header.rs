use super::prelude::*;

/// An ELF object header
#[derive(Derivative, Clone, PartialEq, DekuRead, DekuWrite)]
#[derivative(Debug)]
#[deku(magic = b"\x7FELF")]
pub struct ObjectHeader {
    #[derivative(Debug = "ignore")]
    pub class: ElfClass,
    pub endianness: Endianness,
    /// Always 1
    pub version: u8,
    #[deku(pad_bytes_after = "8")]
    pub os_abi: OsAbi,
    pub r#type: ElfType,
    pub machine: ElfMachine,
    /// Always 1
    pub version_bis: u32,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub entry_point: u64,
    
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub ph_offset: u64,
    #[derivative(Debug(format_with = "hex_fmt"))]
    pub sh_offset: u64,

    #[derivative(Debug(format_with = "hex_fmt"))]
    pub flags: u32,
    pub hdr_size: u16,

    pub ph_entsize: u16,
    pub ph_count: u16,

    pub sh_entsize: u16,
    pub sh_count: u16,
    pub sh_nidx: u16,
}

impl ObjectHeader {
    pub const SIZE: u16 = 64;
}

#[derive(Clone, Copy, DekuRead, DekuWrite, Debug, PartialEq)]
#[deku(type = "u8")]
pub enum ElfClass {
    #[deku(id = "1")]
    Elf32,
    #[deku(id = "2")]
    Elf64,
    #[deku(id_pat = "_")]
    Other(u8),
}

#[derive(Clone, Copy, DekuRead, DekuWrite, Debug, PartialEq)]
#[deku(type = "u8")]
pub enum Endianness {
    #[deku(id = "0x1")]
    Little,
    #[deku(id = "0x2")]
    Big,
    #[deku(id_pat = "_")]
    Other(u8),
}

#[derive(Clone, Copy, DekuRead, DekuWrite, Debug, PartialEq)]
#[deku(type = "u8")]
pub enum OsAbi {
    #[deku(id = "0x0")]
    SysV,
    #[deku(id_pat = "_")]
    Other(u8),
}

#[derive(Clone, Copy, DekuRead, DekuWrite, Debug, PartialEq)]
#[deku(type = "u16")]
pub enum ElfMachine {
    #[deku(id = "0x03")]
    X86,
    #[deku(id = "0x3e")]
    X86_64,
    #[deku(id_pat = "_")]
    Other(u16),
}

#[derive(Clone, Copy, DekuRead, DekuWrite, Debug, PartialEq)]
#[deku(type = "u16")]
pub enum ElfType {
    #[deku(id = "0x2")]
    Exec,
    #[deku(id = "0x3")]
    Dyn,
    #[deku(id_pat = "_")]
    Other(u16),
}