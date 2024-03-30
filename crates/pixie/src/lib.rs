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

mod launch;
pub use launch::*;

use core::ops::Range;

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
    /// can not map non-relocatable object at fixed position 
    CantMapNonRelocatableObjectAtFixedPosition,
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
        let start = header.file_range().start as usize;
        let end = header.file_range().end as usize;
        Self {
            header,
            slice: &full_slice[start..end],
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
    pub fn load_convex_hull(&self) -> Result<Range<u64>, PixieError> {
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

pub struct MappedObject<'a> {
    object: &'a Object<'a>,

    /// Load convex hull
    hull: Range<u64>,

    /// Difference between the start of the load convex hull
    /// and where it's actually mapped. For relocatable objects,
    /// it's the base we picked. For non-relocatable objects,
    /// it's zero.
    base_offset: u64,

    /// Allocated memory for the object
    mem: &'a mut [u8],
}

impl<'a> MappedObject<'a> {
    /// If `at` is Some, map at a specific address. This only works
    /// with relocatable objects.
    pub fn new(object: &'a Object, mut at: Option<u64>) -> Result<Self, PixieError> {
        let hull = object.segments().load_convex_hull()?;
        let is_relocatable = hull.start == 0;

        if is_relocatable == false {
            if at.is_some() {
                return Err(PixieError::CantMapNonRelocatableObjectAtFixedPosition);
            }
            else {
                at = Some(hull.start as u64);
            }
        }
        let mem_len = (hull.end - hull.start) as u64;

        let mut map_opts = MmapOptions::new(mem_len);
        // todo: adjust protections after copying segments
        map_opts.prot(MmapProt::READ | MmapProt::WRITE | MmapProt::EXEC);
        if let Some(at) = at {
            map_opts.at(at);
        }

        let res = map_opts.map()?;
        let base_offset = if is_relocatable { res } else { 0 };
        let mem = unsafe {
            core::slice::from_raw_parts_mut(res as _, mem_len as _)
        };

        let mut mapped = Self {
            object,
            hull,
            base_offset,
            mem,
        };
        mapped.copy_load_segments();
        Ok(mapped)
    }

    /// Copies load segments from the file into the memory we mapped
    fn copy_load_segments(&mut self) {
        for seg in self.object.segments().of_type(SegmentType::Load) {
            let mem_start = self.vaddr_to_mem_offset(seg.header().vaddr);
            let dst = &mut self.mem[mem_start..][..seg.slice().len()];
            dst.copy_from_slice(seg.slice());
        }
    }

    /// Convert a vaddr to a memory offset
    pub fn vaddr_to_mem_offset(&self, vaddr: u64) -> usize {
        (vaddr - self.hull.start) as _
    }

    /// Returns a view of (potentially relocated) `mem` for a given range
    pub fn vaddr_slice(&self, range: Range<u64>) -> &[u8] {
        &self.mem[self.vaddr_to_mem_offset(range.start)..self.vaddr_to_mem_offset(range.end)]
    }

    /// Returns true if the object's base offset is zero, which we assume
    /// means it can be mapped anywhere.
    pub fn is_relocatable(&self) -> bool {
        self.base_offset == 0
    }

    /// Returns the offset between the object's base and where we loaded it
    pub fn base_offset(&self) -> u64 {
        self.base_offset
    }

    /// Returns the base address for this executable
    pub fn base(&self) -> u64 {
        self.mem.as_ptr() as _
    }
}