use libc::{EPOLL_CTL_ADD, epoll_create, epoll_ctl, epoll_event};
use std::io::{Error, Result};

use crate::bitfield;

pub struct EPoll {
    fd: i32,
}

//FIXME
bitfield!(
    EPollEvents,
    u32,
    (EPOLLIN, { 0 }, ("Fd has read data.")),
    //FIXME
    (EPOLLPRI, { 1 }, ("There is an exception on the fd. IDK")),
    (EPOLLOUT, { 2 }, ("Fd has write data.")),
    (EPOLLERR, { 3 }, ("")),
    (EPOLLHUP, { 4 }, ("")),
    (
        EPOLLET,
        { 31 },
        ("Event-triggered not level-triggered so, partial reads wont trigger readablity again.")
    )
);

impl EPoll {
    pub fn new() -> Result<EPoll> {
        let res = unsafe { epoll_create(1) };
        if res < 0 {
            return Err(Error::last_os_error());
        } else {
            return Ok(EPoll { fd: res as i32 });
        }
    }
    pub fn add_fd(&mut self, fd: i32, id: u64, events: EPollEvents) -> Result<()> {
        let mut event = epoll_event {
            events: events.0,
            u64: id,
        };
        let res = unsafe { epoll_ctl(self.fd, EPOLL_CTL_ADD, fd, &mut event) };
        if res != 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
    pub fn fd(&self) -> i32 {
        self.fd
    }
}
