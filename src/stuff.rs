use core::fmt::NumBufferTrait;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct NumberBuffer<T: NumBufferTrait>
where
    [(); T::BUF_SIZE]:,
{
    pub buf: [MaybeUninit<u8>; T::BUF_SIZE],
}

impl<T: NumBufferTrait> NumberBuffer<T>
where
    [(); T::BUF_SIZE]:,
{
    pub fn new() -> NumberBuffer<T> {
        NumberBuffer {
            buf: [MaybeUninit::uninit(); T::BUF_SIZE],
        }
    }
    pub fn as_slice(&self, len: usize) -> &[u8] {
        unsafe{&*(&self.buf[..len] as *const [MaybeUninit<u8>] as *const [u8])};
    }
    pub fn as_str(&self, len: usize) -> &str {
        let slice: &[u8] =
            unsafe { &*(&self.buf[..len] as *const [MaybeUninit<u8>] as *const [u8]) };
        let s = unsafe { std::str::from_utf8_unchecked(slice) };
        s
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
                fn forward_format(&self, _buf: &mut NumberBuffer<Self>)->usize {
                    0
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

                    let mut was_zero = true;
                    let mut len = Self::BUF_SIZE;

                    let mut num = *self;
                    let mut digit;

                    let mut i = 0;
                    let mut i2 = 0;

                    while i < IDK {
                        let step = STEPS[i];
                        digit = num/step;
                        if was_zero {
                            if digit != 0 {
                                was_zero = false;
                                len-=i;
                            } else {
                                i+=1;
                                continue;
                            }
                        }
                        num %= step;
                        buf.buf[i2].write(b'0' + digit as u8);
                        i+=1;
                        i2+=1;
                    }
                    buf.buf[i2].write(b'0' + num as u8);
                    len
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
