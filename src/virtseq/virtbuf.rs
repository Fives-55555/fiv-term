use std::io::{Write, stdout};

use crate::stuff::{FastForwardFormat, NumberBuffer};

pub struct VirtSeqBuf {
    pub buf: [u8; Self::BUFFER_SIZE],
    pub idx: usize,
    pub cap: usize,
}

impl VirtSeqBuf {
    pub const BUFFER_SIZE: usize = 128;
    pub const fn new() -> Self {
        VirtSeqBuf {
            buf: [0; Self::BUFFER_SIZE],
            idx: 0,
            cap: Self::BUFFER_SIZE,
        }
    }
    pub fn format_num(&mut self, num: u16) -> Result<(), ()> {
        let mut buf = NumberBuffer::try_from(&mut self.buf[self.idx..])?;
        let x = num.forward_format(&mut buf);
        self.cap -= x;
        self.idx += x;
        Ok(())
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
        if self.cap >= buf.len() {
            self.buf[self.idx..self.idx + buf.len()].copy_from_slice(buf);
            self.cap -= buf.len();
            self.idx += buf.len();
            return Ok(());
        }
        return Err(());
    }
    // FIXME
    pub fn flush(&mut self) -> std::io::Result<()> {
        stdout().write(&self.buf[0..self.idx])?;
        stdout().flush()?;
        self.cap = Self::BUFFER_SIZE;
        self.idx = 0;
        Ok(())
    }
}

