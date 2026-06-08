use std::{rc::Rc, sync::nonpoison::Mutex};

use crate::{
    TermConfig,
    stdio::{IoUring, StdIo},
    virtseq::{TermControl, VirtSeqBuf},
};

mod pages;

pub use pages::{PageWidget, TermPage};

pub struct TermView<'a, T: TermControl = TermConfig, I: StdIo = IoUring> {
    config: T,
    terminal: TerminalState,
    io: Rc<Mutex<I>>,
    page: Option<&'a mut TermPage>,
    seq_buf: [VirtSeqBuf<{ TermView::BUFFER_SIZE }>; TermView::BUFFER_COUNT],
}

pub struct TerminalState {
    dynamic_view: bool,
    scroll_view: bool,
    input: InputState,
}

impl Default for TerminalState {
    fn default() -> Self {
        TerminalState {
            dynamic_view: false,
            scroll_view: false,
            input: InputState::Basic,
        }
    }
}

pub enum InputState {
    Basic,
    Keyboard,
    NumberField,
    TextField,
    Game,
}

/// The Requested Action for the Terminal
pub enum TerminalAction {
    Quit,
    Back,
    ChangePage,
}

impl TermView<'_> {
    pub const BUFFER_SIZE: usize = 4096;
    pub const BUFFER_COUNT: usize = 4;
    pub fn new<T: TermControl, I: StdIo>() -> TermView<'static, T, I> {
        let io = Rc::new(I::new_stdio().unwrap());
        TermView {
            config: T::new_controls().unwrap(),
            terminal: TerminalState::default(),
            seq_buf: std::array::from_fn(|_| VirtSeqBuf::new_io(io.clone(), 64)),
            io: io,
            page: None,
        }
    }
    /// This dequeues any input and consumes it
    pub fn update(&self) {
        let mut buf = [0; 16];
        loop {
            let read = self.io.read_in(&mut buf).unwrap();
            if read == 0 {
                break;
            }
            let slice = match self.config.filter_in(&mut buf) {
                Some(new_buf) => new_buf,
                None => buf.as_slice(),
            };
            match self.terminal.input {
                InputState::Basic => {
                    //man stdin
                }
                _ => (),
            }
        }
    }
    pub fn set_page<'a>(self, page: &'a mut TermPage) -> TermView<'a> {
        TermView {
            config: self.config,
            terminal: self.terminal,
            io: self.io,
            seq_buf: self.seq_buf,
            page: Some(page),
        }
    }
    pub fn get_page(&mut self) -> Option<&mut TermPage> {
        match self.page {
            Some(ref mut page) => Some(*page),
            None => None,
        }
    }
    pub fn render(&mut self) {
        self.seq_buf.flush().unwrap();
    }
    pub fn render_loop() {}
}
