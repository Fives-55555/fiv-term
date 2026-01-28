use std::{ffi::CStr, fs::OpenOptions, io::Read};

use crate::stuff::SeqBuf;

pub trait TermControl {
    //fn clear_screen(buf: &VirtSeqBuf) -> Result<()>;
}

pub struct VirtSeq<'a> {
    start: &'a [u8],
    input: &'a [()],
}
// Static

pub struct TermInfo<'file> {
    /// File Format
    /// | Legacy
    /// | - Header {
    /// | -- version/magic(u16)
    /// | -- names lenght(u16)
    /// | -- num of bytes([i8])
    /// | -- num of ints([i16], [i32])
    /// | -- num of str_idx([i16])
    /// | -- length string_table(u16)
    /// | }
    /// |
    /// | NAMES
    /// | BOOLS
    /// | INTS
    /// | STR_OFFS
    /// | STR_TABLE
    /// |-----------
    /// | Extended
    /// |
    file: Box<[u8]>,
    // FIXME Overthink this TM
    /// The last is the verbose and descriptive name.
    names: Vec<&'file str>,
    bools: &'file [TermInfoBool],
    ints: FormatnInts<'file>,
    strings: Vec<TermInfoString<'file>>,
    ext_format: Option<TermInfoExt<'file>>,
}

#[repr(u16)]
enum FormatnInts<'file> {
    Legacy(&'file [i16]) = 0x011A,
    Extended(&'file [i32]) = 0x21E,
}

enum TermInfoString<'file> {
    There(&'file str),
    NotImpl,
    IdkCanceled,
}

#[repr(i8)]
enum TermInfoBool {
    Working = 1,
    Disabled = 0,
    NotImpl = -1,
    IdkCanceled = -2,
}

impl TermInfoBool {
    fn valid(bool: i8) -> bool {
        bool < 2 && bool > -3
    }
}

struct TermInfoExt<'file> {
    //FIXME P1
}

impl TermInfoExt<'_> {
    fn parse<'buf, 'file>(buf: &'buf [u8]) -> Result<TermInfoExt<'file>, ()> {}
}

impl TermInfo<'_> {
    pub const PATH: &'static str = "/usr/share/terminfo/";
    pub fn new<'a, 'b>(path: &'a str) -> Result<TermInfo<'b>, ()> {
        // FIXME Use a better Reader
        let mut file = Vec::new();
        _ = OpenOptions::new()
            .read(true)
            .open(path)
            .unwrap()
            .read_to_end(&mut file)
            .unwrap();

        file.shrink_to_fit();

        let file = file.into_boxed_slice();
        let file_ref = &file;

        let header = match file_ref.get_first(12) {
            Some(header) => Header::parse(&header),
            None => return Err(()),
        };

        if !header.valid() {
            return Err(());
        }

        // FIXME Add BIG Endianess
        let names = match file_ref.get_first(header.name_len as usize) {
            Some(names) => {
                let (last, names) = names.split_last().unwrap();
                let str = match str::from_utf8(names) {
                    Ok(str) => str,
                    Err(_) => return Err(()),
                };
                if !names.contains(&0) && *last == 0 {
                    str.split('|').collect::<Vec<&str>>()
                } else {
                    return Err(());
                }
            }
            _ => return Err(()),
        };

        let bools = match file_ref.get_first_to_t::<TermInfoBool>(header.bool_len as usize) {
            Some(bools) => bools,
            _ => return Err(()),
        };

        let ints = {
            match header.version {
                TermInfoVersion::Legacy => {
                    let ints = match file_ref.get_first_to_t::<i16>(header.num_len as usize) {
                        Some(ints) => ints,
                        _ => return Err(()),
                    };
                    FormatnInts::Legacy(ints)
                }
                TermInfoVersion::Extended => {
                    let ints = match file_ref.get_first_to_t::<i32>(header.num_len as usize) {
                        Some(ints) => ints,
                        _ => return Err(()),
                    };
                    FormatnInts::Extended(ints)
                }
            }
        };
        let offs = match file_ref.get_first_to_t::<i16>(header.offs_len as usize) {
            Some(offs) => offs,
            None => return Err(()),
        };
        let string_table = match file_ref.get_first(header.str_table_len as usize) {
            Some(str_table) => str_table,
            None => return Err(()),
        };

        let strings = match offs
            .iter()
            .map(|idx: &i16| match idx {
                (0..) => Ok(TermInfoString::There(
                    match CStr::from_bytes_until_nul(&string_table[*idx as usize..]) {
                        Ok(str) => match str.to_str() {
                            Ok(str) => str,
                            Err(_) => return Err(()),
                        },
                        Err(_) => return Err(()),
                    },
                )),
                -1 => Ok(TermInfoString::NotImpl),
                -2 => Ok(TermInfoString::IdkCanceled),
                _ => return Err(()),
            })
            .collect::<Result<Vec<TermInfoString>, ()>>()
        {
            Ok(str) => str,
            Err(_) => return Err(()),
        };

        let ext_format = if !file_ref.is_empty() {
            match TermInfoExt::parse(file_ref) {
                Ok(ext) => Some(ext),
                Err(_) => return Err(()),
            }
        } else {
            None
        };

        // Check for spoofing
        // Strings
        // FIXME add overlap check
        let mut len = 0;
        for str in strings.iter() {
            match str {
                TermInfoString::There(str) => len += str.len() + 1,
                _ => continue,
            }
        }
        if len != header.str_table_len as usize {
            return Err(());
        }

        //Ints
        match ints {
            FormatnInts::Legacy(intsa) => {
                for int in intsa.iter() {
                    if *int < -2 {
                        return Err(());
                    }
                }
            }
            FormatnInts::Extended(intsa) => {
                for int in intsa.iter() {
                    if *int < -2 {
                        return Err(());
                    }
                }
            }
        }
        // FIXME P1 Add Valitation
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

#[repr(C)]
struct Header {
    version: TermInfoVersion,
    name_len: u16,
    bool_len: u16,
    num_len: u16,
    offs_len: u16,
    str_table_len: u16,
}

#[repr(u16)]
#[non_exhaustive]
#[derive(PartialEq)]
enum TermInfoVersion {
    Legacy = 0x011A,
    Extended = 0x021E,
}

impl Header {
    fn parse<'a>(buf: &'a [u8]) -> &'a Header {
        #[cfg(target_endian = "little")]
        return unsafe { (buf.as_ptr() as *const Header).as_ref_unchecked() };
        // FIXME BROKEN
        #[cfg(target_endian = "big")]
        {
            let mut header: &mut Header = &mut *(buf.as_mut_ptr() as *mut Header);
            header.magic_num_maybe.to_be();
            header.name_len.to_be();
            header.bool_len.to_be();
            header.num_len.to_be();
            header.idk_str_count.to_be();
            header.str_table_len.to_be();
            return;
        }
    }
    fn valid(&self) -> bool {
        self.name_len > 0
            && (self.version == TermInfoVersion::Legacy
                || self.version == TermInfoVersion::Extended)
    }
}

#[test]
fn test() {
    use std::fs::read_dir;
    for dir in read_dir("/usr/share/terminfo").unwrap() {
        for file in read_dir(dir.unwrap().path()).unwrap() {
            let y = file.unwrap().path();
            let y2 = y.to_str().unwrap();
            let ok = std::panic::catch_unwind(|| {
                _ = TermInfo::new(y2);
            });
            if ok.is_err() {
                println!("{:?}", y2)
            }
        }
    }
}