/*

pub enum VirtualAction {
    MoveTo,
    // Move the Cursor one line up.
    ReverseIndex,
}

pub trait VirtualSeq: TerminalTrait {
    fn enable(&self) -> <Self as TerminalTrait>::Result<()>;
    fn command(&self, action: VirtualAction);
}

pub trait ReverseIndex: VirtualSeq {
    // Move the Cursor one line up.
    fn ri(&self, buf: &mut VirtSeqBuf);
}

pub trait StoreCursor: VirtualSeq {
    // Stores the current cursor osition to memory.
    fn decsc(&self, buf: &mut VirtSeqBuf);
    // Restores the saved position. Requires previous store.
    fn decsr(&self, buf: &mut VirtSeqBuf);
}

pub trait CursorPosition: VirtualSeq {
    /// Moves cursor up by <lines>.
    /// <lines> cant be larger then 32.767
    fn cuu(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Moves cursor down by <lines>.
    /// <lines> cant be larger then 32.767
    fn cud(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Moves cursor forward
    /// <lines> cant be larger then 32.767
    fn cuf(&self, buf: &mut VirtSeqBuf, chars: u16);
    /// Moves cursor backward
    /// <lines> cant be larger then 32.767
    fn cub(&self, buf: &mut VirtSeqBuf, chars: u16);
    /// Moves up by <lines> and to the begining of this line
    /// <lines> cant be larger then 32.767
    fn cnl(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Moves up by <lines> and to the begining of this line
    /// <lines> cant be larger then 32.767
    fn cpl(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Moves to the absolute horizontal position. Aka. it moves to charater <y>
    /// <y> cant be larger then 32.767
    fn cha(&self, buf: &mut VirtSeqBuf, x: u16);
    /// Moves to the absolute vertical position. Aka. it moves to line <y>
    /// <y> cant be larger then 32.767
    fn vpa(&self, buf: &mut VirtSeqBuf, y: u16);
    /// VT100-Standart. Moves the Cursor to <x> and <y>.
    /// <x>, <y> cant be larger then 32.767
    fn cup(&self, buf: &mut VirtSeqBuf, x: u16, y: u16);
    /// This is useless. ISO-Standart. Same as CUP.
    /// <x>, <y> cant be larger then 32.767
    fn hvp(&self, buf: &mut VirtSeqBuf, x: u16, y: u16);
}

/// Simulates Ansi.sys Cursor Save/Load Operations
pub trait WinAnsiEmmulation: VirtualSeq {
    /// Stores the Cursor Position
    fn ansisyssc(&self, buf: &mut VirtSeqBuf);
    /// Loads the Cursor Position
    fn ansisysrc(&self, buf: &mut VirtSeqBuf);
}

/// Makes the Cursor Blink
pub trait CursorBlink: VirtualSeq {
    fn att160(&self, buf: &mut VirtSeqBuf, state: bool);
}

/// Changes the Visibility of the cursor
pub trait CursorVisibility: VirtualSeq {
    fn dectcem(&self, buf: &mut VirtSeqBuf, state: bool);
}

/// Changes the Cursor Shape
pub trait CursorShape: VirtualSeq {
    fn decscusr(&self, buf: &mut VirtSeqBuf, shape: CursorShapes);
}

pub trait Scrolling {
    /// Scrolls up by <lines> lines. IMPORTANT: The Viewport moves down.
    fn su(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Scrolls down by <lines> lines. IMPORTANT: The Viewport moves up.
    fn sd(&self, buf: &mut VirtSeqBuf, lines: u16);
}

// FIXME
#[repr(u8)]
pub enum CursorShapes {
    Default = b'0',
    BlockBlinking = b'1',
    BlockSteady = b'2',
    UnderlineBlinking = b'3',
    UnderlineSteady = b'4',
    BarBlinking = b'5',
    BarSteady = b'6',
}

pub trait TextMod {
    /// Inserts <chars> spaces at the cursor position
    fn ich(&self, buf: &mut VirtSeqBuf, chars: u16);
    /// Delets <chars> characters and shifts the other chars to the left
    fn dch(&self, buf: &mut VirtSeqBuf, chars: u16);
    /// Replaces <chars> characters with spaces
    fn ech(&self, buf: &mut VirtSeqBuf, chars: u16);
    /// Inserts <lines> lines over the cursor
    fn il(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Removes <lines> lines from the cursor line on
    fn dl(&self, buf: &mut VirtSeqBuf, lines: u16);
    /// Erase the display based on the mode
    fn ed(&self, buf: &mut VirtSeqBuf, mode: EraseMode);
    /// Erase the line based on the mode
    fn el(&self, buf: &mut VirtSeqBuf, mode: EraseMode);
}

#[repr(u8)]
pub enum EraseMode {
    ToEnd = b'0',
    FromStart = b'1',
    Entire = b'2',
}

pub trait ConsoleFormat {
    fn sgr(&self, buf: &mut VirtSeqBuf, fmt: [ConsoleFmtMode; 16]);
}

#[repr(u8)]
pub enum ConsoleFmtMode {
    Default = 0,
    // Fmt Enable
    BoldBight = 1,
    Underline = 4,
    Negative = 7,
    // Fmt Disable
    NoBoldBiright = 22,
    NoUnderline = 24,
    NoNegative = 27,
    //Color - Foreground
    FgBlack = 30,
    FgRed = 31,
    FgGreen = 32,
    FgYellow = 33,
    FgBlue = 34,
    FgMagenta = 35,
    FgCyan = 36,
    FgWhite = 37,
    /// Expects a following value
    FgExtended = 38,
    FgDefault = 39,
    //Color - Background
    BgBlack = 40,
    BgRed = 41,
    BgGreen = 42,
    BgYellow = 43,
    BgBlue = 44,
    BgMagenta = 45,
    BgCyan = 46,
    BgWhite = 47,
    /// Expects a following value
    BgExtended = 48,
    BgDefault = 49,
}

pub struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}*/
/*
pub struct VT100 {}

impl VT100 {
    pub const LUT: [(str, str, fn(), str)] = [
        ("LMN", "setnl", (), "[20h"),
        ("DECCKM", "setappl", (), "[?1h"),
        ("DECANM", "setansi", (), "[?2h"),
        ("DECCOLM", "setcol", (), "[?3h"),
        ("DECSCLM", "setsmooth", (), "[?4h"),
        ("DECSCNM", "setrevscrn", (), "[?5h"),
        ("DECOM", "setorgrel", (), "[?6h"),
        ("DECAWM", "setwrap", (), "[?7h"),
        ("DECARM", "setrep", (), "[?8h"),
        ("DECINLM", "setinter", (), "[?9h"),
        ("LMN", "setlf", (), "[20l"),
        ("DECCKM", "setcursor", (), "[?1l"),
        ("DECANM", "setvt52", (), "[?2l"),
        ("DECCOLM", "resetcol", (), "[?3l"),
        ("DECSCLM", "setjump", (), "[?4l"),
        ("DECSCNM", "setnormscrn", (), "[?5l"),
        ("DECOM", "setorgabs", (), "[?6l"),
        ("DECAWM", "resetwrap", (), "[?7l"),
        ("DECARM", "resetrep", (), "[?8l"),
        ("DECINLM", "resetinter", (), "[?9l"),
    ];
}*/
