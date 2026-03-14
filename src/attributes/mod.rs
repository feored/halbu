//! Attributes/stat section model.
//!
//! HP/mana/stamina values are stored in save bytes using fixed-point Q8.
//! Use [`AttributeId`] with [`Attributes::stat`] for typed lookup.
//! Use the HP/mana/stamina accessors on [`Attributes`] for game-visible units.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::utils::read_bits;
use crate::utils::write_bits;
use crate::utils::BytePosition;
use crate::ParseHardError;

mod tests;

const SECTION_HEADER: [u8; 2] = [0x67, 0x66];
const SECTION_TRAILER: u32 = 0x1FF;
const STAT_HEADER_LENGTH: usize = 9;
const STAT_NUMBER: usize = 16;

/// Typed identifier for supported attributes/stats in the save format.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AttributeId {
    Strength,
    Energy,
    Dexterity,
    Vitality,
    StatPoints,
    NewSkills,
    Hitpoints,
    MaxHp,
    Mana,
    MaxMana,
    Stamina,
    MaxStamina,
    Level,
    Experience,
    Gold,
    GoldBank,
}

impl AttributeId {
    pub const ALL: [Self; STAT_NUMBER] = [
        Self::Strength,
        Self::Energy,
        Self::Dexterity,
        Self::Vitality,
        Self::StatPoints,
        Self::NewSkills,
        Self::Hitpoints,
        Self::MaxHp,
        Self::Mana,
        Self::MaxMana,
        Self::Stamina,
        Self::MaxStamina,
        Self::Level,
        Self::Experience,
        Self::Gold,
        Self::GoldBank,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Strength => 0,
            Self::Energy => 1,
            Self::Dexterity => 2,
            Self::Vitality => 3,
            Self::StatPoints => 4,
            Self::NewSkills => 5,
            Self::Hitpoints => 6,
            Self::MaxHp => 7,
            Self::Mana => 8,
            Self::MaxMana => 9,
            Self::Stamina => 10,
            Self::MaxStamina => 11,
            Self::Level => 12,
            Self::Experience => 13,
            Self::Gold => 14,
            Self::GoldBank => 15,
        }
    }

    pub const fn id(self) -> u32 {
        self.index() as u32
    }

    pub const fn bit_length(self) -> usize {
        match self {
            Self::Strength
            | Self::Energy
            | Self::Dexterity
            | Self::Vitality
            | Self::StatPoints => 10,
            Self::NewSkills => 8,
            Self::Hitpoints
            | Self::MaxHp
            | Self::Mana
            | Self::MaxMana
            | Self::Stamina
            | Self::MaxStamina => 21,
            Self::Level => 7,
            Self::Experience => 32,
            Self::Gold | Self::GoldBank => 25,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Strength => "strength",
            Self::Energy => "energy",
            Self::Dexterity => "dexterity",
            Self::Vitality => "vitality",
            Self::StatPoints => "statpts",
            Self::NewSkills => "newskills",
            Self::Hitpoints => "hitpoints",
            Self::MaxHp => "maxhp",
            Self::Mana => "mana",
            Self::MaxMana => "maxmana",
            Self::Stamina => "stamina",
            Self::MaxStamina => "maxstamina",
            Self::Level => "level",
            Self::Experience => "experience",
            Self::Gold => "gold",
            Self::GoldBank => "goldbank",
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Strength),
            1 => Some(Self::Energy),
            2 => Some(Self::Dexterity),
            3 => Some(Self::Vitality),
            4 => Some(Self::StatPoints),
            5 => Some(Self::NewSkills),
            6 => Some(Self::Hitpoints),
            7 => Some(Self::MaxHp),
            8 => Some(Self::Mana),
            9 => Some(Self::MaxMana),
            10 => Some(Self::Stamina),
            11 => Some(Self::MaxStamina),
            12 => Some(Self::Level),
            13 => Some(Self::Experience),
            14 => Some(Self::Gold),
            15 => Some(Self::GoldBank),
            _ => None,
        }
    }
}

/// Single stat entry with canonical save-stat metadata.
#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub id: u32,
    pub name: String,
    pub bit_length: usize,
    pub value: u32,
}

impl Stat {
    pub fn max(&self) -> u32 {
        (2u64.pow(self.bit_length as u32) - 1) as u32
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{0} - {1}: {2} -- {3}bits [0-{4}])",
            self.id,
            self.name,
            if self.bit_length == 21 { self.value as f64 / 256f64 } else { self.value.into() },
            self.bit_length,
            self.max()
        )
    }
}

