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
#![allow(non_upper_case_globals)] // https://github.com/rust-lang/rust-analyzer/issues/15344

use bit::BitIndex;
use log::{debug, warn};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

use crate::bit_manipulation::ByteIO;
use std::error::Error;
use std::fmt;
use std::num;
use std::ops::Range;

use attributes::Attributes;
use character::Character;
use items::Items;
use npcs::Placeholder as NPCs;
use quests::Quests;
use skills::SkillSet;
use waypoints::Waypoints;

use crate::convert::u32_from;

pub mod attributes;
mod bit_manipulation;
pub mod character;
mod convert;
mod csv;
pub mod items;
pub mod npcs;
pub mod quests;
pub mod skills;
pub mod waypoints;

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];
const ATTRIBUTES_OFFSET: usize = 765;
const DEFAULT_VERSION: u32 = 99;

#[derive(Debug)]
pub struct FileCutOffError {
    reader: ByteIO,
}

#[derive(Debug)]
pub struct WrongHeaderError {
    section: String,
    reader: ByteIO,
    expected: Vec<u8>,
    actual: Vec<u8>,
}

#[derive(Debug)]
pub struct CustomError {
    message: String,
}

impl fmt::Display for FileCutOffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut cur_byte = self.reader.position.current_byte;
        let cur_bit = match self.reader.position.current_bit {
            8 => {
                cur_byte += 1;
                0
            }
            any => any,
        };
        write!(
            f,
            "Error: Tried to read at byte:{0} bit:{1}, but byte array size \
            is: {2}.\n Array: {3:X?}",
            cur_byte,
            cur_bit,
            self.reader.data.len(),
            self.reader.data,
        )
    }
}

impl fmt::Display for WrongHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut cur_byte = self.reader.position.current_byte;
        let cur_bit = match self.reader.position.current_bit {
            8 => {
                cur_byte += 1;
                0
            }
            any => any,
        };
        write!(
            f,
            "Error: In section {5}, Expected to read header {0:X?} at byte: {1} bit:{2}, but found {3:X?}.\
            \n Array: {4:X?}",
            self.expected,
            cur_byte,
            cur_bit,
            self.actual,
            self.reader.data,
            self.section
        )
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}", self.message)
    }
}

#[derive(Debug)]
pub enum D2SError {
    Parse(num::ParseIntError),
    FileCutOff(FileCutOffError),
    WrongHeader(WrongHeaderError),
    Custom(CustomError),
}

impl fmt::Display for D2SError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            D2SError::Parse(ref err) => err.fmt(f),
            D2SError::FileCutOff(ref err) => err.fmt(f),
            D2SError::WrongHeader(ref err) => err.fmt(f),
            D2SError::Custom(ref err) => err.fmt(f),
        }
    }
}

impl Error for D2SError {}

impl From<WrongHeaderError> for D2SError {
    fn from(value: WrongHeaderError) -> Self {
        D2SError::WrongHeader(value)
    }
}

impl From<FileCutOffError> for D2SError {
    fn from(value: FileCutOffError) -> Self {
        D2SError::FileCutOff(value)
    }
}

impl From<num::ParseIntError> for D2SError {
    fn from(err: num::ParseIntError) -> D2SError {
        D2SError::Parse(err)
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
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
            Section::Version => 4..8,
            Section::FileSize => 8..12,
            Section::Checksum => 12..16,
            Section::Character => 16..335,
            Section::Quests => 335..633,
            Section::Waypoints => 633..713,
            Section::Npcs => 713..765,
        }
    }

    const fn size(self) -> usize {
        self.range().end - self.range().start
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Save {
    pub version: u32,
    pub character: Character,
    pub quests: Quests,
    pub waypoints: Waypoints,
    pub npcs: npcs::Placeholder,
    pub attributes: Attributes,
    pub skills: SkillSet,
    pub items: Items,
}

impl Default for Save {
    fn default() -> Self {
        Save {
            version: DEFAULT_VERSION,
            character: Character::default(),
            quests: Quests::default(),
            waypoints: Waypoints::default(),
            npcs: NPCs::default(),
            attributes: Attributes::default(),
            skills: SkillSet::default(),
            items: Items::default(),
        }
    }
}

impl fmt::Display for Save {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_string = format!("Save:\nVersion: {0}\n", self.version);
        final_string.push_str(&format!("Character:\n{0}\n", self.character));
        final_string.push_str(&format!("Quests:\n{0}\n", self.quests));
        final_string.push_str(&format!("Waypoints:\n{0}\n", self.waypoints));
        //final_string.push_str(&format!("NPCs:\n {0:?}\n", self.npcs));
        final_string.push_str(&format!("Attributes:\n{0}\n", self.attributes));
        final_string.push_str(&format!("Skills:\n{0}\n", self.skills));
        final_string.push_str(&format!("Items:\n {0}\n", self.items));
        write!(f, "{0}", final_string)
    }
}

