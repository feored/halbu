use crate::{D2SError, FileCutOffError};
use bit::BitIndex;
use std::cmp;

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub struct BytePosition {
    pub current_byte: usize,
    pub current_bit: usize,
}

impl BytePosition {
    pub fn total_bits(&self) -> usize {
        self.current_byte * 8 + self.current_bit
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct ByteIO {
    pub data: Vec<u8>,
    pub position: BytePosition,
}

impl ByteIO {
    pub(crate) fn align_position(&mut self) {
        if self.position.current_bit > 0 {
            self.position.current_byte += 1;
            self.position.current_bit = 0;
        }
    }
    pub(crate) fn align_writer(&mut self) {
        if self.position.current_bit > 0 {
            self.write_bits(0u8, 8 - self.position.current_bit);
        }
    }
    pub(crate) fn new(from_data: &[u8], seek_end: bool) -> Self {
        ByteIO {
            data: from_data.to_vec(),
            position: BytePosition {
                current_byte: if seek_end { from_data.len() } else { 0 },
                current_bit: 0,
            },
        }
    }
    pub(crate) fn concat(&mut self, other: &ByteIO) {
        self.align_writer();
        self.data.extend_from_slice(&other.data);
        self.position.current_byte += other.position.current_byte;
        self.position.current_bit = other.position.current_bit;
    }
    pub(crate) fn concat_unaligned(&mut self, other: &ByteIO) {
        for i in 0..other.position.current_byte {
            self.write_bits(other.data[i], 8);
        }
        self.write_bits(other.data[other.position.current_byte], other.position.current_bit);
    }
    /// Write bits_count number of bits (LSB ordering) from bits_source into a vector of bytes.
    pub(crate) fn write_bits_by_byte(&mut self, bits_source: u8, bits_count: usize) {
        let mut bits_left_to_write: usize = bits_count;
        let mut bit_index = 0;
        loop {
            if bits_left_to_write == 0 {
                return;
            }
            if self.data.len() == self.position.current_byte {
                self.data.push(0);
            }

            if self.position.current_bit == 8 {
                self.data.push(0);
                self.position.current_byte += 1;
                self.position.current_bit = 0;
            }

            let bits_can_write_in_byte =
                cmp::min(bits_left_to_write, 8 - self.position.current_bit);

            if bits_can_write_in_byte == 8 {
                // Special case because the bit library seems to fail when trying to set an entire byte using set_bit_range
                // e.g 0x00.set_bit_range(0..8, 0xFF)
                self.data[self.position.current_byte] = bits_source;
            } else {
                self.data[self.position.current_byte].set_bit_range(
                    self.position.current_bit..(self.position.current_bit + bits_can_write_in_byte),
                    bits_source.bit_range(bit_index..(bit_index + bits_can_write_in_byte)),
                );
                bit_index += bits_can_write_in_byte;
            }
            self.position.current_bit += bits_can_write_in_byte;
            bits_left_to_write -= bits_can_write_in_byte;
        }
    }
    /// Write bits_count number of bits from bits_source into a vector of bytes..
    /// This is a wrapper around write_bits_by_byte to easily write ie u32
    pub(crate) fn write_bits<T: Into<u32>>(&mut self, bits_source: T, bits_count: usize) {
        let mut bits_left_to_write: usize = bits_count;

        let byte_source_converted: u32 = bits_source.into();
        let byte_source = byte_source_converted.to_le_bytes();
        let mut byte_source_current = 0;
        loop {
            if bits_left_to_write == 0 {
                return;
            }
            let bits_can_write = cmp::min(bits_left_to_write, 8);
            self.write_bits_by_byte(byte_source[byte_source_current], bits_can_write);
            bits_left_to_write -= bits_can_write;
            byte_source_current += 1;
        }
    }

    pub(crate) fn write_bit(&mut self, value: bool) {
        self.write_bits(u8::from(value), 1);
    }

    /// Read a number of bits in a vector of bytes, starting at a given byte and bit index, and return a u32 with the value read.
    pub(crate) fn read_bits(&mut self, num: usize) -> Result<u32, D2SError> {
        let mut bits_left_to_read: usize = num;
        let mut buffer: u32 = 0;
        let mut buffer_bit_position: usize = 0;
        loop {
            if bits_left_to_read == 0 {
                break;
            }
            if self.position.current_bit > 7 {
                self.position.current_byte += 1;
                self.position.current_bit = 0;
            }
            if self.position.current_byte >= self.data.len() {
                return Err(D2SError::FileCutOff(FileCutOffError { reader: self.clone() }));
            }
            let bits_parsing_count = cmp::min(8 - self.position.current_bit, bits_left_to_read);
            let bits_parsed: u8 = self.data[self.position.current_byte].bit_range(
                self.position.current_bit..(self.position.current_bit + bits_parsing_count),
            );

            buffer.set_bit_range(
                buffer_bit_position..(buffer_bit_position + bits_parsing_count),
                u32::from_le_bytes([bits_parsed, 0x00, 0x00, 0x00]),
            );
            buffer_bit_position += bits_parsing_count;
            bits_left_to_read -= bits_parsing_count;
            self.position.current_bit += bits_parsing_count;
        }
        Ok(buffer)
    }

    pub(crate) fn read_bit(&mut self) -> Result<bool, D2SError> {
        Ok(self.read_bits(1)?.bit(0))
    }
}
