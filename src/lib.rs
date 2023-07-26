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

use std::ops::Range;
use bit::BitIndex;
use std::fmt;
use utils::FileSection;
use utils::BytePosition;

use character::Character as Character;
use quests::Quests as Quests;
use waypoints::Waypoints as Waypoints;
use attributes::Attributes as Attributes;
use skills::SkillSet as SkillSet;

pub mod attributes;
pub mod character;
pub mod items;
pub mod npcs;
pub mod quests;
pub mod skills;
pub mod utils;
pub mod waypoints;

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

const ATTRIBUTES_OFFSET : usize = 765;

const VERSION_100: u32 = 71;
const VERSION_107: u32 = 87;
const VERSION_108: u32 = 89;
const VERSION_109: u32 = 92;
const VERSION_110: u32 = 96;
const VERSION_D2R_100: u32 = 97;
const VERSION_D2R_240: u32 = 98;
const VERSION_D2R_250: u32 = 99;

#[derive(PartialEq, Eq, Debug)]
enum Section{
    Signature,
    Version,
    FileSize,
    Checksum,
    Character,
    Quests,
    Waypoints,
    Npcs
    // Attributes has no fixed length, and therefore the Skills and Item sections that come after have no fixed offset
}

impl From<Section> for FileSection {
    fn from(section: Section) -> FileSection {
        match section {
            Section::Signature => FileSection{offset:0, bytes:4},
            Section::Version => FileSection{offset:4, bytes:4},
            Section::FileSize => FileSection{offset:8, bytes:4},
            Section::Checksum => FileSection{offset:12, bytes:4},
            Section::Character => FileSection{offset:16, bytes:319},
            Section::Quests => FileSection {offset:335, bytes:298},
            Section::Waypoints => FileSection {offset: 633, bytes: 81},
            Section::Npcs => FileSection{offset:714, bytes: 51},
        }
    }
} 

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Save {
    version: Version,
    character: Character,
    quests: Quests,
    waypoints: Waypoints,
    npcs: npcs::Placeholder,
    attributes: Attributes,
    skills: SkillSet,
    items: items::Placeholder
}


pub fn parse(byte_vector: &Vec<u8>) -> Result<Save, ParseError> {
    let mut save : Save = Save::default();
    
    if byte_vector.len() < (765 + 32 + 16) {
        // inferior to size of header + skills + minimum attributes, can't be valid
        return Err(ParseError{message:format!("File is smaller than 765 bytes, the fixed size of the header. Length: {0:?}", byte_vector.len())})
    }

    if byte_vector[Range::<usize>::from(FileSection::from(Section::Signature))] != SIGNATURE {
        return Err(ParseError{message:format!("File signature should be {:0X?} but is {1:X?}", SIGNATURE, &byte_vector[Range::<usize>::from(FileSection::from(Section::Signature))])})
    }


    save.character = character::parse(&byte_vector[Range::<usize>::from(FileSection::from(Section::Character))].try_into().unwrap())?;
    save.quests = quests::parse(&byte_vector[Range::<usize>::from(FileSection::from(Section::Quests))].try_into().unwrap())?;
    save.waypoints = waypoints::parse(&byte_vector[Range::<usize>::from(FileSection::from(Section::Waypoints))].try_into().unwrap())?;
    save.npcs = npcs::parse(&byte_vector[Range::<usize>::from(FileSection::from(Section::Npcs))].try_into().unwrap());

    let mut byte_position : BytePosition = BytePosition::default();
    save.attributes = attributes::parse_with_position(&byte_vector[ATTRIBUTES_OFFSET..byte_vector.len()].try_into().unwrap(), &mut byte_position)?;
    let skills_offset = ATTRIBUTES_OFFSET + byte_position.current_byte + 1;
    save.skills = skills::parse(&byte_vector[skills_offset..(skills_offset+32)].try_into().unwrap(), save.character.class)?;
    let items_offset = skills_offset + 32;
    // TODO make byte_vector not mut
    save.items = items::parse(&mut byte_vector[items_offset..byte_vector.len()].try_into().unwrap());
    Ok(save)
}


#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
}

#[derive(Debug, Clone)]
pub struct GameLogicError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parsing error: {}", self.message)
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
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

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum Difficulty {
    #[default]
    Normal,
    Nightmare,
    Hell,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
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
            _ => Err(ParseError {
                message: format!("Found invalid act: {0:?}.", byte),
            }),
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Class {
    Amazon,
    Sorceress,
    Necromancer,
    Paladin,
    Barbarian,
    Druid,
    Assassin,
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
            _ => Err(ParseError {
                message: format!("Found invalid class: {0:?}.", byte),
            }),
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



// fn check_valid_signature(bytes: &Vec<u8>) -> bool {
//     bytes[get_offset_range(OffsetID::Signature)] == SIGNATURE
// }

// pub fn calc_checksum(bytes: &Vec<u8>) -> i32 {
//     let mut checksum: i32 = 0;
//     let (checksum_start, checksum_end) = get_file_bytes_range(OffsetID::Checksum);
//     for i in 0..bytes.len() {
//         let mut ch: i32 = bytes[i] as i32;
//         if i >= checksum_start && i < checksum_end {
//             ch = 0;
//         }
//         checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
//     }
//     checksum
// }


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    

    #[test]
    fn test_parse_save() {
        let path: &Path = Path::new("C:/Users/feord/Saved Games/Diablo II Resurrected/Nyahallo.d2s");
        let save_file: Vec<u8> = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("File invalid: {e:?}"),
    };

    let save = match parse(&save_file){
        Ok(res) => res,
        Err(e) => panic!("PARSE TEST FAILED WITH ERROR: {e}")
    };

    println!("TEST SUCCESSFUL: {0:?}", save);
    }
}