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

use std::fmt;

pub mod attributes;
pub mod character;
pub mod items;
pub mod npcs;
pub mod quests;
pub mod skills;
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
    section: String,
    message: String,
}

#[derive(Debug, Clone)]
pub struct GameError{
    message: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error parsing section '{}': {}",
            self.section, self.message
        )
    }
}

pub struct Save {
    version: Version,
    character: character::Character,
}

#[derive(Debug)]
pub enum HeaderID {
    Signature,
    VersionID,
    FileSize,
    Checksum,
    WeaponSet,
    Status,
    Progression,
    Class,
    Level,
    CreatedDate,
    LastPlayedDate,
    AssignedSkills,
    LeftMouse,
    RightMouse,
    LeftMouseSwitch,
    RightMouseSwitch,
    MenuAppearance, //Needed?
    Difficulty,
    MapSeed,
    Mercenary,
    ResurrectedMenuAppearance,
    Name,
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

fn get_file_bytes_range(id: HeaderID) -> (usize, usize) {
    let data: FileSection = get_file_data(id);
    (data.offset, data.offset + data.bytes)
}

fn get_file_data(id: HeaderID) -> FileSection {
    match id {
        HeaderID::Signature => FileSection {
            offset: 0,
            bytes: 4,
        },
        HeaderID::VersionID => FileSection {
            offset: 4,
            bytes: 4,
        },
        HeaderID::FileSize => FileSection {
            offset: 8,
            bytes: 4,
        },
        HeaderID::Checksum => FileSection {
            offset: 12,
            bytes: 4,
        },
        HeaderID::WeaponSet => FileSection {
            offset: 16,
            bytes: 4,
        },
        HeaderID::Status => FileSection {
            offset: 36,
            bytes: 1,
        },
        HeaderID::Progression => FileSection {
            offset: 37,
            bytes: 1,
        },
        HeaderID::Class => FileSection {
            offset: 40,
            bytes: 1,
        },
        HeaderID::Level => FileSection {
            offset: 43,
            bytes: 1,
        },
        _ => FileSection {
            offset: 0,
            bytes: 0
        }
    }
}

fn check_valid_signature(bytes: &Vec<u8>) -> bool {
    let (header_start, header_end) = get_file_bytes_range(HeaderID::Signature);
    bytes[header_start..header_end] == SIGNATURE
}

pub fn calc_checksum(bytes: &Vec<u8>) -> i32 {
    let mut checksum: i32 = 0;
    let (checksum_start, checksum_end) = get_file_bytes_range(HeaderID::Checksum);
    for i in 0..bytes.len() {
        let mut ch: i32 = bytes[i] as i32;
        if i >= checksum_start && i < checksum_end {
            ch = 0;
        }
        checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
    }
    checksum
}
