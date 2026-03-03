use std::ops::Range;

use crate::character::codec::CharacterCodec;
use crate::character::common::{
    read_fixed_array, read_name_string, read_range, read_u32_le_at, read_u8_at, write_exact_bytes,
    write_name_string, write_u32_le_at, write_u8_at,
};
use crate::character::mercenary::Mercenary;
use crate::character::{parse_last_act, write_last_act, Character, Status};
use crate::Class;
use crate::ParseHardError;

pub struct CharacterCodecV99;

pub const OFFSET_WEAPON_SET: usize = 0;
pub const OFFSET_STATUS: usize = 20;
pub const OFFSET_PROGRESSION: usize = 21;
pub const OFFSET_CLASS: usize = 24;
pub const OFFSET_RESERVED_VERSION_MARKER_ONE: usize = 25;
pub const OFFSET_RESERVED_VERSION_MARKER_TWO: usize = 26;
pub const OFFSET_LEVEL: usize = 27;
pub const OFFSET_LAST_PLAYED: usize = 32;
pub const RANGE_RESERVED_CHECKSUM_MASK: Range<usize> = 36..40;
pub const RANGE_ASSIGNED_SKILLS: Range<usize> = 40..104;
pub const OFFSET_LEFT_MOUSE_SKILL: usize = 104;
pub const OFFSET_RIGHT_MOUSE_SKILL: usize = 108;
pub const OFFSET_LEFT_MOUSE_SWITCH_SKILL: usize = 112;
pub const OFFSET_RIGHT_MOUSE_SWITCH_SKILL: usize = 116;
pub const RANGE_MENU_APPEARANCE: Range<usize> = 120..152;
pub const RANGE_DIFFICULTY: Range<usize> = 152..155;
pub const OFFSET_MAP_SEED: usize = 155;
pub const RANGE_MERCENARY: Range<usize> = 161..175;
// Unknown legacy bytes block in v99.
pub const RANGE_UNKNOWN_REGION_ONE: Range<usize> = 175..203;
pub const RANGE_RESURRECTED_MENU_APPEARANCE: Range<usize> = 203..251;
pub const RANGE_NAME: Range<usize> = 251..299;
// Unknown trailing bytes block in v99.
pub const RANGE_UNKNOWN_REGION_TWO: Range<usize> = 299..319;

const ASSIGNED_SKILL_SLOT_COUNT: usize = 16;

impl CharacterCodec for CharacterCodecV99 {
    const CHARACTER_LENGTH: usize = 319;

    fn decode(character_section_bytes: &[u8]) -> Result<Character, ParseHardError> {
        if character_section_bytes.len() < Self::CHARACTER_LENGTH {
            return Err(ParseHardError {
                message: format!(
                    "Character section is truncated for v99: expected {} bytes, found {}.",
                    Self::CHARACTER_LENGTH,
                    character_section_bytes.len()
                ),
            });
        }

        let mut character = Character::default();
        character.raw_section = character_section_bytes[..Self::CHARACTER_LENGTH].to_vec();
        let raw_bytes = &character.raw_section;

        character.weapon_switch = read_u32_le_at(raw_bytes, OFFSET_WEAPON_SET, "weapon_set")? != 0;
        character.status = Status::from(read_u8_at(raw_bytes, OFFSET_STATUS, "status")?);
        character.progression = read_u8_at(raw_bytes, OFFSET_PROGRESSION, "progression")?;
        character.class = Class::from_id(read_u8_at(raw_bytes, OFFSET_CLASS, "class")?);
        character.level = read_u8_at(raw_bytes, OFFSET_LEVEL, "level")?;
        character.last_played = read_u32_le_at(raw_bytes, OFFSET_LAST_PLAYED, "last_played")?;

        for assigned_skill_index in 0..ASSIGNED_SKILL_SLOT_COUNT {
            let skill_offset = RANGE_ASSIGNED_SKILLS.start + (assigned_skill_index * 4);
            character.assigned_skills[assigned_skill_index] =
                read_u32_le_at(raw_bytes, skill_offset, "assigned_skill")?;
        }

        character.left_mouse_skill =
            read_u32_le_at(raw_bytes, OFFSET_LEFT_MOUSE_SKILL, "left_mouse_skill")?;
        character.right_mouse_skill =
            read_u32_le_at(raw_bytes, OFFSET_RIGHT_MOUSE_SKILL, "right_mouse_skill")?;
        character.left_mouse_switch_skill =
            read_u32_le_at(raw_bytes, OFFSET_LEFT_MOUSE_SWITCH_SKILL, "left_mouse_switch_skill")?;
        character.right_mouse_switch_skill =
            read_u32_le_at(raw_bytes, OFFSET_RIGHT_MOUSE_SWITCH_SKILL, "right_mouse_switch_skill")?;

        character.menu_appearance =
            read_fixed_array(raw_bytes, RANGE_MENU_APPEARANCE.clone(), "menu_appearance")?;
        let difficulty_bytes = read_fixed_array(raw_bytes, RANGE_DIFFICULTY.clone(), "difficulty")?;
        (character.difficulty, character.act) = parse_last_act(&difficulty_bytes)?;
        character.map_seed = read_u32_le_at(raw_bytes, OFFSET_MAP_SEED, "map_seed")?;
        character.mercenary =
            Mercenary::parse(read_range(raw_bytes, RANGE_MERCENARY.clone(), "mercenary")?)?;
        character.resurrected_menu_appearance = read_fixed_array(
            raw_bytes,
            RANGE_RESURRECTED_MENU_APPEARANCE.clone(),
            "resurrected_menu_appearance",
        )?;
        character.name = read_name_string(raw_bytes, RANGE_NAME.clone(), "name")?;

        Ok(character)
    }

