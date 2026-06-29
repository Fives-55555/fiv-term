use core::fmt::NumBufferTrait;
use std::{fmt::Debug, marker::PhantomData, mem::MaybeUninit, slice};

#[derive(Debug)]
pub struct NumberSlice<'a, T: NumBufferTrait>(&'a mut [u8; T::BUF_SIZE])
where
    [(); T::BUF_SIZE]:;

impl<'a, T: NumBufferTrait> NumberSlice<'a, T>
where
    [(); T::BUF_SIZE]:,
{
    pub fn new(buf: &'a mut [u8]) -> Option<NumberSlice<'a, T>> {
        if buf.len() < T::BUF_SIZE {
            return None;
        }
        Some(NumberSlice(buf[0..T::BUF_SIZE].as_mut_array().unwrap()))
    }
    pub fn as_slice(&self, len: usize) -> &[u8] {
        &self.0[..len]
    }
    pub fn as_str(&self, len: usize) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice(len)) }
    }
}

impl<T: NumBufferTrait> NumberSlice<'_, T>
where
    [(); T::BUF_SIZE]:,
{
    pub fn new_buf() -> [u8; T::BUF_SIZE] {
        [0; T::BUF_SIZE]
    }
}

pub trait FastForwardFormat: NumBufferTrait + Sized
where
    [(); Self::BUF_SIZE]:,
{
    fn forward_format(&self, buf: &mut NumberSlice<Self>) -> usize;
}

macro_rules! fast_forward_format {
    ($($signed:ident, $unsigned:ident,)*) => {
        $(
            impl FastForwardFormat for $signed {
                fn forward_format(&self, buf: &mut NumberSlice<Self>)->usize {
                    const IDK: usize = $signed::BUF_SIZE-2;

                    const STEPS: [$unsigned; IDK] = {
                        let mut arr = [0 as $unsigned; IDK];
                        let mut pow = IDK;
                        let mut i = 0;
                        while pow > 0 {
                            arr[i] = (10 as $unsigned).pow(pow as u32);
                            i+=1;
                            pow-=1;
                        }
                        arr
                    };

                    let num = *self;
                    let mut digit;
                    let mut i = 0;
                    let mut i_buf = 0;

                    let mut num  = if num < 0 {
                        buf.0[i_buf] = b'-';
                        i_buf+=1;
                        num.unsigned_abs()
                    } else {
                        num as $unsigned
                    };

                    while i < IDK && STEPS[i] > num {
                        i+=1;
                        continue;
                    }

                    while i < IDK {
                        let step = STEPS[i];
                        digit = num/step;
                        num %= step;
                        buf.0[i_buf] = b'0' + digit as u8;
                        i+=1;
                        i_buf+=1;
                    }

                    buf.0[i_buf] = b'0' + num as u8;
                    i_buf+1
                }
            }

            impl FastForwardFormat for $unsigned {
                fn forward_format(&self, buf: &mut NumberSlice<Self>)->usize {

                    const IDK: usize = $unsigned::BUF_SIZE-1;

                    const STEPS: [$unsigned; IDK] = {
                        let mut arr = [0 as $unsigned; IDK];
                        let mut pow = IDK;
                        let mut i = 0;
                        while pow > 0 {
                            arr[i] = (10 as $unsigned).pow(pow as u32);
                            i+=1;
                            pow-=1;
                        }
                        arr
                    };

                    let mut num = *self;
                    let mut digit;
                    let mut i = 0;
                    let mut i_buf = 0;

                    while i < IDK && STEPS[i] >= num {
                        i+=1;
                        continue;
                    }

                    while i < IDK {
                        let step = STEPS[i];
                        digit = num/step;
                        num %= step;
                        buf.0[i_buf] = b'0' + digit as u8;
                        i+=1;
                        i_buf+=1;
                    }
                    buf.0[i_buf] = b'0' + num as u8;
                    i_buf+1
                }
            }
        )*
    }
}

fast_forward_format! {
    i8, u8,
    i16, u16,
    i32, u32,
    i64, u64,
    isize, usize,
    i128, u128,
}

#[macro_export]
macro_rules! nullable {
    ($type:ty) => {
        impl $type {
            pub const fn null() -> $type {
                unsafe { std::mem::zeroed() }
            }
        }
    };
}

#[macro_export]
macro_rules! bitfield {
    (
        $name:ident,
        $type:ty,
        $(
            (
            $bitname:ident,
            {
            $(
                $value:expr
            ),*
            }
            $(,
                (
                $(
                    $doc:tt
                )+
                )
            )?
            )
        ),*
    ) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug)]
        pub struct $name(pub $type);

        impl $name {
        $(
            $(#[doc=$($doc)*])?
            pub const $bitname: $name = $name(0 $(| 1<<$value )*);
        )*
            pub const fn contains(&self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
        }
        impl core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }
        impl core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }
        impl core::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }
        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, other: Self) {
                self.0.bitor_assign(other.0)
            }
        }
        impl core::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, other: Self) {
                self.0.bitand_assign(other.0)
            }
        }
        impl core::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, other: Self) {
                self.0.bitxor_assign(other.0)
            }
        }
        impl core::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                Self(self.0.not())
            }
        }
    };
}

