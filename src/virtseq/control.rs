use std::array;

use crate::{
    SeqBuf, Stack, idx_name,
    stuff::{CopyT, SeqSlice},
    virtseq::ParseError,
};

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
    set_pos: ParsedStringCap<2>,
}

macro_rules! get_strs {(
        $entry:expr,
        $buffer:expr,
        $(
            $id:ident,
            $name:literal
            $(, $dir:ident, $io:expr)?
        );+ $(;)?
    ) => {
        $(
            get_strs!(_ $entry, $buffer, $id, $name $(, $dir, $io)?);
        )+
    };
    (_ $entry:expr, $buffer:expr, $id:ident, $name:literal) => {
        let $id = {
            let str = $entry.get_str(idx_name!(STRING, $name));
            match *str.unwrap() {
                TermInfoString::There(slice) => {
                    $buffer.extend(slice);
                    slice.len()
                }
                _ => return Err(()),
            }
        };
    };
    (_ $entry:expr, $buffer:expr, $id:ident, $name:literal, out, $io:expr) => {
        let $id = {
            let str = $entry.get_str(idx_name!(STRING, $name));
            match *str.unwrap() {
                TermInfoString::There(slice) => {
                    ParsedStringCap::<{$io.len()}>::parse_out(&mut $buffer as &mut Vec<u8>, slice as &[u8], $io).unwrap()
                }
                _=>return Err(())
            }
        };
    };
    (_ $entry:expr, $buffer:expr, $id:ident, $name:literal, in, $io:expr) => {
        let str = $entry.get_str(idx_name!(STRING, $name));
        match *str.unwrap() {
            TermInfoString::There(slice) => {
                ParsedStringCap::parse_in(&mut $buffer, slice, io);
            }
            _=>return Err(())
        }
    };
}

impl TermInfoConfig<'_> {
    pub const DEFAULT_PRE_ALLOC: usize = 5 * 3;
    pub fn with_entry(entry: TermInfo) -> Result<TermInfoConfig, ()> {
        let mut buf: Vec<u8> = Vec::with_capacity(Self::DEFAULT_PRE_ALLOC);

        get_strs!(
            entry,
            buf,
            clear, "clear_screen";
            get_pos_res, "user6";
            get_pos_req, "user7";
            set_pos, "cursor_address", out, [(0u8, ExprType::Int),(1u8, ExprType::Int)];
        );

        let mut buf = SeqBuf::new(buf.into_boxed_slice());

        Ok(TermInfoConfig {
            clear: buf.get_next(clear).unwrap(),
            get_pos_res: buf.get_next(get_pos_res).unwrap(),
            get_pos_req: buf.get_next(get_pos_req).unwrap(),
            set_pos: set_pos,
            buf: buf.end().ok_or(())?,
            resp: Vec::with_capacity(4),
        })
    }
}

impl TermControl for TermInfoConfig<'_> {
    type Result<T> = std::io::Result<T>;
    fn clear_screen(&self, buf: &mut VirtSeqBuf) -> Self::Result<()> {
        buf.write(self.clear)
    }
    fn get_cursor_pos(&mut self, buf: &mut VirtSeqBuf, user_data: usize) -> Self::Result<()> {
        self.resp.push((self.get_pos_res, user_data));
        buf.write(self.get_pos_req)
    }
    fn set_cursor_pos(&self, buf: &mut VirtSeqBuf, x: i16, y: i16) -> Self::Result<()> {
        self.set_pos
            .print(&self.buf, buf, [Param::Int(x), Param::Int(y)])
    }
}

enum PState {
    None,
    Byte,
    Slice,
}

// KNOWN Input Type and size
//
//FIXME P1 Size EXPR check u8::MAX
pub struct ParsedStringCap<const N: usize> {
    base: usize,
    data_len: usize,
    print_len: usize,
    expr_len: usize,
}

