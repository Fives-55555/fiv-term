use std::{
    cmp::max,
    ffi::c_void,
    io::{Error, Result},
    ptr,
    rc::Rc,
    slice,
    sync::atomic::{AtomicU32, Ordering},
};

use libc::{MAP_FAILED, MAP_POPULATE, MAP_SHARED, PROT_READ, PROT_WRITE, close, mmap, munmap};

use crate::{
    stdio::{
        StdIo,
        io_uring_def::{
            IORING_FEAT_SINGLE_MMAP, IORING_OFF_CQ_RING, IORING_OFF_SQ_RING, IORING_OFF_SQES,
            IoUringParam, IouCQEntry, IouSQEntry, sys_io_uring_enter, sys_io_uring_setup,
        },
    },
    stuff::{Const, KernelMem, Mut},
};

#[derive(Debug)]
pub struct IoUring {
    fd: i32,
    to_submit: u32,
    // Size Mask
    sring_mask: KernelMem<u32, Const>,
    cring_mask: KernelMem<u32, Const>,
    // S-Ring
    // modified by User
    sring_tail: KernelMem<AtomicU32, Const>,
    //C-Ring
    // Modified by Kernel
    cring_tail: KernelMem<AtomicU32, Const>,
    // Modified by User
    cring_head: KernelMem<AtomicU32, Const>,
    // Index to sqe
    _sarray: KernelMem<[u32], Mut>,
    // The S/C Entries
    sqe: KernelMem<[IouSQEntry], Mut>,
    cqe: KernelMem<[IouCQEntry], Const>,
}

impl IoUring {
    pub fn new(entries: u32) -> Result<IoUring> {
        let mut params = IoUringParam::null();

        let fd = sys_io_uring_setup(entries, &mut params)?;

        let sr_sz =
            (params.sq_offs.array as usize) + ((params.sq_entries as usize) * size_of::<u32>());
        let cr_sz =
            (params.cq_offs.cqes as usize) + (params.cq_entries as usize) * size_of::<IouCQEntry>();

        // FIXME: Add Support for non Feature Kernel
        let (sq_ptr, cq_ptr) = if (params.features & IORING_FEAT_SINGLE_MMAP) != 0 {
            let sq_ptr = unsafe {
                mmap(
                    ptr::null_mut(),
                    max(sr_sz, cr_sz),
                    PROT_READ | PROT_WRITE,
                    MAP_SHARED | MAP_POPULATE,
                    fd as i32,
                    IORING_OFF_SQ_RING,
                )
            } as isize;

            if sq_ptr == MAP_FAILED as isize {
                return Err(Error::last_os_error());
            }
            (sq_ptr, sq_ptr)
        } else {
            let sq_ptr = unsafe {
                mmap(
                    ptr::null_mut(),
                    sr_sz,
                    PROT_READ | PROT_WRITE,
                    MAP_SHARED | MAP_POPULATE,
                    fd as i32,
                    IORING_OFF_SQ_RING,
                )
            } as isize;
            if sq_ptr == MAP_FAILED as isize {
                return Err(Error::last_os_error());
            }
            let cq_ptr = unsafe {
                mmap(
                    ptr::null_mut(),
                    cr_sz,
                    PROT_READ | PROT_WRITE,
                    MAP_SHARED | MAP_POPULATE,
                    fd as i32,
                    IORING_OFF_CQ_RING,
                )
            } as isize;
            if cq_ptr == MAP_FAILED as isize {
                return Err(Error::last_os_error());
            }
            (sq_ptr, cq_ptr)
        };

        let sqes_ptr = unsafe {
            mmap(
                ptr::null_mut(),
                params.sq_entries as usize * size_of::<IouSQEntry>(),
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_POPULATE,
                fd as i32,
                IORING_OFF_SQES,
            )
        } as isize;

        if sqes_ptr == MAP_FAILED as isize {
            return Err(Error::last_os_error());
        }

        let sarray = unsafe {
            slice::from_raw_parts_mut(
                (sq_ptr + params.sq_offs.array as isize) as *mut u32,
                // FIXME is properly wrong
                params.sq_entries as usize,
            )
        };

        for (i, elem) in sarray.iter_mut().enumerate() {
            *elem = i as u32;
        }

        Ok(IoUring {
            fd: fd as i32,
            to_submit: 0,

            sring_tail: KernelMem::from_const(
                (sq_ptr + params.sq_offs.tail as isize) as *const AtomicU32,
            ),
            sring_mask: KernelMem::from_const(
                (sq_ptr + params.sq_offs.ring_mask as isize) as *const u32,
            ),
            cring_head: KernelMem::from_const(
                (cq_ptr + params.cq_offs.head as isize) as *const AtomicU32,
            ),
            cring_tail: KernelMem::from_const(
                (cq_ptr + params.cq_offs.tail as isize) as *const AtomicU32,
            ),
            cring_mask: KernelMem::from_const(
                (cq_ptr + params.cq_offs.ring_mask as isize) as *const u32,
            ),
            _sarray: KernelMem::from_mut(sarray as *mut [u32]),

            sqe: KernelMem::from_mut(unsafe {
                slice::from_raw_parts_mut(sqes_ptr as *mut IouSQEntry, params.sq_entries as usize)
            }),
            cqe: KernelMem::from_const(unsafe {
                slice::from_raw_parts_mut(
                    (cq_ptr + params.cq_offs.cqes as isize) as *mut IouCQEntry,
                    params.cq_entries as usize,
                )
            }),
        })
    }
    pub fn get_cqe(&mut self) -> Option<IouCQEntry> {
        loop {
            let head = self.cring_head.get_ref().load(Ordering::Acquire);
            let tail = self.cring_tail.get_ref().load(Ordering::Acquire);
            if head == tail {
                return None;
            } else {
                let cqe = self.cqe.get_ref()[(head & self.cring_mask.get_ref()) as usize].clone();
                if self
                    .cring_head
                    .get_ref()
                    .compare_exchange_weak(head, head + 1, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    return Some(cqe);
                } else {
                    continue;
                }
            };
        }
    }
    pub fn add_sqe(&mut self, sqe: IouSQEntry) {
        let tail = self.sring_tail.get_ref().load(Ordering::Acquire);
        let index = tail & self.sring_mask.get_ref();
        self.sqe.get_ref_mut()[index as usize] = sqe;
        self.to_submit += 1;
        self.sring_tail.get_ref().store(tail + 1, Ordering::Release);
    }
    pub fn submit(&mut self, wait_for: u32, flags: u32) -> Result<u32> {
        let ret = sys_io_uring_enter(self.fd, self.to_submit, wait_for, flags, None)?;
        self.to_submit = 0;
        Ok(ret)
    }
    pub fn wait(&mut self) -> Result<IouCQResult> {
        loop {
            let _ = sys_io_uring_enter(self.fd, 0, 1, 1, None)?;
            let entry = self.get_cqe();
            if entry.is_some() {
                break Ok(entry.unwrap().as_result());
            }
        }
    }
}

