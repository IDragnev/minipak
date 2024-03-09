#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]

mod cli;

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() {
    use core::arch::asm;
    asm!("mov rdi, rsp", "call pre_main", options(noreturn))
}

use encore::prelude::*;
use pixie::{
    EndMarker,
    Manifest,
    PixieError,
    Resource,
    Writer,
};

#[no_mangle]
unsafe fn pre_main(stack_top: *mut u8) {
    init_allocator();
    main(Env::read(stack_top)).unwrap();
    syscall::exit(0);
}

#[allow(clippy::unnecessary_wraps)]
fn main(env: Env) -> Result<(), PixieError> {
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

fn write_compressed(args: &cli::Args) -> Result<(), PixieError> {
    const PAGE_SIZE: u64 = 4 * 1024;

    let mut output = Writer::new(&args.output, 0o755)?;

    {
        let stage1 = include_bytes!(concat!(env!("OUT_DIR"), "/embeds/release/stage1"));
        output.write_all(stage1)?;
    }

    let guest_offset = output.offset();
    let guest_compressed_len;
    let guest_len;
    {
        let guest = File::open(&args.input)?;
        let guest = guest.map()?;
        let guest = guest.as_ref();
        guest_len = guest.len();

        let guest_compressed = lz4_flex::compress_prepend_size(guest);
        guest_compressed_len = guest_compressed.len();
        output.write_all(&guest_compressed[..])?;
    }

    output.align(PAGE_SIZE)?;

    let manifest_offset = output.offset();
    {
        let manifest = Manifest {
            guest: Resource {
                offset: guest_offset as _,
                len: guest_compressed_len as _,
            },
        };
        output.write_deku(&manifest)?;
    }

    {
        let marker = EndMarker {
            manifest_offset: manifest_offset as _,
        };
        output.write_deku(&marker)?;
    }

    println!(
        "Wrote {} ({:.2}% of input)",
        args.output,
        (output.offset() as f64 / guest_len as f64) * 100.0,
    );

    Ok(())
}