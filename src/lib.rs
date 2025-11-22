use std::iter::FusedIterator;
use std::ops::BitAnd;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

mod terminal;

mod commands;
mod loadbar;

mod color;
mod page;

pub(crate) mod virtkeys;

pub use crate::color::{Attributes, Color};

pub use crate::page::{Content, Page, PageUtils};

pub use crate::loadbar::Loadbar;

// pub trait LenLinesAdd {
//     fn lenlines(&self, len: usize) -> LenLines<'_>;
// }

// #[derive(Clone, Copy)]
// pub struct LenLines<'a> {
//     str: &'a str,
//     len: usize,
// }

// impl<'a> Iterator for LenLines<'a> {
//     type Item = &'a str;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.str.is_empty() {
//             return None;
//         };

//         let mut chars = self.str.chars();

//         let mut b = false;

//         for i in 0..self.len {
//             if b {
//                 b = false;
//                 continue;
//             }

//             let char = match chars.next() {
//                 Some(char) => char,
//                 None => {
//                     let line = self.str;
//                     self.str = &self.str[0..0];
//                     return Some(line);
//                 }
//             };

//             if char == '\r' {
//                 match chars.next() {
//                     Some(char) if char == '\n' => (),
//                     _ => {
//                         b = true;
//                         continue;
//                     }
//                 };
//                 let line = &self.str[0..i - 1];
//                 self.str = &self.str[i + 1..];
//                 return Some(line);
//             }

//             if char == '\n' {
//                 let line = &self.str[0..i];
//                 self.str = &self.str[i + 1..];
//                 return Some(line);
//             }
//         }
//         let line = &self.str[0..self.len];
//         self.str = &self.str[self.len..];
//         return Some(line);
//     }
// }

// impl<'a> DoubleEndedIterator for LenLines<'a> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         if self.str.is_empty() {
//             return None;
//         };

//         let len = self.str.len();

//         let mut chars = self.str.chars();

//         let mut b = false;

//         for i in 0..self.len {
//             if b {
//                 b = false;
//                 continue;
//             }

//             let char = match chars.next_back() {
//                 Some(char) => char,
//                 None => {
//                     let line = self.str;
//                     self.str = &self.str[0..0];
//                     return Some(line);
//                 }
//             };

//             if char == '\n' {
//                 let line = &self.str[len - i..];
//                 match chars.next_back() {
//                     Some(char) if char == '\r' => {
//                         self.str = &self.str[..len - (i + 1)];
//                     }
//                     _ => {
//                         self.str = &self.str[..len - i];
//                     }
//                 };
//                 return Some(line);
//             }
//         }
//         let line = &self.str[len - self.len..];
//         self.str = &self.str[..len - self.len];
//         return Some(line);
//     }
// }

// impl FusedIterator for LenLines<'_> {}

// impl LenLinesAdd for &str {
//     fn lenlines(&self, len: usize) -> LenLines<'_> {
//         LenLines {
//             str: self,
//             len: len,
//         }
//     }
// }

// impl LenLinesAdd for String {
//     fn lenlines(&self, len: usize) -> LenLines<'_> {
//         LenLines {
//             str: self,
//             len: len,
//         }
//     }
// }

pub struct Terminal {
   // Two Dim Render 
}


#[cfg(not(feature = "ter_test"))]
#[test]
fn try_vis() -> Result<()> {
    use crate::terminal::Terminal;

    let terminal = Terminal::new();
    Ok(())
}

// -----------------------------
// |                           |
// |                           |
// |                           |
// |                           |
// |                           |
// |===========================|
// |                           |
// -----------------------------

pub trait Bitfield {
    fn has(&self, bit: &Self)->bool {
        (self & bit) != 0
    }
}

pub(crate) enum Footer {
    Basic = 1,
    Loadbar = 2,
    Keys = 4,
}

impl Footer {
    // Calculates the Area of the Loadbar (including the Delimiter('|')?)
    pub const fn calc_loadbar(size: (usize, usize)) -> (usize, usize) {
        let loadbar_size = size.0 / 4;
        let loadbar_pos = size.0 - loadbar_size - 1;
        (loadbar_pos, loadbar_size)
    }
}
