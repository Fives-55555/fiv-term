use std::ops::{Index, IndexMut};

use fiv_log::{log, ERROR};
#[cfg(not(feature = "debug"))]
use windows::Win32::System::Console::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};
use windows::{
    core::Result,
    Win32::{
        Foundation::HANDLE,
        System::Console::{
            CreateConsoleScreenBuffer, GetNumberOfConsoleInputEvents, GetStdHandle,
            ReadConsoleInputW, SetConsoleActiveScreenBuffer, SetConsoleCursorPosition,
            WriteConsoleOutputCharacterW, WriteConsoleOutputW, CHAR_INFO, CHAR_INFO_0,
            CONSOLE_TEXTMODE_BUFFER, COORD, INPUT_RECORD, KEY_EVENT, SMALL_RECT, STD_INPUT_HANDLE,
            STD_OUTPUT_HANDLE,
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
        unsafe {
            match ScreenBuffer::get_std_handles() {
                Ok(handles) => {
                    #[cfg(not(feature = "debug"))]
                    let attr = {
                        use windows::Win32::System::Console::{
                            GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO,
                        };

                        let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
                        GetConsoleScreenBufferInfo(
                            handles.output.0,
                            &mut info as *mut CONSOLE_SCREEN_BUFFER_INFO,
                        )?;
                        Attributes::from(info.wAttributes)
                    };
                    #[cfg(feature = "debug")]
                    let attr = {
                        use windows::Win32::System::Console::CONSOLE_CHARACTER_ATTRIBUTES;
                        Attributes::from(CONSOLE_CHARACTER_ATTRIBUTES(0))
                    };
                    return Ok(Terminal {
                        active_buffer: handles.output,
                        std_handles: handles,
                        attributes: attr,
                        title_size: 0,
                        colors: (),
                    });
                }
                Err(err) => {
                    log(ERROR, "Couldnt get Console Handle");
                    return Err(err);
                }
            }
        }
    }
    pub fn set_buffer(&mut self, buffer: ScreenBuffer) -> Result<()> {
        unsafe { SetConsoleActiveScreenBuffer(buffer.0)? };
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
}

impl Terminal {
    fn set_pos(&self, x: i16, y: i16) -> Result<()> {
        unsafe {
            let handle = self.active_buffer;
            let pos = COORD { X: x, Y: y };
            SetConsoleCursorPosition(handle.0, pos)
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
    pub fn get_key(&self) -> Option<(Key, char)> {
        unsafe {
            let handle = self.std_handles.input;
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
                            return Some((
                                buffer[0].Event.KeyEvent.wVirtualKeyCode,
                                char::from_u32(buffer[0].Event.KeyEvent.uChar.UnicodeChar.into())
                                    .or(Some(' '))
                                    .unwrap(),
                            ));
                        }
                    }
                    _ => return None,
                }
            }
            return None;
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

pub struct StdHandles {
    input: HANDLE,
    output: ScreenBuffer,
}

#[derive(Clone, Copy)]
pub struct ScreenBuffer {
    handle: HANDLE,
    attr: Attributes,
}

impl ScreenBuffer {
    pub fn get_std_handles() -> Result<StdHandles> {
        unsafe {
            let input = GetStdHandle(STD_INPUT_HANDLE)?;
            let output = ScreenBuffer(GetStdHandle(STD_OUTPUT_HANDLE)?);
            Ok(StdHandles {
                input: input,
                output: output,
            })
        }
    }
    pub fn new_buffer() -> Result<ScreenBuffer> {
        unsafe {
            CreateConsoleScreenBuffer(0, 0, None, CONSOLE_TEXTMODE_BUFFER, None).map(ScreenBuffer)
        }
    }
    pub fn get_size(&self) -> Result<(usize, usize)> {
        let size: (usize, usize) = {
            #[cfg(not(feature = "debug"))]
            {
                let mut buf: CONSOLE_SCREEN_BUFFER_INFO = CONSOLE_SCREEN_BUFFER_INFO::default();
                unsafe {
                    GetConsoleScreenBufferInfo(
                        self.0,
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
        self.0
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
                self.0,
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
                self.0,
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
