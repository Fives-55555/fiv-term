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

/// [Stack]
/// --------
/// [data(valid u8)(FUCK) | Maybe parse and create runtime function | Escape parsing at usage]
/// --------
/// Custom data
/// With logic needed to be called at some point
pub struct StringCapParser {}

impl StringCapParser {
    fn parse(str: &[u8]) -> Result<Box<dyn Fn(&mut VirtSeqBuf, i16, i16) -> i16>, ()> {
        let mut op_stack: Vec<Op> = Vec::new();

        let mut bytes = str.iter();
        let mut char = *bytes.next().ok_or(())?;

        loop {
            op_stack.push(if char == b'%' {
                char = *bytes.next().ok_or(())?;
                match char {
                    b'p' => {
                        char = *bytes.next().ok_or(())?;
                        if char >= b'1' && char <= b'9' {
                            Op::Push(Vars::Param((char - (b'1' - 1), VarType::Unknown)))
                        } else {
                            return Err(());
                        }
                    }
                    b'+' => Op::Add,
                    b'&' => Op::And,
                    b'A' => Op::CondAnd,
                    b'!' => Op::CondNot,
                    b'O' => Op::CondOR,
                    b'/' => Op::Div,
                    b'=' => Op::Eq,
                    b'?' => Op::If,
                    b'e' => Op::IfElse,
                    b'i' => Op::Inc,
                    b'>' => Op::LargerThen,
                    b'm' => Op::Mod,
                    b'*' => Op::Mul,
                    b'|' => Op::Or,
                    b'<' => Op::SmallerThen,
                    b'l' => Op::StrLen,
                    b'-' => Op::Sub,
                    b'^' => Op::Xor,
                    b'%' => Op::Byte(b'%'),
                    _ => todo!(),
                }
            } else {
                Op::Byte(char)
            });
            char = match bytes.next() {
                Some(char) => *char,
                None => break,
            };
        }

        Ok(Box::new(|x, z, y| z))
    }
}

#[repr(u8)]
enum Op {
    //Num
    Add,         // +
    Sub,         // -
    Mul,         // *
    Div,         // /
    Mod,         // m
    And,         // &
    Or,          // |
    Xor,         // ^
    Eq,          // =
    LargerThen,  // >
    SmallerThen, // <
    Inc,         // i Adds to first to parameters
    //Str
    StrLen, // l
    //Bool
    CondOR,  // O
    CondAnd, // A
    CondNot, // !
    //Branching
    If,     // ?
    Then,   // t
    IfElse, // e
    IfStop, // %
    //Vars
    PrintOOO(Vars), // Expects a ???? o?
    PrintHex(Vars), // Expects a i16? X? x?
    PrintInt(Vars), // Expects a i16? d?
    PrintStr(Vars), // Expects a *const char? s?
    Push(Vars),
    PushConst(u8), // Bounds not clear
    Byte(u8),
}

enum Vars {
    Param((u8, VarType)),
}

enum VarType {
    String,
    Int,
    BoolMAYBE,
    Unknown,
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
