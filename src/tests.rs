
#[test]
fn test_format_int_zero() {
    use crate::{NumberBuffer, FastForwardFormat};
    
    let mut buf = NumberBuffer::new();
    let mut len = 0.forward_format(&mut buf);
    
    assert_eq!(buf.as_str(len), "0");

    // again, with unsigned
    
    let mut buf = NumberBuffer::new();
    len = 0u32.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "0");
}

#[test]
fn test_format_int_one() {
    use crate::{FastForwardFormat, NumberBuffer};
    let mut buf = NumberBuffer::new();
    let mut len;

    // Formatting integers should select the right implementation based off
    // the type of the argument. Also, hex/octal/binary should be defined
    // for integers, but they shouldn't emit the negative sign.
    
    len = 1isize.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1i8.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1i16.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1i32.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1i64.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1i128.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();

    // again, with unsigned
    len = 1usize.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1u8.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1u16.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1u32.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1u64.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();
    len = 1u128.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "1");
    let mut buf = NumberBuffer::new();

    // again, with negative
    len = (-1isize).forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-1");
    let mut buf = NumberBuffer::new();
    len = (-1i8).forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-1");
    let mut buf = NumberBuffer::new();
    len = (-1i16).forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-1");
    let mut buf = NumberBuffer::new();
    len = (-1i32).forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-1");
    let mut buf = NumberBuffer::new();
    len = (-1i64).forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-1");
    let mut buf = NumberBuffer::new();
    len = (-1i128).forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-1");
}

#[test]
fn test_format_int_misc() {
    use crate::{FastForwardFormat, NumberBuffer};
    let mut buf = NumberBuffer::new();
    let len = 55.forward_format(&mut buf);

    assert_eq!(buf.as_str(len), "55");
}

#[test]
fn test_format_int_limits() {
    use crate::{FastForwardFormat, NumberBuffer};
    let mut buf = NumberBuffer::new();
    let mut len;

    len = i8::MIN.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-128");
    len = i8::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "127");
    
    let mut buf = NumberBuffer::new();

    len = i16::MIN.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-32768");
    len = i16::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "32767");

    let mut buf = NumberBuffer::new();
    
    len = i32::MIN.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-2147483648");
    len = i32::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "2147483647");
    
    let mut buf = NumberBuffer::new();

    len = i64::MIN.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-9223372036854775808");
    len = i64::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "9223372036854775807");
    
    let mut buf = NumberBuffer::new();
    
    len = i128::MIN.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "-170141183460469231731687303715884105728");
    len = i128::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "170141183460469231731687303715884105727");

    let mut buf = NumberBuffer::new();
    
    len = u8::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "255");
    
    let mut buf = NumberBuffer::new();

    len = u16::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "65535");
    
    let mut buf = NumberBuffer::new();

    len = u32::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "4294967295");
    
    let mut buf = NumberBuffer::new();

    len = u64::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "18446744073709551615");
    
    let mut buf = NumberBuffer::new();
    
    len = u128::MAX.forward_format(&mut buf);
    assert_eq!(buf.as_str(len), "340282366920938463463374607431768211455");
}
