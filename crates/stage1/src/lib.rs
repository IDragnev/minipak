#![feature(naked_functions)]
#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use encore::prelude::*;

macro_rules! info {
    ($($tokens: tt)*) => {
        println!("[stage1] {}", alloc::format!($($tokens)*));
    }
}

/// # Safety
/// Uses inline assembly so it can behave as the entry point of a static
/// executable.
#[no_mangle]
#[naked]
pub unsafe extern "C" fn entry() {
    use core::arch::asm;
    asm!("mov rdi, rsp", "call premain", options(noreturn))
}

/// # Safety
/// Initializes the allocator.
#[no_mangle]
#[inline(never)]
unsafe fn premain(stack_top: *mut u8) -> ! {
    init_allocator();
    crate::main(stack_top)
}

/// # Safety
/// Nothing bad so far.
#[inline(never)]
unsafe fn main(stack_top: *mut u8) -> ! {
    info!("Stack top: {:?}", stack_top);

    syscall::exit(0)
}