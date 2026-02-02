use std::{error::Error, ffi::CStr, fmt::Display, fs::OpenOptions, io::Read};

use crate::stuff::SeqBuf;

const MAXENTRYSIZE: usize = 4096;
const MAXEXTENTRYSIZE: usize = 32768;

pub struct TermInfo<'file> {
    file: Box<[u8]>,
    // FIXME Overthink this TM
    /// The last is the verbose and descriptive name.
    names: Vec<&'file str>,
    bools: &'file [TermInfoBool],
    ints: FormatnInts<'file>,
    strings: Vec<TermInfoString<'file>>,
    ext_format: Option<TermInfoExt<'file>>,
}

#[derive(Debug)]
pub enum ParseError {
    Debug,

    NoPadding,
    FileNotEmpty,

    NoHeader,
    InvalidHeader,
    NoExtHeader,
    InvalidExtHeader,

    NoNames,
    InvalidNames,
    InvalidUTF8Name,

    NoBools,
    InvalidBool,
    NoExtBools,
    InvalidExtBool,

    NoInts,
    InvalidInt(i32),
    NoExtInts,
    InvalidExtInt(i32),

    NoStrOffs,
    NoStrTable,
    InvalidStrIdx,
    InvalidCStr,
    InvalidStrTable,
    NoExtStrOffs,
    NoExtNameOffs,
    NoExtStrTable,
    InvalidExtStrIdx,
    InvalidExtCStr,
    InvalidExtStrTable,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

impl TermInfo<'_> {
    pub const PATH: &'static str = "/usr/share/terminfo/";
    pub fn new<'a, 'b>(path: &'a str) -> Result<TermInfo<'b>, ParseError> {
        // FIXME Use a better Reader
        let mut file = Vec::new();
        _ = OpenOptions::new()
            .read(true)
            .open(path)
            .unwrap()
            .read_to_end(&mut file)
            .unwrap();

        let file = file.into_boxed_slice();
        let mut file_ref = file.as_slice();

        let header = match file_ref.get_first_to_t::<Header>(1) {
            Some(header) => &header[0],
            None => return Err(ParseError::NoHeader),
        };

        if !header.valid() {
            return Err(ParseError::InvalidHeader);
        }

        // FIXME Add BIG Endianess
        let names = match file_ref.get_first(header.name_size as usize) {
            Some(names) => {
                let cstr = match CStr::from_bytes_with_nul(names) {
                    Ok(str) => str,
                    Err(_) => return Err(ParseError::InvalidNames),
                };
                let str = match cstr.to_str() {
                    Ok(str) => str,
                    Err(_) => return Err(ParseError::InvalidUTF8Name),
                };
                str.split('|').collect::<Vec<&str>>()
            }
            _ => return Err(ParseError::NoNames),
        };

        let bools = TermInfoBool::parse::<false>(&mut file_ref, header.bool_count)?;

        if file_ref.as_ptr().mask(1) as usize != 0
            && file_ref
                .get_first(1)
                .map(|val| if val[0] == 0 { Some(()) } else { None })
                .is_none()
        {
            return Err(ParseError::NoPadding);
        }

        let ints = FormatnInts::parse::<false>(&mut file_ref, header.int_count, header.version)?;

        let (strings, _) = TermInfoString::parse::<false>(
            &mut file_ref,
            header.offs_count as usize,
            None,
            header.str_table_size as usize,
        )?;

        let ext_format = if !file_ref.is_empty() {
            Some(TermInfoExt::parse(&mut file_ref, header.version)?)
        } else {
            None
        };

        return Ok(TermInfo {
            file,
            names,
            bools,
            ints,
            strings,
            ext_format,
        });
    }
}

pub struct TermInfoExt<'file> {
    name_table: Vec<TermInfoString<'file>>,
    bools: &'file [TermInfoBool],
    ints: FormatnInts<'file>,
    str_table: Vec<TermInfoString<'file>>,
}

