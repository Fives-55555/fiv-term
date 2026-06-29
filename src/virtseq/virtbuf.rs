use crate::{
    stdio::{StdIo, StdIoImpl},
    stuff::{FastForwardFormat, NumberSlice},
};
use core::fmt::NumBufferTrait;
use std::{fmt::Debug, rc::Rc, sync::nonpoison::Mutex};

#[derive(Debug)]
pub struct VirtSeqBuf<const B: usize = 128, I: StdIo = StdIoImpl> {
    pub buf: [u8; B],
    pub idx: usize,
    pub min_flush_size: usize,
    // FIXME
    pub io: Rc<Mutex<I>>,
}

impl<const B: usize> VirtSeqBuf<B> {
    pub const fn new(io: Rc<Mutex<StdIoImpl>>) -> Self {
        VirtSeqBuf {
            buf: [0; B],
            idx: 0,
            min_flush_size: Self::MIN_FLUSH_SIZE,
            io: io,
        }
    }
}

impl<const B: usize, I: StdIo> VirtSeqBuf<B, I> {
    pub const MIN_FLUSH_SIZE: usize = 32;

    pub fn new_io(io: Rc<Mutex<I>>, min_flush_size: usize) -> VirtSeqBuf<B, I> {
        VirtSeqBuf {
            buf: [0; B],
            idx: 0,
            min_flush_size: min_flush_size,
            io: io,
        }
    }
    pub fn format_num(&mut self, num: i16) -> std::io::Result<()> {
        if self.cap_left() < i16::BUF_SIZE {
            self.flush()?
        }
        let mut buf = NumberSlice::new(&mut self.buf[self.idx..]).expect("Your buf is to small");
        let x = num.forward_format(&mut buf);
        self.idx += x;
        Ok(())
    }
    // FIXME ADD Length based writes
    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if buf.len() >= self.min_flush_size {
            self.flush()?;
            return self.flush_buf(buf);
        }

        let cap = self.cap_left();
        if cap >= buf.len() {
            self.buf[self.idx..self.idx + buf.len()].copy_from_slice(buf);
            self.idx += buf.len();
            return Ok(());
        } else {
            if cap != 0 {
                self.buf[self.idx..B].copy_from_slice(&buf[0..cap]);
            }
            self.flush()?;
            self.idx = buf.len() - cap;
            self.buf[0..self.idx].copy_from_slice(&buf[cap..]);
            Ok(())
        }
    }
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buf(&self.buf[0..self.idx])?;
        self.idx = 0;
        Ok(())
    }
    pub fn flush_buf(&self, buf: &[u8]) -> std::io::Result<()> {
        (*self.io.lock()).write_all(buf)
    }
    pub fn write_byte(&mut self, byte: u8) -> std::io::Result<()> {
        if self.idx == B {
            self.flush()?;
        }
        self.buf[self.idx] = byte;
        self.idx += 1;
        Ok(())
    }
    pub fn cap_left(&self) -> usize {
        B - self.idx
    }
}
