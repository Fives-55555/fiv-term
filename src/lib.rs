#![feature(int_from_ascii)]
#![feature(int_format_into)]
#![feature(stmt_expr_attributes)]
#![feature(ptr_mask)]
#![allow(internal_features)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(str_as_str)]
#![feature(const_heap)]
#![feature(box_vec_non_null)]
#![feature(write_all_vectored)]
#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]
#![feature(exact_size_is_empty)]

mod stuff;

pub use stuff::{/*AlignedPop,*/ AlignedPush, SeqBuf, Stack};

pub mod stdio;

pub mod virtseq;

pub use virtseq::TermInfoConfig;

mod tests;

pub mod pages;

pub use pages::TermView;

// pub use crate::stuff::{FastForwardFormat, NumberBuffer};

// mod tests;

// #[cfg(target_os = "linux")]
// mod linux;

#[cfg(target_os = "windows")]
mod windows;

// pub mod virtkeys;

/*
mod terminal;
mod stuff;

mod commands;
mod loadbar;

mod color;
mod page;

pub(crate) mod virtkeys;

pub use crate::color::{Attributes, Color};

pub use crate::page::{Content, Page, PageUtils};

pub use crate::loadbar::Loadbar;
*/
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

#[cfg(feature = "ter_test")]
#[test]
fn try_vis() {}

// -----------------------------
// |                           |
// |                           |
// |                           |
// |                           |
// |                           |
// |===========================|
// |                           |
// -----------------------------

pub(crate) enum _Footer {
    Basic = 1,
    Loadbar = 2,
    Keys = 4,
}

impl _Footer {
    // Calculates the Area of the Loadbar (including the Delimiter('|')?)
    pub const fn _calc_loadbar(size: (usize, usize)) -> (usize, usize) {
        let loadbar_size = size.0 / 4;
        let loadbar_pos = size.0 - loadbar_size - 1;
        (loadbar_pos, loadbar_size)
    }
}

pub trait TerminalTrait {
    type Result<V>;
}

#[cfg(target_os = "linux")]
pub type TermConfig = TermInfoConfig;
