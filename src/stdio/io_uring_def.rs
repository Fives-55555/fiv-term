use crate::{bitfield, nullable, stdio::io_uring::IouCQResult};
use libc::{
    __kernel_rwf_t, SYS_io_uring_enter, SYS_io_uring_register, SYS_io_uring_setup, epoll_event,
    off_t, sigset_t, syscall,
};
use std::{
    fmt::Debug,
    io::{Error, Result},
    os::raw::c_void,
    ptr,
};

pub const IORING_OFF_SQ_RING: off_t = 0;
pub const IORING_OFF_CQ_RING: off_t = 0x8000000;
pub const IORING_OFF_SQES: off_t = 0x10000000;
pub const IORING_FEAT_SINGLE_MMAP: u32 = 1 << 0;

bitfield!(
    IoUringSQFlags,
    u8,
    (
        IOSQE_FIXED_FILE,
        { 0 },
        ("The fd is index to pre-registered fd")
    ),
    (
        IOSQE_IO_DRAIN,
        { 1 },
        ("Make this SQE blocking and let it wait for the previous.")
    ),
    (
        IOSQE_IO_LINK,
        { 2 },
        ("Connects the consecutives SQEs in one submission to be sequential. An error terminates the rest of the chain with -ECANCELED")
    ),
    // FIXME
    (
        IOSQE_IO_HARDLINK,
        { 3 },
        ("Like IOSQE_IO_LINK but ignores the result of the entries.")
    ),
    //FIXME
    (IOSQE_ASYNC, { 4 }, ("")),
    (
        IOSQE_BUFFER_SELECT,
        { 5 },
        ("The SQE will choose a preregistered buffer and return its id in the CQE, if there is non it returns -ENOBUFS. A used buffer needs too be reregistered.")
    ),
    //FIXME
    (
        IOSQE_CQE_SKIP_SUCCESS,
        { 6 },
        ("The CQE wont be posted on success. On error the CQE will be posted.")
    )
);

#[repr(C)]
pub struct IoUringParam {
    pub sq_entries: u32,
    pub cq_entries: u32,
    // To Configure
    pub flags: u32,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    // End
    pub features: u32,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_offs: SqOffset,
    pub cq_offs: CqOffset,
}

nullable!(IoUringParam);

#[repr(C)]
pub struct SqOffset {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub flags: u32,
    pub dropped: u32,
    pub array: u32,
    pub resv1: u32,
    pub user_addr: u64,
}

