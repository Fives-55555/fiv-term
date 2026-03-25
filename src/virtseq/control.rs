use std::array;

use crate::{SeqBuf, idx_name, stuff::Stack, virtseq::ParseError};

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
    fn set_cursor_pos(&self, _buf: &mut VirtSeqBuf, _x: i16, _y: i16) -> Self::Result<()> {
        Err(())
    }
}
pub struct ParaStrCap;

pub struct ParsedStringCap<'buf>(Box<dyn Fn(&mut VirtSeqBuf)>);

impl<'buf> ParsedStringCap<'buf> {
    const OP_STACK_SIZE: usize = 16;
    fn parse<F>(
        str: &[u8],
        inputs: &[(u8, ExprType)],
    ) -> Result<Box<dyn Fn(&mut VirtSeqBuf, F)>, ParseError> {
        let mut stack: Stack<usize, { Self::OP_STACK_SIZE }> = Stack::new();
        let mut params: Vec<Var> = Vec::new();
        let mut expr: Vec<Expr> = Vec::new();
        let mut print: Vec<Print> = Vec::new();

        let mut one = None;
        let mut bytes = str.iter();
        let mut char;

        loop {
            char = match bytes.next() {
                Some(char) => *char,
                None => break,
            };
            if char == b'%' {
                char = *bytes.next().ok_or(ParseError::IncompleteMod).unwrap();
                let exp = match char {
                    b'p' => {
                        char = *bytes.next().ok_or(ParseError::IncompleteParam).unwrap();
                        if char >= b'1' && char <= b'9' {
                            let id = char - b'1';
                            let param =
                                params.iter().position(|p| p.id == id).unwrap_or_else(|| {
                                    params.push(Var {
                                        id,
                                        expr_type: ExprType::Unknown,
                                    });
                                    params.len() - 1
                                });
                            let exp = Expr {
                                expr_type: params[param].expr_type,
                                exprs: ExprEnum::Ground(param),
                            };
                            if one.is_some() && (id == 1 || id == 2) {
                                let i = expr.len();
                                //FIXME Check for Int
                                expr.push(exp);
                                Expr {
                                    expr_type: ExprType::Int,
                                    exprs: ExprEnum::Add((i, one.unwrap())),
                                }
                            } else {
                                exp
                            }
                        } else {
                            return Err(ParseError::InvalidParam);
                        }
                    }
                    b'[' => match bytes.find(|x| **x == b']') {
                        Some(_) => return Err(ParseError::MissingFeatureMatch),
                        None => return Err(ParseError::InvalidInputMatch),
                    },
                    b'{' => match bytes.find(|x| **x == b'}') {
                        Some(_) => return Err(ParseError::MissingFeatureConsts),
                        None => return Err(ParseError::InvalidConsts),
                    },
                    b'+' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Add((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'&' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::And((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'A' => {
                        let opr =
                            Self::get_exprs::<2>(&mut stack, &expr, ExprType::BoolMAYBE).unwrap();

                        Expr {
                            exprs: ExprEnum::CondAnd((opr[0], opr[1])),
                            expr_type: ExprType::BoolMAYBE,
                        }
                    }
                    b'!' => {
                        let opr =
                            Self::get_exprs::<1>(&mut stack, &expr, ExprType::BoolMAYBE).unwrap();

                        Expr {
                            exprs: ExprEnum::CondNot(opr[0]),
                            expr_type: ExprType::BoolMAYBE,
                        }
                    }
                    b'O' => {
                        let opr =
                            Self::get_exprs::<2>(&mut stack, &expr, ExprType::BoolMAYBE).unwrap();

                        Expr {
                            exprs: ExprEnum::CondOr((opr[0], opr[1])),
                            expr_type: ExprType::BoolMAYBE,
                        }
                    }
                    b'/' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Div((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'=' => {
                        let opr = Self::get_cmp_exprs(&mut stack, &expr).unwrap();

                        Expr {
                            exprs: ExprEnum::CondAnd((opr[0], opr[1])),
                            expr_type: ExprType::BoolMAYBE,
                        }
                    }
                    b'i' => {
                        one = Some(expr.len());
                        expr.push(Expr {
                            exprs: ExprEnum::Const(1),
                            expr_type: ExprType::Int,
                        });
                        continue;
                    }
                    b'>' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::LargerThen((opr[0], opr[1])),
                            expr_type: ExprType::BoolMAYBE,
                        }
                    }
                    b'm' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Mod((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'*' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Mul((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'|' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Or((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'<' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::SmallerThen((opr[0], opr[1])),
                            expr_type: ExprType::BoolMAYBE,
                        }
                    }
                    b'l' => {
                        let opr =
                            Self::get_exprs::<1>(&mut stack, &expr, ExprType::String).unwrap();

                        Expr {
                            exprs: ExprEnum::StrLen(opr[0]),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'-' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Sub((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'^' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Int).unwrap();

                        Expr {
                            exprs: ExprEnum::Xor((opr[0], opr[1])),
                            expr_type: ExprType::Int,
                        }
                    }
                    b'\'' => {
                        let c = *bytes.next().ok_or(ParseError::IncompleteCharConst).unwrap();
                        char = *bytes.next().ok_or(ParseError::IncompleteCharConst).unwrap();
                        if char == b'\'' {
                            Expr {
                                exprs: ExprEnum::Const(c as u32),
                                expr_type: ExprType::Int,
                            }
                        } else {
                            return Err(ParseError::InvalidCharConst);
                        }
                    }
                    _ => {
                        print.push(match char {
                            b'%' => Print::Byte(b'%'),
                            b'd' => Print::PrintDec(
                                Self::get_exprs::<1>(&mut stack, &expr, ExprType::Int).unwrap()[0],
                            ),
                            b'o' => Print::PrintOct(
                                Self::get_exprs::<1>(&mut stack, &expr, ExprType::Int).unwrap()[0],
                            ),
                            b'x' => Print::PrintHex(
                                Self::get_exprs::<1>(&mut stack, &expr, ExprType::Int).unwrap()[0],
                            ),
                            b'X' => Print::PrintLHex(
                                Self::get_exprs::<1>(&mut stack, &expr, ExprType::Int).unwrap()[0],
                            ),
                            b's' => Print::PrintStr(
                                Self::get_exprs::<1>(&mut stack, &expr, ExprType::String).unwrap()
                                    [0],
                            ),
                            b'c' => Print::PrintChar(
                                Self::get_exprs::<1>(&mut stack, &expr, ExprType::Int).unwrap()[0],
                            ),
                            b'?' => return Err(ParseError::MissingFeatureIf),
                            _ => return Err(ParseError::InvalidMod),
                        });
                        continue;
                    }
                };
                stack.push(expr.len());
                expr.push(exp);
            } else {
                print.push(Print::Byte(char))
            };
        }
        if stack.len() != 0 {
            println!("{stack:?}");
            return Err(ParseError::NotEmptyStack);
        }
        // FIXME Add Static check
        Ok(Box::new(|bufs: &mut VirtSeqBuf, params: F| {
            for print_thing in print {
                match print_thing {
                    Print::Byte(byte) => bufs.add_char(byte),
                    Print::PrintDec(dec) => {
                        let num = expr[dec].eval();
                        match bufs.format_num(num) {
                            Ok(_) => continue,
                            Err(_) => {
                                bufs.flush();
                                bufs.format_num(num).unwrap()
                            }
                        }
                    }
                }
            }
        }))
    }
    fn get_param(
        params: &mut Vec<Var>,
        id: u8,
        type_filter: Option<ExprType>,
    ) -> Result<usize, ParseError> {
        Ok(match params.iter().position(|elem| elem.id == id) {
            Some(idx) => {
                if params[idx].expr_type == type_filter.unwrap_or(ExprType::Unknown)
                    || params[idx].expr_type == ExprType::Unknown
                {
                    idx
                } else {
                    return Err(ParseError::WrongType);
                }
            }
            None => {
                let i = params.len();
                params.push(Var {
                    id,
                    expr_type: type_filter.unwrap_or(ExprType::Unknown),
                });
                i
            }
        })
    }
    // FIXME P! Type BACKpatching
    fn get_exprs<const N: usize>(
        stack: &mut Stack<usize, { Self::OP_STACK_SIZE }>,
        exprs: &Vec<Expr>,
        type_filter: ExprType,
    ) -> Result<[usize; N], ParseError> {
        if stack.len() >= N {
            let res = array::from_fn(|_| stack.pop().unwrap());
            for elem in res.iter() {
                if !(exprs[*elem].expr_type == ExprType::Unknown
                    || exprs[*elem].expr_type == type_filter)
                {
                    return Err(ParseError::WrongType);
                }
            }
            Ok(res)
        } else {
            return Err(ParseError::EmptyStack);
        }
    }
    // FIXME P2
    fn get_cmp_exprs(
        stack: &mut Stack<usize, { Self::OP_STACK_SIZE }>,
        exprs: &Vec<Expr>,
    ) -> Result<[usize; 2], ParseError> {
        todo!();
        if stack.len() >= 2 {
            let res = array::from_fn(|_| stack.pop().unwrap());
            let t = ExprType::Unknown;
            for elem in res.iter() {}
            Ok(res)
        } else {
            return Err(ParseError::Debug);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Expr {
    exprs: ExprEnum,
    expr_type: ExprType,
}

impl Expr {
    pub fn eval(&self) {
        //Rec?
        match self
    }
}

#[derive(Clone, Copy, Debug)]
enum ExprEnum {
    //Num
    Add((usize, usize)), // +
    Sub((usize, usize)), // -
    Mul((usize, usize)), // *
    Div((usize, usize)), // /
    Mod((usize, usize)), // m
    And((usize, usize)), // &
    Or((usize, usize)),  // |
    Xor((usize, usize)), // ^
    //Cond
    Eq((usize, usize)),          // =
    LargerThen((usize, usize)),  // >
    SmallerThen((usize, usize)), // <
    //Str
    StrLen(usize), // l
    //Bool
    CondOr((usize, usize)),  // O
    CondAnd((usize, usize)), // A
    CondNot(usize),          // !
    //Branching
    If(usize), // ?
    /// Index into the Params
    Ground(usize),
    /// Limited consts to u32
    Const(u32),
}

#[repr(u8)]
#[derive(Debug)]
enum Print {
    PrintOct(usize),  // Expects a ???? o?
    PrintHex(usize),  // Expects a i16? x
    PrintLHex(usize), // Expects a i16? X
    PrintDec(usize),  // Expects a i16? d
    PrintStr(usize),  // Expects a *const char? s?
    PrintChar(usize), // Expects a int
    Byte(u8),
}

/// Param id is (0..9)
#[derive(Clone, Copy, Debug)]
struct Var {
    id: u8,
    expr_type: ExprType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ExprType {
    String,
    Int,
    Bool,
    Unknown,
}
