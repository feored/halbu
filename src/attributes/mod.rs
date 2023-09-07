use std::cmp;
use std::fmt;

use log::warn;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};

use crate::bit_manipulation::read_bits;
use crate::bit_manipulation::write_bits;
use crate::bit_manipulation::BytePosition;
use crate::csv::read_csv;
use crate::csv::Record;
use crate::Class;

mod tests;

const SECTION_HEADER: [u8; 2] = [0x67, 0x66];
const SECTION_TRAILER: u32 = 0x1FF;
const STAT_HEADER_LENGTH: usize = 9;
const STAT_NUMBER: usize = 16;

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u32)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]

pub enum StatKey {
    #[default]
    Strength = 0,
    Energy = 1,
    Dexterity = 2,
    Vitality = 3,
    StatPointsLeft = 4,
    SkillPointsLeft = 5,
    LifeCurrent = 6,
    LifeBase = 7,
    ManaCurrent = 8,
    ManaBase = 9,
    StaminaCurrent = 10,
    StaminaBase = 11,
    Level = 12,
    Experience = 13,
    GoldInventory = 14,
    GoldStash = 15,
}

/// Length in bits of each stat
const STAT_BITLENGTH: [usize; STAT_NUMBER] =
    [10, 10, 10, 10, 10, 8, 21, 21, 21, 21, 21, 21, 7, 32, 25, 25];

/// Representation of a single stat, with data taken from itemstatcosts.txt
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stat {
    pub bit_length: usize,
    pub value: u32,
}

impl Stat {
    pub fn set(&mut self, value: u32) {
        self.value = cmp::min(value, self.max());
    }
    pub fn max(&self) -> u32 {
        (2u64.pow(self.bit_length as u32) - 1) as u32
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{0} -- {1}bits [0-{2}])",
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
    pub stat_points_left: Stat,
    pub skill_points_left: Stat,
    pub life_current: Stat,
    pub life_base: Stat,
    pub mana_current: Stat,
    pub mana_base: Stat,
    pub stamina_current: Stat,
    pub stamina_base: Stat,
    pub level: Stat,
    pub experience: Stat,
    pub gold_inventory: Stat,
    pub gold_stash: Stat,
}

impl Attributes {
    pub fn get_stat(&self, key: StatKey) -> Stat {
        match key {
            StatKey::Strength => self.strength,
            StatKey::Energy => self.energy,
            StatKey::Dexterity => self.dexterity,
            StatKey::Vitality => self.vitality,
            StatKey::StatPointsLeft => self.stat_points_left,
            StatKey::SkillPointsLeft => self.skill_points_left,
            StatKey::LifeCurrent => self.life_current,
            StatKey::LifeBase => self.life_base,
            StatKey::ManaCurrent => self.mana_current,
            StatKey::ManaBase => self.mana_base,
            StatKey::StaminaCurrent => self.stamina_current,
            StatKey::StaminaBase => self.stamina_base,
            StatKey::Level => self.level,
            StatKey::Experience => self.experience,
            StatKey::GoldInventory => self.gold_inventory,
            StatKey::GoldStash => self.gold_stash,
        }
    }

