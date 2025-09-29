use fiv_log::{log, ERROR};
use std::iter::FusedIterator;

use windows::{
    core::Result,
    Win32::{
        Foundation::HANDLE,
        System::Console::{
            CreateConsoleScreenBuffer, GetConsoleScreenBufferInfo, GetNumberOfConsoleInputEvents,
            GetStdHandle, ReadConsoleInputW, SetConsoleActiveScreenBuffer,
            SetConsoleCursorPosition, WriteConsoleW, CONSOLE_SCREEN_BUFFER_INFO,
            CONSOLE_TEXTMODE_BUFFER, COORD, INPUT_RECORD, KEY_EVENT, STD_INPUT_HANDLE,
            STD_OUTPUT_HANDLE,
        },
    },
};

mod terminal;

mod commands;
mod loadbar;

mod color;
mod macros;
mod page;

pub use crate::color::{Attributes, Color, ColorUtils};

pub use crate::page::{Content, Page, PageUtils};

pub use crate::loadbar::Loadbar;

pub trait LenLinesAdd {
    fn lenlines(&self, len: usize) -> LenLines<'_>;
}

#[derive(Clone, Copy)]
pub struct LenLines<'a> {
    str: &'a str,
    len: usize,
}

impl<'a> Iterator for LenLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.str.is_empty() {
            return None;
        };

        let mut chars = self.str.chars();

        let mut b = false;

        for i in 0..self.len {
            if b {
                b = false;
                continue;
            }

            let char = match chars.next() {
                Some(char) => char,
                None => {
                    let line = self.str;
                    self.str = &self.str[0..0];
                    return Some(line);
                }
            };

            if char == '\r' {
                match chars.next() {
                    Some(char) if char == '\n' => (),
                    _ => {
                        b = true;
                        continue;
                    }
                };
                let line = &self.str[0..i - 1];
                self.str = &self.str[i + 1..];
                return Some(line);
            }

            if char == '\n' {
                let line = &self.str[0..i];
                self.str = &self.str[i + 1..];
                return Some(line);
            }
        }
        let line = &self.str[0..self.len];
        self.str = &self.str[self.len..];
        return Some(line);
    }
}

impl<'a> DoubleEndedIterator for LenLines<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.str.is_empty() {
            return None;
        };

        let len = self.str.len();

        let mut chars = self.str.chars();

        let mut b = false;

        for i in 0..self.len {
            if b {
                b = false;
                continue;
            }

            let char = match chars.next_back() {
                Some(char) => char,
                None => {
                    let line = self.str;
                    self.str = &self.str[0..0];
                    return Some(line);
                }
            };

            if char == '\n' {
                let line = &self.str[len - i..];
                match chars.next_back() {
                    Some(char) if char == '\r' => {
                        self.str = &self.str[..len - (i + 1)];
                    }
                    _ => {
                        self.str = &self.str[..len - i];
                    }
                };
                return Some(line);
            }
        }
        let line = &self.str[len - self.len..];
        self.str = &self.str[..len - self.len];
        return Some(line);
    }
}

impl FusedIterator for LenLines<'_> {}

impl LenLinesAdd for &str {
    fn lenlines(&self, len: usize) -> LenLines<'_> {
        LenLines {
            str: self,
            len: len,
        }
    }
}

impl LenLinesAdd for String {
    fn lenlines(&self, len: usize) -> LenLines<'_> {
        LenLines {
            str: self,
            len: len,
        }
    }
}

#[cfg(not(feature = "ter_test"))]
#[test]
fn try_vis() -> Result<()> {
    use crate::terminal::Terminal;

    let terminal = Terminal::new();
    Ok(())
}
