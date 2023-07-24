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
use std::fmt;
use std::ops::Range;

pub mod attributes;
pub mod character;
pub mod items;
pub mod npcs;
pub mod quests;
pub mod skills;
pub mod utils;
pub mod waypoints;

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

const VERSION_100: u32 = 71;
const VERSION_107: u32 = 87;
const VERSION_108: u32 = 89;
const VERSION_109: u32 = 92;
const VERSION_110: u32 = 96;
const VERSION_D2R_100: u32 = 97;
const VERSION_D2R_240: u32 = 98;
const VERSION_D2R_250: u32 = 99;

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

pub struct Save {
    version: Version,
    character: character::Character,
}

#[derive(Debug)]
pub enum OffsetID {
    Signature,
    VersionID,
    FileSize,
    Checksum,
    WeaponSet,
    Status,
    Progression,
    Class,
    Level,
    LastPlayedDate,
    AssignedSkills,
    LeftMouseSkill,
    RightMouseSkill,
    LeftMouseSwitchSkill,
    RightMouseSwitchSkill,
    MenuAppearance,
    Difficulty,
    MapSeed,
    Mercenary,
    ResurrectedMenuAppearance,
    Name,
    Quests,
    Waypoints,
    NPCs,
    Attributes,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Version {
    V100,
    V107,
    V108,
    V109,
    V110,
    V200R,
    V240R,
    V250R,
}

#[derive(PartialEq, Eq, Debug)]
struct FileSection {
    offset: usize,
    bytes: usize,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Difficulty {
    Normal,
    Nightmare,
    Hell,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Act {
    Act1,
    Act2,
    Act3,
    Act4,
    Act5,
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

pub fn get_offset_from_position(id: OffsetID, start: usize) -> usize {
    let data: FileSection = get_file_data(id);
    if start > data.offset {
        panic!("Start is after offset!");
    }
    data.offset - start
}

pub fn get_offset_range_from_position(id: OffsetID, start: usize) -> Range<usize> {
    let data: FileSection = get_file_data(id);
    if start > data.offset {
        panic!("Start is after offset!");
    }
    (data.offset - start)..(data.offset + data.bytes - start)
}

pub fn get_offset_range(id: OffsetID) -> Range<usize> {
    get_offset_range_from_position(id, 0)
}

fn get_file_data(id: OffsetID) -> FileSection {
    match id {
        OffsetID::Signature => FileSection {
            offset: 0,
            bytes: 4,
        },
        OffsetID::VersionID => FileSection {
            offset: 4,
            bytes: 4,
        },
        OffsetID::FileSize => FileSection {
            offset: 8,
            bytes: 4,
        },
        OffsetID::Checksum => FileSection {
            offset: 12,
            bytes: 4,
        },
        OffsetID::WeaponSet => FileSection {
            offset: 16,
            bytes: 4,
        },
        OffsetID::Status => FileSection {
            offset: 36,
            bytes: 1,
        },
        OffsetID::Progression => FileSection {
            offset: 37,
            bytes: 1,
        },
        OffsetID::Class => FileSection {
            offset: 40,
            bytes: 1,
        },
        OffsetID::Level => FileSection {
            offset: 43,
            bytes: 1,
        },
        OffsetID::LastPlayedDate => FileSection {
            offset: 48,
            bytes: 4,
        },
        OffsetID::AssignedSkills => FileSection {
            offset: 56,
            bytes: 64,
        },
        OffsetID::LeftMouseSkill => FileSection {
            offset: 120,
            bytes: 4,
        },
        OffsetID::RightMouseSkill => FileSection {
            offset: 124,
            bytes: 4,
        },
        OffsetID::LeftMouseSwitchSkill => FileSection {
            offset: 128,
            bytes: 4,
        },
        OffsetID::RightMouseSwitchSkill => FileSection {
            offset: 132,
            bytes: 4,
        },
        OffsetID::MenuAppearance => FileSection {
            offset: 136,
            bytes: 32,
        },
        OffsetID::Difficulty => FileSection {
            offset: 168,
            bytes: 3,
        },
        OffsetID::MapSeed => FileSection {
            offset: 171,
            bytes: 4,
        },
        OffsetID::Mercenary => FileSection {
            offset: 177,
            bytes: 14,
        },
        OffsetID::ResurrectedMenuAppearance => FileSection {
            offset: 249,
            bytes: 48,
        },
        OffsetID::Name => FileSection {
            offset: 267,
            bytes: 16,
        },
        OffsetID::Quests => FileSection {
            offset: 335,
            bytes: 298,
        },
        OffsetID::Waypoints => FileSection {
            offset: 633,
            bytes: 80,
        },
        OffsetID::NPCs => FileSection {
            offset: 713,
            bytes: 52,
        },
        OffsetID::Attributes => FileSection {
            offset: 765,
            bytes: 0,
        },
    }
}

fn check_valid_signature(bytes: &Vec<u8>) -> bool {
    bytes[get_offset_range(OffsetID::Signature)] == SIGNATURE
}

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
