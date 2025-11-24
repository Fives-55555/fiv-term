use core::fmt::NumBuffer;
use std::io::Result;

use windows::{
    Win32::{
        Foundation::HANDLE,
        System::Console::{COORD, GetStdHandle, SMALL_RECT, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
    },
    core::Result,
};

use crate::{
    Terminal, TerminalTrait,
    terminal::TerminalStr,
    virtkeys::{CursorPosition, ReverseIndex, StoreCursor, VirtualSeq, VirtualSequence},
};

#[derive(Clone, Copy)]
pub struct ScreenBuffer {
    handle: HANDLE,
    attr: Attributes,
}

impl ScreenBuffer {
    pub fn get_std_handles() -> Result<StdHandles> {
        unsafe {
            let input = GetStdHandle(STD_INPUT_HANDLE)?;
            let output = GetStdHandle(STD_OUTPUT_HANDLE)?;
            Ok(StdHandles {
                input: input,
                output: output,
            })
        }
    }
    pub fn new_buffer() -> Result<ScreenBuffer> {
        unsafe {
            let handle = CreateConsoleScreenBuffer(0, 0, None, CONSOLE_TEXTMODE_BUFFER, None)?;
            let attrs = Attributes::get_attrs(handle)?;
            Ok(ScreenBuffer {
                handle: handle,
                attr: attrs,
            })
        }
    }
    pub fn get_size(&self) -> Result<(usize, usize)> {
        let size: (usize, usize) = {
            #[cfg(not(feature = "debug"))]
            {
                let mut buf: CONSOLE_SCREEN_BUFFER_INFO = CONSOLE_SCREEN_BUFFER_INFO::default();
                unsafe {
                    GetConsoleScreenBufferInfo(
                        self.handle(),
                        &mut buf as *mut CONSOLE_SCREEN_BUFFER_INFO,
                    )?;
                }
                (buf.dwSize.X as usize, buf.dwSize.Y as usize)
            }
            #[cfg(feature = "debug")]
            {
                (75, 15)
            }
        };
        Ok(size)
    }
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
    pub fn write_buffer(&mut self, cords: (usize, usize), buffer: TerminalStr) -> Result<()> {
        let mut region = SMALL_RECT {
            Left: cords.0 as i16,
            Top: cords.1 as i16,
            Right: (cords.0 + buffer.x) as i16,
            Bottom: (cords.1 + buffer.y) as i16,
        };
        unsafe {
            WriteConsoleOutputW(
                self.handle(),
                buffer.buf.as_ptr(),
                COORD {
                    X: buffer.x as i16,
                    Y: buffer.y as i16,
                },
                COORD {
                    X: cords.0 as i16,
                    Y: cords.1 as i16,
                },
                &mut region as *mut SMALL_RECT,
            )
        }
    }
    pub fn write_lin(&mut self, cords: (usize, usize), buffer: &[u16]) -> Result<()> {
        let mut written: u32 = 0;
        unsafe {
            WriteConsoleOutputCharacterW(
                self.handle(),
                buffer,
                COORD {
                    X: cords.0 as i16,
                    Y: cords.1 as i16,
                },
                &mut written as *mut u32,
            )
        }
    }
    pub fn blank(&mut self) -> Result<()> {
        let size = self.get_size()?;
        self.write_buffer((0, 0), TerminalStr::new(size.0, size.1, None))
    }
}

pub struct StdHandles {
    input: HANDLE,
    output: HANDLE,
}

pub struct WindowsTerminal;

pub const ESC_SEQ: u8 = 0x1b;

impl TerminalTrait for WindowsTerminal {
    type Result<V> = Result<V>;
}

// See https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#example
impl VirtualSeq for WindowsTerminal {
    fn enable(&self) -> Result<()> {}
}

impl ReverseIndex for WindowsTerminal {
    fn ri(&self) -> &[u8] {
        &[ESC_SEQ, b'M']
    }
}

impl StoreCursor for WindowsTerminal {
    fn decsc(&self) -> &[u8] {
        &[ESC_SEQ, b'7']
    }
    fn decsr(&self) -> &[u8] {
        &[ESC_SEQ, b'8']
    }
}

impl CursorPosition for WindowsTerminal {
    fn cuu(&self, lines: u16) -> VirtualSequence<8> {
        let whole = VirtualSequence {
            buf: [ESC_SEQ, b'[', 0, 0, 0, 0, 0, 0],
            len: 0
        };
        let mut buf: NumBuffer<u16> = NumBuffer;
        let num = lines.format_into(&mut buf);
        whole
    }
}