pub struct SeqBuf {
    buf: Box<[u8]>,
    base: *const u8,
    len: usize,
}

impl SeqBuf {
    pub fn new(buf: Box<[u8]>) -> SeqBuf {
        SeqBuf {
            base: buf.as_ptr(),
            len: buf.len(),
            buf,
        }
    }
    pub fn get_next<'ret>(&mut self, block_len: usize) -> Option<&'ret [u8]> {
        if self.len >= block_len {
            let ptr = self.base;
            self.len -= block_len;
            self.base = unsafe { self.base.add(block_len) };
            Some(unsafe { slice::from_raw_parts(ptr, block_len) })
        } else {
            None
        }
    }
    pub fn get_next_t<'ret, T>(&mut self, block_len: usize) -> Option<&'ret [T]> {
        let real_len = block_len * size_of::<T>();
        if self.len >= real_len {
            let ptr = self.base;
            self.len -= real_len;
            self.base = unsafe { self.base.add(real_len) };
            Some(unsafe { slice::from_raw_parts(ptr as *const T, block_len) })
        } else {
            None
        }
    }
    pub fn get_next_aligned_t<'ret, T>(&mut self, block_len: usize) -> Option<&'ret [T]> {
        let alignment = align_of::<T>() as isize;

        let offset = ((-(self.base as isize)) & (alignment - 1)) as usize;
        let real_len = block_len * size_of::<T>();

        if self.len >= real_len + offset {
            self.base = unsafe { self.base.add(offset) };
            let ptr = self.base;
            self.len -= real_len + offset;
            self.base = unsafe { self.base.add(real_len) };
            Some(unsafe { slice::from_raw_parts(ptr as *const T, block_len) })
        } else {
            None
        }
    }
    pub fn base(&self) -> *const u8 {
        self.base
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn end(self) -> Option<Box<[u8]>> {
        if self.len == 0 { Some(self.buf) } else { None }
    }
}

pub struct SeqSlice<'a> {
    buf: &'a Box<[u8]>,
    base: *const u8,
    len: usize,
}