impl<const N: usize> ParsedStringCap<N> {
    const OP_STACK_SIZE: usize = 16;
    fn parse_out(
        buf: &mut Vec<u8>,
        str: &[u8],
        inputs: [(u8, ExprType); N],
    ) -> Result<ParsedStringCap<N>, ParseError>
    where
        [(); Self::OP_STACK_SIZE]:,
    {
        let mut stack: Stack<u8, { Self::OP_STACK_SIZE }> = Stack::new();
        let mut params: Vec<Var> = Vec::new();
        let mut expr: Vec<Expr> = Vec::new();
        let mut print: Vec<Print> = Vec::new();

        let base = buf.len();
        let mut state = PState::None;
        let mut one = None;
        let mut bytes = str.iter();
        let mut char;

        loop {
            char = match bytes.next() {
                Some(char) => *char,
                None => break,
            };
            if char == b'%' {
                state = PState::None;
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
                                exprs: ExprEnum::Ground(param as u8),
                            };
                            if one.is_some() && (id == 1 || id == 2) {
                                let i = expr.len() as u8;
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
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Bool).unwrap();

                        Expr {
                            exprs: ExprEnum::CondAnd((opr[0], opr[1])),
                            expr_type: ExprType::Bool,
                        }
                    }
                    b'!' => {
                        let opr = Self::get_exprs::<1>(&mut stack, &expr, ExprType::Bool).unwrap();

                        Expr {
                            exprs: ExprEnum::CondNot(opr[0]),
                            expr_type: ExprType::Bool,
                        }
                    }
                    b'O' => {
                        let opr = Self::get_exprs::<2>(&mut stack, &expr, ExprType::Bool).unwrap();

                        Expr {
                            exprs: ExprEnum::CondOr((opr[0], opr[1])),
                            expr_type: ExprType::Bool,
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
                            expr_type: ExprType::Bool,
                        }
                    }
                    b'i' => {
                        one = Some(expr.len() as u8);
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
                            expr_type: ExprType::Bool,
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
                            expr_type: ExprType::Bool,
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
                            b'X' => Print::PrintBHex(
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
                stack.push(expr.len() as u8);
                expr.push(exp);
            } else {
                match state {
                    PState::None => {
                        print.push(Print::Byte(char));
                        state = PState::Byte;
                    }
                    PState::Byte => {
                        let value = print.last_mut().unwrap();
                        if let Print::Byte(byte) = value {
                            buf.push(*byte);
                            buf.push(char);
                            *value = Print::Slice(2);
                            state = PState::Slice;
                        };
                    }
                    PState::Slice => {
                        let value = print.last_mut().unwrap();
                        if let Print::Slice(len) = value {
                            *value = Print::Slice(*len + 1);
                            buf.push(char);
                        }
                    }
                }
            };
        }

        if stack.len() != 0 {
            println!("{stack:?}");
            return Err(ParseError::NotEmptyStack);
        }

        let print_base = buf.copy_t_aligned(print);

        Ok(ParsedStringCap {
            data_len: print_base - base,
            print_len: expr_base - print_base,
            expr_len: buf.len() - expr_base,
            base,
        })
    }
    pub fn print<F>(
        &self,
        data: &Box<[u8]>,
        buf: &mut VirtSeqBuf,
        params: F,
    ) -> std::io::Result<()> {
        let mut data = SeqSlice::new(data);
        _ = data.get_next(self.base);
        let bytes: &[u8] = data.get_next(self.data_len).unwrap();
        let print: &[Print] = data.get_next_aligned_t(self.print_len).unwrap();
        let expr: &[Expr] = data.get_next_aligned_t(self.expr_len).unwrap();

        for print_thing in print {
            match print_thing {
                Print::Byte(byte) => buf.write_byte(*byte)?,
                Print::Slice(slice_len) => {
                    buf.write(data.get_next(*slice_len as usize).unwrap())?
                }
                Print::PrintDec(_) => {
                    let num: u16 = data.get_next_aligned_t::<Expr>(1).unwrap()[0].eval();
                    buf.format_num(num)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
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
    fn get_exprs<const C: usize>(
        stack: &mut Stack<u8, { Self::OP_STACK_SIZE }>,
        exprs: &Vec<Expr>,
        type_filter: ExprType,
    ) -> Result<[u8; C], ParseError> {
        if stack.len() >= C {
            let res = array::from_fn(|_| stack.pop().unwrap());
            for elem in res.iter() {
                if !(exprs[*elem as usize].expr_type == ExprType::Unknown
                    || exprs[*elem as usize].expr_type == type_filter)
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
        _stack: &mut Stack<u8, { Self::OP_STACK_SIZE }>,
        _exprs: &Vec<Expr>,
    ) -> Result<[u8; 2], ParseError> {
        todo!();
        // if stack.len() >= 2 {
        //     let res = array::from_fn(|_| stack.pop().unwrap());
        //     let t = ExprType::Unknown;
        //     for elem in res.iter() {}
        //     Ok(res)
        // } else {
        //     return Err(ParseError::Debug);
        // }
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
        match self {
            _ => (),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ExprEnum {
    //Num
    Add((u8, u8)), // +
    Sub((u8, u8)), // -
    Mul((u8, u8)), // *
    Div((u8, u8)), // /
    Mod((u8, u8)), // m
    And((u8, u8)), // &
    Or((u8, u8)),  // |
    Xor((u8, u8)), // ^
    //Cond
    Eq((u8, u8)),          // =
    LargerThen((u8, u8)),  // >
    SmallerThen((u8, u8)), // <
    //Str
    StrLen(u8), // l
    //Bool
    CondOr((u8, u8)),  // O
    CondAnd((u8, u8)), // A
    CondNot(u8),       // !
    //Branching
    If(u8), // ?
    /// Index into the Params
    Ground(u8),
    /// Limited consts to u32
    Const(u32),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Print {
    PrintOct(u8),  // Expects a ???? o?
    PrintHex(u8),  // Expects a i16? x
    PrintBHex(u8), // Expects a i16? X
    PrintDec(u8),  // Expects a i16? d
    PrintStr(u8),  // Expects a *const char? s?
    PrintChar(u8), // Expects a int
    Byte(u8),
    Slice(u8), // Lenght
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
