pub mod epoll;
pub mod io_uring;
mod io_uring_def;

pub use io_uring::IoUring;
pub use io_uring_def::IouSQEntry;
use std::io::{Read, Result, Write, stdin, stdout};

pub trait StdIo: Read + Write + Sized {
    fn new_stdio() -> Result<Self>;
}

pub struct StdIoImpl;

impl StdIo for StdIoImpl {
    fn new_stdio() -> Result<Self> {
        Ok(StdIoImpl)
    }
}

impl Write for StdIoImpl {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        stdout().write(buf)
    }
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        stdout().write_all(buf)
    }
    fn write_all_vectored(&mut self, bufs: &mut [std::io::IoSlice<'_>]) -> Result<()> {
        stdout().write_all_vectored(bufs)
    }
    fn flush(&mut self) -> Result<()> {
        stdout().flush()
    }
}

impl Read for StdIoImpl {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        stdin().read(buf)
    }
}

pub trait Io {
    type Read;
    fn read(fd: i32, buf: &mut [u8]) -> Result<Self::Read>;
    type Write;
    fn write(fd: i32, buf: &[u8]) -> Result<Self::Write>;
}
