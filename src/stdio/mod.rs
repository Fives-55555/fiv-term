pub mod epoll;
pub mod io_uring;
mod io_uring_def;

pub use io_uring::IoUring;
pub use io_uring_def::IouSQEntry;
use std::io::Result;

pub trait StdIo: Default {
    fn is_blocking() -> bool;
    fn read_in(&self, buf: &mut [u8]) -> Result<usize>;
    fn write_out(&self, buf: &[u8]) -> Result<usize>;
}

pub trait Io {
    type Read;
    fn read(fd: i32, buf: &mut [u8]) -> Result<Self::Read>;
    type Write;
    fn write(fd: i32, buf: &[u8]) -> Result<Self::Write>;
}
