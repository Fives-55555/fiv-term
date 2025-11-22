use std::ops::{Index, IndexMut};

use fiv_log::{log, ERROR};
#[cfg(not(feature = "debug"))]
use windows::Win32::System::Console::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};
use windows::{
    core::Result,
    Win32::{
        Foundation::HANDLE,
        System::Console::{
            CreateConsoleScreenBuffer, GetStdHandle, ReadConsoleInputW,
            SetConsoleActiveScreenBuffer, SetConsoleCursorPosition, WriteConsoleOutputCharacterW,
            WriteConsoleOutputW, CHAR_INFO, CHAR_INFO_0, CONSOLE_TEXTMODE_BUFFER, COORD,
            INPUT_RECORD, KEY_EVENT, SMALL_RECT, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
        },
    },
};

use crate::Attributes;

const IKEY_EVENT: u16 = KEY_EVENT as u16;

pub struct Terminal {
    active_buffer: ScreenBuffer,
    std_handles: StdHandles,
    title_size: usize,
    colors: (),
}

impl Terminal {
    pub fn new() -> Result<Terminal> {
        match ScreenBuffer::get_std_handles() {
            Ok(handles) => {
                return Ok(Terminal {
                    active_buffer: handles.output,
                    std_handles: handles,
                    title_size: 0,
                    colors: (),
                });
            }
            Err(err) => {
                log(ERROR, "Could not get Standart Handles");
                return Err(err);
            }
        }
    }
    pub fn set_buffer(&mut self, buffer: ScreenBuffer) -> Result<()> {
        unsafe { SetConsoleActiveScreenBuffer(buffer.handle())? };
        self.active_buffer = buffer;
        Ok(())
    }
    pub fn clear(&self) {
        todo!()
    }
    pub fn clear_history(&self) -> Result<()> {
        //ScrollConsoleScreenBufferA dosen t work because in cmd there is no scrollback only a main buffer, but in modern term. there are
        #[cfg(feature = "virt_keys")]
        unsafe {
            let handle = self.active_buffer;
            let sequence = "\x1b[3J".encode_utf16().collect::<Vec<u16>>();
            WriteConsoleW(handle.0, &sequence, None, None)
        }
        todo!()
    }
    pub fn blank(&self) -> Result<()> {
        #[cfg(feature = "virt_keys")]
        unsafe {
            let handle = self.active_buffer;
            let sequence = "\x1b[2J".encode_utf16().collect::<Vec<u16>>();
            WriteConsoleW(handle.0, &sequence, None, None)
        }
        todo!()
    }
    pub fn get_buffer(&self) -> ScreenBuffer {
        self.active_buffer
    }
    pub fn input_handle(&self) -> HANDLE {
        self.std_handles.input
    }
}

impl Terminal {
    fn set_pos(&self, x: i16, y: i16) -> Result<()> {
        unsafe {
            let handle = self.active_buffer;
            let pos = COORD { X: x, Y: y };
            SetConsoleCursorPosition(handle.handle(), pos)
        }
    }
    fn get_size(&self) -> Result<(usize, usize)> {
        let handle = self.active_buffer;
        handle.get_size()
    }
    fn clear_footer(&mut self) {
        let size = self.get_size().unwrap();
        let buffer = std::iter::repeat(' ')
            .take(size.0)
            .collect::<String>()
            .encode_utf16()
            .collect::<Vec<u16>>();
        self.active_buffer
            .write_lin((0, size.1 - 1), &buffer)
            .unwrap()
    }
}

impl Terminal {
    pub fn poll_events(&mut self) -> Result<Option<Event>> {
        unsafe {
            let handle = self.input_handle();
            let mut buffer: [INPUT_RECORD; 1] = [INPUT_RECORD::default()];
            let mut read = 0;
            ReadConsoleInputW(handle, &mut buffer, &mut read)?;
            if read == 1 {
                Ok(Some(Event::from_input(&buffer[0])))
            } else {
                Ok(None)
            }
        }
    }
}

enum Event {
    KeyEvent {
        keydown: bool,
        repeat_counter: u16,
        virt_scan_code: u16,
        virt_key_code: u16,
        control_keys: u32,
        as_char: char,
    },
    MouseEvent {
        pos: (usize, usize),
        button_state: u32,
        control_keys: u32,
        event: u32,
    },
    ResizeEvent(usize, usize),
    FocusEvent(bool),
    MenüEvent(u32),
}

impl Event {
    pub fn from_input(value: &INPUT_RECORD) -> Event {
        match value.EventType {
            1 => {
                let event = unsafe { value.Event.KeyEvent };
                Event::KeyEvent {
                    keydown: event.bKeyDown.as_bool(),
                    repeat_counter: event.wRepeatCount,
                    virt_scan_code: event.wVirtualScanCode,
                    virt_key_code: event.wVirtualKeyCode,
                    control_keys: event.dwControlKeyState,
                    as_char: unsafe { char::from_u32_unchecked(event.uChar.UnicodeChar as u32) },
                }
            }
            2 => {
                let event = unsafe { value.Event.MouseEvent };
                Event::MouseEvent {
                    pos: (
                        event.dwMousePosition.X as usize,
                        event.dwMousePosition.Y as usize,
                    ),
                    button_state: event.dwButtonState,
                    control_keys: event.dwControlKeyState,
                    event: event.dwEventFlags,
                }
            }
            4 => {
                let event = unsafe { value.Event.WindowBufferSizeEvent };
                Event::ResizeEvent(event.dwSize.X as usize, event.dwSize.Y as usize)
            }
            8 => {
                let event = unsafe { value.Event.MenuEvent };
                Event::MenüEvent(event.dwCommandId)
            }
            16 => {
                let event = unsafe { value.Event.FocusEvent };
                Event::FocusEvent(event.bSetFocus.as_bool())
            }
            _ => panic!("Corrupt Event Record"),
        }
    }
}

pub struct TerminalStr {
    x: usize,
    y: usize,
    buf: Vec<CHAR_INFO>,
}

impl TerminalStr {
    pub fn new(x: usize, y: usize, char: Option<CHAR_INFO>) -> TerminalStr {
        let buf: Vec<CHAR_INFO> = vec![char.unwrap_or(CHAR_INFO {
            Char: CHAR_INFO_0 {
                UnicodeChar: b' ' as u16,
            },
            Attributes: 0,
        })];
        TerminalStr {
            x: x,
            y: y,
            buf: buf,
        }
    }
    pub fn resize(&mut self, x: usize, y: usize) {
        let diff = (x * y) as isize - (self.buf.len() as isize);
        match diff {
            0 => (),
            1.. => {
                self.buf.reserve_exact(diff as usize);
                self.x = x;
                self.y = y;
            }
            ..0 => {
                self.buf.shrink_to(x * y);
                self.x = x;
                self.y = y;
            }
        }
    }
}

impl Index<usize> for TerminalStr {
    type Output = [CHAR_INFO];
    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index * self.x..(index + 1) * self.x]
    }
}

impl IndexMut<usize> for TerminalStr {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index * self.x..(index + 1) * self.x]
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
}

//https://learn.microsoft.com/de-de/windows/win32/inputdev/virtual-key-codes
pub type Key = u16;
