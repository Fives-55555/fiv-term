#[test]
fn test_format_int_zero() {
    use crate::stuff::{FastForwardFormat, NumberSlice};

    let mut len;

    let mut buf = NumberSlice::<i32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 0.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "0");

    // again, with unsigned

    let mut buf = NumberSlice::<u32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 0u32.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "0");
}

#[test]
fn test_format_int_one() {
    use crate::stuff::{FastForwardFormat, NumberSlice};

    // Formatting integers should select the right implementation based off
    // the type of the argument. Also, hex/octal/binary should be defined
    // for integers, but they shouldn't emit the negative sign.

    let mut len;

    let mut buf = NumberSlice::<isize>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1isize.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<i8>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1i8.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<i16>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1i16.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<i32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1i32.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<i64>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1i64.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<i128>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1i128.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    // again, with unsigned
    let mut buf = NumberSlice::<usize>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1usize.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<u8>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1u8.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<u16>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1u16.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<u32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1u32.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<u64>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1u64.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    let mut buf = NumberSlice::<u128>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 1u128.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "1");

    // again, with negative

    let mut buf = NumberSlice::<isize>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = (-1isize).forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-1");

    let mut buf = NumberSlice::<i8>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = (-1i8).forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-1");

    let mut buf = NumberSlice::<i16>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = (-1i16).forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-1");

    let mut buf = NumberSlice::<i32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = (-1i32).forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-1");

    let mut buf = NumberSlice::<i64>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = (-1i64).forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-1");

    let mut buf = NumberSlice::<i128>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = (-1i128).forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-1");
}

#[test]
fn test_format_int_misc() {
    use crate::stuff::{FastForwardFormat, NumberSlice};

    let mut len;

    let mut buf = NumberSlice::<i32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = 55.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "55");
    len = 5.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "5");
}

#[test]
fn test_format_int_limits() {
    use crate::stuff::{FastForwardFormat, NumberSlice};

    let mut len;

    let mut buf = NumberSlice::<i8>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = i8::MIN.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-128");
    len = i8::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "127");

    let mut buf = NumberSlice::<i16>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = i16::MIN.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-32768");
    len = i16::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "32767");

    let mut buf = NumberSlice::<i32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = i32::MIN.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-2147483648");
    len = i32::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "2147483647");

    let mut buf = NumberSlice::<i64>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = i64::MIN.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "-9223372036854775808");
    len = i64::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "9223372036854775807");

    let mut buf = NumberSlice::<i128>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = i128::MIN.forward_format(&mut slice);
    assert_eq!(
        slice.as_str(len),
        "-170141183460469231731687303715884105728"
    );
    len = i128::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "170141183460469231731687303715884105727");

    let mut buf = NumberSlice::<u8>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = u8::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "255");

    let mut buf = NumberSlice::<u16>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = u16::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "65535");

    let mut buf = NumberSlice::<u32>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = u32::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "4294967295");

    let mut buf = NumberSlice::<u64>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = u64::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "18446744073709551615");

    let mut buf = NumberSlice::<u128>::new_buf();
    let mut slice = NumberSlice::new(&mut buf).unwrap();
    len = u128::MAX.forward_format(&mut slice);
    assert_eq!(slice.as_str(len), "340282366920938463463374607431768211455");
}
