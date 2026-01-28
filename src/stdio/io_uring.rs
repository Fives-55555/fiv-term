use std::{
    cmp::max,
    io::{Error, Result},
    ptr, slice,
    sync::atomic::{AtomicU32, Ordering},
};

use libc::{MAP_FAILED, MAP_POPULATE, MAP_SHARED, PROT_READ, PROT_WRITE, mmap};

use crate::stdio::io_uring_def::{
    IORING_FEAT_SINGLE_MMAP, IORING_OFF_CQ_RING, IORING_OFF_SQ_RING, IORING_OFF_SQES, IoUringParam,
    IouCQEntry, IouSQEntry, sys_io_uring_enter, sys_io_uring_setup,
};

#[derive(Debug)]
pub struct IoUring<'a> {
    fd: i32,
    to_submit: u32,
    // Size Mask
    sring_mask: &'a u32,
    cring_mask: &'a u32,
    // S-Ring
    // modified by User
    sring_tail: &'a AtomicU32,
    //C-Ring
    // Modified by Kernel
    cring_tail: &'a AtomicU32,
    // Modified by User
    cring_head: &'a AtomicU32,
    // Index to sqe
    _sarray: &'a mut [u32],
    // The S/C Entries
    sqe: &'a mut [IouSQEntry],
    cqe: &'a [IouCQEntry],
}

impl<'a> IoUring<'a> {
    pub fn new(entries: u32) -> Result<IoUring<'a>> {
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

            sring_tail: unsafe {
                AtomicU32::from_ptr((sq_ptr + params.sq_offs.tail as isize) as *mut u32)
            },
            sring_mask: unsafe {
                ((sq_ptr + params.sq_offs.ring_mask as isize) as *mut u32).as_ref()
            }
            .unwrap(),

            cring_head: unsafe {
                AtomicU32::from_ptr((cq_ptr + params.cq_offs.head as isize) as *mut u32)
            },
            cring_tail: unsafe {
                AtomicU32::from_ptr((cq_ptr + params.cq_offs.tail as isize) as *mut u32)
            },
            cring_mask: unsafe {
                ((cq_ptr + params.cq_offs.ring_mask as isize) as *mut u32).as_ref()
            }
            .unwrap(),

            _sarray: sarray,

            sqe: unsafe {
                slice::from_raw_parts_mut(sqes_ptr as *mut IouSQEntry, params.sq_entries as usize)
            },
            cqe: unsafe {
                slice::from_raw_parts_mut(
                    (cq_ptr + params.cq_offs.cqes as isize) as *mut IouCQEntry,
                    params.cq_entries as usize,
                )
            },
        })
    }
    pub fn get_cqe(&mut self) -> Option<IouCQEntry> {
        loop {
            let head = self.cring_head.load(Ordering::Acquire);
            let tail = self.cring_tail.load(Ordering::Acquire);
            if head == tail {
                return None;
            } else {
                let cqe = self.cqe[(head & self.cring_mask) as usize].clone();
                if self
                    .cring_head
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
        let tail = self.sring_tail.load(Ordering::Acquire);
        let index = tail & self.sring_mask;
        self.sqe[index as usize] = sqe;
        self.to_submit += 1;
        self.sring_tail.store(tail + 1, Ordering::Release);
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

pub type IouCQResult = std::result::Result<IouCQEntry, (IouCQEntry, Error)>;

#[test]
fn mark_test() {
    let mut buf = [0u8; 16];

    let input = 0;

    let mut io_ring = IoUring::new(16).unwrap();

    let mut read = IouSQEntry::null();
    read.set_read(0x830, input, &mut buf, 0);

    // wait.flags = IouSQFlags::IOSQE_IO_LINK;
    io_ring.add_sqe(read);
    io_ring.submit(1, 1).unwrap();
    let x = io_ring.get_cqe().unwrap();
    println!("{:?}", x);
    println!("{:?}", buf);
    x.get_result().unwrap();
}
