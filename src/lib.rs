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
//! Diablo II save-file library focused on practical editing workflows.
//!
//! `halbu` parses, edits, and writes D2R save data while preserving unknown/raw bytes
//! where possible so files can roundtrip cleanly.
//!
//! Supported top-level save layouts:
//! - [`format::FormatId::V99`]
//! - [`format::FormatId::V105`]
//!
//! Parsing modes:
//! - [`Strictness::Strict`]: fail fast on invalid/truncated data
//! - [`Strictness::Lax`]: continue parsing and collect [`ParseIssue`] values
//!
//! # Quick start
//! ```rust,no_run
//! use halbu::{Save, Strictness};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let bytes = std::fs::read("Hero.d2s")?;
//!     let parsed = Save::parse(&bytes, Strictness::Strict)?;
//!     let mut save = parsed.save;
//!     let target_format = save.format_id();
//!
//!     save.character.name = "Demo".to_string();
//!     save.set_level(10);
//!
//!     let output = save.to_bytes_for(target_format)?;
//!     let reparsed = Save::parse(&output, Strictness::Strict)?;
//!
//!     Ok(())
//! }
//! ```
use bit::BitIndex;
use serde::{Deserialize, Serialize};

use std::fmt;

use attributes::Attributes;
use character::Character;
use npcs::Placeholder as NPCs;
use quests::Quests;
use skills::SkillPoints;
use waypoints::Waypoints;

/// Attributes/stat section model and bit-level serializer.
pub mod attributes;
/// Character section model and per-format codecs.
pub mod character;
/// Save-layout detection and top-level encode/decode glue.
pub mod format;
/// Item section placeholder/raw-preserving support.
pub mod items;
/// NPC section placeholder/raw-preserving support.
pub mod npcs;
/// Quest section model.
pub mod quests;
/// Skill section model and optional default-D2R name helpers.
pub mod skills;
/// Internal byte utilities shared across sections.
pub mod utils;
/// Waypoint section model.
pub mod waypoints;

const CHECKSUM_START: usize = 12;
const CHECKSUM_END: usize = 16;

use crate::format::FormatId;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct SaveMeta {
    /// Layout selected/observed for this save model.
    #[serde(default)]
    pub format: FormatId,
}

/// Severity classification for a parse issue in lax mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Warning,
    Error,
}

/// High-level category for a parse issue in lax mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueKind {
    TruncatedSection,
    InvalidSignature,
    UnsupportedVersion,
    InvalidValue,
    InconsistentLayout,
    Other,
}

/// Detailed parse diagnostic emitted in lax mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseIssue {
    /// Error/warning level.
    pub severity: IssueSeverity,
    /// Broad issue category.
    pub kind: IssueKind,
    /// Section name when known (`"character"`, `"attributes"`, ...).
    pub section: Option<String>,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Byte offset when known.
    pub offset: Option<usize>,
    /// Expected byte count/value when known.
    pub expected: Option<usize>,
    /// Observed byte count/value when known.
    pub found: Option<usize>,
}

/// Hard parse error returned when strict validation fails.
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
    /// Parsed save model (possibly partial in lax mode).
    pub save: Save,
    /// Non-fatal issues collected during parsing in lax mode.
    pub issues: Vec<ParseIssue>,
}

/// Controls parse behavior when malformed data is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Strictness {
    /// Return an error on the first hard validation failure.
    Strict,
    /// Continue parsing when possible and accumulate [`ParseIssue`]s.
    #[default]
    Lax,
}

/// Full in-memory save model.
///
/// Unknown payloads for currently unmodeled sections are preserved in placeholder structs.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Save {
    /// Numeric version stored in the file header.
    pub version: u32,
    /// Character section.
    pub character: Character,
    /// Quests section.
    pub quests: Quests,
    /// Waypoints section.
    pub waypoints: Waypoints,
    /// NPC section (placeholder model).
    pub npcs: npcs::Placeholder,
    /// Attributes section.
    pub attributes: Attributes,
    /// Skills section.
    pub skills: SkillPoints,
    /// Items section (placeholder model).
    pub items: items::Placeholder,
    /// Auxiliary metadata kept by this library.
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
    /// Build a new blank save for a target format/class.
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

    /// Set output format and synchronize the numeric version field.
    pub fn set_format(&mut self, format: FormatId) {
        self.set_format_id(format);
    }

    pub fn format_id(&self) -> FormatId {
        self.meta.format
    }

    /// Set output format and synchronize the numeric version field.
    pub fn set_format_id(&mut self, format: FormatId) {
        self.meta.format = format;
        self.version = format.version();
    }

    /// Set both character level fields kept in separate sections.
    ///
    /// This updates:
    /// - `character.level`
    /// - `attributes.level.value`
    pub fn set_level(&mut self, level: u8) {
        self.character.level = level;
        self.attributes.level.value = level as u32;
    }

    /// Parse a save with explicit strictness.
    pub fn parse(byte_slice: &[u8], strictness: Strictness) -> Result<ParsedSave, ParseHardError> {
        format::decode_with_strictness(byte_slice, strictness)
    }

    /// Parse a save in lax mode.
    pub fn parse_lax(byte_slice: &[u8]) -> Result<ParsedSave, ParseHardError> {
        Self::parse(byte_slice, Strictness::Lax)
    }

    /// Encode using the explicit `version` field when recognized, otherwise `meta.format`.
    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let explicit_format = FormatId::from_version(self.version);
        let target_format = explicit_format.unwrap_or(self.meta.format);
        self.to_bytes_for(target_format)
    }

    /// Encode to a specific output format.
    pub fn to_bytes_for(&self, format: FormatId) -> Result<Vec<u8>, EncodeError> {
        format::encode(self, format)
    }
}

/// Save encoding error.
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

/// In-game difficulty.
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

/// In-game act.
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

/// Character class id used by the save format.
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
    /// Convert raw class id from save bytes.
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

    /// Convert class to raw id used in save bytes.
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

/// Compute save checksum using D2 save algorithm.
pub fn calc_checksum(bytes: &[u8]) -> i32 {
    let mut checksum: i32 = 0;
    for (i, byte) in bytes.iter().enumerate() {
        let mut ch = *byte as i32;
        if (CHECKSUM_START..CHECKSUM_END).contains(&i) {
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

    #[test]
    fn test_set_level_syncs_character_and_attributes() {
        let mut save = Save::default();
        save.set_level(75);

        assert_eq!(save.character.level, 75);
        assert_eq!(save.attributes.level.value, 75);
    }
}
