//! Attributes/stat section model.
//!
//! HP/mana/stamina values are stored in save bytes using fixed-point Q8.
//! Use the convenience getters/setters on [`Attributes`] to work with game-visible units.

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

const STAT_KEY: [&str; STAT_NUMBER] = [
    "strength",
    "energy",
    "dexterity",
    "vitality",
    "statpts",
    "newskills",
    "hitpoints",
    "maxhp",
    "mana",
    "maxmana",
    "stamina",
    "maxstamina",
    "level",
    "experience",
    "gold",
    "goldbank",
];

/// Length in bits of each stat
const STAT_BITLENGTH: [usize; STAT_NUMBER] =
    [10, 10, 10, 10, 10, 8, 21, 21, 21, 21, 21, 21, 7, 32, 25, 25];

/// Representation of a single stat, with data taken from itemstatcosts.txt
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

/// Representation of a character's attributes.
///
/// Names are taken from itemstatcost.txt
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
    pub level: Stat,
    pub experience: Stat,
    pub gold: Stat,
    pub goldbank: Stat,
}

impl Attributes {
    /// Get a cloned stat by canonical stat key (for example `"strength"`).
    pub fn stat(&self, s: &'static str) -> Result<Stat, ParseHardError> {
        match s {
            "strength" => Ok(self.strength.clone()),
            "energy" => Ok(self.energy.clone()),
            "dexterity" => Ok(self.dexterity.clone()),
            "vitality" => Ok(self.vitality.clone()),
            "statpts" => Ok(self.statpts.clone()),
            "newskills" => Ok(self.newskills.clone()),
            "hitpoints" => Ok(self.hitpoints.clone()),
            "maxhp" => Ok(self.maxhp.clone()),
            "mana" => Ok(self.mana.clone()),
            "maxmana" => Ok(self.maxmana.clone()),
            "stamina" => Ok(self.stamina.clone()),
            "maxstamina" => Ok(self.maxstamina.clone()),
            "level" => Ok(self.level.clone()),
            "experience" => Ok(self.experience.clone()),
            "gold" => Ok(self.gold.clone()),
            "goldbank" => Ok(self.goldbank.clone()),
            _ => Err(ParseHardError {
                message: format!("Requested a stat not found in itemstatcost.txt: {s}"),
            }),
        }
    }

    /// Replace a full stat object by canonical stat key.
    pub fn set_stat(&mut self, stat_name: String, stat: &Stat) -> Result<(), ParseHardError> {
        let s: &str = stat_name.as_str();
        match s {
            "strength" => self.strength = stat.clone(),
            "energy" => self.energy = stat.clone(),
            "dexterity" => self.dexterity = stat.clone(),
            "vitality" => self.vitality = stat.clone(),
            "statpts" => self.statpts = stat.clone(),
            "newskills" => self.newskills = stat.clone(),
            "hitpoints" => self.hitpoints = stat.clone(),
            "maxhp" => self.maxhp = stat.clone(),
            "mana" => self.mana = stat.clone(),
            "maxmana" => self.maxmana = stat.clone(),
            "stamina" => self.stamina = stat.clone(),
            "maxstamina" => self.maxstamina = stat.clone(),
            "level" => self.level = stat.clone(),
            "experience" => self.experience = stat.clone(),
            "gold" => self.gold = stat.clone(),
            "goldbank" => self.goldbank = stat.clone(),
            _ => {
                return Err(ParseHardError {
                    message: format!("Tried to set a stat not found in itemstatcost.txt: {s}"),
                })
            }
        };
        Ok(())
    }

    /// Set only the numeric value for a stat by canonical stat key.
    pub fn set_stat_value(&mut self, stat_name: String, value: u32) -> Result<(), ParseHardError> {
        let s: &str = stat_name.as_str();
        match s {
            "strength" => self.strength.value = value,
            "energy" => self.energy.value = value,
            "dexterity" => self.dexterity.value = value,
            "vitality" => self.vitality.value = value,
            "statpts" => self.statpts.value = value,
            "newskills" => self.newskills.value = value,
            "hitpoints" => self.hitpoints.value = value,
            "maxhp" => self.maxhp.value = value,
            "mana" => self.mana.value = value,
            "maxmana" => self.maxmana.value = value,
            "stamina" => self.stamina.value = value,
            "maxstamina" => self.maxstamina.value = value,
            "level" => self.level.value = value,
            "experience" => self.experience.value = value,
            "gold" => self.gold.value = value,
            "goldbank" => self.goldbank.value = value,
            _ => {
                return Err(ParseHardError {
                    message: format!("Tried to set a stat not found in itemstatcost.txt: {s}"),
                })
            }
        };
        Ok(())
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
    /// Parse vector of bytes containing attributes data while storing byte position and return an Attributes struct.
    ///
    /// This function borrows a byte_position, which will store the length in bytes of the
    /// attributes section to help find the offset at which to start reading the next section.
    ///
    /// Attributes are stored in a pair format (header:value). Not all attributes are required to be
    /// present. Headers are always 9 bits. Values span different number of bits found in itemstatcost.txt
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

        // In case all stats are written down, parse one more to make sure we parse 0x1FF trailer
        for _i in 0..(STAT_NUMBER + 1) {
            let header: u32 =
                read_bits(byte_slice, byte_position, STAT_HEADER_LENGTH).map_err(|error| {
                    ParseHardError {
                        message: format!("Error while parsing attributes header {_i}: {error}"),
                    }
                })?;
            if header == SECTION_TRAILER {
                break;
            } else if header as usize >= STAT_KEY.len() {
                return Err(ParseHardError {
                    message: format!(
                        "Invalid attributes header value {header} at index {_i}; expected < {} or trailer.",
                        STAT_KEY.len()
                    ),
                });
            }
            let stat: Stat = attributes.stat(STAT_KEY[header as usize])?;
            attributes.set_stat_value(
                stat.name.clone(),
                read_bits(byte_slice, byte_position, stat.bit_length).map_err(|error| {
                    ParseHardError {
                        message: format!(
                            "Error while parsing attributes value {_i} (header {header}): {error}"
                        ),
                    }
                })?,
            )?;
        }
        Ok(attributes)
    }

    /// Get a byte-aligned vector of bytes representing a character's attribute.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ParseHardError> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        let mut byte_position: BytePosition = BytePosition::default();
        result.append(&mut SECTION_HEADER.to_vec());
        byte_position.current_byte += 2;

        for s in STAT_KEY {
            let stat = self.stat(s).unwrap_or_else(|_| Stat::default());
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
        for (i, s) in STAT_KEY.iter().enumerate() {
            let mut stat: Stat = Stat::default();
            stat.name = s.to_string();
            stat.bit_length = STAT_BITLENGTH[i];
            stat.id = i as u32;
            let _ = attributes.set_stat(stat.name.clone(), &stat);
        }
        attributes
    }
}
