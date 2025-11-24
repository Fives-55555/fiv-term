use crate::TerminalTrait;

pub struct VirtualSequence<const M: usize> {
    pub buf: [u8; M],
    pub len: usize,
}

pub trait VirtualSeq: TerminalTrait {
    fn enable(&self)-><Self as TerminalTrait>::Result<()>;
}

pub trait ReverseIndex: VirtualSeq {
    // Move the Cursor one line up.
    fn ri(&self)->&[u8];
}

pub trait StoreCursor: VirtualSeq {
    // Stores the current cursor osition to memory.
    fn decsc(&self)->&[u8];
    // Restores the saved position. Requires previous store.
    fn decsr(&self)->&[u8];
}

pub trait CursorPosition: VirtualSeq {
    /// Moves cursor up by <lines>.
    /// <lines> cant be larger then 32.767
    fn cuu(&self, lines: u16)->VirtualSequence<8>;
    /// Moves cursor down by <lines>.
    /// <lines> cant be larger then 32.767
    fn cud(&self, lines: u16)->VirtualSequence<8>;
    /// Moves cursor forward
    /// <lines> cant be larger then 32.767
    fn cuf(&self, chars: u16)->VirtualSequence<8>;
    /// Moves cursor backward
    /// <lines> cant be larger then 32.767
    fn cub(&self, chars: u16)->VirtualSequence<8>;
    /// Moves up by <lines> and to the begining of this line
    /// <lines> cant be larger then 32.767
    fn cnl(&self, lines: u16)->VirtualSequence<8>;
    /// Moves up by <lines> and to the begining of this line
    /// <lines> cant be larger then 32.767
    fn cpl(&self, lines: u16)->VirtualSequence<8>;
    /// Moves to the absolute horizontal position. Aka. it moves to charater <y> 
    /// <y> cant be larger then 32.767
    fn cha(&self, x: u16)->VirtualSequence<8>;
    /// Moves to the absolute vertical position. Aka. it moves to line <y> 
    /// <y> cant be larger then 32.767
    fn vpa(&self, y: u16)->VirtualSequence<8>;
    /// VT100-Standart. Moves the Cursor to <x> and <y>.
    /// <x>, <y> cant be larger then 32.767
    fn cup(&self, x: u16, y: u16)->VirtualSequence<14>;
    /// This is useless. ISO-Standart. Same as CUP.
    /// <x>, <y> cant be larger then 32.767
    fn hvp(&self, x: u16, y: u16)->VirtualSequence<14>;
}

pub trait WinAnsiEmmulation {
    fn ansisyssc(&self)->&[u8];
    fn ansisysrc(&self)->&[u8];
}