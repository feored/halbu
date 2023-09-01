#![warn(
    anonymous_parameters,
    nonstandard_style,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

use bit::BitIndex;
use serde::{Deserialize, Serialize};

use std::fmt;
use std::ops::Range;
use utils::BytePosition;

use attributes::Attributes;
use character::Character;
use quests::Quests;
use skills::SkillSet;
use waypoints::Waypoints;

pub mod attributes;
pub mod character;
pub mod items;
pub mod npcs;
pub mod quests;
pub mod skills;
pub mod utils;
pub mod waypoints;

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

const ATTRIBUTES_OFFSET: usize = 765;

#[derive(PartialEq, Eq, Debug)]
enum Section {
    Signature,
    Version,
    FileSize,
    Checksum,
    Character,
    Quests,
    Waypoints,
    Npcs, // Attributes has no fixed length, and therefore the Skills and Item sections that come after have no fixed offset
}

impl Section {
    const fn range(self) -> Range<usize> {
        match self {
            Section::Signature => 0..4,
            Section::Version => 4..4,
            Section::FileSize => 8..4,
            Section::Checksum => 12..4,
            Section::Character => 16..335,
            Section::Quests => 335..633,
            Section::Waypoints => 633..713,
            Section::Npcs => 713..765,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub struct Save {
    pub version: Version,
    pub character: Character,
    pub quests: Quests,
    pub waypoints: Waypoints,
    pub npcs: npcs::Placeholder,
    pub attributes: Attributes,
    pub skills: SkillSet,
    pub items: items::Placeholder,
}

impl fmt::Display for Save {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_string = format!("Save:\nVersion: {0}\n", self.version);
        final_string.push_str(&format!("Character:\n {0}\n", self.character));
        final_string.push_str(&format!("Quests:\n {0}\n", self.quests));
        final_string.push_str(&format!("Waypoints:\n {0}\n", self.waypoints));
        //final_string.push_str(&format!("NPCs:\n {0:?}\n", self.npcs));
        final_string.push_str(&format!("Attributes:\n {0}\n", self.attributes));
        final_string.push_str(&format!("Skills:\n {0}\n", self.skills));
        //final_string.push_str(&format!("Items:\n {0:?}\n", self.items));
        write!(f, "{0}", final_string)
    }
}

pub fn parse(byte_vector: &Vec<u8>) -> Result<Save, ParseError> {
    let mut save: Save = Save::default();

    if byte_vector.len() < (765 + 32 + 16) {
        // inferior to size of header + skills + minimum attributes, can't be valid
        return Err(ParseError {
            message: format!(
                "File is smaller than 765 bytes, the fixed size of the header. Length: {0:?}",
                byte_vector.len()
            ),
        });
    }

    if byte_vector[Section::Signature.range()] != SIGNATURE {
        return Err(ParseError {
            message: format!(
                "File signature should be {:0X?} but is {1:X?}",
                SIGNATURE,
                &byte_vector[Section::Signature.range()]
            ),
        });
    }

    save.character =
        character::parse(&byte_vector[Section::Character.range()].try_into().unwrap())?;
    save.quests = Quests::parse(&byte_vector[Section::Quests.range()])?;
    save.waypoints =
        waypoints::parse(&byte_vector[Section::Waypoints.range()].try_into().unwrap())?;
    save.npcs = npcs::parse(&byte_vector[Section::Npcs.range()].try_into().unwrap())?;

    let mut byte_position: BytePosition = BytePosition::default();
    save.attributes = attributes::parse(
        &byte_vector[ATTRIBUTES_OFFSET..byte_vector.len()].try_into().unwrap(),
        &mut byte_position,
    )?;
    let skills_offset = ATTRIBUTES_OFFSET + byte_position.current_byte + 1;
    save.skills = skills::parse(
        &byte_vector[skills_offset..(skills_offset + 32)].try_into().unwrap(),
        save.character.class,
    )?;
    let items_offset = skills_offset + 32;
    // TODO make byte_vector not mut
    save.items = items::parse(&byte_vector[items_offset..byte_vector.len()]);
    Ok(save)
}

impl Save {
    pub fn write(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        result.resize(765, 0x00);

        result[Section::Signature.range()].copy_from_slice(&SIGNATURE);
        result[Section::Version.range()]
            .copy_from_slice(&u32::to_le_bytes(u32::from(self.version)));
        result[Section::Character.range()].copy_from_slice(&self.character.write());
        result[Section::Quests.range()].copy_from_slice(&self.quests.to_bytes());
        result[Section::Waypoints.range()].copy_from_slice(&waypoints::generate(&self.waypoints));
        result[Section::Npcs.range()].copy_from_slice(&npcs::generate(self.npcs));
        result.append(&mut self.attributes.write());
        result.append(&mut self.skills.write());
        result.append(&mut items::generate(&self.items, self.character.mercenary.is_hired()));

        let length = result.len() as u32;
        result[Section::FileSize.range()].copy_from_slice(&u32::to_le_bytes(length));
        let checksum = calc_checksum(&result);
        result[Section::Checksum.range()].copy_from_slice(&i32::to_le_bytes(checksum));

        result
    }

    pub fn default_class(class: Class) -> Self {
        let default_class: Save = Save {
            attributes: Attributes::default_class(class),
            character: Character::default_class(class),
            skills: SkillSet::default_class(class),
            ..Default::default()
        };
        default_class
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLogicError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parsing error: {}", self.message)
    }
}

impl fmt::Display for GameLogicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Game logic error: {}", self.message)
    }
}

