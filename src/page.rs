use crate::{get_handle_output, get_key, LenLinesAdd, Terminal, TERMINAL};
use std::{thread::sleep, time::Duration};
use windows::Win32::System::Console::WriteConsoleW;

pub const MIN_HEIGHT: usize = 7;
pub const MIN_WIDTH: usize = 75;

#[derive(Clone)]
pub enum Content {
    Info(String),
    SubMenü(Vec<Page>),
    Select((String, Vec<String>, fn(&str)->Page)),
    TextInput((String, String, fn(&mut String)->Page)),
}

#[derive(Clone)]
pub struct Page {
    //UFT-16 Title
    title: String,
    //Updates the Page
    update: Option<fn(menü: &mut Content)>,
    //Content
    content: Content,
}

pub trait PageUtils {
    fn open(&self, page: &mut Page) -> Result<(), ()>;
}

impl PageUtils for Terminal {
    fn open(&self, page: &mut Page) -> Result<(), ()> {
        let mut line = 0;

        #[cfg(not(feature = "debug"))]
        let size = self.get_size()?.0;

        #[cfg(feature = "debug")]
        let size = 75;

        self.clear();
        loop {
            sleep(Duration::from_millis(50));

            match get_key() {
                Some(x) => match x {
                    0x25 => return Ok(()),
                    0x27 => match &mut page.content {
                        Content::SubMenü(sm) => {
                            self.open(&mut sm[line])?;
                        },
                        Content::Select((_, vec, func)) => {
                            self.open(&mut func(vec[line].as_str()))?;
                        },
                        Content::TextInput((_, input, func))=> {
                            self.open(&mut func(input))?;
                        }
                        Content::Info(_) => (),
                    },
                    0x26 => {
                        if line > 0 {
                            line -= 1;
                        }
                    }
                    0x28 => {
                        if page.content.len(size) > 0 && line < page.content.len(size) - 1 {
                            line += 1;
                        }
                    }
                    _ => (),
                },
                None => (),
            };

            match page.update {
                Some(func) => func(&mut page.content),
                None => (),
            }

            Page::print_page(self, &page.title, &page.content, line)?;
        }
    }
}

impl Content {
    fn len(&self, terlen: usize) -> usize {
        match self {
            Content::Info(inner) => inner.lenlines(terlen).count(),
            Content::SubMenü(inner) => inner.len(),
            Content::TextInput(_)=> 1,
            Content::Select((_, v, _)) => v.len()
        }
    }
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: String::from("Default-Page"),
            update: None,
            content: Content::Info(String::from("Default-Info")),
        }
    }
    pub fn info<A: ToString>(self, str: A) -> Page {
        let mut x = self;
        x.content = Content::Info(str.to_string());
        x
    }
    pub fn menü(self, v: Vec<Page>) -> Page {
        let mut x = self;
        x.content = Content::SubMenü(v);
        x
    }
    pub fn select(self, desc: String, vec: Vec<String>, func: fn(&str)->Page) -> Page {
        let mut x = self;
        for i in vec.iter() {
            if i.len() >= MIN_WIDTH-1 {
                panic!("OPtion too large-Later adding of scrolling")
            }
        }
        x.content = Content::Select((desc, vec, func));
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
    pub fn update(self, func: fn(&mut Content)) -> Page {
        let mut x = self;
        x.update = Some(func);
        x
    }
    fn print_page(ter: &Terminal, title: &String, page: &Content, pline: usize) -> Result<(), ()> {
        #[cfg(not(feature = "debug"))]
        let size = ter.get_size().unwrap();

        #[cfg(feature = "debug")]
        let size = (75, 20);

        if size.0 < MIN_WIDTH && size.1 < MIN_HEIGHT {
            return Err(());
        }

        unsafe {
            let handle = get_handle_output!();
            let entitle = title.encode_utf16().collect::<Vec<u16>>();
            let content = match page {
                Content::Info(inner) => {
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
                Content::Select(inner)=> {
                    let mut str = inner.0.clone();
                    str.push_str(":\n>");
                    str.push_str(&inner.1[pline]);
                    str.push_str("<\n");
                    let len = inner.1.len();
                    for i in 0..size.1 - 6 {
                        if i + pline + 1 < len {
                            str.push_str(&inner.1[pline + i + 1]);
                            str.push('\n');
                        }
                    }
                    str
                }
                Content::TextInput(inner)=>{
                    let mut str = inner.0.clone();
                    str.push_str(":\n>");
                    str.push_str(&inner.1);
                    str.push('<');
                    str
                }
            }
            .encode_utf16()
            .collect::<Vec<u16>>();
            ter.set_pos((size.0-entitle.len()) as i16/2, 0)?;
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
    let x = Page { title: String::from("Page-X"), update: None, content: Content::Info("Hi this is the mn\nddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\ndddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddain Page".to_string()) };
    let y = Page { title: String::from("Page-Y"), update: None, content: Content::Info("Hi this is the mn\nddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\ndddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddain Page".to_string()) };
    let mut menü = Page {
        title: String::from("Main Page"),
        content: Content::SubMenü(vec![x, y]),
        update: None,
    };
    let ter = Terminal::new().unwrap();
    ter.clear();
    ter.open(&mut menü).unwrap();
}
