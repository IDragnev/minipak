use core::arch::asm;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(pub u64);

impl FileDescriptor {
    pub const STDIN: Self = Self(0);
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);
}

/// # Safety
/// Calls into the Kernel
#[inline(always)]
pub unsafe fn write(fd: FileDescriptor, buf: *const u8, count: u64) -> u64 {
    let syscall_num: u64 = 1;
    let mut rax = syscall_num;

    asm!(
        "syscall",
        inout("rax") rax,
        in("rdi") fd.0,
        in("rsi") buf,
        in("rdx") count,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    rax
}