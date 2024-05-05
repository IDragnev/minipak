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
/// Maps and calls into another ELF object
#[inline(never)]
unsafe fn main(stack_top: *mut u8) -> ! {
    info!("Stack top: {:?}", stack_top);

    let file = File::open("/proc/self/exe").unwrap();
    let map = file.map().unwrap();
    let full_slice = map.as_ref();
    let manifest = pixie::Manifest::read_from_full_slice(full_slice).unwrap();

    let stage2_slice = &full_slice[manifest.stage2.as_range()];
    let stage2_obj = pixie::Object::new(stage2_slice).unwrap();
    let mut stage2_mapped = pixie::MappedObject::new(&stage2_obj, None).unwrap();
    info!(
        "Mapped stage2 at base 0x{:x} (offset 0x{:x})",
        stage2_mapped.base(),
        stage2_mapped.base_offset(),
    );
    info!("Relocating stage2...");
    stage2_mapped.relocate(stage2_mapped.base_offset()).unwrap();
    info!("Relocating stage2 done!");

    let s2_entry = stage2_mapped.lookup_sym("entry").unwrap();
    info!("Found entry sym {:?}", s2_entry);
    let entry: unsafe extern "C" fn(*mut u8) -> ! = 
        core::mem::transmute(stage2_mapped.base_offset() + s2_entry.value);
    entry(stack_top);
}