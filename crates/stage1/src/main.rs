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
unsafe fn pre_main(stack_top: *mut u8) {
    init_allocator();
    main(Env::read(stack_top)).unwrap();
    syscall::exit(0);
}

#[allow(clippy::unnecessary_wraps)]
fn main(_env: Env) -> Result<(), EncoreError> {
    println!("Hello from stage1!");

    Ok(())
}