impl TermInfoExt<'_> {
    fn parse<'buf, 'file>(
        buf: &mut &'buf [u8],
        version: TermInfoVersion,
    ) -> Result<TermInfoExt<'file>, ParseError> {
        if buf.as_ptr().mask(1) as usize != 0
            && buf
                .get_first(1)
                .map(|val| if val[0] == 0 { Some(()) } else { None })
                .is_none()
        {
            return Err(ParseError::NoPadding);
        }

        let ext_header = match buf.get_first_to_t::<ExtHeader>(1) {
            Some(head) => &head[0],
            None => return Err(ParseError::NoExtHeader),
        };

        if !ext_header.valid(if version == TermInfoVersion::Legacy {
            MAXENTRYSIZE
        } else {
            MAXEXTENTRYSIZE
        }) {
            return Err(ParseError::InvalidExtHeader);
        }

        let bools = TermInfoBool::parse::<true>(buf, ext_header.bool_count)?;

        if buf.as_ptr().mask(1) as usize != 0
            && buf
                .get_first(1)
                .map(|val| if val[0] == 0 { Some(()) } else { None })
                .is_none()
        {
            return Err(ParseError::NoPadding);
        }

        let ints = FormatnInts::parse::<true>(buf, ext_header.int_count, version)?;

        let (str_table, name_table) = TermInfoString::parse::<true>(
            buf,
            ext_header.str_count as usize,
            Some(
                ext_header.bool_count as usize
                    + ext_header.int_count as usize
                    + ext_header.str_count as usize,
            ),
            ext_header.str_table_size as usize,
        )?;

        if !buf.is_empty() {
            return Err(ParseError::FileNotEmpty);
        }

        return Ok(TermInfoExt {
            name_table,
            bools,
            ints,
            str_table,
        });
    }
}

#[repr(C)]
struct Header {
    version: TermInfoVersion,
    name_size: i16,
    bool_count: i16,
    int_count: i16,
    offs_count: i16,
    str_table_size: i16,
}

impl Header {
    fn valid(&self) -> bool {
        if !(self.version == TermInfoVersion::Legacy || self.version == TermInfoVersion::Extended) {
            return false;
        }

        if self.name_size <= 0
            || self.name_size >= 512
            || self.bool_count < 0
            || self.bool_count > 44
            || self.int_count < 0
            || self.int_count > 39
            || self.offs_count < 0
            || self.offs_count > 414
            || self.str_table_size < 0
        {
            return false;
        }
        return true;
    }
}

#[repr(C)]
#[derive(Debug)]
struct ExtHeader {
    bool_count: i16,
    int_count: i16,
    str_count: i16,
    str_table_count: i16,
    str_table_size: i16,
}

impl ExtHeader {
    fn valid(&self, max_entry: usize) -> bool {
        if self.bool_count < 0
            || self.int_count < 0
            || self.str_count < 0
            || self.str_table_count < 0
            || self.str_table_size < 0
            || self.str_table_size as usize >= max_entry
            || self.str_table_count as usize >= max_entry
            || self.bool_count as usize + self.int_count as usize + self.str_count as usize
                >= max_entry / 2
        {
            return false;
        }
        true
    }
}

#[repr(i16)]
#[derive(PartialEq, Clone, Copy, Debug)]
enum TermInfoVersion {
    Legacy = 0x011A,
    Extended = 0x021E,
}

