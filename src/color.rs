use windows::{
    core::Result,
    Win32::{
        Foundation::HANDLE,
        System::Console::{
            GetConsoleScreenBufferInfo, SetConsoleTextAttribute, CONSOLE_CHARACTER_ATTRIBUTES,
            CONSOLE_SCREEN_BUFFER_INFO,
        },
    },
};

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Attributes {
    // Foreground Color
    FGBlue = 1,      //FOREGROUND_BLUE
    FGGreen = 2,     //FOREGROUND_GREEN
    FGRed = 4,       //FOREGROUND_RED
    FGIntensity = 8, //FOREGROUND_INTENSITY
    // Background Color
    BGBlue = 16,       //BACKGROUND_BLUE
    BGGreen = 32,      //BACKGROUND_GREEN
    BGRed = 64,        //BACKGROUND_RED
    BDIntensity = 128, //BACKGROUND_INTENSITY
    // Common idk
    CMLVBLeading = 256,  //COMMON_LVB_LEADING_BYTE
    CMLVBTrailing = 512, //COMMON_LVB_TRAILING_BYTE
    // Common Grid Idk
    CMLVBHorizontal = 1024, //COMMON_LVB_
    CMLVBLVertical = 2048,  //COMMON_LVB_
    CMLVBRVertical = 4096,  //COMMON_LVB_
    // Common Tool?
    CMLVBReverse = 16384,    //COMMON_LVB_REVERSE_VIDEO
    CMLVBUnderscore = 32768, //COMMON_LVB_UNDERSCORE
}

impl Attributes {
    pub fn is_contained(&self, value: Attributes) -> bool {
        (*self as u16 & value as u16) != 0
    }
    pub fn add(&mut self, value: Attributes) {
        *self = unsafe { std::mem::transmute(*self as u16 | value as u16) };
    }
    pub fn set(buffer: ScreenBuffer, attr: Attributes) -> Result<()> {
        unsafe { SetConsoleTextAttribute(buffer.handle(), attr.as_attr()) }
    }
    pub fn as_attr(&self) -> CONSOLE_CHARACTER_ATTRIBUTES {
        CONSOLE_CHARACTER_ATTRIBUTES(*self as u16)
    }
    pub fn as_color(&self) -> Color {
        Color()
    }
    pub fn get_attrs(handle: HANDLE) -> Result<Attributes> {
        let mut buffer = CONSOLE_SCREEN_BUFFER_INFO::default();
        #[cfg(not(feature = "debug"))]
        unsafe {
            GetConsoleScreenBufferInfo(handle, &mut buffer)?
        }
        Ok(unsafe { std::mem::transmute(buffer.wAttributes) })
    }
}

impl Color {
    pub fn as_attr(&self) -> CONSOLE_CHARACTER_ATTRIBUTES {
        CONSOLE_CHARACTER_ATTRIBUTES(self.0 as u16)
    }
    pub const fn get(&self, id: u8) -> bool {
        self.0 & (1 << id) != 0
    }
    pub const fn set(&mut self, id: u8, value: bool) {
        self.0 = (self.0 & !(1 << id)) | ((value as u8) << id)
    }
    pub const ID_FG_BLUE: u8 = 0;
    pub const ID_FG_GREEN: u8 = 1;
    pub const ID_FG_RED: u8 = 2;
    pub const ID_FG_INT: u8 = 3;
    pub const ID_BG_BLUE: u8 = 4;
    pub const ID_BG_GREEN: u8 = 5;
    pub const ID_BG_RED: u8 = 6;
    pub const ID_BG_INT: u8 = 7;

    pub const FG_WHITE: Color = Color(0b00000111);
    pub const BG_WHITE: Color = Color(0b01110000);

    pub const BLACK: Color = Color(0);
}

impl From<CONSOLE_CHARACTER_ATTRIBUTES> for Attributes {
    fn from(value: CONSOLE_CHARACTER_ATTRIBUTES) -> Self {
        unsafe { std::mem::transmute(value.0) }
    }
}

//4BIt
#[repr(u8)]
pub enum AnsiColor {
    Black,
    Red
}

pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}
