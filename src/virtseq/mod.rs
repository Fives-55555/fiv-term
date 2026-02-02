mod terminfo;

pub use terminfo::TermInfo;

mod virtbuf;

pub use virtbuf::VirtSeqBuf;

pub trait TermControl {
    type Result<T>;
    fn clear_screen(&self, buf: &) -> Self::Result<()>;
}