impl Save {
    fn section_readable(byte_vector: &Vec<u8>, section: Section) -> bool {
        if (&byte_vector.len() - section.range().start) < section.size() {
            warn!(
                "File was cut off early, cannot read section {0:?} (Expected: {1} bytes in section, Found: {2})",
                section,
                section.size(),
                &byte_vector.len() - section.range().start
            );
            false
        } else {
            true
        }
    }
    pub fn parse(byte_vector: &Vec<u8>) -> Self {
        // (Save, Vec<ParseError>) {
        let mut save: Save = Save::default();

        debug!("Started parsing file of length {0}", byte_vector.len());

        if byte_vector.len() < (765 + 32 + 16) {
            // inferior to size of header + skills + minimum attributes, can't be valid
            warn!(
                "File is smaller than {0} bytes, the fixed size of the header + skills + attributes section. Parts of it will be replaced with default data.",
                765+32+16
            );
        }

        if !Save::section_readable(&byte_vector, Section::Signature) {
            return save;
        }
        if byte_vector[Section::Signature.range()] != SIGNATURE {
            warn!(
                "File signature should be {:0X?} but is {1:X?}",
                SIGNATURE,
                &byte_vector[Section::Signature.range()]
            );
        }

        if !Save::section_readable(&byte_vector, Section::Version) {
            return save;
        }
        save.version = u32_from(&byte_vector[Section::Version.range()], "save.version").into();
        if !Save::section_readable(&byte_vector, Section::Character) {
            return save;
        }
        save.character = Character::parse(&byte_vector[Section::Character.range()]);

        if !Save::section_readable(&byte_vector, Section::Quests) {
            return save;
        }
        save.quests = Quests::parse(&byte_vector[Section::Quests.range()]);

        if !Save::section_readable(&byte_vector, Section::Waypoints) {
            return save;
        }
        save.waypoints = Waypoints::parse(&byte_vector[Section::Waypoints.range()]);

        if !Save::section_readable(&byte_vector, Section::Npcs) {
            return save;
        }
        save.npcs = NPCs::parse(&byte_vector[Section::Npcs.range()]);

        let mut reader: ByteIO =
            ByteIO::new(&byte_vector[ATTRIBUTES_OFFSET..byte_vector.len()], false);
        save.attributes = Attributes::parse(&mut reader);
        let skills_offset = ATTRIBUTES_OFFSET + reader.position.current_byte + 1;
        save.skills = SkillSet::parse(
            &byte_vector[skills_offset..(skills_offset + 32)],
            save.character.class,
        );
        let items_offset = skills_offset + 32;

        save.items = Items::parse(
            &byte_vector[items_offset..byte_vector.len()],
            save.character.status.expansion,
            save.character.mercenary.is_hired(),
        );
        save
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        result.resize(765, 0x00);

        result[Section::Signature.range()].copy_from_slice(&SIGNATURE);
        result[Section::Version.range()]
            .copy_from_slice(&u32::to_le_bytes(u32::from(self.version)));
        result[Section::Character.range()].copy_from_slice(&self.character.to_bytes());
        result[Section::Quests.range()].copy_from_slice(&self.quests.to_bytes());
        result[Section::Waypoints.range()].copy_from_slice(&self.waypoints.to_bytes());
        result[Section::Npcs.range()].copy_from_slice(&self.npcs.to_bytes());
        result.append(&mut self.attributes.to_bytes());
        result.append(&mut self.skills.to_bytes());
        result.append(
            &mut self
                .items
                .to_bytes(self.character.status.expansion, self.character.mercenary.is_hired())
                .data,
        );

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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parsing error: {}", self.message)
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

#[derive(IntoPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Act {
    #[default]
    Act1 = 0,
    Act2 = 1,
    Act3 = 2,
    Act4 = 3,
    Act5 = 4,
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

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Class {
    #[default]
    Amazon = 0,
    Sorceress = 1,
    Necromancer = 2,
    Paladin = 3,
    Barbarian = 4,
    Druid = 5,
    Assassin = 6,
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
    // use crate::csv::*;
    use std::path::Path;
    // use std::time::Duration;
    #[test]
    fn test_parse_save() {
        let path: &Path = Path::new("assets/test/Joe.d2s");
        let save_file: Vec<u8> = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => panic!("File invalid: {e:?}"),
        };

        let _save = Save::parse(&save_file);
    }
}
