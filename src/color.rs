use crate::{get_handle_output, TERMINAL};
use windows::Win32::System::Console::{
    SetConsoleTextAttribute, BACKGROUND_BLUE, BACKGROUND_GREEN, BACKGROUND_INTENSITY,
    BACKGROUND_RED, COMMON_LVB_REVERSE_VIDEO, CONSOLE_CHARACTER_ATTRIBUTES, FOREGROUND_BLUE,
    FOREGROUND_GREEN, FOREGROUND_INTENSITY, FOREGROUND_RED,
};

pub struct Attributes {
    text_color: Color,
    backgound_color: Color,
}

pub struct Color {
    red: bool,
    green: bool,
    blue: bool,
    intensity: bool,
}

pub trait ColorUtils {
    fn set_attr(attr: Attributes) -> Result<(), ()> {
        unsafe {
            let handle = get_handle_output!();
            if SetConsoleTextAttribute(
                handle,
                attr.backgound_color.get(false) | attr.text_color.get(true),
            )
            .is_ok()
            {
                match TERMINAL {
                    Some(ref ter) => ter.lock().unwrap().attr = attr,
                    None => panic!("Terminal not initilised"),
                }
                Ok(())
            } else {
                Err(())
            }
        }
    }
    fn switch_colors() -> Result<(), ()> {
        unsafe {
            let handle = get_handle_output!();
            if SetConsoleTextAttribute(handle, COMMON_LVB_REVERSE_VIDEO).is_ok() {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

impl Color {
    fn get(&self, con: bool) -> CONSOLE_CHARACTER_ATTRIBUTES {
        if con {
            (if self.blue {
                FOREGROUND_BLUE
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            }) | (if self.red {
                FOREGROUND_RED
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            }) | (if self.green {
                FOREGROUND_GREEN
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            }) | (if self.intensity {
                FOREGROUND_INTENSITY
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            })
        } else {
            (if self.blue {
                BACKGROUND_BLUE
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            }) | (if self.red {
                BACKGROUND_RED
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            }) | (if self.green {
                BACKGROUND_GREEN
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            }) | (if self.intensity {
                BACKGROUND_INTENSITY
            } else {
                CONSOLE_CHARACTER_ATTRIBUTES(0)
            })
        }
    }
    pub fn black() -> Self {
        Color {
            red: false,
            green: false,
            blue: false,
            intensity: false,
        }
    }
    pub fn white() -> Self {
        Color {
            red: true,
            green: true,
            blue: true,
            intensity: false,
        }
    }
    pub fn red() -> Self {
        Color {
            red: true,
            blue: false,
            green: false,
            intensity: false,
        }
    }
    pub fn blue() -> Self {
        Color {
            red: false,
            blue: true,
            green: false,
            intensity: false,
        }
    }
    pub fn green() -> Self {
        Color {
            red: false,
            blue: false,
            green: true,
            intensity: false,
        }
    }
}

impl From<CONSOLE_CHARACTER_ATTRIBUTES> for Attributes {
    fn from(value: CONSOLE_CHARACTER_ATTRIBUTES) -> Self {
        let mut text = Color::white();
        let mut background = Color::black();
        if (value & FOREGROUND_BLUE).0 != 0 {
            text.blue = true
        }
        if (value & FOREGROUND_RED).0 != 0 {
            text.red = true
        }
        if (value & FOREGROUND_GREEN).0 != 0 {
            text.green = true
        }
        if (value & FOREGROUND_INTENSITY).0 != 0 {
            text.intensity = true
        }
        if (value & BACKGROUND_BLUE).0 != 0 {
            background.blue = true
        }
        if (value & BACKGROUND_RED).0 != 0 {
            background.red = true
        }
        if (value & BACKGROUND_GREEN).0 != 0 {
            background.green = true
        }
        if (value & BACKGROUND_INTENSITY).0 != 0 {
            background.intensity = true
        }
        Attributes {
            text_color: text,
            backgound_color: background,
        }
    }
}