impl StdIo for IoUring {
    fn new_stdio() -> Result<Self> {
        IoUring::new(32)
    }
}

// FIXME ADD Support for Application allocated Memory
impl Drop for IoUring {
    fn drop(&mut self) {
        let r_sqe = unsafe {
            munmap(
                self.sqe.get_ref().as_mut_ptr() as *mut c_void,
                self.sqe.get_ref().len() * size_of::<IouSQEntry>(),
            )
        };
        let r_sq = unsafe {
            munmap(
                self._sarray.get_ref().as_mut_ptr() as *mut c_void,
                self._sarray.get_ref().len() * size_of::<u32>(),
            )
        };
        let r_cq = unsafe {
            munmap(
                self.cqe.get_ref().as_mut_ptr() as *mut c_void,
                self.cqe.get_ref().len() * size_of::<IouCQEntry>(),
            )
        };
        if r_sqe != 0 || r_sq != 0 || r_cq != 0 {
            Result::<(), Error>::Err(Error::last_os_error())
                .expect("A MEM_UNMAP was unsuccessful!");
        }
        let r_fd = unsafe { close(self.fd) };
        if r_fd != 0 {
            Result::<(), Error>::Err(Error::last_os_error())
                .expect("The closure of the IORing was unsuccessful!");
        }
    }
}

pub type IouCQResult = std::result::Result<IouCQEntry, (IouCQEntry, Error)>;

//FIXME maybe add wrapper
// FIXME add Chained Requests
pub trait IoUringOps: Sized {
    fn get_uring(&self) -> &IoUring;
    fn from_uring<T>(ring: Rc<IoUring>, args: T) -> Result<Self>;
    fn read(&self, buf: &mut [u8]) -> Result<()>;
    fn write(&self, buf: &[u8]) -> Result<()>;
    fn flush_uring(&self) -> Result<usize> {
        self.get_uring().submit(0, flags)
    }
}