#[repr(u16)]
#[derive(Debug)]
pub enum FormatnInts<'file> {
    Legacy(&'file [TermInfoInt]) = 0x011A,
    Extended(&'file [TermInfoExtInt]) = 0x21E,
}

impl FormatnInts<'_> {
    fn parse<'file, const EXT: bool>(
        buf: &mut &[u8],
        count: i16,
        version: TermInfoVersion,
    ) -> Result<FormatnInts<'file>, ParseError> {
        Ok(match version {
            TermInfoVersion::Legacy => {
                FormatnInts::Legacy(match buf.get_first_to_t::<TermInfoInt>(count as usize) {
                    Some(ints) => {
                        for int in ints {
                            if *int < -2 {
                                if EXT {
                                    return Err(ParseError::InvalidExtInt(*int as i32));
                                } else {
                                    return Err(ParseError::InvalidInt(*int as i32));
                                }
                            }
                        }
                        ints
                    }
                    _ => {
                        if EXT {
                            return Err(ParseError::NoExtInts);
                        } else {
                            return Err(ParseError::NoInts);
                        }
                    }
                })
            }
            TermInfoVersion::Extended => {
                FormatnInts::Extended(match buf.get_first_to_t::<TermInfoExtInt>(count as usize) {
                    Some(ints) => {
                        for slice in ints {
                            let int = i32::from_le_bytes(*slice);
                            if int < -2 {
                                if EXT {
                                    return Err(ParseError::InvalidExtInt(int));
                                } else {
                                    return Err(ParseError::InvalidInt(int));
                                }
                            }
                        }
                        ints
                    }
                    _ => {
                        if EXT {
                            return Err(ParseError::NoExtInts);
                        } else {
                            return Err(ParseError::NoInts);
                        }
                    }
                })
            }
        })
    }
}

pub enum TermInfoString<'file> {
    There(&'file [u8]),
    NotImpl,
    IdkCanceled,
}

impl TermInfoString<'_> {
    fn parse<'file, const EXT: bool>(
        buf: &mut &[u8],
        str_count: usize,
        name_count: Option<usize>,
        str_table_size: usize,
    ) -> Result<(Vec<TermInfoString<'file>>, Vec<TermInfoString<'file>>), ParseError> {
        let mut str_table = Vec::with_capacity(str_count);
        let mut name_table = if EXT {
            Vec::with_capacity(name_count.unwrap())
        } else {
            Vec::new()
        };

        let str_offs = match buf.get_first_to_t::<i16>(str_count) {
            Some(offs) => offs,
            None => {
                if EXT {
                    return Err(ParseError::NoExtStrOffs);
                } else {
                    return Err(ParseError::NoStrOffs);
                }
            }
        };

        let name_offs = if EXT {
            Some(match buf.get_first_to_t::<i16>(name_count.unwrap()) {
                Some(name_offs) => name_offs,
                None => return Err(ParseError::NoExtNameOffs),
            })
        } else {
            None
        };

        if str_table_size == 0 {
            return Ok((str_table, name_table));
        }

        let mut string_table = match buf.get_first(str_table_size) {
            Some(str_table) => str_table,
            None => {
                if EXT {
                    return Err(ParseError::NoExtStrTable);
                } else {
                    return Err(ParseError::NoStrTable);
                }
            }
        };

        fn get_next_offs(buf: &[i16], i: usize) -> Option<usize> {
            let mut ret = None;
            if buf.len() > i + 1 {
                for n_off in &buf[i + 1..] {
                    match n_off {
                        (1..) => {
                            ret = Some(*n_off as usize);
                            break;
                        }
                        _ => continue,
                    }
                }
            }
            return ret;
        }

        let mut idx: usize = 0;
        let mut n_idx: Option<usize>;

        if str_offs.len() != 0 {
            for (i, off) in str_offs.iter().enumerate() {
                str_table.push(match off {
                    (0..) => {
                        //Fixme P1
                        n_idx = get_next_offs(str_offs, i);
                        let cstr = match n_idx {
                            Some(n_idx) => {
                                match CStr::from_bytes_with_nul(&string_table[idx as usize..n_idx])
                                {
                                    Ok(cstr) => {
                                        idx = n_idx;
                                        cstr
                                    }
                                    Err(_) => {
                                        return if EXT {
                                            Err(ParseError::InvalidExtCStr)
                                        } else {
                                            Err(ParseError::InvalidCStr)
                                        };
                                    }
                                }
                            }
                            None => match CStr::from_bytes_until_nul(&string_table[idx..]) {
                                Ok(cstr) => {
                                    if EXT {
                                        if name_count.unwrap() == 0
                                            && idx + cstr.count_bytes() + 1 != string_table.len()
                                        {
                                            return Err(ParseError::InvalidExtStrTable);
                                        }
                                        string_table =
                                            &string_table[idx + cstr.count_bytes() + 1..];
                                        idx = 0;
                                    } else {
                                        if idx + cstr.count_bytes() + 1 != string_table.len() {
                                            return Err(ParseError::InvalidStrTable);
                                        }
                                    }
                                    cstr
                                }
                                Err(_) => {
                                    return if EXT {
                                        Err(ParseError::InvalidExtCStr)
                                    } else {
                                        Err(ParseError::InvalidCStr)
                                    };
                                }
                            },
                        };
                        TermInfoString::There(cstr.to_bytes())
                    }
                    -1 => TermInfoString::NotImpl,
                    -2 => TermInfoString::IdkCanceled,
                    _ => {
                        return if EXT {
                            Err(ParseError::InvalidExtStrIdx)
                        } else {
                            Err(ParseError::InvalidStrIdx)
                        };
                    }
                });
            }
        }

        if EXT && name_count.unwrap() != 0 {
            let name_offs = name_offs.unwrap();
            for (i, off) in name_offs.iter().enumerate() {
                name_table.push(match off {
                    (0..) => {
                        n_idx = get_next_offs(name_offs, i);
                        let cstr = match n_idx {
                            Some(n_idx) => {
                                match CStr::from_bytes_with_nul(&string_table[idx as usize..n_idx])
                                {
                                    Ok(cstr) => {
                                        idx = n_idx;
                                        cstr
                                    }
                                    Err(_) => {
                                        return Err(ParseError::InvalidExtCStr);
                                    }
                                }
                            }
                            None => match CStr::from_bytes_until_nul(&string_table[idx..]) {
                                Ok(cstr) => {
                                    if idx + cstr.count_bytes() + 1 != string_table.len() {
                                        return Err(ParseError::InvalidStrTable);
                                    }

                                    cstr
                                }
                                Err(_) => {
                                    return Err(ParseError::InvalidExtCStr);
                                }
                            },
                        };
                        TermInfoString::There(cstr.to_bytes())
                    }
                    -1 => TermInfoString::NotImpl,
                    -2 => TermInfoString::IdkCanceled,
                    _ => {
                        return if EXT {
                            Err(ParseError::InvalidExtStrIdx)
                        } else {
                            Err(ParseError::InvalidStrIdx)
                        };
                    }
                });
            }
        }

        Ok((str_table, name_table))
    }
}

