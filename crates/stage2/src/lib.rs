#![no_std]
#![feature(naked_functions)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use encore::prelude::*;

macro_rules! info {
    ($($tokens: tt)*) => {
        println!("[stage2] {}", alloc::format!($($tokens)*));
    }
}

/// # Safety
/// Does a raw syscall, initializes the global allocator
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn entry(stack_top: *mut u8) -> ! {
    init_allocator();
    crate::main(stack_top);
}

/// # Safety
/// Maps and jumps to another ELF object
#[inline(never)]
unsafe fn main(stack_top: *mut u8) -> ! {
    info!("Stack top: {:?}", stack_top);
    syscall::exit(0);
}