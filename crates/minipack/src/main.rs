#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]

mod cli;
mod error;

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() {
    use core::arch::asm;
    asm!("mov rdi, rsp", "call pre_main", options(noreturn))
}

use error::Error;
use encore::prelude::*;
use pixie::{
    ObjectHeader,
    ProgramHeader,
    Writer,
};
use core::ops::Range;

#[no_mangle]
unsafe fn pre_main(stack_top: *mut u8) {
    init_allocator();
    main(Env::read(stack_top)).unwrap();
    syscall::exit(0);
}

#[allow(clippy::unnecessary_wraps)]
fn main(env: Env) -> Result<(), Error> {
    match cli::Args::parse(&env) {
        Ok(args) => {
            return write_compressed(&args);
        },
        Err(err) => {
            println!("{}", err);
            syscall::exit(1);
        },
    }
}

fn write_compressed(args: &cli::Args) -> Result<(), Error> {
    println!("Packing guest {:?}", args.input);
    let guest_file = File::open(args.input)?;
    let guest_map = guest_file.map()?;
    let guest_obj = pixie::Object::new(guest_map.as_ref())?;

    let guest_hull = guest_obj.segments().load_convex_hull()?;
    let mut output = Writer::new(&args.output, 0o755)?;
    relink_stage1(guest_hull, &mut output)?;

    Ok(())
}

fn relink_stage1(guest_hull: Range<u64>, writer: &mut Writer) -> Result<(), Error> {
    let obj = pixie::Object::new(include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/embeds/libstage1.so"
        )
    ))?;

    let hull = obj.segments().load_convex_hull()?;
    assert_eq!(hull.start, 0, "stage1 must be relocatable");

    // Pick a base offset. If our guest is a relocatable executable, pick a
    // random one, otherwise, pick theirs.
    let (base_offset, adjusted_guest_hull) = if guest_hull.start == 0 {
        let offset = 0x800000; // by fair dice roll
        let adjusted_guest_hull = (guest_hull.start + offset)..(guest_hull.end + offset);
        (offset, adjusted_guest_hull)
    } else {
        (guest_hull.start, guest_hull.clone())
    };
    println!("Picked base_offset 0x{:x}", base_offset);

    let hull = (hull.start + base_offset)..(hull.end + base_offset);
    println!("Stage1 hull: {:x?}", hull);
    println!(" Guest hull: {:x?}", guest_hull);
    println!("AGuest hull: {:x?}", adjusted_guest_hull);

    // map stage1 wherever
    let mut mapped = pixie::MappedObject::new(&obj, None)?;
    println!("Loaded stage1");

    // then relocate it as if it was mapped at `base_offset`
    mapped.relocate(base_offset)?;
    println!("Relocated stage1");

    println!("Looking for `entry` in stage1...");
    let entry_sym = mapped.lookup_sym("entry")?;
    let entry_point = base_offset + entry_sym.value;

    let mut load_segs = obj
        .segments()
        .of_type(pixie::SegmentType::Load)
        .collect::<Vec<_>>();

    let out_header = ObjectHeader {
        class: pixie::ElfClass::Elf64,
        endianness: pixie::Endianness::Little,
        version: 1,
        os_abi: pixie::OsAbi::SysV,
        r#type: pixie::ElfType::Exec,
        machine: pixie::ElfMachine::X86_64,
        version_bis: 1,
        entry_point,

        flags: 0,
        hdr_size: ObjectHeader::SIZE,
        // Two additional segments: one for `brk` alignment, and GNU_STACK.
        ph_count: load_segs.len() as u16 + 2,
        ph_offset: ObjectHeader::SIZE as _,
        ph_entsize: ProgramHeader::SIZE,
        // We're not adding any sections, our object will be opaque to debuggers
        sh_count: 0,
        sh_entsize: 0,
        sh_nidx: 0,
        sh_offset: 0,
    };

    writer.write_deku(&out_header)?;

    let static_headers = load_segs.iter().map(|seg| {
        let mut ph = seg.header().clone();
        ph.vaddr += base_offset;
        ph.paddr += base_offset;
        ph
    });
    for ph in static_headers {
        writer.write_deku(&ph)?;
    }

    // Insert dummy segment to offset the `brk` to its original position
    // for the guest, if we can.
    {
        let current_hull = pixie::align_hull(hull);
        let desired_hull = pixie::align_hull(adjusted_guest_hull);

        let pad_size = if current_hull.end > desired_hull.end {
            println!("WARNING: Guest executable is too small, the `brk` will be wrong.");
            0x0
        } else {
            desired_hull.end - current_hull.end
        };
        println!("pad segment size = 0x{:x}", pad_size);

        let ph = pixie::ProgramHeader {
            paddr: current_hull.end,
            vaddr: current_hull.end,
            mem_size: pad_size,
            file_size: 0,
            offset: 0,
            align: 0x1000,
            r#type: pixie::SegmentType::Load,
            flags: ProgramHeader::WRITE | ProgramHeader::READ,
        };
        writer.write_deku(&ph)?;
    }

    // Add a GNU_STACK program header for alignment and make it
    // non-executable.
    {
        let ph = ProgramHeader {
            paddr: 0,
            vaddr: 0,
            mem_size: 0,
            file_size: 0,
            offset: 0,
            align: 0x10,
            r#type: pixie::SegmentType::GnuStack,
            flags: ProgramHeader::WRITE | ProgramHeader::READ,
        };
        writer.write_deku(&ph)?;
    }

    // Sort load segments by file offset and copy them.
    {
        load_segs.sort_by_key(|&seg| seg.header().offset);

        println!("Copying stage1 segments...");
        let copy_start_offset = writer.offset();
        println!("copy_start_offset = 0x{:x}", copy_start_offset);
        let copied_segments = load_segs
            .into_iter()
            .filter(move |seg| seg.header().offset > copy_start_offset);

        for cp_seg in copied_segments {
            let ph = cp_seg.header();
            println!("copying {:?}", ph);

            // Pad space between segments with zeros:
            writer.pad(ph.offset - writer.offset())?;

            // Then copy.
            let start = ph.vaddr;
            let len = ph.file_size;
            let end = start + len;

            writer.write_all(mapped.vaddr_slice(start..end))?;
        }
    }

    // Pad end of last segment with zeros:
    writer.align(0x1000)?;

    Ok(())
}