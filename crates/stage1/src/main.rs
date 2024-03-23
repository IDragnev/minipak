#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() {
    use core::arch::asm;
    asm!("mov rdi, rsp", "call pre_main", options(noreturn))
}

use encore::prelude::*;
use pixie::{
    Manifest,
    PixieError,
};

#[no_mangle]
unsafe fn pre_main(stack_top: *mut u8) {
    init_allocator();
    main(Env::read(stack_top)).unwrap();
    syscall::exit(0);
}

#[allow(clippy::unnecessary_wraps)]
fn main(env: Env) -> Result<(), PixieError> {
    println!("Hello from stage1!");

    let host = File::open("/proc/self/exe")?;
    let host = host.map()?;
    let host = host.as_ref();
    let manifest = Manifest::read_from_full_slice(host)?;

    let guest_range = manifest.guest.as_range();
    println!("The guest is at {:x?}", guest_range);

    let guest_slice = &host[guest_range];
    let uncompressed_guest = lz4_flex::decompress_size_prepended(guest_slice).expect("invalid lz4 payload");

    let guest_obj = pixie::Object::new(&uncompressed_guest[..])?;
    println!("Parsed {:#?}", guest_obj.header());

    /*
    let tmp_path = "/tmp/minipak-guest";
    {
        let mut guest = File::create(tmp_path, 0o755)?;
        guest.write_all(&uncompressed_guest[..])?;
    }

    {
        extern crate alloc;
        
        let tmp_path_nullter = format!("{}\0", tmp_path);
        let argv: Vec<*const u8> = env
            .args
            .iter()
            .copied()
            .map(str::as_ptr)
            .chain(core::iter::once(core::ptr::null()))
            .collect();
        let envp: Vec<*const u8> = env
            .vars
            .iter()
            .copied()
            .map(str::as_ptr)
            .chain(core::iter::once(core::ptr::null()))
            .collect();

            unsafe {
                use core::arch::asm;

                asm!(
                    "syscall",
                    in("rax") 59, // `execve` syscall
                    in("rdi") tmp_path_nullter.as_ptr(), // `filename`
                    in("rsi") argv.as_ptr(), // `argv`
                    in("rdx") envp.as_ptr(), // `envp`
                    options(noreturn),
                )
            }
    }
    */

    Ok(())
}