    pub fn set(&mut self, key: StatKey, value: u32) {
        match key {
            StatKey::Strength => self.strength.set(value),
            StatKey::Energy => self.energy.set(value),
            StatKey::Dexterity => self.dexterity.set(value),
            StatKey::Vitality => self.vitality.set(value),
            StatKey::StatPointsLeft => self.stat_points_left.set(value),
            StatKey::SkillPointsLeft => self.skill_points_left.set(value),
            StatKey::LifeCurrent => self.life_current.set(value),
            StatKey::LifeBase => self.life_base.set(value),
            StatKey::ManaCurrent => self.mana_current.set(value),
            StatKey::ManaBase => self.mana_base.set(value),
            StatKey::StaminaCurrent => self.stamina_current.set(value),
            StatKey::StaminaBase => self.stamina_base.set(value),
            StatKey::Level => self.level.set(value),
            StatKey::Experience => self.experience.set(value),
            StatKey::GoldInventory => self.gold_inventory.set(value),
            StatKey::GoldStash => self.gold_stash.set(value),
        }
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Strength: {}\nEnergy: {}\nDexterity: {}\nVitality: {}\nStat Points Left: {}\nSkill Points Left: {}\n\
            Current Life: {}\nBase Life: {}\nCurrent Mana: {}\nBase Mana: {}\nCurrent Stamina: {}\nBase Stamina: {}\n\
            Level: {}\nExperience: {}\nGold in Inventory: {}\nGold in Stash: {}",
            self.strength,
            self.energy,
            self.dexterity,
            self.vitality,
            self.stat_points_left,
            self.skill_points_left,
            self.life_current,
            self.life_base,
            self.mana_current,
            self.mana_base,
            self.stamina_current,
            self.stamina_base,
            self.level,
            self.experience,
            self.gold_inventory,
            self.gold_stash
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
    pub fn parse(byte_vector: &Vec<u8>, byte_position: &mut BytePosition) -> Attributes {
        if byte_vector[0..2] != SECTION_HEADER {
            warn!(
                "Found wrong header for attributes, expected {0:X?} but found {1:X?}",
                SECTION_HEADER,
                &byte_vector[0..2]
            );
        }
        byte_position.current_byte = 2;

        let mut attributes = Attributes::default();

        // In case all stats are written down, parse one more to make sure we parse 0x1FF trailer
        for i in 0..(STAT_NUMBER + 1) {
            let header: u32 = match read_bits(byte_vector, byte_position, STAT_HEADER_LENGTH) {
                Ok(res) => res,
                Err(e) => {
                    warn!("Error while parsing attributes header {0}: {1}", i, e.to_string());
                    break;
                }
            };

            if header == SECTION_TRAILER {
                break;
            } else if header as usize >= STAT_NUMBER {
                warn!("Error while parsing attributes header {0}: {1}, using default values for all following attributes.", i, header);
                break;
            }
            let stat_key: StatKey = StatKey::from(header);
            let bit_length = STAT_BITLENGTH[header as usize];
            let stat_value = match read_bits(byte_vector, byte_position, bit_length) {
                Ok(res) => res,
                Err(e) => {
                    warn!(
                        "Error while parsing attributes value {0} (header {1}): {2}. Using default values for all following attributes.",
                        i,
                        header,
                        e.to_string()
                    );
                    return attributes;
                }
            };
            attributes.set(stat_key, stat_value);
        }

        attributes
    }

    /// Get a byte-aligned vector of bytes representing a character's attribute.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        let mut byte_position: BytePosition = BytePosition::default();
        result.append(&mut SECTION_HEADER.to_vec());
        byte_position.current_byte += 2;

        for i in 0..STAT_NUMBER {
            write_bits(&mut result, &mut byte_position, i as u32, STAT_HEADER_LENGTH);
            let stat_key = StatKey::from(i as u32);
            let stat = self.get_stat(stat_key);
            write_bits(&mut result, &mut byte_position, stat.value, stat.bit_length);
        }
        write_bits(&mut result, &mut byte_position, SECTION_TRAILER, STAT_HEADER_LENGTH);
        result
    }

    pub fn default_class(class: Class) -> Self {
        let mut class_attributes: Attributes = Attributes::default();
        let charstats: Vec<Record> =
            read_csv(include_bytes!("../../assets/data/charstats.txt")).unwrap();

        let class_id_in_csv = match class {
            Class::Amazon => 0,
            Class::Sorceress => 1,
            Class::Necromancer => 2,
            Class::Paladin => 3,
            Class::Barbarian => 4,
            Class::Druid => 6,
            Class::Assassin => 7,
        };
        class_attributes.level.value = 1;

        class_attributes.strength.value = charstats[class_id_in_csv]["str"].parse::<u32>().unwrap();
        class_attributes.energy.value = charstats[class_id_in_csv]["int"].parse::<u32>().unwrap();
        class_attributes.dexterity.value =
            charstats[class_id_in_csv]["dex"].parse::<u32>().unwrap();
        class_attributes.vitality.value = charstats[class_id_in_csv]["vit"].parse::<u32>().unwrap();

        class_attributes.life_base.value = (class_attributes.vitality.value
            + charstats[class_id_in_csv]["hpadd"].parse::<u32>().unwrap())
            * 256;
        class_attributes.life_current.value = class_attributes.life_base.value;

        class_attributes.mana_base.value = class_attributes.energy.value * 256;
        class_attributes.mana_current.value = class_attributes.mana_base.value;

        class_attributes.stamina_base.value =
            charstats[class_id_in_csv]["stamina"].parse::<u32>().unwrap() * 256;
        class_attributes.stamina_current.value = class_attributes.stamina_base.value;

        class_attributes
    }
}

impl Default for Attributes {
    fn default() -> Self {
        let attributes = Attributes {
            strength: Stat { bit_length: (10), value: (0) },
            energy: Stat { bit_length: 10, value: 0 },
            dexterity: Stat { bit_length: 10, value: 0 },
            vitality: Stat { bit_length: 10, value: 0 },
            stat_points_left: Stat { bit_length: 10, value: 0 },
            skill_points_left: Stat { bit_length: 8, value: 0 },
            life_current: Stat { bit_length: 21, value: 0 },
            life_base: Stat { bit_length: 21, value: 0 },
            mana_current: Stat { bit_length: 21, value: 0 },
            mana_base: Stat { bit_length: 21, value: 0 },
            stamina_current: Stat { bit_length: 21, value: 0 },
            stamina_base: Stat { bit_length: 21, value: 0 },
            level: Stat { bit_length: 7, value: 0 },
            experience: Stat { bit_length: 32, value: 0 },
            gold_inventory: Stat { bit_length: 25, value: 0 },
            gold_stash: Stat { bit_length: 25, value: 0 },
        };
        attributes
    }
}
