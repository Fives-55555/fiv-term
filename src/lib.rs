use fiv_log::{log, ERROR};
use std::{iter::FusedIterator, sync::Mutex};

use windows::Win32::{
    Foundation::HANDLE,
    System::Console::{
        GetConsoleScreenBufferInfo, GetNumberOfConsoleInputEvents, GetStdHandle, ReadConsoleInputW,
        SetConsoleCursorPosition, WriteConsoleW, CONSOLE_SCREEN_BUFFER_INFO, COORD, INPUT_RECORD,
        KEY_EVENT, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    },
};

mod commands;
mod loadbar;

mod macros;
mod page;
mod color;

pub use crate::color::{ColorUtils, Color, Attributes};

pub use crate::page::{Content, Page, PageUtils};

pub use crate::loadbar::Loadbar;

static mut TERMINAL: Option<Mutex<TerminalStat>> = None;

struct TerminalStat {
    pub output: HANDLE,
    pub input: HANDLE,
    pub attr: Attributes,
}

pub struct Terminal {}

pub(crate) fn get_console_handle() -> Result<(HANDLE, HANDLE), ()> {
    unsafe {
        let output = GetStdHandle(STD_OUTPUT_HANDLE);
        let input = GetStdHandle(STD_INPUT_HANDLE);
        match (input, output) {
            (Ok(input), Ok(output)) if !(input.is_invalid() && output.is_invalid()) => {
                return Ok((input, output))
            }
            _ => return Err(()),
        }
    }
}

impl Terminal {
    pub fn new() -> Result<Terminal, ()> {
        unsafe {
            if TERMINAL.is_some() {
                return Err(());
            }
            match get_console_handle() {
                Ok(handle) => {
                    let attr = {
                        let mut info = std::mem::zeroed();
                        if GetConsoleScreenBufferInfo(handle.1, core::ptr::addr_of_mut!(info))
                            .is_err()
                        {
                            return Err(());
                        }
                        Attributes::from(info.wAttributes)
                    };
                    TERMINAL = Some(Mutex::new(TerminalStat {
                        input: handle.0,
                        output: handle.1,
                        attr: attr,
                    }));
                    return Ok(Terminal {});
                }
                Err(_) => log(ERROR, "Couldnt get Console Handle"),
            }
        }
        Err(())
    }
    pub fn clear(&self) {
        self.clear_history().unwrap();
        self.blank().unwrap();
    }
    pub fn clear_history(&self) -> Result<(), ()> {
        //ScrollConsoleScreenBufferA dosen t work because in cmd there is no scrollback only a main buffer, but in modern term. there are
        unsafe {
            let handle = get_handle_output!(noerr);
            let sequence = "\x1b[3J".encode_utf16().collect::<Vec<u16>>();
            if WriteConsoleW(handle, &sequence, None, None).is_err() {
                return Err(());
            };
            self.set_pos(0, 0)?;
            Ok(())
        }
    }
    pub fn blank(&self) -> Result<(), ()> {
        unsafe {
            let handle = get_handle_output!(noerr);
            let sequence = "\x1b[2J".encode_utf16().collect::<Vec<u16>>();
            if WriteConsoleW(handle, &sequence, None, None).is_err() {
                return Err(());
            };
            Ok(())
            //let size = self.get_size()?;let len = size.0 as u32 * size.1 as u32;let handle = get_handle_output!();let mut chars_written = 0;if FillConsoleOutputCharacterW(handle,' ' as u16,len,COORD { X: 0, Y: 0 },&mut chars_written,).is_err() {return Err(());}Ok(())
        }
    }
}

impl Terminal {
    fn set_pos(&self, x: i16, y: i16) -> Result<(), ()> {
        unsafe {
            let handle = get_handle_output!();
            let pos = COORD { X: x, Y: y };
            if SetConsoleCursorPosition(handle, pos).is_err() {
                return Err(());
            };
            Ok(())
        }
    }
    fn get_size(&self) -> Result<(usize, usize), ()> {
        unsafe {
            let handle = get_handle_output!();

            let mut coninfo: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
            if GetConsoleScreenBufferInfo(handle, &mut coninfo).is_ok() {
                let size: COORD = coninfo.dwSize;
                let (x, y) = (size.X, size.Y);
                return Ok((x as usize, y as usize));
            } else {
                return Err(());
            }
        }
    }
    fn clear_footer(&self) {
        unsafe {
            let size = self.get_size().unwrap();
            self.set_pos(0, size.1 as i16 - 1).unwrap();
            let handle = get_handle_output!(noerr);
            let buffer = std::iter::repeat(' ')
                .take(size.0)
                .collect::<String>()
                .encode_utf16()
                .collect::<Vec<u16>>();
            WriteConsoleW(handle, &buffer, None, None).unwrap()
        }
    }
}

