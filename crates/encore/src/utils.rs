use core::fmt;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
            use ::core::fmt::Write;
            ::core::writeln!($crate::utils::Stdout, $($arg)*).ok();
        }
    };
}

pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            crate::syscall::write(
                crate::syscall::FileDescriptor::STDOUT,
                s.as_ptr(),
                s.len() as _,
            );
        }
        Ok(())
    }
}