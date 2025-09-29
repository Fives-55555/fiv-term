use std::{
    sync::{Arc, Mutex},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};
use windows::Win32::System::Console::{WriteConsoleA, WriteConsoleW};

use crate::{get_handle_output, Terminal};

///A Loadbar at the footer
pub trait Loadbar {
    fn loadbar(self, an: Arc<Mutex<u8>>) -> JoinHandle<Terminal>;
    fn loadbarb(self, an: Arc<Mutex<u8>>) -> Terminal;
}

impl Loadbar for Terminal {
    fn loadbar(self, an: Arc<Mutex<u8>>) -> JoinHandle<Terminal> {
        thread::spawn(move || {
            let handle = unsafe { get_handle_output!(noerr) };
            let mut size = self.get_size().unwrap();
            let mut s = String::with_capacity(size.0);
            loop {
                size = self.get_size().unwrap();
                s.clear();
                self.clear_footer();
                self.set_pos(0, size.1 as i16 - 1).unwrap();
                let bar = size.0 - 9;
                let n = loop {
                    match an.lock() {
                        Ok(x) => {
                            if *x <= 100 {
                                break *x;
                            } else {
                                continue;
                            }
                        }
                        Err(_) => continue,
                    }
                };
                if n == 100 {
                    self.clear_footer();
                    s.push_str(" 100% Complete");
                    unsafe { WriteConsoleA(handle, s.as_bytes(), None, None).unwrap() };
                    sleep(Duration::from_secs(1));
                    break;
                }
                let f = bar as f64 * (n as f64 / 100.0);
                s.push_str(" [");
                s.push_str(
                    std::iter::repeat('█')
                        .take(f.floor() as usize)
                        .collect::<String>()
                        .as_str(),
                );
                if f.floor() != f {
                    s.push('▌');
                } else {
                    s.push(' ');
                }
                if (f as usize) < bar - 1 {
                    s.push_str(
                        std::iter::repeat(' ')
                            .take((bar - 1) - f as usize)
                            .collect::<String>()
                            .as_str(),
                    );
                };
                s.push_str(format!("]  {}%", n).as_str());
                unsafe {
                    WriteConsoleW(handle, &s.encode_utf16().collect::<Vec<u16>>(), None, None)
                        .unwrap()
                };
                sleep(Duration::from_micros(16666));
            }
            return self;
        })
    }
    fn loadbarb(self, an: Arc<Mutex<u8>>) -> Terminal {
        let handle = unsafe { get_handle_output!(noerr) };
        let mut size = self.get_size().unwrap();
        let mut s = String::with_capacity(size.0);
        loop {
            size = self.get_size().unwrap();
            s.clear();
            self.clear_footer();
            self.set_pos(0, size.1 as i16 - 1).unwrap();
            let bar = size.0 - 9;
            let n = loop {
                match an.lock() {
                    Ok(x) => {
                        if *x <= 100 {
                            break *x;
                        } else {
                            continue;
                        }
                    }
                    Err(_) => continue,
                }
            };
            if n == 100 {
                s.push_str(" 100% Complete");
                unsafe { WriteConsoleA(handle, s.as_bytes(), None, None).unwrap() };
                sleep(Duration::from_secs(1));
                break;
            }
            let f = bar as f64 * (n as f64 / 100.0);
            s.push_str(" [");
            s.push_str(
                std::iter::repeat('█')
                    .take(f.floor() as usize)
                    .collect::<String>()
                    .as_str(),
            );
            if f.floor() != f {
                s.push('▌');
            } else {
                s.push(' ');
            }
            if (f as usize) < bar - 1 {
                s.push_str(
                    std::iter::repeat(' ')
                        .take((bar - 1) - f as usize)
                        .collect::<String>()
                        .as_str(),
                );
            };
            s.push_str(format!("]  {}%", n).as_str());
            unsafe {
                WriteConsoleW(handle, &s.encode_utf16().collect::<Vec<u16>>(), None, None).unwrap()
            };
            sleep(Duration::from_micros(16666));
        }
        return self;
    }
}

#[test]
fn loadbar() {
    let t = Terminal::new().unwrap();
    let an = Arc::new(Mutex::new(0));
    let a = an.clone();
    t.clear();
    t.loadbar(an);
    for i in 0..101 {
        loop {
            match a.lock() {
                Ok(mut x) => {
                    *x = i;
                }
                Err(_) => continue,
            }
            sleep(Duration::from_millis(50));
            break;
        }
    }
    sleep(Duration::from_secs(3))
}
