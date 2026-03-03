use std::ops::Range;

use crate::ParseHardError;

pub(crate) fn read_range<'a>(
    bytes: &'a [u8],
    range: Range<usize>,
    field_name: &str,
) -> Result<&'a [u8], ParseHardError> {
    bytes.get(range.clone()).ok_or_else(|| ParseHardError {
        message: format!(
            "Character field {field_name} is out of bounds: requested {}..{}, available length {}.",
            range.start,
            range.end,
            bytes.len()
        ),
    })
}

pub(crate) fn write_range<'a>(
    bytes: &'a mut [u8],
    range: Range<usize>,
    field_name: &str,
) -> Result<&'a mut [u8], ParseHardError> {
    let start = range.start;
    let end = range.end;
    let bytes_length = bytes.len();
    bytes.get_mut(start..end).ok_or_else(|| ParseHardError {
        message: format!(
            "Character field {field_name} is out of bounds for write: requested {start}..{end}, buffer length {bytes_length}."
        ),
    })
}

pub(crate) fn read_u8_at(
    bytes: &[u8],
    offset: usize,
    field_name: &str,
) -> Result<u8, ParseHardError> {
    bytes.get(offset).copied().ok_or_else(|| ParseHardError {
        message: format!(
            "Character field {field_name} is out of bounds: requested byte {offset}, available length {}.",
            bytes.len()
        ),
    })
}

pub(crate) fn write_u8_at(
    bytes: &mut [u8],
    offset: usize,
    value: u8,
    field_name: &str,
) -> Result<(), ParseHardError> {
    let bytes_length = bytes.len();
    let target_byte = bytes.get_mut(offset).ok_or_else(|| ParseHardError {
        message: format!(
            "Character field {field_name} is out of bounds for write: requested byte {offset}, buffer length {}.",
            bytes_length
        ),
    })?;
    *target_byte = value;
    Ok(())
}

pub(crate) fn read_u32_le_at(
    bytes: &[u8],
    offset: usize,
    field_name: &str,
) -> Result<u32, ParseHardError> {
    let slice = read_range(bytes, offset..(offset + 4), field_name)?;
    let parsed_bytes: [u8; 4] = slice.try_into().map_err(|_| ParseHardError {
        message: format!("Failed to read 4-byte value for {field_name}."),
    })?;
    Ok(u32::from_le_bytes(parsed_bytes))
}

pub(crate) fn write_u32_le_at(
    bytes: &mut [u8],
    offset: usize,
    value: u32,
    field_name: &str,
) -> Result<(), ParseHardError> {
    let target_slice = write_range(bytes, offset..(offset + 4), field_name)?;
    target_slice.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub(crate) fn read_fixed_array<const ARRAY_LENGTH: usize>(
    bytes: &[u8],
    range: Range<usize>,
    field_name: &str,
) -> Result<[u8; ARRAY_LENGTH], ParseHardError> {
    let source_slice = read_range(bytes, range, field_name)?;
    source_slice.try_into().map_err(|_| ParseHardError {
        message: format!(
            "Character field {field_name} has invalid length: expected {ARRAY_LENGTH}, found {}.",
            source_slice.len()
        ),
    })
}

pub(crate) fn write_exact_bytes(
    bytes: &mut [u8],
    range: Range<usize>,
    source_bytes: &[u8],
    field_name: &str,
) -> Result<(), ParseHardError> {
    let target_slice = write_range(bytes, range.clone(), field_name)?;
    if source_bytes.len() != target_slice.len() {
        return Err(ParseHardError {
            message: format!(
                "Character field {field_name} length mismatch: target {} bytes for range {}..{}, source {} bytes.",
                target_slice.len(),
                range.start,
                range.end,
                source_bytes.len()
            ),
        });
    }
    target_slice.copy_from_slice(source_bytes);
    Ok(())
}

pub(crate) fn read_name_string(
    bytes: &[u8],
    range: Range<usize>,
    field_name: &str,
) -> Result<String, ParseHardError> {
    let raw_name_bytes = read_range(bytes, range, field_name)?;
    let decoded_name =
        String::from_utf8_lossy(raw_name_bytes).trim_matches(char::from(0)).to_string();
    Ok(decoded_name)
}

pub(crate) fn write_name_string(
    bytes: &mut [u8],
    range: Range<usize>,
    value: &str,
    field_name: &str,
) -> Result<(), ParseHardError> {
    let target_slice = write_range(bytes, range, field_name)?;
    target_slice.fill(0x00);
    let source_bytes = value.as_bytes();
    let copied_length = usize::min(source_bytes.len(), target_slice.len());
    target_slice[..copied_length].copy_from_slice(&source_bytes[..copied_length]);
    Ok(())
}
