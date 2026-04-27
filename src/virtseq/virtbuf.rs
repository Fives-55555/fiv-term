use crate::stuff::{FastForwardFormat, NumberBuffer};
use std::io::{Write, stdout};

pub struct VirtSeqBuf {
    pub buf: [u8; Self::BUFFER_SIZE],
    pub idx: usize,
    pub min_flush_size: usize,
}

impl VirtSeqBuf {
    pub const BUFFER_SIZE: usize = 128;
    pub const MIN_FLUSH_SIZE: usize = 32;

    pub const fn new() -> Self {
        VirtSeqBuf {
            buf: [0; Self::BUFFER_SIZE],
            idx: 0,
            min_flush_size: Self::MIN_FLUSH_SIZE,
        }
    }
    pub fn format_num(&mut self, num: i16) -> std::io::Result<()> {
        let mut buf = match NumberBuffer::try_from(&mut self.buf[self.idx..]) {
            Ok(buf) => buf,
            Err(_) => {
                self.flush()?;
                NumberBuffer::try_from(&mut self.buf[self.idx..]).expect("The BUFFER is to small.")
            }
        };
        let x = num.forward_format(&mut buf);
        self.idx += x;
        Ok(())
    }
    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if buf.len() >= self.min_flush_size {
            self.flush()?;
            return Self::flush_buf(buf);
        }

        let cap = self.cap_left();
        if cap >= buf.len() {
            self.buf[self.idx..self.idx + buf.len()].copy_from_slice(buf);
            self.idx += buf.len();
            return Ok(());
        } else {
            if cap != 0 {
                self.buf[self.idx..Self::BUFFER_SIZE].copy_from_slice(&buf[0..cap]);
            }
            self.flush()?;
            self.idx = buf.len() - cap;
            self.buf[0..self.idx].copy_from_slice(&buf[cap..]);
            Ok(())
        }
    }
    #[inline]
    pub fn flush(&mut self) -> std::io::Result<()> {
        Self::flush_buf(&self.buf[0..self.idx])?;
        self.idx = 0;
        Ok(())
    }
    // FIXME
    pub fn flush_buf(buf: &[u8]) -> std::io::Result<()> {
        stdout().write_all(buf)?;
        stdout().flush()?;
        Ok(())
    }
    pub fn write_byte(&mut self, byte: u8) -> std::io::Result<()> {
        if self.idx == Self::BUFFER_SIZE {
            self.flush()?;
        }
        self.buf[self.idx] = byte;
        self.idx += 1;
        Ok(())
    }
    pub fn cap_left(&self) -> usize {
        Self::BUFFER_SIZE - self.idx
    }
}