impl<'a> SeqSlice<'a> {
    pub fn new(buf: &'a Box<[u8]>) -> SeqSlice<'a> {
        SeqSlice {
            base: buf.as_ptr(),
            len: buf.len(),
            buf,
        }
    }
    pub fn get_next<'ret>(&mut self, block_len: usize) -> Option<&'ret [u8]> {
        if self.len >= block_len {
            let ptr = self.base;
            self.len -= block_len;
            self.base = unsafe { self.base.add(block_len) };
            Some(unsafe { slice::from_raw_parts(ptr, block_len) })
        } else {
            None
        }
    }
    pub fn get_next_t<'ret, T>(&mut self, block_len: usize) -> Option<&'ret [T]> {
        let real_len = block_len * size_of::<T>();
        if self.len >= real_len {
            let ptr = self.base;
            self.len -= real_len;
            self.base = unsafe { self.base.add(real_len) };
            Some(unsafe { slice::from_raw_parts(ptr as *const T, block_len) })
        } else {
            None
        }
    }
    pub fn get_next_aligned_t<'ret, T>(&mut self, block_len: usize) -> Option<&'ret [T]> {
        let alignment = align_of::<T>() as isize;

        let offset = ((-(self.base as isize)) & (alignment - 1)) as usize;
        let real_len = block_len * size_of::<T>();

        if self.len >= real_len + offset {
            self.base = unsafe { self.base.add(offset) };
            let ptr = self.base;
            self.len -= real_len + offset;
            self.base = unsafe { self.base.add(real_len) };
            Some(unsafe { slice::from_raw_parts(ptr as *const T, block_len) })
        } else {
            None
        }
    }
    pub fn base(&self) -> *const u8 {
        self.base
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn end(self) -> Option<()> {
        if self.len == 0 { Some(()) } else { None }
    }
}

pub trait AlignedPush {
    fn align_to_t<T>(&mut self);
    fn push_aligned<T>(&mut self, value: T);
}

impl AlignedPush for Vec<u8> {
    fn align_to_t<T>(&mut self) {
        let alignment = align_of::<T>() as isize;

        if alignment == 1 {
            return;
        }

        let ptr = self.as_ptr() as isize + self.len() as isize;
        let offset = ((-ptr) & (alignment - 1)) as usize;

        self.reserve(offset);
        unsafe { self.set_len(self.len() + offset) };
    }
    fn push_aligned<T>(&mut self, value: T) {
        let size = size_of::<T>();
        let alignment = align_of::<T>() as isize;

        let ptr = self.as_ptr() as isize + self.len() as isize;
        let offset = ((-ptr) & (alignment - 1)) as usize;

        self.reserve(offset + size);

        let elem = (ptr as usize + offset) as *mut T;

        unsafe {
            *elem = value;
            self.set_len(self.len() + offset + size)
        };
    }
}

// pub trait AlignedPop {
//     fn pop_aligned_to_t<T>(&mut self);
//     fn pop_aligned<'a, T>(&mut self) -> &'a T;
// }

// impl AlignedPop for &[u8] {
//     fn pop_aligned_to_t<T>(&mut self) {
//         let alignment = align_of::<T>() as isize;

//         let ptr = self.as_ptr() as isize + self.len() as isize;
//         let offset = ((-ptr) & (alignment - 1)) as usize;

//         *self = &self[offset..];
//     }
//     fn pop_aligned<'a, T>(&mut self) -> &'a T {
//         self.pop_aligned_to_t::<T>();
//         let size = size_of::<T>();

//         let ptr = unsafe { (self.as_ptr() as *mut T).as_ref_unchecked() };
//         *self = &self[size..];
//         return ptr;
//     }
// }

pub struct Stack<T: Copy, const SIZE: usize>
where
    [(); SIZE]:,
{
    stack: [MaybeUninit<T>; SIZE],
    head: usize,
}

impl<T: Debug + Copy, const N: usize> Debug for Stack<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stack: [")?;
        for i in 0..self.head {
            write!(f, "{:?}", unsafe { self.stack[i].assume_init_ref() })?;
        }
        write!(f, "]")
    }
}

impl<T: Copy, const N: usize> Stack<T, N> {
    pub fn new() -> Stack<T, N> {
        Stack {
            stack: [MaybeUninit::uninit(); N],
            head: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.head
    }
    pub fn push(&mut self, value: T) -> Option<()> {
        if self.head < N {
            self.stack[self.head].write(value);
            self.head += 1;
            Some(())
        } else {
            None
        }
    }
    pub fn push_unchecked(&mut self, value: T) {
        self.stack[self.head].write(value);
        self.head += 1;
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.head != 0 {
            self.head -= 1;
            Some(unsafe { self.stack[self.head].assume_init() })
        } else {
            None
        }
    }
    pub unsafe fn slice_last(&mut self, len: usize) -> Option<&[T]> {
        if len <= self.len() {
            Some(unsafe { self.stack[self.head - len..self.head].assume_init_ref() })
        } else {
            None
        }
    }
    pub unsafe fn mut_slice_last(&mut self, len: usize) -> Option<&mut [T]> {
        if len <= self.len() {
            Some(unsafe { self.stack[self.head - len..self.head].assume_init_mut() })
        } else {
            None
        }
    }
}

pub trait CopyT {
    fn copy_t_aligned<T: Copy>(&mut self, slice: Vec<T>) -> usize;
}

impl CopyT for Vec<u8> {
    fn copy_t_aligned<T: Copy>(&mut self, slice: Vec<T>) -> usize {
        self.align_to_t::<T>();
        let base = self.len();

        let slice_len = slice.len() * size_of::<T>();

        self.reserve(slice_len);
        unsafe { self.set_len(base + slice_len) };

        let s = unsafe {
            slice::from_raw_parts_mut((&mut self[base]) as *mut u8 as *mut T, slice.len())
        };

        s.copy_from_slice(&slice);

        return base;
    }
}

//FIXME Restrict A further
/// Requires the caller to drop the alloc after this thing.
#[derive(Debug)]
pub struct KernelMem<T: Debug + ?Sized, A = Const> {
    ptr: *mut T,
    _marker: PhantomData<A>,
}

impl<T: Debug + ?Sized> KernelMem<T> {
    pub fn from_const(ptr: *const T) -> KernelMem<T, Const> {
        KernelMem::<T, Const> {
            ptr: ptr.cast_mut(),
            _marker: PhantomData::default(),
        }
    }
    pub fn from_mut(ptr: *mut T) -> KernelMem<T, Mut> {
        KernelMem::<T, Mut> {
            ptr: ptr,
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Debug + ?Sized, A> KernelMem<T, A> {
    pub fn get_ref<'a>(&'a self) -> &'a T {
        unsafe { self.ptr.as_ref().unwrap_unchecked() }
    }
}

impl<T: Debug + ?Sized> KernelMem<T, Mut> {
    pub fn get_ref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { self.ptr.as_mut().unwrap_unchecked() }
    }
}

#[derive(Debug)]
pub struct Const;
#[derive(Debug)]
pub struct Mut;
