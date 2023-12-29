#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() {
    use  core::arch::asm;
    asm!("mov rdi, rsp", "call pre_main", options(noreturn))
}

#[no_mangle]
unsafe fn pre_main(_stack_top: *mut u8) {
    encore::items::init_allocator();
}