// pub mod epoll;
// pub mod io_uring;
// mod io_uring_def;

// pub use io_uring::IoUring;
// pub use io_uring_def::IouSQEntry;
use std::io::{IoSlice, Read, Result, Write, stdin, stdout};

pub trait StdIo: Sized + Write + Read {
    fn new_stdio() -> Result<Self>;
    fn io_type() -> IoType;
}

pub enum IoType {
    Synchronous,
    BusyAsynchronous,
    EventAsynchronous,
}

pub struct StdIoImpl;

impl StdIo for StdIoImpl {
    fn new_stdio() -> Result<Self> {
        Ok(StdIoImpl)
    }
    fn io_type() -> IoType {
        IoType::Synchronous
    }
}

impl Write for StdIoImpl {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        stdout().write(buf)
    }
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        stdout().write_all(buf)
    }
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<()> {
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
