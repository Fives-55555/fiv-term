use crate::{
    get_handle_output,
    terminal::{ScreenBuffer, Terminal, TerminalStr},
    LenLinesAdd,
};
use std::{thread::sleep, time::Duration};
use windows::{
    core::Result,
    Win32::{
        Foundation::HANDLE,
        System::Console::{WriteConsoleW, CHAR_INFO, CHAR_INFO_0},
    },
};

pub const MIN_HEIGHT: usize = 7;
pub const MIN_WIDTH: usize = 75;

#[derive(Clone)]
pub enum Content {
    //The Page itself
    InfoPage(String),
    //An Info Page with a Confirm Function
    ConfirmPage((String, fn() -> Page)),
    //The possible Sites
    SubMenü(Vec<Page>),
    //The Question; The possible Answers; The Answer Processing
    SelectList((String, Vec<String>, fn(&str) -> Page)),
    //The Currrent Search; The Question; The possible Answers; The Answer Processing
    SearchSelectList((String, String, Vec<usize>, fn(&str) -> Page, Vec<String>)),
    //The Question; Entered Text
    TextInput((String, String, fn(&mut String) -> Page)),
    //KeyInp TerSize VisOut Page Ok
    CustomPageRender(fn((u16, char), (usize, usize)) -> (Option<String>, Option<Page>)),
}
#[derive(Clone)]
/// This is an Page Renderer
/// It allows:
///  -> Basic Text Pages(InfoPage)
///  -> Text Input Pages(TextInput)
///  -> Selection Pages(SelectList)
///  -> Custom Rendered Pages with Key Inputs
///  -> Sub Menü Pages
///
/// Possible a enum with Display Trait?
/// Additionally are Page update functions avalible
pub struct Page {
    //UFT-16 Title
    title: String,
    //Updates the Page
    update: Option<fn(menü: &mut Content) -> bool>,
    //Content
    content: Content,
    //Terminal Buffer
    buffer: HANDLE,
}

pub trait PageUtils {
    fn open(&self, page: &mut Page) -> Result<()>;
}

impl PageUtils for Terminal {
    ///Opens a Pages
    fn open(&self, page: &mut Page) -> Result<()> {
        let mut line = 0;

        let screen = ScreenBuffer::new_buffer()?;

        Page::print_page(self, &page.title, &page.content, line)?;

        loop {
            sleep(Duration::from_millis(30));

            match self.get_key() {
                Some(x) => match x.0 {
                    //At every Page Type
                    0x25 => {
                        match &mut page.content {
                            Content::SearchSelectList(inner) => {
                                inner.0.clear();
                                inner.2.clear();
                                for i in 0..inner.4.len() {
                                    inner.2.push(i);
                                }
                            }
                            _ => (),
                        }
                        return Ok(());
                    }
                    0x27 => {
                        match &mut page.content {
                            Content::SubMenü(sm) => {
                                if sm.is_empty() {
                                    ()
                                } else {
                                    self.open(&mut sm[line])?;
                                }
                            }
                            Content::SelectList((_, vec, func)) => {
                                if vec.is_empty() {
                                    ()
                                } else {
                                    self.open(&mut func(&vec[line]))?;
                                }
                            }
                            Content::SearchSelectList((_, _, vec, func, x)) => {
                                if vec.is_empty() {
                                    ()
                                } else {
                                    self.open(&mut func(&x[vec[line]]))?;
                                }
                            }
                            Content::TextInput((_, input, func)) => {
                                self.open(&mut func(input))?;
                            }
                            Content::CustomPageRender(func) => {
                                match func(x, (size.0, size.1 - 4)).1 {
                                    Some(mut page) => {
                                        self.open(&mut page)?;
                                    }
                                    None => (),
                                }
                            }
                            Content::ConfirmPage((_, func)) => {
                                self.open(&mut func())?;
                            }
                            Content::InfoPage(_) => (),
                        }
                        Page::print_page(self, &page.title, &page.content, line)?;
                    }
                    0x26 => {
                        if line > 0 {
                            line -= 1;
                            Page::print_page(self, &page.title, &page.content, line)?;
                        }
                    }
                    0x28 => {
                        if page.content.len(size.0) > 0 && line < page.content.len(size.0) - 1 {
                            line += 1;
                            Page::print_page(self, &page.title, &page.content, line)?;
                        }
                    }
                    //Only Custom Keybinds
                    _ => match &mut page.content {
                        Content::TextInput(inner) => {
                            match x.0 {
                                0x08 => {
                                    inner.1.pop();
                                }
                                0x0D => (),
                                _ => {
                                    let char = x.1;
                                    inner.1.push(char)
                                }
                            };
                            Page::print_page(self, &page.title, &page.content, line)?;
                        }
                        Content::CustomPageRender(func) => match func(x, (size.0, size.1 - 4)).0 {
                            Some(mut view) => {
                                if view.lenlines(size.0).count() > size.1 - 4 {
                                    view.truncate(size.0 * size.1 - 4);
                                }
                                view.shrink_to_fit();
                                unsafe {
                                    self.set_pos(0, 2)?;
                                    let handle = get_handle_output!();
                                    WriteConsoleW(
                                        handle,
                                        &view.encode_utf16().collect::<Vec<u16>>(),
                                        None,
                                        None,
                                    )
                                }
                            }
                            None => Ok(()),
                        },
                        Content::SearchSelectList(inner) => {
                            match x.0 {
                                0x08 => {
                                    inner.0.pop();

                                    inner.2.clear();
                                    line = 0;

                                    let mut i = 0;

                                    for elem in inner.4.iter() {
                                        if elem.to_lowercase().contains(&inner.0.to_lowercase()) {
                                            inner.2.push(i);
                                        }
                                        i += 1;
                                    }
                                    Page::print_page(self, &page.title, &page.content, line)?
                                }
                                0x0D => (),
                                _ => {
                                    let char = x.1;
                                    inner.0.push(char);

                                    inner.2.clear();
                                    line = 0;

                                    let mut i = 0;

                                    for elem in inner.4.iter() {
                                        if elem.to_lowercase().contains(&inner.0.to_lowercase()) {
                                            inner.2.push(i);
                                        }
                                        i += 1;
                                    }
                                    Page::print_page(self, &page.title, &page.content, line)?
                                }
                            };
                        }
                        _ => (),
                    },
                },
                None => (),
            };
            match page.update {
                Some(func) => {
                    if func(&mut page.content) {
                        Page::print_page(self, &page.title, &page.content, line)?;
                    }
                }
                None => (),
            }
        }
    }
}