/// Character attributes as stored in the attributes bitstream.
///
/// Stat names follow the save format's canonical stat keys.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: Stat,
    pub energy: Stat,
    pub dexterity: Stat,
    pub vitality: Stat,
    pub statpts: Stat,
    pub newskills: Stat,
    pub hitpoints: Stat,
    pub maxhp: Stat,
    pub mana: Stat,
    pub maxmana: Stat,
    pub stamina: Stat,
    pub maxstamina: Stat,
    level: Stat,
    pub experience: Stat,
    pub gold: Stat,
    pub goldbank: Stat,
}

impl Attributes {
    /// Get a stat by typed id.
    pub fn stat(&self, stat_id: AttributeId) -> &Stat {
        match stat_id {
            AttributeId::Strength => &self.strength,
            AttributeId::Energy => &self.energy,
            AttributeId::Dexterity => &self.dexterity,
            AttributeId::Vitality => &self.vitality,
            AttributeId::StatPoints => &self.statpts,
            AttributeId::NewSkills => &self.newskills,
            AttributeId::Hitpoints => &self.hitpoints,
            AttributeId::MaxHp => &self.maxhp,
            AttributeId::Mana => &self.mana,
            AttributeId::MaxMana => &self.maxmana,
            AttributeId::Stamina => &self.stamina,
            AttributeId::MaxStamina => &self.maxstamina,
            AttributeId::Level => &self.level,
            AttributeId::Experience => &self.experience,
            AttributeId::Gold => &self.gold,
            AttributeId::GoldBank => &self.goldbank,
        }
    }

    fn stat_mut(&mut self, stat_id: AttributeId) -> &mut Stat {
        match stat_id {
            AttributeId::Strength => &mut self.strength,
            AttributeId::Energy => &mut self.energy,
            AttributeId::Dexterity => &mut self.dexterity,
            AttributeId::Vitality => &mut self.vitality,
            AttributeId::StatPoints => &mut self.statpts,
            AttributeId::NewSkills => &mut self.newskills,
            AttributeId::Hitpoints => &mut self.hitpoints,
            AttributeId::MaxHp => &mut self.maxhp,
            AttributeId::Mana => &mut self.mana,
            AttributeId::MaxMana => &mut self.maxmana,
            AttributeId::Stamina => &mut self.stamina,
            AttributeId::MaxStamina => &mut self.maxstamina,
            AttributeId::Level => &mut self.level,
            AttributeId::Experience => &mut self.experience,
            AttributeId::Gold => &mut self.gold,
            AttributeId::GoldBank => &mut self.goldbank,
        }
    }

    /// Set current HP in game-visible units (internal encoding is Q8).
    pub fn set_hp(&mut self, value: u32) {
        self.hitpoints.value = value.saturating_mul(256);
    }

    /// Get current HP in game-visible units.
    pub fn get_hp(&self) -> u32 {
        self.hitpoints.value / 256
    }

    /// Set max HP in game-visible units (internal encoding is Q8).
    pub fn set_max_hp(&mut self, value: u32) {
        self.maxhp.value = value.saturating_mul(256);
    }

    /// Get max HP in game-visible units.
    pub fn get_max_hp(&self) -> u32 {
        self.maxhp.value / 256
    }

    /// Set current mana in game-visible units (internal encoding is Q8).
    pub fn set_mana(&mut self, value: u32) {
        self.mana.value = value.saturating_mul(256);
    }

    /// Get current mana in game-visible units.
    pub fn get_mana(&self) -> u32 {
        self.mana.value / 256
    }

    /// Set max mana in game-visible units (internal encoding is Q8).
    pub fn set_max_mana(&mut self, value: u32) {
        self.maxmana.value = value.saturating_mul(256);
    }

    /// Get max mana in game-visible units.
    pub fn get_max_mana(&self) -> u32 {
        self.maxmana.value / 256
    }

    /// Set current stamina in game-visible units (internal encoding is Q8).
    pub fn set_stamina(&mut self, value: u32) {
        self.stamina.value = value.saturating_mul(256);
    }

    /// Get current stamina in game-visible units.
    pub fn get_stamina(&self) -> u32 {
        self.stamina.value / 256
    }

    /// Set max stamina in game-visible units (internal encoding is Q8).
    pub fn set_max_stamina(&mut self, value: u32) {
        self.maxstamina.value = value.saturating_mul(256);
    }

    /// Get max stamina in game-visible units.
    pub fn get_max_stamina(&self) -> u32 {
        self.maxstamina.value / 256
    }

    /// Character level mirrored by [`crate::character::Character::level`].
    pub const fn level(&self) -> u8 {
        self.level.value as u8
    }