    fn encode(character: &Character) -> Result<Vec<u8>, ParseHardError> {
        let mut encoded_bytes = if character.raw_section.len() == Self::CHARACTER_LENGTH {
            character.raw_section.clone()
        } else {
            vec![0u8; Self::CHARACTER_LENGTH]
        };

        write_u32_le_at(
            &mut encoded_bytes,
            OFFSET_WEAPON_SET,
            u32::from(character.weapon_switch),
            "weapon_set",
        )?;
        write_u8_at(&mut encoded_bytes, OFFSET_STATUS, u8::from(character.status), "status")?;
        write_u8_at(&mut encoded_bytes, OFFSET_PROGRESSION, character.progression, "progression")?;
        write_u8_at(&mut encoded_bytes, OFFSET_CLASS, u8::from(character.class), "class")?;
        write_u8_at(&mut encoded_bytes, OFFSET_LEVEL, character.level, "level")?;
        write_u32_le_at(
            &mut encoded_bytes,
            OFFSET_LAST_PLAYED,
            character.last_played,
            "last_played",
        )?;

        for assigned_skill_index in 0..ASSIGNED_SKILL_SLOT_COUNT {
            let skill_offset = RANGE_ASSIGNED_SKILLS.start + (assigned_skill_index * 4);
            write_u32_le_at(
                &mut encoded_bytes,
                skill_offset,
                character.assigned_skills[assigned_skill_index],
                "assigned_skill",
            )?;
        }

        write_u32_le_at(
            &mut encoded_bytes,
            OFFSET_LEFT_MOUSE_SKILL,
            character.left_mouse_skill,
            "left_mouse_skill",
        )?;
        write_u32_le_at(
            &mut encoded_bytes,
            OFFSET_RIGHT_MOUSE_SKILL,
            character.right_mouse_skill,
            "right_mouse_skill",
        )?;
        write_u32_le_at(
            &mut encoded_bytes,
            OFFSET_LEFT_MOUSE_SWITCH_SKILL,
            character.left_mouse_switch_skill,
            "left_mouse_switch_skill",
        )?;
        write_u32_le_at(
            &mut encoded_bytes,
            OFFSET_RIGHT_MOUSE_SWITCH_SKILL,
            character.right_mouse_switch_skill,
            "right_mouse_switch_skill",
        )?;

        write_exact_bytes(
            &mut encoded_bytes,
            RANGE_MENU_APPEARANCE.clone(),
            &character.menu_appearance,
            "menu_appearance",
        )?;
        write_exact_bytes(
            &mut encoded_bytes,
            RANGE_DIFFICULTY.clone(),
            &write_last_act(character.difficulty, character.act),
            "difficulty",
        )?;
        write_u32_le_at(&mut encoded_bytes, OFFSET_MAP_SEED, character.map_seed, "map_seed")?;
        write_exact_bytes(
            &mut encoded_bytes,
            RANGE_MERCENARY.clone(),
            &character.mercenary.write(),
            "mercenary",
        )?;
        write_exact_bytes(
            &mut encoded_bytes,
            RANGE_RESURRECTED_MENU_APPEARANCE.clone(),
            &character.resurrected_menu_appearance,
            "resurrected_menu_appearance",
        )?;
        write_name_string(&mut encoded_bytes, RANGE_NAME.clone(), &character.name, "name")?;

        write_u8_at(
            &mut encoded_bytes,
            OFFSET_RESERVED_VERSION_MARKER_ONE,
            0x10,
            "reserved_version_marker_one",
        )?;
        write_u8_at(
            &mut encoded_bytes,
            OFFSET_RESERVED_VERSION_MARKER_TWO,
            0x1E,
            "reserved_version_marker_two",
        )?;
        write_exact_bytes(
            &mut encoded_bytes,
            RANGE_RESERVED_CHECKSUM_MASK.clone(),
            &[0xFF, 0xFF, 0xFF, 0xFF],
            "reserved_checksum_mask",
        )?;

        Ok(encoded_bytes)
    }
}
