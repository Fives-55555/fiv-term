use windows::Win32::{
    Foundation::HANDLE,
    System::Console::{
        CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle,
        STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
    },
};

use crate::{
    FastForwardFormat, NumberBuffer, TerminalTrait,
    virtkeys::{
        CursorBlink, CursorPosition, CursorShape, CursorShapes, CursorVisibility, ReverseIndex,
        Scrolling, StoreCursor, TextMod, VirtualSeq, VirtualSequenceBuffer,
    },
};
use std::io::Result;

/*
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
*/
pub struct WindowsTerminal {
    input_handle: HANDLE,
    output_handle: HANDLE,
}

impl WindowsTerminal {
    pub fn new() -> <WindowsTerminal as TerminalTrait>::Result<WindowsTerminal> {
        unsafe {
            let input = GetStdHandle(STD_INPUT_HANDLE)?;
            let output = GetStdHandle(STD_OUTPUT_HANDLE)?;
            Ok(WindowsTerminal {
                input_handle: input,
                output_handle: output,
            })
        }
    }
}

const ESC_SEQ: u8 = 0x1b;

impl TerminalTrait for WindowsTerminal {
    type Result<V> = Result<V>;
}

// See https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#example
impl VirtualSeq for WindowsTerminal {
    fn enable(&self) -> Result<()> {
        let mut mode = CONSOLE_MODE::default();
        unsafe { GetConsoleMode(self.output_handle, &mut mode) }?;
        mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        unsafe { SetConsoleMode(self.output_handle, mode) }?;
        Ok(())
    }
}

impl ReverseIndex for WindowsTerminal {
    fn ri(&self, buf: &mut VirtualSequenceBuffer) {
        // FIXME
        assert!(buf.cap() >= 2);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'm';
        buf.len += 1;
    }
}

impl StoreCursor for WindowsTerminal {
    fn decsc(&self, buf: &mut VirtualSequenceBuffer) {
        // FIXME
        assert!(buf.cap() >= 2);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'7';
        buf.len += 1;
    }
    fn decsr(&self, buf: &mut VirtualSequenceBuffer) {
        // FIXME
        assert!(buf.cap() >= 2);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'8';
        buf.len += 1;
    }
}

impl CursorPosition for WindowsTerminal {
    fn cuu(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(lines <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'A';
        buf.len += 1;
    }
    fn cud(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(lines <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'B';
        buf.len += 1;
    }
    fn cuf(&self, buf: &mut VirtualSequenceBuffer, chars: u16) {
        assert!(chars <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = chars.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'C';
        buf.len += 1;
    }
    fn cub(&self, buf: &mut VirtualSequenceBuffer, chars: u16) {
        assert!(chars <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = chars.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'D';
        buf.len += 1;
    }
    fn cnl(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(lines <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'E';
        buf.len += 1;
    }
    fn cpl(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(lines <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'F';
        buf.len += 1;
    }
    fn cha(&self, buf: &mut VirtualSequenceBuffer, x: u16) {
        assert!(x <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = x.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'G';
        buf.len += 1;
    }
    fn vpa(&self, buf: &mut VirtualSequenceBuffer, y: u16) {
        assert!(y <= 32_767);
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = y.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'G';
        buf.len += 1;
    }
    fn cup(&self, buf: &mut VirtualSequenceBuffer, x: u16, y: u16) {
        assert!(x <= 32_767);
        assert!(y <= 32_767);
        assert!(buf.cap() >= 14);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = y.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b';';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = x.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'H';
        buf.len += 1;
    }
    fn hvp(&self, buf: &mut VirtualSequenceBuffer, x: u16, y: u16) {
        assert!(x <= 32_767);
        assert!(y <= 32_767);
        assert!(buf.cap() >= 14);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = y.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b';';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = x.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'f';
        buf.len += 1;
    }
}

impl CursorBlink for WindowsTerminal {
    fn att160(&self, buf: &mut VirtualSequenceBuffer, state: bool) {
        assert!(buf.cap() >= 6);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        buf.buf[buf.len] = b'?';
        buf.len += 1;
        buf.buf[buf.len] = b'1';
        buf.len += 1;
        buf.buf[buf.len] = b'2';
        buf.len += 1;
        buf.buf[buf.len] = if state { b'h' } else { b'l' };
        buf.len += 1;
    }
}

impl CursorVisibility for WindowsTerminal {
    fn dectcem(&self, buf: &mut VirtualSequenceBuffer, state: bool) {
        assert!(buf.cap() >= 6);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        buf.buf[buf.len] = b'?';
        buf.len += 1;
        buf.buf[buf.len] = b'2';
        buf.len += 1;
        buf.buf[buf.len] = b'5';
        buf.len += 1;
        buf.buf[buf.len] = if state { b'h' } else { b'l' };
        buf.len += 1;
    }
}

impl CursorShape for WindowsTerminal {
    fn decscusr(&self, buf: &mut VirtualSequenceBuffer, shape: CursorShapes) {
        assert!(buf.cap() >= 6);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        buf.buf[buf.len] = shape as u8;
        buf.len += 1;
        buf.buf[buf.len] = b'S';
        buf.len += 1;
        buf.buf[buf.len] = b'P';
        buf.len += 1;
        buf.buf[buf.len] = b'q';
        buf.len += 1;
    }
}

impl Scrolling for WindowsTerminal {
    fn su(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(buf.cap() >= 8);
        assert!(lines <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'S';
        buf.len += 1;
    }
    fn sd(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(buf.cap() >= 8);
        assert!(lines <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'T';
        buf.len += 1;
    }
}

impl TextMod for WindowsTerminal {
    fn ich(&self, buf: &mut VirtualSequenceBuffer, chars: u16) {
        assert!(buf.cap() >= 8);
        assert!(chars <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = chars.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'@';
        buf.len += 1;
    }
    fn dch(&self, buf: &mut VirtualSequenceBuffer, chars: u16) {
        assert!(buf.cap() >= 8);
        assert!(chars <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = chars.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'P';
        buf.len += 1;
    }
    fn ech(&self, buf: &mut VirtualSequenceBuffer, chars: u16) {
        assert!(buf.cap() >= 8);
        assert!(chars <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = chars.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'X';
        buf.len += 1;
    }
    fn il(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(buf.cap() >= 8);
        assert!(lines <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'L';
        buf.len += 1;
    }
    fn dl(&self, buf: &mut VirtualSequenceBuffer, lines: u16) {
        assert!(buf.cap() >= 8);
        assert!(lines <= 32_767);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        let mut num_buf: NumberBuffer<u16> = NumberBuffer::from(&mut buf.buf[buf.len..]);
        let num = lines.forward_format(&mut num_buf);
        buf.len += num;
        buf.buf[buf.len] = b'M';
        buf.len += 1;
    }
    fn ed(&self, buf: &mut VirtualSequenceBuffer, mode: crate::virtkeys::EraseMode) {
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        buf.buf[buf.len] = mode as u8;
        buf.len += 1;
        buf.buf[buf.len] = b'J';
        buf.len += 1;
    }
    fn el(&self, buf: &mut VirtualSequenceBuffer, mode: crate::virtkeys::EraseMode) {
        assert!(buf.cap() >= 8);
        buf.buf[buf.len] = ESC_SEQ;
        buf.len += 1;
        buf.buf[buf.len] = b'[';
        buf.len += 1;
        buf.buf[buf.len] = mode as u8;
        buf.len += 1;
        buf.buf[buf.len] = b'K';
        buf.len += 1;
    }
}