const IKEY_EVENT: u16 = KEY_EVENT as u16;

pub fn get_key() -> Option<Key> {
    unsafe {
        let handle = get_handle_input!(noerr);
        let mut events: u32 = 0;
        //Gets the amount of Input Events
        if GetNumberOfConsoleInputEvents(handle, core::ptr::addr_of_mut!(events)).is_err() {
            return None;
        };
        if events > 0 {
            let mut buffer: [INPUT_RECORD; 1] = std::mem::zeroed();
            let mut read: u32 = 0;
            if ReadConsoleInputW(handle, &mut buffer, core::ptr::addr_of_mut!(read)).is_err() {
                return None;
            }
            match buffer[0].EventType {
                IKEY_EVENT => {
                    if buffer[0].Event.KeyEvent.bKeyDown.as_bool() {
                        return Some(buffer[0].Event.KeyEvent.wVirtualKeyCode);
                    }
                }
                _ => return None,
            }
        }
        return None;
    }
}

#[test]
fn test() {
    use crate::commands::{Action, Arg, Commands};
    let t = std::time::SystemTime::now();
    let ta = t.clone();

    let _ = std::thread::spawn(move || {
        if std::time::SystemTime::duration_since(&std::time::SystemTime::now(), ta)
            .unwrap()
            .as_secs()
            > 20
        {
            println!("x");
            std::process::exit(0x0)
        }
    });

    log(fiv_log::INFO, "0");
    let c = Commands::from_string("rust".to_string()).unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(c.args() == Vec::new());
    log(fiv_log::INFO, "1");

    let c = Commands::from_string("stop".to_string()).unwrap();
    assert!(c.action() == Action::Stop);
    assert!(c.args() == Vec::new());
    log(fiv_log::INFO, "2");

    let c = Commands::from_string("help".to_string()).unwrap();
    assert!(c.action() == Action::Help);
    assert!(c.args() == Vec::new());
    log(fiv_log::INFO, "3");

    let c = Commands::from_string("rust -t".to_string()).unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(c.args() == vec![Arg::Flag("t".to_string())]);
    log(fiv_log::INFO, "4");

    let c = Commands::from_string("rust --terminal".to_string()).unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(c.args() == vec![Arg::Flag("terminal".to_string())]);
    log(fiv_log::INFO, "5");

    let c = Commands::from_string("rust --terminal=true".to_string()).unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(c.args() == vec![Arg::Inner(("terminal".to_string(), "true".to_string()))]);
    log(fiv_log::INFO, "6");

    let c = Commands::from_string("rust --terminal=\"Is true\"".to_string()).unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(c.args() == vec![Arg::Inner(("terminal".to_string(), "Is true".to_string()))]);
    log(fiv_log::INFO, "7");

    let c = Commands::from_string("rust --terminal=\"Is true\" --rust=\"Is stupid\"".to_string())
        .unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(
        c.args()
            == vec![
                Arg::Inner(("terminal".to_string(), "Is true".to_string())),
                Arg::Inner(("rust".to_string(), "Is stupid".to_string()))
            ]
    );
    log(fiv_log::INFO, "8");

    let c = Commands::from_string(
        "rust -t -r --terminal --rust --terminal=\"Is true\" --rust=\"Is stupid\"".to_string(),
    )
    .unwrap();
    assert!(c.action() == Action::Unknown("rust".to_string()));
    assert!(
        c.args()
            == vec![
                Arg::Flag("t".to_string()),
                Arg::Flag("r".to_string()),
                Arg::Flag("terminal".to_string()),
                Arg::Flag("rust".to_string()),
                Arg::Inner(("terminal".to_string(), "Is true".to_string())),
                Arg::Inner(("rust".to_string(), "Is stupid".to_string()))
            ]
    );
    log(fiv_log::INFO, "9");

    assert!(Commands::from_string("".to_string()).is_err());
    log(fiv_log::INFO, "10");

    println!(
        "{:?}",
        std::time::SystemTime::duration_since(&std::time::SystemTime::now(), t).unwrap()
    )
}

//https://learn.microsoft.com/de-de/windows/win32/inputdev/virtual-key-codes
pub type Key = u16;

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
