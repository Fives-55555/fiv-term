use std::{
    io::{Result, Stderr, Stdin, Stdout, stderr, stdin, stdout},
    os::fd::{AsRawFd, OwnedFd},
    sync::LazyLock,
};

use crate::{TerminalTrait, virtkeys::VirtualSeq};

pub struct Terminal {}

impl TerminalTrait for Terminal {
    type Result<V> = Result<V>;
}

impl VirtualSeq for Terminal {
    fn enable(&self) -> <Self as crate::TerminalTrait>::Result<()> {
        Ok(())
    }
}
