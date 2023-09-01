use std::fmt;

use serde::{Deserialize, Serialize};

use crate::utils::read_bits;
use crate::utils::read_csv;
use crate::utils::write_bits;
use crate::utils::BytePosition;
use crate::utils::Record;
use crate::Class;
use crate::ParseError;

mod tests;

const SECTION_HEADER: [u8; 2] = [0x67, 0x66];
const SECTION_TRAILER: u32 = 0x1FF;
const STAT_HEADER_LENGTH: usize = 9;
const STAT_NUMBER: usize = 16;

const STAT_KEY: [&'static str; STAT_NUMBER] = [
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
            self.value,
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
    pub fn stat(&self, s: &'static str) -> Stat {
        match s {
            "strength" => self.strength.clone(),
            "energy" => self.energy.clone(),
            "dexterity" => self.dexterity.clone(),
            "vitality" => self.vitality.clone(),
            "statpts" => self.statpts.clone(),
            "newskills" => self.newskills.clone(),
            "hitpoints" => self.hitpoints.clone(),
            "maxhp" => self.maxhp.clone(),
            "mana" => self.mana.clone(),
            "maxmana" => self.maxmana.clone(),
            "stamina" => self.stamina.clone(),
            "maxstamina" => self.maxstamina.clone(),
            "level" => self.level.clone(),
            "experience" => self.experience.clone(),
            "gold" => self.gold.clone(),
            "goldbank" => self.goldbank.clone(),
            _ => panic!("Requested a stat not found in itemstatcost.txt: {}", s),
        }
    }

    pub fn set_stat(&mut self, stat_name: String, stat: &Stat) {
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
            _ => panic!("Tried to set a stat not found in itemstatcost.txt: {}", s),
        };
    }

    pub fn set_stat_value(&mut self, stat_name: String, value: u32) {
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
            _ => panic!("Tried to set a stat not found in itemstatcost.txt: {}", s),
        };
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
    /// Get a byte-aligned vector of bytes representing a character's attribute.
    pub fn write(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        let mut byte_position: BytePosition = BytePosition::default();
        result.append(&mut SECTION_HEADER.to_vec());
        byte_position.current_byte += 2;

        for s in STAT_KEY {
            write_bits(&mut result, &mut byte_position, self.stat(s).id, STAT_HEADER_LENGTH);
            write_bits(
                &mut result,
                &mut byte_position,
                self.stat(s).value,
                self.stat(s).bit_length,
            );
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

        class_attributes.maxhp.value = (class_attributes.vitality.value
            + charstats[class_id_in_csv]["hpadd"].parse::<u32>().unwrap())
            * 256;
        class_attributes.hitpoints.value = class_attributes.maxhp.value;

        class_attributes.maxmana.value = class_attributes.energy.value * 256;
        class_attributes.mana.value = class_attributes.maxmana.value;

        class_attributes.maxstamina.value =
            charstats[class_id_in_csv]["stamina"].parse::<u32>().unwrap() * 256;
        class_attributes.stamina.value = class_attributes.maxstamina.value;

        class_attributes
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
        // initialize all fields using csv
        for (i, s) in STAT_KEY.iter().enumerate() {
            let mut stat: Stat = Stat::default();
            stat.name = s.to_string();
            stat.bit_length = STAT_BITLENGTH[i];
            stat.id = i as u32;
            attributes.set_stat(stat.name.clone(), &stat);
        }
        attributes
    }
}
/// Parse vector of bytes containing attributes data while storing byte position and return an Attributes struct.
///
/// This function borrows a byte_position, which will store the length in bytes of the
/// attributes section to help find the offset at which to start reading the next section.
///
/// Attributes are stored in a pair format (header:value). Not all attributes are required to be
/// present. Headers are always 9 bits. Values span different number of bits found in itemstatcost.txt
pub fn parse(
    byte_vector: &Vec<u8>,
    byte_position: &mut BytePosition,
) -> Result<Attributes, ParseError> {
    if byte_vector[0..2] != SECTION_HEADER {
        return Err(ParseError {
            message: format!(
                "Found wrong header for attributes, expected {0:X?} but found {1:X?}",
                SECTION_HEADER,
                &byte_vector[0..2]
            ),
        });
    }
    byte_position.current_byte = 2;

    let mut attributes = Attributes::default();

    // In case all stats are written down, parse one more to make sure we parse 0x1FF trailer
    for _i in 0..(STAT_NUMBER + 1) {
        let header: u32 = read_bits(byte_vector, byte_position, STAT_HEADER_LENGTH);
        if header == SECTION_TRAILER {
            break;
        }
        let stat: Stat = attributes.stat(STAT_KEY[header as usize]);
        attributes.set_stat_value(
            stat.name.clone(),
            read_bits(byte_vector, byte_position, stat.bit_length),
        );
    }
    Ok(attributes)
}
