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

#[no_mangle]
unsafe fn pre_main(_stack_top: *mut u8) {
    init_allocator();
    main().unwrap();
    syscall::exit(0);
}

fn main() -> Result<(), EncoreError> {
    let file = File::open("/etc/lsb-release")?;
    let map = file.map()?;

    let s = core::str::from_utf8(&map[..]).unwrap();
    for l in s.lines() {
        println!("> {}", l);
    }

    Ok(())
}