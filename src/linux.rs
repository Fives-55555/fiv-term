use std::{io::Result, os::fd::OwnedFd};

pub struct StdHandles {
    input: OwnedFd,
    output: OwnedFd,
    error: OwnedFd,
}

impl StdHandles {
    pub fn new() -> Result<StdHandles> {
        Ok(StdHandles { input: stdin })
    }
}
