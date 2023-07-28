use std::ops::Range;
use std::time::SystemTime;

pub fn get_sys_time_in_secs() -> u32 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as u32,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct FileSection {
    pub offset: usize,
    pub bytes: usize,
}

impl From<FileSection> for Range<usize> {
    fn from(file_section: FileSection) -> Range<usize> {
        file_section.offset..(file_section.offset + file_section.bytes)
    }
}

/// Keep track of current byte and bit index in the attributes byte vector.
#[derive(Default, PartialEq, Eq, Debug)]
pub struct BytePosition {
    pub current_byte: usize,
    pub current_bit: usize,
}

pub fn u32_from(slice: &[u8]) -> u32 {
    u32::from_le_bytes(slice.try_into().unwrap())
}

pub fn u16_from(slice: &[u8]) -> u16 {
    u16::from_le_bytes(slice.try_into().unwrap())
}

pub fn u8_from(slice: &[u8]) -> u8 {
    slice[0]
}
