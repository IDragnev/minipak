#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]

extern crate alloc;

pub mod items;
pub mod error;
pub mod memmap;
pub mod syscall;
pub mod utils;
pub mod prelude;
pub mod fs;
pub mod env;