#[derive(PartialEq, Eq, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub enum Version {
    V100,
    V107,
    V108,
    V109,
    V110,
    V200R,
    V240R,
    #[default]
    V250R,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Version::V100 => write!(f, "Diablo II: Lord of Destruction Patch 1.00"),
            Version::V107 => write!(f, "Diablo II: Lord of Destruction Patch 1.07"),
            Version::V108 => write!(f, "Diablo II: Lord of Destruction Patch 1.08"),
            Version::V109 => write!(f, "Diablo II: Lord of Destruction Patch 1.09"),
            Version::V110 => write!(f, "Diablo II: Lord of Destruction Patch 1.10"),
            Version::V200R => write!(f, "Diablo II: Resurrected Patch 2.0"),
            Version::V240R => write!(f, "Diablo II: Resurrected Patch 2.4"),
            Version::V250R => write!(f, "Diablo II: Resurrected Patch 2.5"),
        }
    }
}

impl From<Version> for u32 {
    fn from(version: Version) -> u32 {
        match version {
            Version::V100 => 71,
            Version::V107 => 87,
            Version::V108 => 89,
            Version::V109 => 92,
            Version::V110 => 96,
            Version::V200R => 97,
            Version::V240R => 98,
            Version::V250R => 99,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Difficulty {
    #[default]
    Normal,
    Nightmare,
    Hell,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Normal => write!(f, "Normal"),
            Difficulty::Nightmare => write!(f, "Nightmare"),
            Difficulty::Hell => write!(f, "Hell"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Act {
    #[default]
    Act1,
    Act2,
    Act3,
    Act4,
    Act5,
}

impl fmt::Display for Act {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Act::Act1 => write!(f, "Act I"),
            Act::Act2 => write!(f, "Act II"),
            Act::Act3 => write!(f, "Act III"),
            Act::Act4 => write!(f, "Act IV"),
            Act::Act5 => write!(f, "Act V"),
        }
    }
}

impl TryFrom<u8> for Act {
    type Error = ParseError;
    fn try_from(byte: u8) -> Result<Act, ParseError> {
        let mut relevant_bits: u8 = 0;
        relevant_bits.set_bit_range(0..3, byte.bit_range(0..3));
        match relevant_bits {
            0x00 => Ok(Act::Act1),
            0x01 => Ok(Act::Act2),
            0x02 => Ok(Act::Act3),
            0x03 => Ok(Act::Act4),
            0x04 => Ok(Act::Act5),
            _ => Err(ParseError { message: format!("Found invalid act: {0:?}.", byte) }),
        }
    }
}

impl From<Act> for u8 {
    fn from(act: Act) -> u8 {
        match act {
            Act::Act1 => 0x00,
            Act::Act2 => 0x01,
            Act::Act3 => 0x02,
            Act::Act4 => 0x03,
            Act::Act5 => 0x04,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Class {
    Amazon,
    Sorceress,
    Necromancer,
    Paladin,
    Barbarian,
    Druid,
    Assassin,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class: &'static str = match self {
            Class::Amazon => "Amazon",
            Class::Sorceress => "Sorceress",
            Class::Necromancer => "Necromancer",
            Class::Paladin => "Paladin",
            Class::Barbarian => "Barbarian",
            Class::Druid => "Druid",
            Class::Assassin => "Assassin",
        };
        write!(f, "{0}", class)
    }
}

impl TryFrom<u8> for Class {
    type Error = ParseError;
    fn try_from(byte: u8) -> Result<Class, ParseError> {
        match byte {
            0x00 => Ok(Class::Amazon),
            0x01 => Ok(Class::Sorceress),
            0x02 => Ok(Class::Necromancer),
            0x03 => Ok(Class::Paladin),
            0x04 => Ok(Class::Barbarian),
            0x05 => Ok(Class::Druid),
            0x06 => Ok(Class::Assassin),
            _ => Err(ParseError { message: format!("Found invalid class: {0:?}.", byte) }),
        }
    }
}

impl From<Class> for u8 {
    fn from(class: Class) -> u8 {
        match class {
            Class::Amazon => 0x00,
            Class::Sorceress => 0x01,
            Class::Necromancer => 0x02,
            Class::Paladin => 0x03,
            Class::Barbarian => 0x04,
            Class::Druid => 0x05,
            Class::Assassin => 0x06,
        }
    }
}

pub fn calc_checksum(bytes: &Vec<u8>) -> i32 {
    let mut checksum: i32 = 0;
    let range = Section::Checksum.range();
    for i in 0..bytes.len() {
        let mut ch: i32 = bytes[i] as i32;
        if i >= range.start && i < range.end {
            ch = 0;
        }
        checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;
    use std::{path::Path, time::Duration};

    #[test]
    fn test_parse_save() {
        let path: &Path = Path::new("assets/test/Joe.d2s");
        let save_file: Vec<u8> = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => panic!("File invalid: {e:?}"),
        };

        let _save = match parse(&save_file) {
            Ok(res) => res,
            Err(e) => panic!("test_parse_save failed: {e}"),
        };
    }
}
