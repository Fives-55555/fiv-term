use std::io::{Write, stdout};

use crate::{SeqBuf, idx_name};

use super::{TermInfo, TermInfoString, VirtSeqBuf};

pub trait TermControl {
    type Result<T>;
    fn clear_screen(&self, buf: &mut VirtSeqBuf) -> Self::Result<()>;
    /// The Cursor Position will be returned through the StdIn. So you need to process the Input to get the Cursor Postion.
    fn get_cursor_pos(&mut self, buf: &mut VirtSeqBuf, user_data: usize) -> Self::Result<()>;
    fn set_cursor_pos(&self, buf: &mut VirtSeqBuf, x: i16, y: i16) -> Self::Result<()>;
}

pub struct TermInfoConfig<'me> {
    resp: Vec<(&'me [u8], usize)>,
    buf: Box<[u8]>,
    clear: &'me [u8],
    get_pos_res: &'me [u8],
    get_pos_req: &'me [u8],
    set_pos: &'me [u8],
}

macro_rules! get_strs {
    ($entry:expr, $buf:expr, $(($id:ident, $nc_name:literal),)*) => {
        $(
            let $id = {
                let str = $entry.get_str(idx_name!(STRING, $nc_name));
                match *str.unwrap() {
                    TermInfoString::There(slice) => {
                        $buf.extend(slice);
                        slice.len()
                    }
                    _ => return Err(()),
                }
            };
        )*
    };
}

impl TermInfoConfig<'_> {
    pub const DEFAULT_PRE_ALLOC: usize = 5 * 3;
    pub fn with_entry(entry: TermInfo) -> Result<TermInfoConfig, ()> {
        let mut buf: Vec<u8> = Vec::with_capacity(Self::DEFAULT_PRE_ALLOC);

        get_strs!(
            entry,
            buf,
            (clear, "clear_screen"),
            (get_pos_res, "user6"),
            (get_pos_req, "user7"),
            (set_pos, "cursor_address"),
        );

        let mut buf = SeqBuf::new(buf.into_boxed_slice());

        Ok(TermInfoConfig {
            clear: buf.get_first(clear).unwrap(),
            get_pos_res: buf.get_first(get_pos_res).unwrap(),
            get_pos_req: buf.get_first(get_pos_req).unwrap(),
            set_pos: buf.get_first(set_pos).unwrap(),
            buf: buf.end().ok_or(())?,
            resp: Vec::with_capacity(4),
        })
    }
}

impl TermControl for TermInfoConfig<'_> {
    type Result<T> = Result<T, ()>;
    fn clear_screen(&self, buf: &mut VirtSeqBuf) -> Self::Result<()> {
        buf.write(self.clear)
    }
    fn get_cursor_pos(&mut self, buf: &mut VirtSeqBuf, user_data: usize) -> Self::Result<()> {
        self.resp.push((self.get_pos_res, user_data));
        buf.write(self.get_pos_req)
    }
    fn set_cursor_pos(&self, buf: &mut VirtSeqBuf, x: i16, y: i16) -> Self::Result<()> {
        println!("{:?}", self.set_pos);
        Err(())
    }
}

#[test]
fn test() -> std::io::Result<()> {
    let mut buf = VirtSeqBuf::new();
    let entry = TermInfo::from_term().unwrap();
    println!("{}", entry);
    let term = TermInfoConfig::with_entry(entry).unwrap();
    term.set_cursor_pos(&mut buf, 5, 15).unwrap();
    Ok(())
}
