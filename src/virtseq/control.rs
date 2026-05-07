use crate::{
    SeqBuf, Stack, idx_name,
    stuff::{CopyT, SeqSlice},
    virtseq::ParseError,
};

use std::ops::{BitAnd, BitOr, BitXor};

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
                _ => return Err(ParseError::EmptyStr),
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
                _=>return Err(ParseError::EmptyStrOut)
            }
        };
    };
    (_ $entry:expr, $buffer:expr, $id:ident, $name:literal, in, $io:expr) => {
        let str = $entry.get_str(idx_name!(STRING, $name));
        match *str.unwrap() {
            TermInfoString::There(slice) => {
                ParsedStringCap::parse_in(&mut $buffer, slice, io);
            }
            _=>return Err(ParseError::EmptyStrIn)
        }
    };
}

impl TermInfoConfig<'_> {
    pub const DEFAULT_PRE_ALLOC: usize = 5 * 3;
    pub fn with_entry(entry: TermInfo) -> Result<TermInfoConfig, ParseError> {
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

        fn dyn_str<const N: usize>(
            buf: &mut SeqBuf,
            str: ParsedStringCap<N>,
        ) -> Result<ParsedStringCap<N>, ParseError> {
            buf.get_next(str.op_len + str.slice_len)
                .ok_or(ParseError::UnknownError)?;
            Ok(str)
        }
        Ok(TermInfoConfig {
            clear: buf.get_next(clear).unwrap(),
            get_pos_res: buf.get_next(get_pos_res).unwrap(),
            get_pos_req: buf.get_next(get_pos_req).unwrap(),
            set_pos: dyn_str(&mut buf, set_pos).unwrap(),
            buf: buf.end().ok_or(ParseError::UnknownError)?,
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

#[derive(Clone, Copy)]
pub enum Param<'a> {
    Str(&'a str),
    Int(i16),
    Bool(bool),
}

impl<'a> Param<'a> {
    fn to_var(self) -> Variable<'a> {
        match self {
            Param::Str(str) => Variable { str },
            Param::Int(int) => Variable { int },
            Param::Bool(bool) => Variable { bool },
        }
    }
}

#[derive(Clone, Copy)]
union Variable<'a> {
    str: &'a str,
    int: i16,
    bool: bool,
}

#[repr(u8)]
enum PState {
    None,
    Byte,
    Slice,
}

// KNOWN Input Type and size
pub struct ParsedStringCap<const N: usize> {
    base: usize,
    slice_len: usize,
    op_len: usize,
}

macro_rules! parse_n_check {
    (
        $char:expr,
        $ops:expr,
        $stack:expr,
        (
            $(($filter:expr, $code:expr, $ret_type:expr, $argc:expr, $type_filter:expr)),*
        ),
        (
            $(($print_filter:expr, $print_code:expr, $print_type_filter:expr)),*
        )
    )=>{
        match $char {

            $($filter => {
                Self::get_exprs::<$argc, _>(&mut $stack, $type_filter)
                .unwrap();
                ($code as u8, $ret_type)
            })*
            _=>{
                $ops.push(match $char {
                    $(
                        $print_filter=>{
                            Self::get_exprs::<1, _>(&mut $stack, $print_type_filter).unwrap();
                            $print_code as u8
                        }
                    )*
                    _ => return Err(ParseError::InvalidMod),
                });
                continue;
            }
        }
    };
}

macro_rules! switch_and_exec {
    (
        $op:expr,
        $stack:expr,
        (
            $(
                ($code:pat, $func:expr, $in_type:ident, $type:ident)
            ),*
        )
    ) => {
        match $op {
            $($code => {
                let lhs = $stack.pop().unwrap();
                let rhs = &mut unsafe {$stack.mut_slice_last(1).unwrap()[0]};
                *rhs = Variable {$type: $func(unsafe {lhs.$in_type}, unsafe {rhs.$in_type})};
            })*
            _ => unimplemented!(),
        }
    };
}

// FIXME P2 Remove unwraps
// FIXME Feature add OPs for strs
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
        let mut stack: Stack<ExprType, { Self::OP_STACK_SIZE }> = Stack::new();
        let mut ops: Vec<u8> = Vec::new();
        // FIXME CHeck for right use
        let mut used: u16 = 0;

        let base = buf.len();
        let mut state = PState::None;

        let mut one = false;

        let mut bytes = str.iter();
        let mut char: u8;

        loop {
            char = match bytes.next() {
                Some(char) => *char,
                None => break,
            };
            if char == b'%' {
                state = PState::None;
                char = *bytes.next().ok_or(ParseError::IncompleteMod).unwrap();
                let op: (u8, ExprType) = match char {
                    b'?' => return Err(ParseError::MissingFeatureIf),
                    b'%' => {
                        OpCode::push_byte(&mut ops, char);
                        continue;
                    }
                    b'p' => {
                        char = *bytes.next().ok_or(ParseError::IncompleteParam).unwrap();
                        if char >= b'1' && char <= b'9' {
                            let id = char - b'1';
                            let idx = inputs
                                .iter()
                                .position(|p| p.0 == id)
                                .ok_or(ParseError::InvalidId)
                                .unwrap();
                            used |= 1 << idx;
                            stack.push(inputs[idx].1);
                            OpCode::push_p(&mut ops, idx as u8);
                            if one && (id == 1 || id == 2) {
                                ops.push(OpCode::AddOne as u8);
                            }
                        } else {
                            return Err(ParseError::InvalidId);
                        }
                        continue;
                    }
                    // FIXME IDk
                    b'[' => match bytes.find(|x| **x == b']') {
                        Some(_) => todo!(),
                        None => return Err(ParseError::InvalidInputMatch),
                    },
                    b'{' => {
                        let mut num = [0; 6];
                        let mut idx = 0;
                        char = *bytes.next().ok_or(ParseError::IncompleteCharConst).unwrap();
                        while char != b'}' {
                            if idx == 6 {
                                return Err(ParseError::InvalidConsts);
                            }
                            num[idx] = char;
                            idx += 1;
                            char = *bytes.next().ok_or(ParseError::IncompleteCharConst).unwrap();
                        }
                        let num = i16::from_ascii(&num[0..idx])
                            .ok()
                            .ok_or(ParseError::InvalidConsts)?;
                        OpCode::push_const(&mut ops, num);
                        continue;
                    }
                    b'i' => {
                        one = true;
                        continue;
                    }
                    // FIXME IDK
                    b'\'' => {
                        let c = *bytes.next().ok_or(ParseError::IncompleteCharConst).unwrap();
                        char = *bytes.next().ok_or(ParseError::IncompleteCharConst).unwrap();
                        if char == b'\'' {
                            OpCode::push_const(&mut ops, c as i16);
                            stack.push(ExprType::Int);
                            continue;
                        } else {
                            return Err(ParseError::InvalidCharConst);
                        }
                    }
                    _ => {
                        #[rustfmt::skip]
                        parse_n_check!(
                            char,
                            ops,
                            stack,
                            (
                                (b'+', OpCode::Add, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int),
                                (b'&', OpCode::BitAnd, ExprType::Int, 2, |arr| arr[0]==ExprType::Int && arr[1] == ExprType::Int),
                                (b'A', OpCode::CondAnd, ExprType::Bool, 2, |arr| arr[0] == ExprType::Bool && arr[1] == ExprType::Bool),
                                (b'!', OpCode::CondNot, ExprType::Bool, 1, |arr| arr[0] == ExprType::Bool),
                                (b'O', OpCode::CondOr, ExprType::Bool, 2, |arr| arr[0] == ExprType::Bool && arr[1] == ExprType::Bool),
                                (b'/', OpCode::Div, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int),
                                (b'=', OpCode::CondEq, ExprType::Bool, 2, |_arr| false),
                                (b'>', OpCode::CondLargerThen, ExprType::Bool, 2, |_arr| false),
                                (b'm', OpCode::Mod, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int),
                                (b'*', OpCode::Mul, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int),
                                (b'|', OpCode::BitOr, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int),
                                (b'<', OpCode::CondSmallerThen, ExprType::Bool, 2, |_arr| false),
                                (b'l', OpCode::StrLen, ExprType::Int, 1, |arr| arr[0] == ExprType::String),
                                (b'-', OpCode::Sub, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int),
                                (b'^', OpCode::BitXor, ExprType::Int, 2, |arr| arr[0] == ExprType::Int && arr[1] == ExprType::Int)
                            ),
                            (
                                (b'd', OpCode::PrintDec, |arr| arr[0] == ExprType::Int),
                                (b'o', OpCode::PrintOct, |arr| arr[0] == ExprType::Int),
                                (b'x', OpCode::PrintHex, |arr| arr[0] == ExprType::Int),
                                (b'X', OpCode::PrintBHex, |arr| arr[0] == ExprType::Int),
                                (b's', OpCode::PrintStr, |arr| arr[0] == ExprType::String),
                                (b'c', OpCode::PrintChar, |arr| arr[0] == ExprType::Int)
                            )
                        )
                    }
                };
                ops.push(op.0);
                stack.push(op.1);
            } else {
                match state {
                    PState::None => {
                        OpCode::push_byte(&mut ops, char);
                        state = PState::Byte;
                    }
                    PState::Byte => {
                        let idx = ops.len() - 1;
                        let prev_char = ops[idx] as u8;
                        ops[idx] = unsafe { std::mem::transmute(2_u8) };

                        ops[idx - 1] = OpCode::PrintSlice as u8;

                        buf.push(prev_char);
                        buf.push(char);

                        state = PState::Slice;
                    }
                    PState::Slice => {
                        let elem = unsafe { ops.last_mut().unwrap_unchecked() };
                        let val = *elem as u8 + 1;
                        *elem = unsafe { std::mem::transmute(val) };
                        buf.push(char);
                    }
                }
            };
        }

        if stack.len() != 0 {
            return Err(ParseError::NotEmptyStack);
        }

        let ops_base = buf.copy_t_aligned(ops);

        if used.count_ones() != N as u32 {
            return Err(ParseError::UnusedParam);
        }

        Ok(ParsedStringCap {
            slice_len: ops_base - base,
            op_len: buf.len() - ops_base,
            base,
        })
    }
    pub fn print(
        &self,
        data: &Box<[u8]>,
        buf: &mut VirtSeqBuf,
        params: [Param; N],
    ) -> std::io::Result<()>
    where
        [(); Self::OP_STACK_SIZE]:,
    {
        fn pop<T: Copy>(slice: &mut &[T]) -> T {
            let val = slice[0];
            *slice = &slice[1..];
            return val;
        }
        fn pop_slices<'a>(slice: &mut &'a [u8], len: usize) -> &'a [u8] {
            let val = &slice[0..len];
            *slice = &slice[len..];
            return val;
        }
        let vars = std::array::from_fn::<Variable, N, _>(|idx| params[idx].to_var());
        let mut data = SeqSlice::new(data);
        _ = data.get_next(self.base);
        let mut slices: &[u8] = data.get_next(self.slice_len).unwrap();
        let mut ops: &[u8] = data.get_next(self.op_len).unwrap();
        let mut stack: Stack<Variable, { Self::OP_STACK_SIZE }> = Stack::new();

        while !ops.is_empty() {
            let op = unsafe { std::mem::transmute(pop(&mut ops)) };
            match op {
                OpCode::PrintByte => buf.write_byte(pop(&mut ops) as u8)?,
                OpCode::PrintSlice => {
                    buf.write(pop_slices(&mut slices, pop(&mut ops) as u8 as usize))?
                }
                OpCode::PrintDec => {
                    let num = stack.pop().unwrap();
                    buf.format_num(unsafe { num.int })?;
                }
                OpCode::PushParam => {
                    let idx = pop(&mut ops);
                    stack.push_unchecked(vars[idx as usize]);
                }
                OpCode::PushConst => {
                    let value: [u8; 2] = std::array::from_fn(|_| pop(&mut ops) as u8);
                    stack.push_unchecked(Variable {
                        int: i16::from_ne_bytes(value),
                    });
                }
                OpCode::AddOne => unsafe { stack.mut_slice_last(1).unwrap()[0].int += 1 },
                OpCode::StrLen => {
                    let x = &mut unsafe { stack.mut_slice_last(1).unwrap()[0] };
                    x.int = unsafe { x.str.len() as i16 };
                }
                OpCode::CondNot => {
                    let cond = &mut unsafe { stack.mut_slice_last(1).unwrap()[0].bool };
                    *cond = !*cond;
                }
                _ => {
                    #[rustfmt::skip]
                    switch_and_exec!(
                        op,
                        stack,
                        (
                            (OpCode::Add, i16::wrapping_add, int, int),
                            (OpCode::Sub, i16::wrapping_sub, int, int),
                            (OpCode::Mul, i16::wrapping_mul, int, int),
                            (OpCode::Div, i16::wrapping_div, int, int),
                            (OpCode::Mod, i16::wrapping_rem, int, int),
                            (OpCode::BitAnd, i16::bitand, int, int),
                            (OpCode::BitOr, i16::bitor, int, int),
                            (OpCode::BitXor, i16::bitxor, int, int),
                            (OpCode::CondAnd, |a, b| a && b, bool, bool),
                            (OpCode::CondOr, |a, b| a || b, bool, bool),
                            (OpCode::CondLargerThen, |a, b| a > b, int, bool),
                            (OpCode::CondSmallerThen, |a, b| a < b, int, bool)
                        )
                    );
                }
            }
        }
        Ok(())
    }
    fn get_exprs<const C: usize, F>(
        stack: &mut Stack<ExprType, { Self::OP_STACK_SIZE }>,
        type_filter: F,
    ) -> Result<(), ParseError>
    where
        F: FnOnce(&[ExprType; C]) -> bool,
    {
        if type_filter(unsafe {
            stack
                .slice_last(C)
                .ok_or(ParseError::EmptyStack)
                .unwrap()
                .as_array()
                .unwrap_unchecked()
        }) {
            for _ in 0..C {
                _ = stack.pop();
            }
            Ok(())
        } else {
            Err(ParseError::WrongType)
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
enum OpCode {
    //Num
    Add,    // +
    AddOne, // i
    Sub,    // -
    Mul,    // *
    Div,    // /
    Mod,    // m
    BitAnd, // &
    BitOr,  // |
    BitXor, // ^
    //Cond
    CondEq,          // =
    CondLargerThen,  // >
    CondSmallerThen, // <
    CondOr,          // O
    CondAnd,         // A
    CondNot,         // !
    //Str
    StrLen, // l

    // FIXME P3 Add Branching

    // Following 2 Bytes are:
    // The constant
    PushConst,

    // Followiing Byte is
    // The Id
    PushParam,
    // The Char
    PrintByte,
    // The SliceLen
    PrintSlice,

    PrintOct,
    PrintDec,
    PrintHex,
    PrintBHex,
    PrintStr,
    PrintChar,
}

impl OpCode {
    pub fn push_p(ops: &mut Vec<u8>, id: u8) {
        ops.push(OpCode::PushParam as u8);
        ops.push(id);
    }
    pub fn push_byte(ops: &mut Vec<u8>, byte: u8) {
        ops.push(OpCode::PrintByte as u8);
        ops.push(byte);
    }
    pub fn push_const(ops: &mut Vec<u8>, num: i16) {
        ops.push(OpCode::PushConst as u8);
        ops.extend(num.to_ne_bytes());
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ExprType {
    String,
    Int,
    Bool,
}