    /// Set level stat value.
    ///
    /// Keep this synchronized with character level through [`crate::Save::set_level`].
    pub(crate) fn set_level(&mut self, level: u8) {
        self.level.value = u32::from(level);
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.strength,
            self.energy,
            self.dexterity,
            self.vitality,
            self.statpts,
            self.newskills,
            self.hitpoints,
            self.maxhp,
            self.mana,
            self.maxmana,
            self.stamina,
            self.maxstamina,
            self.level,
            self.experience,
            self.gold,
            self.goldbank
        )
    }
}

impl Attributes {
    /// Parse attributes from a packed bitstream and advance `byte_position`.
    ///
    /// Each entry is encoded as `9-bit header + value bits`.
    /// Not every stat must be present in the stream.
    pub fn parse(
        byte_slice: &[u8],
        byte_position: &mut BytePosition,
    ) -> Result<Attributes, ParseHardError> {
        if byte_slice.len() < 2 {
            return Err(ParseHardError {
                message: format!(
                    "Attributes section is truncated: expected at least 2 bytes, found {}.",
                    byte_slice.len()
                ),
            });
        }
        if byte_slice[0..2] != SECTION_HEADER {
            return Err(ParseHardError {
                message: format!(
                    "Found wrong header for attributes, expected {SECTION_HEADER:X?} but found {:X?}.",
                    &byte_slice[0..2]
                ),
            });
        }
        byte_position.current_byte = 2;

        let mut attributes = Attributes::default();

        // Parse one extra header to allow a fully populated stream plus trailer.
        for _i in 0..(STAT_NUMBER + 1) {
            let header: u32 =
                read_bits(byte_slice, byte_position, STAT_HEADER_LENGTH).map_err(|error| {
                    ParseHardError {
                        message: format!("Error while parsing attributes header {_i}: {error}"),
                    }
                })?;
            if header == SECTION_TRAILER {
                break;
            }

            let stat_id = AttributeId::from_index(header as usize).ok_or_else(|| ParseHardError {
                message: format!(
                    "Invalid attributes header value {header} at index {_i}; expected < {} or trailer.",
                    STAT_NUMBER
                ),
            })?;
            let value = read_bits(byte_slice, byte_position, stat_id.bit_length()).map_err(
                |error| ParseHardError {
                    message: format!(
                        "Error while parsing attributes value {_i} (header {header}): {error}"
                    ),
                },
            )?;
            attributes.stat_mut(stat_id).value = value;
        }
        Ok(attributes)
    }

    /// Encode attributes into a byte-aligned bitstream.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ParseHardError> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        let mut byte_position: BytePosition = BytePosition::default();
        result.extend_from_slice(&SECTION_HEADER);
        byte_position.current_byte += 2;

        for stat_id in AttributeId::ALL {
            let stat = self.stat(stat_id);
            write_bits(&mut result, &mut byte_position, stat.id, STAT_HEADER_LENGTH)?;
            write_bits(&mut result, &mut byte_position, stat.value, stat.bit_length)?;
        }
        write_bits(&mut result, &mut byte_position, SECTION_TRAILER, STAT_HEADER_LENGTH)?;
        Ok(result)
    }

    pub fn new_save_defaults() -> Self {
        let mut attributes = Attributes::default();

        attributes.level.value = 1;

        attributes.strength.value = 10;
        attributes.energy.value = 10;
        attributes.dexterity.value = 10;
        attributes.vitality.value = 10;

        attributes.maxhp.value = 50;
        attributes.hitpoints.value = attributes.maxhp.value;

        attributes.maxmana.value = 25;
        attributes.mana.value = attributes.maxmana.value;

        attributes.maxstamina.value = 100;
        attributes.stamina.value = attributes.maxstamina.value;

        attributes
    }
}

impl Default for Attributes {
    fn default() -> Self {
        let mut attributes = Attributes {
            strength: Stat::default(),
            energy: Stat::default(),
            dexterity: Stat::default(),
            vitality: Stat::default(),
            statpts: Stat::default(),
            newskills: Stat::default(),
            hitpoints: Stat::default(),
            maxhp: Stat::default(),
            mana: Stat::default(),
            maxmana: Stat::default(),
            stamina: Stat::default(),
            maxstamina: Stat::default(),
            level: Stat::default(),
            experience: Stat::default(),
            gold: Stat::default(),
            goldbank: Stat::default(),
        };
        // Initialize all fields using static stat metadata.
        for stat_id in AttributeId::ALL {
            *attributes.stat_mut(stat_id) = Stat {
                id: stat_id.id(),
                name: stat_id.name().to_string(),
                bit_length: stat_id.bit_length(),
                value: 0,
            };
        }
        attributes
    }
}