#[repr(C)]
pub struct CqOffset {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub overflow: u32,
    pub cqes: u32,
    pub flags: u32,
    pub resv1: u32,
    pub user_addr: u64,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct IouSQEntry {
    opcode: IouOpcode,
    flags: IouSQFlags,
    //FIXME man ioprio_get
    ioprio: u16,
    fd: i32,
    first_arg: FirstArg,
    second_arg: SecondArg,
    len: u32, /* buffer size or number of iovecs */
    io_flags: IoFlags,
    /// User defined SQE data
    user_data: u64,
    /* pack this to avoid bogus arm OABI complaints */
    attribute: IouAttribute,
    /* personality to use, if used */
    personality: u16,
    idk2: Idk2,
    idk3: Idk3,
}

nullable!(IouSQEntry);

impl IouSQEntry {
    pub fn set_epoll_wait(
        &mut self,
        userdata: u64,
        epfd: i32,
        events: &mut [epoll_event],
        flags: u32,
    ) {
        self.user_data = userdata;
        self.opcode = IouOpcode::IORING_OP_EPOLL_WAIT;
        self.fd = epfd;
        self.second_arg.addr = events.as_mut_ptr() as u64;
        self.len = events.len() as u32;
        self.io_flags.rw_flags = flags as i32;
    }
    pub fn set_read(&mut self, userdata: u64, fd: i32, buf: &mut [u8], flags: i32) {
        self.user_data = userdata;
        self.opcode = IouOpcode::IORING_OP_READ;
        self.fd = fd;
        self.second_arg.addr = buf.as_mut_ptr() as u64;
        self.len = buf.len() as u32;
        self.io_flags.rw_flags = flags;
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum IouOpcode {
    IORING_OP_NOP,
    IORING_OP_READV,
    IORING_OP_WRITEV,
    IORING_OP_FSYNC,
    IORING_OP_READ_FIXED,
    IORING_OP_WRITE_FIXED,
    IORING_OP_POLL_ADD,
    IORING_OP_POLL_REMOVE,
    IORING_OP_SYNC_FILE_RANGE,
    IORING_OP_SENDMSG,
    IORING_OP_RECVMSG,
    IORING_OP_TIMEOUT,
    IORING_OP_TIMEOUT_REMOVE,
    IORING_OP_ACCEPT,
    IORING_OP_ASYNC_CANCEL,
    IORING_OP_LINK_TIMEOUT,
    IORING_OP_CONNECT,
    IORING_OP_FALLOCATE,
    IORING_OP_OPENAT,
    IORING_OP_CLOSE,
    IORING_OP_FILES_UPDATE,
    IORING_OP_STATX,
    IORING_OP_READ,
    IORING_OP_WRITE,
    IORING_OP_FADVISE,
    IORING_OP_MADVISE,
    IORING_OP_SEND,
    IORING_OP_RECV,
    IORING_OP_OPENAT2,
    IORING_OP_EPOLL_CTL,
    IORING_OP_SPLICE,
    IORING_OP_PROVIDE_BUFFERS,
    IORING_OP_REMOVE_BUFFERS,
    IORING_OP_TEE,
    IORING_OP_SHUTDOWN,
    IORING_OP_RENAMEAT,
    IORING_OP_UNLINKAT,
    IORING_OP_MKDIRAT,
    IORING_OP_SYMLINKAT,
    IORING_OP_LINKAT,
    IORING_OP_MSG_RING,
    IORING_OP_FSETXATTR,
    IORING_OP_SETXATTR,
    IORING_OP_FGETXATTR,
    IORING_OP_GETXATTR,
    IORING_OP_SOCKET,
    IORING_OP_URING_CMD,
    IORING_OP_SEND_ZC,
    IORING_OP_SENDMSG_ZC,
    IORING_OP_READ_MULTISHOT,
    IORING_OP_WAITID,
    IORING_OP_FUTEX_WAIT,
    IORING_OP_FUTEX_WAKE,
    IORING_OP_FUTEX_WAITV,
    IORING_OP_FIXED_FD_INSTALL,
    IORING_OP_FTRUNCATE,
    IORING_OP_BIND,
    IORING_OP_LISTEN,
    IORING_OP_RECV_ZC,

    /// Layout:
    ///     fd: epfd,
    ///     addr: events[n],
    ///     len: n,
    ///     io_flags: flags
    IORING_OP_EPOLL_WAIT,
    IORING_OP_READV_FIXED,
    IORING_OP_WRITEV_FIXED,
    IORING_OP_PIPE,
    IORING_OP_NOP128,
    IORING_OP_URING_CMD128,

    /* this goes last, obviously */
    IORING_OP_LAST,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FirstArg {
    pub off: i64, /* offset into file */
    pub addr2: u64,
    pub cmd_op: CmdOp,
}

impl Debug for FirstArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FIRST Arg: {:X}", unsafe { self.addr2 })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CmdOp {
    pub cmd_op: u32,
    pub pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union SecondArg {
    pub addr: u64, /* pointer to buffer or iovecs */
    pub splice_off_in: u64,
    pub idk1: Idk1,
}

impl Debug for SecondArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Second Arg: {:X}", unsafe { self.addr })
    }
}
// FIXME
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Idk1 {
    pub level: u32,
    pub optname: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union IoFlags {
    pub rw_flags: __kernel_rwf_t,
    pub fsync_flags: u32,
    pub poll_events: u16,   /* compatibility */
    pub poll32_events: u32, /* word-reversed for BE */
    pub sync_range_flags: u32,
    pub msg_flags: u32,
    pub timeout_flags: u32,
    pub accept_flags: u32,
    pub cancel_flags: u32,
    pub open_flags: u32,
    pub statx_flags: u32,
    pub fadvise_advice: u32,
    pub splice_flags: u32,
    pub rename_flags: u32,
    pub unlink_flags: u32,
    pub hardlink_flags: u32,
    pub xattr_flags: u32,
    pub msg_ring_flags: u32,
    pub uring_cmd_flags: u32,
    pub waitid_flags: u32,
    pub futex_flags: u32,
    pub install_fd_flags: u32,
    pub nop_flags: u32,
}

impl Debug for IoFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SQE FLAGS: {:X}", unsafe { self.nop_flags })
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union IouAttribute {
    /* index into fixed buffers, if used */
    pub buf_index: u16,
    /* for grouped buffer selection */
    pub buf_group: u16,
}

impl Debug for IouAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IO ATTR: {:X}", unsafe { self.buf_index })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union Idk3 {
    pub addr3: IoAddr3,
    pub optval: u64,
    /*
     * If the ring is initialized with IORING_SETUP_SQE128, then
     * this field is used for 80 bytes of arbitrary command data
     */
    pub cmds: [u8; 0],
}

impl Debug for Idk3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IDK2: {:?}", unsafe { self.addr3 })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IoAddr3 {
    pub addr3: u64,
    pub pad: u64,
}

nullable!(IoAddr3);

#[repr(C)]
#[derive(Clone, Copy)]
pub union Idk2 {
    pub splice_fd_in: i32,
    pub file_index: u32,
    pub optlen: u32,
    pub addr_len: IoAddrLen,
}

impl Debug for Idk2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IDK3: {:X}", unsafe { self.file_index })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IoAddrLen {
    pub addr_len: u16,
    pub pad: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IouCQEntry {
    pub user_data: u64,
    pub result: i32,
    pub flags: u32,
}

impl IouCQEntry {
    pub fn as_result(self) -> IouCQResult {
        if self.result < 0 {
            return Err((self, Error::from_raw_os_error(-self.result)));
        } else {
            return Ok(self);
        }
    }
    pub fn get_result(&self) -> Result<()> {
        if self.result < 0 {
            return Err(Error::from_raw_os_error(-self.result));
        } else {
            return Ok(());
        }
    }
}

pub fn sys_io_uring_setup(entries: u32, params: &mut IoUringParam) -> Result<u32> {
    let ret: i32 =
        unsafe { syscall(SYS_io_uring_setup, entries, params as *mut IoUringParam) } as i32;
    if ret < 0 {
        Err(Error::from_raw_os_error(-ret))
    } else {
        Ok(ret as u32)
    }
}

// FIXME: sig & or &mut
pub fn sys_io_uring_enter(
    fd: i32,
    to_submit: u32,
    min_complete: u32,
    flags: u32,
    sig: Option<&mut sigset_t>,
) -> Result<u32> {
    let ret = unsafe {
        syscall(
            SYS_io_uring_enter,
            fd,
            to_submit,
            min_complete,
            flags,
            sig.map_or(ptr::null_mut(), |ptr| ptr as *mut sigset_t) as *mut sigset_t,
        )
    } as i32;
    if ret < 0 {
        return Err(Error::from_raw_os_error(-ret));
    } else {
        return Ok(ret as u32);
    }
}

pub fn sys_io_uring_register(fd: u32, opcode: u32, arg: *mut c_void, nr_arg: u32) -> Result<u32> {
    let ret = unsafe { syscall(SYS_io_uring_register, fd, opcode, arg, nr_arg) } as i32;
    if ret < 0 {
        return Err(Error::from_raw_os_error(-ret));
    } else {
        Ok(ret as u32)
    }
}

bitfield!(
    IoUringEnterFlags,
    u32,
    (
        IORING_ENTER_GETEVENTS,
        { 0 },
        ("The request will be waiting for min_complete CQs to be finished.")
    ),
    (
        IORING_ENTER_SQ_WAKEUP,
        { 1 },
        ("Wake the SQ-Kernelthread setup by IORING_SETUP_SQPOLL.")
    ),
    (
        IORING_ENTER_SQ_WAIT,
        { 2 },
        ("This lets the syscall wait for a free SQEntry to return.")
    ),
    (
        IORING_ENTER_EXT_ARG,
        { 3 },
        ("Allows arg to be a pointer to io_uring_getevents_arg by setting argsz to the size of the struct.")
    ),
    (
        IORING_ENTER_REGISTERED_RING,
        { 4 },
        ("This sets ring_fd to be a offset to the registered ring. Requires registration through IORING_REGISTER_RING_FDS.")
    ),
    (
        IORING_ENTER_ABS_TIMER,
        { 5 },
        ("This sets the timeout of the io_uring_getevents_arg to be an absolute timestamp and not a timeout.")
    ),
    (
        IORING_ENTER_EXT_ARG_REG,
        { 6 },
        ("The arg must be an offset to a io_uring_getevents_arg struct in a memory region from io_uring_register(2) IORING_REGISTER_MEM_REGION.")
    ),
    (
        IORING_ENTER_NO_IOWAIT,
        { 7 },
        ("Requires IORING_FEAT_NO_IOWAIT and ")
    )
);