#[repr(i8)]
#[derive(Clone, Copy)]
pub enum TermInfoBool {
    Working = 1,
    Disabled = 0,
    NotImpl = -1,
    IdkCanceled = -2,
}

impl TermInfoBool {
    fn parse<'file, const EXT: bool>(
        buf: &mut &[u8],
        count: i16,
    ) -> Result<&'file [TermInfoBool], ParseError> {
        match buf.get_first_to_t::<TermInfoBool>(count as usize) {
            Some(slice) => {
                for bool in slice {
                    if !bool.valid() {
                        if EXT {
                            return Err(ParseError::InvalidExtBool);
                        } else {
                            return Err(ParseError::InvalidBool);
                        }
                    }
                }
                return Ok(slice);
            }
            None => {
                if EXT {
                    return Err(ParseError::NoExtBools);
                } else {
                    return Err(ParseError::NoBools);
                }
            }
        }
    }
    fn valid(&self) -> bool {
        ((*self) as i8) < 2 && (*self) as i8 > -3
    }
}

type TermInfoInt = i16;
// FIXME Fix the Alignment Problem
type TermInfoExtInt = [u8; 4];

#[test]
fn test() {
    use std::fs::read_dir;
    let mut success = 0;
    let mut err = 0;
    for dir in read_dir("/usr/share/terminfo").unwrap() {
        for file in read_dir(dir.unwrap().path()).unwrap() {
            let y = file.unwrap().path();
            let y2 = y.to_str().unwrap();
            let ok = std::panic::catch_unwind(|| {
                let res = TermInfo::new(y2);
                res.unwrap();
            });
            if ok.is_err() {
                err += 1;
                println!("{:?}", y2)
            } else {
                success += 1;
            }
        }
    }
    println!("{} | {}", success, err);
}
