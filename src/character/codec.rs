use crate::character::v105::CharacterCodecV105;
use crate::character::v99::CharacterCodecV99;
use crate::character::Character;
use crate::format::FormatId;
use crate::ParseHardError;

pub trait CharacterCodec {
    const CHARACTER_LENGTH: usize;

    fn decode(character_section_bytes: &[u8]) -> Result<Character, ParseHardError>;
    fn encode(character: &Character) -> Result<Vec<u8>, ParseHardError>;
}

pub fn decode_for_format(
    format_id: FormatId,
    character_section_bytes: &[u8],
) -> Result<Character, ParseHardError> {
    match format_id {
        FormatId::V99 => CharacterCodecV99::decode(character_section_bytes),
        FormatId::V105 | FormatId::Unknown(_) => {
            CharacterCodecV105::decode(character_section_bytes)
        }
    }
}

pub fn encode_for_format(
    format_id: FormatId,
    character: &Character,
) -> Result<Vec<u8>, ParseHardError> {
    match format_id {
        FormatId::V99 => CharacterCodecV99::encode(character),
        FormatId::V105 | FormatId::Unknown(_) => CharacterCodecV105::encode(character),
    }
}

pub fn expected_length_for_format(format_id: FormatId) -> usize {
    match format_id {
        FormatId::V99 => CharacterCodecV99::CHARACTER_LENGTH,
        FormatId::V105 | FormatId::Unknown(_) => CharacterCodecV105::CHARACTER_LENGTH,
    }
}
