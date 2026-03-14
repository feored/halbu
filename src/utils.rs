use std::cmp;
use std::time::SystemTime;

use crate::ParseHardError;
use bit::BitIndex;

const BITS_PER_BYTE: usize = 8;
const MAX_U32_BIT_WIDTH: usize = 32;

pub fn get_sys_time_in_secs() -> u32 {
    let seconds_since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    cmp::min(seconds_since_epoch, u32::MAX as u64) as u32
}

fn parse_fixed_array<const ARRAY_LENGTH: usize>(
    slice: &[u8],
    field_name: &str,
) -> Result<[u8; ARRAY_LENGTH], ParseHardError> {
    slice.try_into().map_err(|_| ParseHardError {
        message: format!(
            "Expected {ARRAY_LENGTH} bytes for {field_name}, found {} bytes.",
            slice.len()
        ),
    })
}

fn normalize_position(byte_position: &mut BytePosition) -> Result<(), ParseHardError> {
    if byte_position.current_bit < BITS_PER_BYTE {
        return Ok(());
    }

    let extra_full_bytes = byte_position.current_bit / BITS_PER_BYTE;
    byte_position.current_byte =
        byte_position.current_byte.checked_add(extra_full_bytes).ok_or_else(|| ParseHardError {
            message: "Byte position overflow while normalizing cursor.".to_string(),
        })?;
    byte_position.current_bit %= BITS_PER_BYTE;
    Ok(())
}

fn absolute_bit_offset(byte_position: &BytePosition) -> Result<usize, ParseHardError> {
    byte_position
        .current_byte
        .checked_mul(BITS_PER_BYTE)
        .and_then(|offset| offset.checked_add(byte_position.current_bit))
        .ok_or_else(|| ParseHardError {
            message: "Bit offset overflow while computing cursor position.".to_string(),
        })
}

fn validate_u32_bit_width(bits_count: usize, context: &str) -> Result<(), ParseHardError> {
    if bits_count > MAX_U32_BIT_WIDTH {
        return Err(ParseHardError {
            message: format!(
                "{context} supports at most {MAX_U32_BIT_WIDTH} bits, received {bits_count}."
            ),
        });
    }
    Ok(())
}

fn advance_bits(byte_position: &mut BytePosition, bits_count: usize) -> Result<(), ParseHardError> {
    let combined_bits = byte_position.current_bit.checked_add(bits_count).ok_or_else(|| {
        ParseHardError { message: "Bit offset overflow while advancing cursor.".to_string() }
    })?;

    let full_bytes = combined_bits / BITS_PER_BYTE;
    byte_position.current_byte =
        byte_position.current_byte.checked_add(full_bytes).ok_or_else(|| ParseHardError {
            message: "Byte offset overflow while advancing cursor.".to_string(),
        })?;
    byte_position.current_bit = combined_bits % BITS_PER_BYTE;
    Ok(())
}

fn ensure_bits_available(
    byte_slice: &[u8],
    byte_position: &BytePosition,
    bits_needed: usize,
    context: &str,
) -> Result<(), ParseHardError> {
    let total_bits_available =
        byte_slice.len().checked_mul(BITS_PER_BYTE).ok_or_else(|| ParseHardError {
            message: "Bit capacity overflow while checking available input bits.".to_string(),
        })?;
    let current_bit_offset = absolute_bit_offset(byte_position)?;
    if current_bit_offset > total_bits_available {
        return Err(ParseHardError {
            message: format!(
                "{context} cursor is out of bounds: bit offset {current_bit_offset}, total bits {total_bits_available}."
            ),
        });
    }
    let remaining_bits = total_bits_available - current_bit_offset;
    if bits_needed > remaining_bits {
        return Err(ParseHardError {
            message: format!(
                "{context} needs {bits_needed} bits but only {remaining_bits} remain from bit offset {current_bit_offset}."
            ),
        });
    }
    Ok(())
}

fn ensure_writable_byte(
    byte_vector: &mut Vec<u8>,
    byte_index: usize,
) -> Result<(), ParseHardError> {
    if byte_index < byte_vector.len() {
        return Ok(());
    }

    let required_length = byte_index.checked_add(1).ok_or_else(|| ParseHardError {
        message: "Byte vector length overflow while expanding output buffer.".to_string(),
    })?;
    byte_vector.resize(required_length, 0);
    Ok(())
}

pub fn u32_from(slice: &[u8], name: &'static str) -> Result<u32, ParseHardError> {
    let parsed_bytes = parse_fixed_array::<4>(slice, name)?;
    Ok(u32::from_le_bytes(parsed_bytes))
}

pub fn u16_from(slice: &[u8], name: &'static str) -> Result<u16, ParseHardError> {
    let parsed_bytes = parse_fixed_array::<2>(slice, name)?;
    Ok(u16::from_le_bytes(parsed_bytes))
}

