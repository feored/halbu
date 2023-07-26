use crate::ParseError;

const SECTION_HEADER: [u8; 4] = [0x01, 0x77, 0x34, 0x00];

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Placeholder {
    data: [u8; 52],
}

impl Default for Placeholder {
    fn default() -> Self {
        let mut placeholder = Placeholder { data: [0x00; 52] };
        placeholder.data[0..4].copy_from_slice(&SECTION_HEADER);
        placeholder
    }
}

pub fn parse(bytes: &[u8; 52]) -> Result<Placeholder, ParseError> {
    if bytes[0..4] != SECTION_HEADER{
        return Err(ParseError{message: format!("Found wrong header for NPC section, expected {0:X?} but found {1:X?}", SECTION_HEADER, &bytes[0..4])})
    }
    let mut placeholder: Placeholder = Placeholder { data: [0x00; 52] };
    placeholder.data.copy_from_slice(bytes);

    Ok(placeholder)
}

pub fn generate(placeholder: Placeholder) -> [u8; 52] {
    let mut bytes: [u8; 52] = [0x00; 52];
    bytes.copy_from_slice(&placeholder.data[0..52]);

    bytes
}
