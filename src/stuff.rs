use core::fmt::NumBufferTrait;
use std::{mem::MaybeUninit, slice};

#[derive(Debug)]
pub struct NumberBuffer<T: NumBufferTrait>
where
    [(); T::BUF_SIZE]:,
{
    pub buf: [u8; T::BUF_SIZE],
}

impl<T: NumBufferTrait> NumberBuffer<T>
where
    [(); T::BUF_SIZE]:,
{
    pub fn new() -> NumberBuffer<T> {
        NumberBuffer {
            buf: unsafe { MaybeUninit::array_assume_init([MaybeUninit::uninit(); T::BUF_SIZE]) },
        }
    }
    pub fn as_slice(&self, len: usize) -> &[u8] {
        &self.buf[..len]
    }
    pub fn as_str(&self, len: usize) -> &str {
        let slice: &[u8] = &self.buf[..len];
        let s = unsafe { std::str::from_utf8_unchecked(slice) };
        s
    }
}

impl<F: NumBufferTrait> From<&mut [u8]> for NumberBuffer<F>
where
    [(); F::BUF_SIZE]:,
{
    fn from(value: &mut [u8]) -> Self {
        debug_assert!(value.len() > F::BUF_SIZE);
        let buf: &mut [u8; F::BUF_SIZE] = value.try_into().expect("Debug Assert Failed");
        NumberBuffer { buf: *buf }
    }
}

pub trait FastForwardFormat: NumBufferTrait + Sized
where
    [(); Self::BUF_SIZE]:,
{
    fn forward_format(&self, buf: &mut NumberBuffer<Self>) -> usize;
}

macro_rules! fast_forward_format {
    ($($signed:ident, $unsigned:ident,)*) => {
        $(
            impl FastForwardFormat for $signed {
                fn forward_format(&self, buf: &mut NumberBuffer<Self>)->usize {
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
                        buf.buf[i_buf] = b'-';
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
                        buf.buf[i_buf] = b'0' + digit as u8;
                        i+=1;
                        i_buf+=1;
                    }
                    buf.buf[i_buf] = b'0' + num as u8;
                    i_buf+1
                }
            }

            impl FastForwardFormat for $unsigned {
                fn forward_format(&self, buf: &mut NumberBuffer<Self>)->usize {

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

                    while i < IDK && STEPS[i] > num {
                        i+=1;
                        continue;
                    }

                    while i < IDK {
                        let step = STEPS[i];
                        digit = num/step;
                        num %= step;
                        buf.buf[i_buf] = b'0' + digit as u8;
                        i+=1;
                        i_buf+=1;
                    }
                    buf.buf[i_buf] = b'0' + num as u8;
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

pub trait SeqBuf {
    fn get_first<'ret>(self, len: usize) -> Option<&'ret [u8]>;
    fn get_first_to_t<'ret, T>(self, len: usize) -> Option<&'ret [T]>;
}

impl SeqBuf for &[u8] {
    fn get_first<'ret>(mut self, len: usize) -> Option<&'ret [u8]> {
        if self.len() >= len {
            self = &self[len..];
            Some(unsafe { slice::from_raw_parts(self.as_ptr(), len) })
        } else {
            None
        }
    }
    fn get_first_to_t<'ret, T>(mut self, len: usize) -> Option<&'ret [T]> {
        let real_len = len * size_of::<T>();
        if self.len() >= real_len {
            self = &self[real_len..];
            Some(unsafe { slice::from_raw_parts(self.as_ptr() as *const T, len) })
        } else {
            None
        }
    }
}
