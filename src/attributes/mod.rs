use std::fmt;

use serde::{Deserialize, Serialize};

use crate::utils::read_bits;
use crate::utils::write_bits;
use crate::utils::BytePosition;
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
#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
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
            _ => panic!("Requested a stat not found in itemstatcost.txt: {}", s)
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
            _ => panic!("Tried to set a stat not found in itemstatcost.txt: {}", s)
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
            write_bits(&mut result, &mut byte_position, self.stat(s).value, self.stat(s).bit_length);
        }
        write_bits(&mut result, &mut byte_position, SECTION_TRAILER, STAT_HEADER_LENGTH);
        result
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
        //println!("Found header: {0:?} == ? {1:?}", header, SECTION_TRAILER);
        if header == SECTION_TRAILER {
            //println!("BREAAAAAAAAAAK");
            break;
        }
        //println!("Ok no break, {0}", header as usize);
        let mut stat : Stat = Stat::default();
        stat.name = String::from(STAT_KEY[header as usize]);
        stat.id = header;
        stat.bit_length = STAT_BITLENGTH[header as usize];
        //println!("{0:?}", stat);
        stat.value = read_bits(byte_vector, byte_position, stat.bit_length);
        //println!("{0:?}", stat);
        attributes.set_stat(stat.name.clone(), &stat);
    }
    Ok(attributes)
}
