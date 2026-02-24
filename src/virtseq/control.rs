use crate::{SeqBuf, idx_name, stuff::Stack};

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
///
pub enum ParsedStringCap<'buf> {
    Static(&'buf [u8]),
    Dynamic {},
    Branching,
}

impl<'buf> ParsedStringCap<'buf> {
    const OP_STACK_SIZE: usize = 16;
    fn parse<F>(str: &[u8], mut buf: Vec<u8>) -> Result<Box<dyn Fn(&mut VirtSeqBuf)>, ()> {
        let mut stack: Stack<&Expr, Self::OP_STACK_SIZE> = Stack::new();

        let mut params: Vec<Var> = Vec::new();

        let mut expr: Vec<Expr> = Vec::new();

        let mut print: Vec<Print> = Vec::new();

        let mut bytes = str.iter();
        let mut char = *bytes.next().ok_or(())?;

        let mut dyn_str_cap = false;

        loop {
            if char == b'%' {
                char = *bytes.next().ok_or(())?;
                if char == b'%' {
                    print.push(Print::Byte(b'%'));
                    char = match bytes.next() {
                        Some(char) => *char,
                        None => break,
                    };
                    continue;
                } else if !dyn_str_cap {
                        dyn_str_cap = true;
                    }

                expr.push(match char {
                    b'p' => {
                        char = *bytes.next().ok_or(())?;
                        if char >= b'1' && char <= b'9' {
                            let id = char - (b'1' - 1);
                            let mut exists = None;
                            for param in params.iter() {
                                if param.id == id {
                                    exists = Some(param);
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            let mut expr_type = match exists {
                                Some(para) => para.expr_type,
                                None => ExprType::Unknown,
                            };
                            let param = exists.unwrap_or_else(|| {
                                &*params.push_mut(Var {
                                    id,
                                    expr_type: ExprType::Unknown,
                                })
                            });
                            expr.push(Expr {
                                exprs: ExprEnum::Ground(param),
                                expr_type,
                            });
                        } else {
                            return Err(());
                        }
                    }
                    b'+' => {
                        let opr = Self::get_exprs::<2>(&mut stack)?;
                        
                            if matches!(opr[0].expr_type, ExprType::Unknown | ExprType::Int) && matches!(opr[1].expr_type, ExprType::Unknown | ExprType::Int)
                            Expr{
                                exprs: ExprEnum::Add((opr[0], opr[1])),
                                expr_type: ExprType::Int,
                            }
                    },
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
                    _ => todo!(),
                })
            } else {
                print.push(Byte(char))
            };
            char = match bytes.next() {
                Some(char) => *char,
                None => break,
            };
        }
        // FIXME Add Static check
        Ok(Box::new(|x, z, y| z))
    }
    fn get_exprs<const N: usize>(stack: &mut Stack)->Result<[&Expr;N], ()> {
        if stack.len() >= N {
            let res: [&Expr; N];
            for elem in res.iter_mut() {
                *elem = stack.pop().unwrap();
            }
            Ok(res)
        }else {
            return Err(())
        }
    }
}

#[derive(Clone, Copy)]
struct Expr<'a> {
    exprs: ExprEnum<'a>,
    expr_type: ExprType,
}

#[derive(Clone, Copy)]
enum ExprEnum<'a> {
    //Num
    Add((&'a Expr<'a>, &'a Expr<'a>)), // +
    Sub((&'a Expr<'a>, &'a Expr<'a>)), // -
    Mul((&'a Expr<'a>, &'a Expr<'a>)), // *
    Div((&'a Expr<'a>, &'a Expr<'a>)), // /
    Mod((&'a Expr<'a>, &'a Expr<'a>)), // m
    And((&'a Expr<'a>, &'a Expr<'a>)), // &
    Or((&'a Expr<'a>, &'a Expr<'a>)),  // |
    Xor((&'a Expr<'a>, &'a Expr<'a>)), // ^
    //Cond
    Eq((&'a Expr<'a>, &'a Expr<'a>)),          // =
    LargerThen((&'a Expr<'a>, &'a Expr<'a>)),  // >
    SmallerThen((&'a Expr<'a>, &'a Expr<'a>)), // <
    //FIXME P1
    Inc, // i Adds to first to parameters
    //Str
    StrLen(&'a Expr<'a>), // l
    //Bool
    CondOR((&'a Expr<'a>, &'a Expr<'a>)),  // O
    CondAnd((&'a Expr<'a>, &'a Expr<'a>)), // A
    CondNot(&'a Expr<'a>),                 // !
    //Branching
    If(&'a Expr<'a>), // ?
    // vars
    Ground(&'a Var),
}

#[repr(u8)]
enum Print<'a> {
    PrintOOO(Expr<'a>), // Expects a ???? o?
    PrintHex(Expr<'a>), // Expects a i16? X? x?
    PrintInt(Expr<'a>), // Expects a i16? d?
    PrintStr(Expr<'a>), // Expects a *const char? s?
    Push(Expr<'a>),
    Byte(u8),
}

/// Param id is (0..9)
#[derive(Clone, Copy)]
struct Var {
    id: u8,
    expr_type: ExprType,
}

#[derive(Clone, Copy)]
enum ExprType {
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
