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
    /// no segments found
    NoSegmentsFound,
    /// could not find segment of type `{0:?}`
    SegmentNotFound(SegmentType),
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

pub struct Segment<'a> {
    header: ProgramHeader,
    slice: &'a [u8],
}

impl<'a> Segment<'a> {
    fn new(header: ProgramHeader, full_slice: &'a [u8]) -> Self {
        let file_rng = header.file_range();
        Self {
            header,
            slice: &full_slice[file_rng],
        }
    }

    pub fn typ(&self) -> SegmentType {
        self.header.r#type
    }

    pub fn header(&self) -> &ProgramHeader {
        &self.header
    }
    
    pub fn slice(&self) -> &[u8] {
        &self.slice
    }
}

#[derive(Default)]
pub struct Segments<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'a> Segments<'a> {
    pub fn all(&self) -> &[Segment] {
        &self.segments
    }

    /// Returns all the segments of a given type
    pub fn of_type(&self, typ: SegmentType) -> impl Iterator<Item = &Segment<'a>> + '_ {
        self.segments
            .iter()
            .filter(move |&seg| seg.typ() == typ)
    }

    /// Returns the first segment of a given type or none
    pub fn find(&self, typ: SegmentType) -> Result<&Segment, PixieError> {
        self.of_type(typ)
            .next()
            .ok_or(PixieError::SegmentNotFound(typ))
    }

    /// Returns a 4K-aligned convex hull of all the load segments
    pub fn load_convex_hull(&self) -> Result<core::ops::Range<usize>, PixieError> {
        self.of_type(SegmentType::Load)
            .map(|s| s.header().mem_range())
            .reduce(|acc, x| { 
                use core::cmp::{ max, min };
                let start = min(acc.start, x.start);
                let end = max(acc.end, x.end);
                start..end
             })
            .ok_or(PixieError::NoSegmentsFound)
    }
}

pub struct Object<'a> {
    header: ObjectHeader,
    slice: &'a [u8],
    segments: Segments<'a>,
}

impl<'a> Object<'a> {
    /// Read an ELF object from a given slice
    pub fn new(slice: &'a [u8]) -> Result<Self, PixieError> {
        let (_, header) = ObjectHeader::from_bytes((slice, 0))?;
        let segments = {
            let mut result = Segments::default();
            let mut segs_input = (&slice[header.ph_offset as usize..], 0);
            for _ in 0..header.ph_count {
                let (rest, phed) = ProgramHeader::from_bytes(segs_input)?;
                result.segments.push(Segment::new(phed, slice));
                segs_input = rest;
            }

            result
        };

        Ok(Self {
            slice,
            header,
            segments,
         })
    }

    /// Returns the ELF object header
    pub fn header(&self) -> &ObjectHeader {
        &self.header
    }

    /// Returns the full slice
    pub fn slice(&self) -> &[u8] {
        &self.slice
    }

    /// Returns all the program's segments
    pub fn segments(&self) -> &Segments {
        &self.segments
    }
}
