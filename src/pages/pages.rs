use std::sync::LazyLock;

pub struct TermPage {
    pub content: Vec<PageWidget>,
    pub page_size: SizeConfig,
}

impl TermPage {
    pub const DEFAULT: LazyLock<TermPage> = LazyLock::new(|| TermPage {
        content: vec![PageWidget::DEFAULT],
    });

    pub fn new() -> TermPage {
        TermPage {
            content: Vec::new(),
        }
    }
    pub fn add_widget(&mut self, widget: PageWidget) {
        self.content.push(widget);
        self.resize();
    }
    pub fn resize(&mut self) {
        let size = self.page_size;
        let mut widgets = self.content.iter();

        let wid = match widgets.next() {
            Some(wid) => wid,
            None => return,
        };

        while !widgets.is_empty() {}
        // How to SIZE Stuff
        // FIXME
    }
}

pub struct PageWidget {
    widget_type: WidgetType,
    size_config: SizeConfig,
    // Top-left Corner
    corner: (u16, u16),
    size: (u16, u16),
}

impl PageWidget {
    pub const DEFAULT: PageWidget = PageWidget::from_static_str(
        "!!!!!!!!!!!!!!-DEFAULT PAGE-!!!!!!!!!!!!!!!!!!!!!\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!--SOMETHING HAS GONE WRONG--!!!!!!!!!!!!",
    );
    pub const fn from_static_str(str: &'static str) -> PageWidget {
        PageWidget {
            widget_type: WidgetType::StaticTextbox(str),
            size_config: SizeConfig::default(),
            corner: (0, 0),
            size: (0, 0),
        }
    }
    pub fn from_final_string(string: String) -> PageWidget {
        PageWidget {
            widget_type: WidgetType::RuntimeTextbox(string.into_boxed_str()),
            size_config: SizeConfig::default(),
            corner: (0, 0),
            size: (0, 0),
        }
    }
    pub fn from_string(string: String) -> PageWidget {
        PageWidget {
            widget_type: WidgetType::MutableTextbox(string),
            size_config: SizeConfig::default(),
            corner: (0, 0),
            size: (0, 0),
        }
    }
    pub fn text_field(string: String, prefix: u8, max_size: u8, suffix: u8) -> PageWidget {
        assert!(string.len() == prefix as usize + suffix as usize);
        PageWidget {
            widget_type: WidgetType::TextField(String::new()),
            size_config: SizeConfig::from_text_field(prefix, max_size, suffix),
            corner: (0, 0),
            size: (0, 0),
        }
    }
}

/// The Struct consists out of the width and height size config
/// |  SizeConfig   |
/// | width | height|
/// |  u32  |  u32  |
///
/// |             Structure (u32)              |
/// |------------------------------------------|
/// | Type | minSize |        TypeData         |
/// |  u2  |   u14   |           u16           |
/// |------------------------------------------|
/// |                |           Pad           |
/// |           Flex |           u16           |
/// |------------------------------------------|
/// |                | numerator | denominator |
/// |          Ratio |    u8     |     u8      |
/// |------------------------------------------|
/// |                |       Fixed Size        |
/// |          Fixed |           u16           |
///
#[repr(align(8))]
pub struct SizeConfig([u32; 2]);

impl SizeConfig {
    pub const U14_MAX: u16 = 0b11111111111111;

    pub const fn default() -> SizeConfig {
        SizeConfig([SizeConfig::flex(), SizeConfig::flex()])
    }
    pub fn get_type(&self) -> SizeType {
        unsafe { std::mem::transmute(self.0.to_le_bytes()[3]) }
    }
    pub const fn flex(min_size: u16) -> u32 {
        debug_assert!(min_size <= Self::U14_MAX);
        let min = (min_size & Self::U14_MAX) as u32;
        ((SizeType::Flex as u32) << 14 | min) << 16
    }
    pub fn from_ratio(x: u16, y: u16) -> SizeConfig {
        assert!(x <= Self::U12MAX && y <= Self::U12MAX);
        SizeConfig((SizeType::Ratio as u32) << 24 | (x as u32) << 12 | y as u32)
    }
    pub fn from_fixed(x: u16, y: u16) -> SizeConfig {
        assert!(x <= Self::U12MAX && y <= Self::U12MAX);
        SizeConfig((SizeType::Fixed as u32) << 24 | (x as u32) << 12 | y as u32)
    }
    pub fn get_ratio(&self) -> (u8, u8) {
        let bytes = self.0.to_le_bytes();
        (bytes[0], bytes[1])
    }
    pub fn get_fixed(&self) -> (u16, u16) {
        let num = self.0.to_le();
        (num as u16 & Self::U12MAX, (num >> 12) as u16 & Self::U12MAX)
    }
}

#[repr(u8)]
pub enum SizeType {
    Flex = 0,
    Ratio = 1,
    Fixed = 2,
}

pub enum WidgetType {
    Line {
        len: usize,
        // Bitfield (0 | 1)
        // bit 0 (LSB) = alignleft | alignsides
        bitfield: u8,
    },
    /// A Textbox which content is set at compile-time
    StaticTextbox(&'static str),
    /// A Textbox which needs to be computed during runtime
    RuntimeTextbox(Box<str>),
    /// A Textbox which needs to be changed during runtime
    MutableTextbox(String),
    /// Set max char size by size_config
    TextField(String),
    // FIXME
    /// Current Number, min, max
    NumberField {
        num: usize,
        min_bound: usize,
        max_bound: usize,
    },
}