pub fn u8_from(slice: &[u8], name: &'static str) -> Result<u8, ParseHardError> {
    slice.first().copied().ok_or_else(|| ParseHardError {
        message: format!("Expected 1 byte for {name}, found 0 bytes."),
    })
}

#[derive(Default, PartialEq, Eq, Debug)]
pub struct BytePosition {
    pub current_byte: usize,
    pub current_bit: usize,
}

/// Write `bits_count` bits (LSB-first) from `bits_source` into `byte_vector`.
pub fn write_byte(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: u8,
    bits_count: usize,
) -> Result<(), ParseHardError> {
    if bits_count > BITS_PER_BYTE {
        return Err(ParseHardError {
            message: format!(
                "write_byte supports at most {BITS_PER_BYTE} bits, received {bits_count}."
            ),
        });
    }
    if bits_count == 0 {
        return Ok(());
    }
    if bits_count < BITS_PER_BYTE && (bits_source >> bits_count) != 0 {
        return Err(ParseHardError {
            message: format!(
                "write_byte source {bits_source:#010b} does not fit in {bits_count} bits."
            ),
        });
    }

    normalize_position(byte_position)?;
    let mut bits_left_to_write: usize = bits_count;
    let mut source_bit_index = 0;
    loop {
        if bits_left_to_write == 0 {
            return Ok(());
        }

        normalize_position(byte_position)?;
        ensure_writable_byte(byte_vector, byte_position.current_byte)?;

        let bits_can_write_in_byte =
            cmp::min(bits_left_to_write, BITS_PER_BYTE - byte_position.current_bit);
        let bits_from_source =
            bits_source.bit_range(source_bit_index..(source_bit_index + bits_can_write_in_byte));

        if bits_can_write_in_byte == BITS_PER_BYTE && byte_position.current_bit == 0 {
            byte_vector[byte_position.current_byte] = bits_from_source;
        } else {
            byte_vector[byte_position.current_byte].set_bit_range(
                byte_position.current_bit..(byte_position.current_bit + bits_can_write_in_byte),
                bits_from_source,
            );
        }
        source_bit_index += bits_can_write_in_byte;
        advance_bits(byte_position, bits_can_write_in_byte)?;
        bits_left_to_write -= bits_can_write_in_byte;
    }
}

/// Write `bits_count` bits (LSB-first) from `bits_source` into `byte_vector`.
pub fn write_bits<T: Into<u32>>(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: T,
    bits_count: usize,
) -> Result<(), ParseHardError> {
    validate_u32_bit_width(bits_count, "write_bits")?;
    if bits_count == 0 {
        return Ok(());
    }

    let source_value = bits_source.into();
    if bits_count < MAX_U32_BIT_WIDTH && (source_value >> bits_count) != 0 {
        return Err(ParseHardError {
            message: format!("Value {source_value} does not fit in {bits_count} bits."),
        });
    }

    let mut bits_left_to_write: usize = bits_count;
    let mut bits_written = 0;
    loop {
        if bits_left_to_write == 0 {
            return Ok(());
        }

        let bits_can_write = cmp::min(bits_left_to_write, BITS_PER_BYTE);
        let source_byte = ((source_value >> bits_written) & 0xFF) as u8;
        write_byte(byte_vector, byte_position, source_byte, bits_can_write)?;
        bits_left_to_write -= bits_can_write;
        bits_written += bits_can_write;
    }
}

/// Read `bits_to_read` bits (LSB-first) from `byte_slice` at `byte_position`.
///
/// This supports packed bitstreams with non-byte-aligned fields.
pub fn read_bits(
    byte_slice: &[u8],
    byte_position: &mut BytePosition,
    bits_to_read: usize,
) -> Result<u32, ParseHardError> {
    validate_u32_bit_width(bits_to_read, "read_bits")?;
    if bits_to_read == 0 {
        return Ok(0);
    }

    normalize_position(byte_position)?;
    ensure_bits_available(byte_slice, byte_position, bits_to_read, "read_bits")?;

    let mut bits_left_to_read: usize = bits_to_read;
    let mut buffer: u32 = 0;
    let mut buffer_bit_position: usize = 0;
    loop {
        if bits_left_to_read == 0 {
            return Ok(buffer);
        }

        let bits_parsing_count =
            cmp::min(BITS_PER_BYTE - byte_position.current_bit, bits_left_to_read);
        let bits_parsed: u8 = byte_slice[byte_position.current_byte]
            .bit_range(byte_position.current_bit..(byte_position.current_bit + bits_parsing_count));

        buffer.set_bit_range(
            buffer_bit_position..(buffer_bit_position + bits_parsing_count),
            u32::from_le_bytes([bits_parsed, 0x00, 0x00, 0x00]),
        );
        buffer_bit_position += bits_parsing_count;
        bits_left_to_read -= bits_parsing_count;
        advance_bits(byte_position, bits_parsing_count)?;
    }
}
