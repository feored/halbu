use std::time::SystemTime;
use std::ops::Range;

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
        return file_section.offset..(file_section.offset + file_section.bytes);
    }
}

pub fn u32_from(slice: &[u8]) -> u32 {
    u32::from_le_bytes(
        slice
            .try_into()
            .unwrap(),
    )
}

pub fn u16_from(slice: &[u8]) -> u16 {
    u16::from_le_bytes(
        slice
            .try_into()
            .unwrap(),
    )
}

pub fn u8_from(slice: &[u8]) -> u8 {
    slice[0]
}
