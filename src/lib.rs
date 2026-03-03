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

use attributes::Attributes;
use character::Character;
use npcs::Placeholder as NPCs;
use quests::Quests;
use skills::SkillPoints;
use waypoints::Waypoints;

pub mod attributes;
pub mod character;
pub mod format;
pub mod items;
pub mod npcs;
pub mod quests;
pub mod skills;
pub mod utils;
pub mod waypoints;

const CHECKSUM_START: usize = 12;
const CHECKSUM_END: usize = 16;

use crate::format::FormatId;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct SaveMeta {
    #[serde(default)]
    pub format: FormatId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueKind {
    TruncatedSection,
    InvalidSignature,
    UnsupportedVersion,
    InvalidValue,
    InconsistentLayout,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseIssue {
    pub severity: IssueSeverity,
    pub kind: IssueKind,
    pub section: Option<String>,
    pub message: String,
    pub offset: Option<usize>,
    pub expected: Option<usize>,
    pub found: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseHardError {
    pub message: String,
}

impl fmt::Display for ParseHardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse hard error: {}", self.message)
    }
}

impl std::error::Error for ParseHardError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSave {
    pub save: Save,
    pub issues: Vec<ParseIssue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Strictness {
    Strict,
    #[default]
    Lax,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Save {
    pub version: u32,
    pub character: Character,
    pub quests: Quests,
    pub waypoints: Waypoints,
    pub npcs: npcs::Placeholder,
    pub attributes: Attributes,
    pub skills: SkillPoints,
    pub items: items::Placeholder,
    #[serde(default)]
    pub meta: SaveMeta,
}

impl Default for Save {
    fn default() -> Self {
        let mut character = Character::default_class(Class::Amazon);
        character.last_played = 0;

        Save {
            version: FormatId::V99.version(),
            character,
            quests: Quests::default(),
            waypoints: Waypoints::default(),
            npcs: NPCs::default(),
            attributes: Attributes::new_save_defaults(),
            skills: SkillPoints::default(),
            items: items::Placeholder::default(),
            meta: SaveMeta { format: FormatId::V99 },
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
        //final_string.push_str(&format!("Items:\n {0:?}\n", self.items));
        write!(f, "{0}", final_string)
    }
}

impl Save {
    pub fn new(format: FormatId, class: Class) -> Save {
        let mut character = Character::default_class(class);
        character.last_played = 0;
        character.raw_section = Vec::new();

        Save {
            version: format.version(),
            character,
            quests: Quests::default(),
            waypoints: Waypoints::default(),
            npcs: NPCs::default(),
            attributes: Attributes::new_save_defaults(),
            skills: SkillPoints::default(),
            items: items::Placeholder::default(),
            meta: SaveMeta { format },
        }
    }

    pub fn format(&self) -> FormatId {
        self.meta.format
    }

    pub fn set_format(&mut self, format: FormatId) {
        self.set_format_id(format);
    }

    pub fn format_id(&self) -> FormatId {
        self.meta.format
    }

    pub fn set_format_id(&mut self, format: FormatId) {
        self.meta.format = format;
        self.version = format.version();
    }

    pub fn parse(byte_slice: &[u8], strictness: Strictness) -> Result<ParsedSave, ParseHardError> {
        format::decode_with_strictness(byte_slice, strictness)
    }

    pub fn parse_lax(byte_slice: &[u8]) -> Result<ParsedSave, ParseHardError> {
        Self::parse(byte_slice, Strictness::Lax)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let explicit_format = FormatId::from_version(self.version);
        let target_format = explicit_format.unwrap_or(self.meta.format);
        self.to_bytes_for(target_format)
    }

    pub fn to_bytes_for(&self, format: FormatId) -> Result<Vec<u8>, EncodeError> {
        format::encode(self, format)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeError {
    message: String,
}

impl EncodeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Encoding error: {}", self.message)
    }
}

impl std::error::Error for EncodeError {}

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
    type Error = ParseHardError;
    fn try_from(byte: u8) -> Result<Act, ParseHardError> {
        let mut relevant_bits: u8 = 0;
        relevant_bits.set_bit_range(0..3, byte.bit_range(0..3));
        match relevant_bits {
            0x00 => Ok(Act::Act1),
            0x01 => Ok(Act::Act2),
            0x02 => Ok(Act::Act3),
            0x03 => Ok(Act::Act4),
            0x04 => Ok(Act::Act5),
            _ => Err(ParseHardError { message: format!("Found invalid act: {0:?}.", byte) }),
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
    Warlock,
    Unknown(u8),
}

impl Class {
    pub const fn from_id(class_id: u8) -> Self {
        match class_id {
            0x00 => Class::Amazon,
            0x01 => Class::Sorceress,
            0x02 => Class::Necromancer,
            0x03 => Class::Paladin,
            0x04 => Class::Barbarian,
            0x05 => Class::Druid,
            0x06 => Class::Assassin,
            0x07 => Class::Warlock,
            _ => Class::Unknown(class_id),
        }
    }

    pub const fn id(self) -> u8 {
        match self {
            Class::Amazon => 0x00,
            Class::Sorceress => 0x01,
            Class::Necromancer => 0x02,
            Class::Paladin => 0x03,
            Class::Barbarian => 0x04,
            Class::Druid => 0x05,
            Class::Assassin => 0x06,
            Class::Warlock => 0x07,
            Class::Unknown(class_id) => class_id,
        }
    }
}

impl From<u8> for Class {
    fn from(class_id: u8) -> Self {
        Class::from_id(class_id)
    }
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
            Class::Warlock => "Warlock",
            Class::Unknown(_) => "Unknown",
        };
        match self {
            Class::Unknown(class_id) => write!(f, "{0}({1})", class, class_id),
            _ => write!(f, "{0}", class),
        }
    }
}

impl From<Class> for u8 {
    fn from(class: Class) -> u8 {
        class.id()
    }
}

pub fn calc_checksum(bytes: &Vec<u8>) -> i32 {
    let mut checksum: i32 = 0;
    for i in 0..bytes.len() {
        let mut ch: i32 = bytes[i] as i32;
        if i >= CHECKSUM_START && i < CHECKSUM_END {
            ch = 0;
        }
        checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_save() {
        let path: &Path = Path::new("assets/test/Joe.d2s");
        let save_file: Vec<u8> = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => panic!("File invalid: {e:?}"),
        };

        let _save = Save::parse_lax(&save_file).expect("save should parse");
    }
}