impl Content {
    fn len(&self, terlen: usize) -> usize {
        match self {
            Content::InfoPage(inner) | Content::ConfirmPage((inner, _)) => {
                inner.lenlines(terlen).count()
            }
            Content::SubMenü(inner) => inner.len(),
            Content::TextInput(_) => 1,
            Content::SelectList((_, v, _)) => v.len(),
            Content::SearchSelectList((_, _, v, _, _)) => v.len(),
            Content::CustomPageRender(_) => 0,
        }
    }
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: String::from("Default-Page"),
            update: None,
            content: Content::InfoPage(String::from("Default-Info")),
        }
    }
    pub fn info<A: ToString>(self, str: A) -> Page {
        let mut x = self;
        x.content = Content::InfoPage(str.to_string());
        x
    }
    pub fn menü(self, v: Vec<Page>) -> Page {
        let mut x = self;
        x.content = Content::SubMenü(v);
        x
    }
    pub fn select<A: ToString>(self, desc: A, vec: &[A], func: fn(&str) -> Page) -> Page {
        let mut x = self;
        let v = vec.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        for i in v.iter() {
            if i.len() >= MIN_WIDTH - 1 {
                panic!("OPtion too large-Later adding of scrolling")
            }
        }
        x.content = Content::SelectList((desc.to_string(), v, func));
        x
    }
    pub fn selectnsearch<A: ToString>(self, desc: A, vec: &[A], func: fn(&str) -> Page) -> Page {
        let mut x = self;
        let v = vec.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        for i in v.iter() {
            if i.len() >= MIN_WIDTH - 1 {
                panic!("OPtion too large-Later adding of scrolling")
            }
        }
        let mut y = Vec::new();
        for i in 0..v.len() {
            y.push(i);
        }
        x.content = Content::SearchSelectList((String::new(), desc.to_string(), y, func, v));
        x
    }
    pub fn input<A: ToString>(self, str: A, func: fn(&mut String) -> Page) -> Page {
        let mut x = self;
        x.content = Content::TextInput((str.to_string(), String::new(), func));
        x
    }
    pub fn title<A: ToString>(self, str: A) -> Page {
        let mut x = self;
        x.title = str.to_string();
        x
    }
    pub fn update(self, func: fn(&mut Content) -> bool) -> Page {
        let mut x = self;
        x.update = Some(func);
        x
    }
    ///It is very important to apply the following rules to the func for thr Page Gen
    /// -> If the u16 is 0 the function must generate the current Page and return Some()
    /// -> If the u16 or the char does not match any Key Binding you should not generate the Page again and return None
    pub fn custom(
        self,
        func: fn((u16, char), (usize, usize)) -> (Option<String>, Option<Page>),
    ) -> Page {
        let mut x = self;
        x.content = Content::CustomPageRender(func);
        x
    }
    ///The Confirm Page calls if the user confirms the function.
    /// You can add custom logic to your function.
    pub fn confirm<A: ToString>(self, str: A, func: fn() -> Page) -> Page {
        let mut x = self;
        x.content = Content::ConfirmPage((str.to_string(), func));
        x
    }
    fn print_page(
        handle: ScreenBuffer,
        title: &String,
        page: &Content,
        pline: usize,
    ) -> Result<()> {
        #[cfg(not(feature = "debug"))]
        let size = handle.get_size()?;

        #[cfg(feature = "debug")]
        let size = (75, 20);

        if size.0 < MIN_WIDTH && size.1 < MIN_HEIGHT {
            todo!()
        }

        let limiter = TerminalStr::new(
            size.0,
            1,
            Some(CHAR_INFO {
                Attributes: 0,
                Char: CHAR_INFO_0 {
                    UnicodeChar: b'=' as u16,
                },
            }),
        );

        let enc_title = TerminalStr::new(title.len(), 1, None);

        let i = 0;
        let slice = &mut enc_title[0];

        for char in title.encode_utf16() {
            slice[i].Char.UnicodeChar = char;
            i += 1;
        }

        let content = match page {
            Content::InfoPage(inner) => {
                let mut str = String::new();
                let mut i = 0;
                for line in inner.lenlines(size.0) {
                    if i >= pline && i < pline + size.1 - 4 {
                        str.push_str(line);
                        str.push('\n');
                    }
                    i += 1;
                }
                str
            }
            Content::ConfirmPage((inner, _)) => {
                let mut str = String::new();
                let mut i = 0;
                str.push_str("Please confirm or reject:\n");
                for line in inner.lenlines(size.0) {
                    if i >= pline && i < pline + size.1 - 5 {
                        str.push_str(line);
                        str.push('\n');
                    }
                    i += 1;
                }
                str
            }
            Content::SubMenü(inner) => {
                let mut str = String::from("Choose a Page:\n>");
                str.push_str(&inner[pline].title);
                str.push_str("<\n");
                let len = inner.len();
                for i in 0..size.1 - 6 {
                    if i + pline + 1 < len {
                        str.push_str(&inner[pline + i + 1].title);
                        str.push('\n');
                    }
                }
                str
            }
            Content::SelectList(inner) => {
                let mut str = inner.0.clone();
                str.push_str(":\n>");
                if inner.1.is_empty() {
                    str.push_str("Nothing in here");
                    str.push_str("<\n");
                } else {
                    str.push_str(&inner.1[pline]);
                    str.push_str("<\n");
                    let len = inner.1.len();
                    for i in 0..size.1 - 6 {
                        if i + pline + 1 < len {
                            str.push_str(&inner.1[pline + i + 1]);
                            str.push('\n');
                        }
                    }
                }
                str
            }
            Content::SearchSelectList(inner) => {
                let mut str = format!("{}:\nSearch: \"{}\"\n>", inner.1, inner.0);
                if inner.2.is_empty() {
                    str.push_str("Nothing found");
                    str.push_str("<\n");
                } else {
                    str.push_str(&inner.4[inner.2[pline]]);
                    str.push_str("<\n");
                    let len = inner.2.len();
                    for i in 0..size.1 - 7 {
                        if i + pline + 1 < len {
                            str.push_str(&inner.4[inner.2[pline + i + 1]]);
                            str.push('\n');
                        }
                    }
                }
                str
            }
            Content::TextInput(inner) => {
                let mut str = inner.0.clone();
                str.push_str(":\n>");
                str.push_str(&inner.1);
                str.push('<');
                str
            }
            Content::CustomPageRender(inner) => {
                inner((0, char::from_u32(0).unwrap()), (size.0, size.1 - 4))
                    .0
                    .unwrap()
            }
        };
        let content = {
            let mut str = String::new();
            for i in content.lenlines(size.0) {
                str.push_str(i);
                str.push('\n');
            }
            str.encode_utf16().collect::<Vec<u16>>()
        };
        ter.set_pos((size.0 - entitle.len()) as i16 / 2, 0)?;

        unsafe {
            if WriteConsoleW(handle, &entitle, None, None).is_err() {
                return Err(());
            };
            ter.set_pos(0, 1)?;
            if WriteConsoleW(
                handle,
                &std::iter::repeat('=' as u16)
                    .take(size.0)
                    .collect::<Vec<u16>>(),
                None,
                None,
            )
            .is_err()
            {
                return Err(());
            };
            ter.set_pos(0, 2)?;
            if WriteConsoleW(
                handle,
                &std::iter::repeat(' ' as u16)
                    .take(size.0 * (size.1 - 4))
                    .collect::<Vec<u16>>(),
                None,
                None,
            )
            .is_err()
            {
                return Err(());
            };
            ter.set_pos(0, 2)?;
            if WriteConsoleW(handle, &content, None, None).is_err() {
                return Err(());
            };
            ter.set_pos(0, size.1 as i16 - 2)?;
            if WriteConsoleW(
                handle,
                &std::iter::repeat('=' as u16)
                    .take(size.0)
                    .collect::<Vec<u16>>(),
                None,
                None,
            )
            .is_err()
            {
                return Err(());
            };
        }
        return Ok(());
    }
}

#[test]
fn page() {
    let x = Page { title: String::from("Page-X"), update: None, content: Content::InfoPage("Hi this is the mn\nddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\ndddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddain Page".to_string()) };
    let y = Page { title: String::from("Page-Y"), update: None, content: Content::InfoPage("Hi this is the mn\nddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\ndddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddain Page".to_string()) };
    let mut menü = Page {
        title: String::from("Main Page"),
        content: Content::SubMenü(vec![x, y]),
        update: None,
    };
    let ter = Terminal::new().unwrap();
    ter.clear();
    ter.open(&mut menü).unwrap();
}
