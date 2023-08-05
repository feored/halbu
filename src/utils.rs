use std::ops::Range;
use std::cmp;
use std::time::SystemTime;

use bit::BitIndex;


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



/// Write bits_count number of bits (LSB ordering) from bits_source into a vector of bytes.
pub fn write_byte(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: u8,
    bits_count: usize,
) {
    let mut bits_left_to_write: usize = bits_count;
    let mut bit_index = 0;
    loop {
        if bits_left_to_write == 0 {
            return;
        }
        if byte_vector.len() == byte_position.current_byte {
            byte_vector.push(0);
        }

        if byte_position.current_bit == 8 {
            byte_vector.push(0);
            byte_position.current_byte += 1;
            byte_position.current_bit = 0;
        }

        let bits_can_write_in_byte = cmp::min(bits_left_to_write, 8 - byte_position.current_bit);

        if bits_can_write_in_byte == 8 {
            // Special case because the bit library seems to fail when trying to set an entire byte using set_bit_range
            // e.g 0x00.set_bit_range(0..8, 0xFF)
            byte_vector[byte_position.current_byte] = bits_source;
        } else {
            byte_vector[byte_position.current_byte].set_bit_range(
                byte_position.current_bit..(byte_position.current_bit + bits_can_write_in_byte),
                bits_source.bit_range(bit_index..(bit_index + bits_can_write_in_byte)),
            );
            bit_index += bits_can_write_in_byte;
        }
        byte_position.current_bit += bits_can_write_in_byte;
        bits_left_to_write -= bits_can_write_in_byte;
    }
}

/// Write bits_count number of bits (LSB ordering) from bits_source into a vector of u8.
pub fn write_bits<T: Into<u32>>(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: T,
    bits_count: usize,
) {
    let mut bits_left_to_write: usize = bits_count;
    let byte_source = bits_source.into().to_le_bytes();
    let mut byte_source_current = 0;
    loop {
        if bits_left_to_write == 0 {
            return;
        }
        let bits_can_write = cmp::min(bits_left_to_write, 8);
        write_byte(byte_vector, byte_position, byte_source[byte_source_current], bits_can_write);
        bits_left_to_write -= bits_can_write;
        byte_source_current += 1;
    }
}


/// Read a certain number of bits in a vector of bytes, starting at a given byte and bit index, and return a u32 with the value.
///
/// The attributes are stored in a packed struct with non-aligned bytes.
/// Headers for instance contain 9 bits, so they must be read over multiple bytes.
pub fn read_bits(byte_slice: &[u8], byte_position: &mut BytePosition, bits_to_read: usize) -> u32 {
    let mut bits_left_to_read: usize = bits_to_read;
    let mut buffer: u32 = 0;
    let mut buffer_bit_position: usize = 0;
    loop {
        if bits_left_to_read == 0 {
            break;
        }
        if byte_position.current_bit > 7 {
            byte_position.current_byte += 1;
            byte_position.current_bit = 0;
        }
        let bits_parsing_count = cmp::min(8 - byte_position.current_bit, bits_left_to_read);
        let bits_parsed: u8 = byte_slice[byte_position.current_byte]
            .bit_range(byte_position.current_bit..(byte_position.current_bit + bits_parsing_count));

        buffer.set_bit_range(
            buffer_bit_position..(buffer_bit_position + bits_parsing_count),
            u32::from_le_bytes([bits_parsed, 0x00, 0x00, 0x00]),
        );
        buffer_bit_position += bits_parsing_count;
        bits_left_to_read -= bits_parsing_count;
        byte_position.current_bit += bits_parsing_count;
    }
    buffer
}