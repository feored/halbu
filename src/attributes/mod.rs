use bit::BitIndex;
use std::cmp;
use std::fmt;

use serde::{Serialize, Deserialize};


use crate::GameLogicError;
use crate::utils::BytePosition;
use crate::Class;
use crate::ParseError;

mod tests;
pub mod consts;

use consts::*;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Stat {
    Strength,
    Energy,
    Dexterity,
    Vitality,
    StatPointsLeft,
    SkillPointsLeft,
    LifeCurrent,
    LifeBase,
    ManaCurrent,
    ManaBase,
    StaminaCurrent,
    StaminaBase,
    Level,
    Experience,
    GoldInventory,
    GoldStash,
}

impl TryFrom<String> for Stat{
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, ParseError> {
        let stripped_string: String = value.trim().to_lowercase();
        match stripped_string.as_str() {
            "strength" => Ok(Stat::Strength),
            "energy" => Ok(Stat::Energy),
            "dexterity" => Ok(Stat::Dexterity),
            "vitality" => Ok(Stat::Vitality),
            "statpointsleft" => Ok(Stat::StatPointsLeft),
            "skillpointsleft" => Ok(Stat::SkillPointsLeft),
            "lifecurrent" => Ok(Stat::LifeCurrent),
            "lifebase" => Ok(Stat::LifeBase),
            "manacurrent" => Ok(Stat::ManaCurrent),
            "manabase" => Ok(Stat::ManaBase),
            "staminacurrent" => Ok(Stat::StaminaCurrent),
            "staminabase" => Ok(Stat::StaminaBase),
            "level" => Ok(Stat::Level),
            "experience" => Ok(Stat::Experience),
            "goldinventory" => Ok(Stat::GoldInventory),
            "goldstash" => Ok(Stat::GoldStash),
            _ => Err(ParseError { message: format!("Cannot find corresponding stat for: {0}", value)})
            
        }
    }
}

/// Store integer and fraction parts of a fixed point number.
///
/// Life, mana and stamina are represented
/// as 21 bit fixed point numbers, 13 bit
/// for the integer and 8 for the fraction.
#[derive(Default, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct FixedPointStat {
    integer: u32,
    fraction: u32,
}

impl From<u32> for FixedPointStat {
    fn from(fixed_point_number: u32) -> FixedPointStat {
        let integer: u32 = fixed_point_number.bit_range(8..21);
        let fraction: u32 = fixed_point_number.bit_range(0..8);
        FixedPointStat {
            integer,
            fraction,
        }
    }
}

impl From<&FixedPointStat> for u32 {
    fn from(fixed_point_number: &FixedPointStat) -> u32 {
        let mut result = 0u32;
        result.set_bit_range(0..8, fixed_point_number.fraction.bit_range(0..8));
        result.set_bit_range(8..21, fixed_point_number.integer.bit_range(0..13));
        result
    }
}

impl fmt::Debug for FixedPointStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}.{}]", self.integer, self.fraction)
    }
}

/// Representation of a character's attributes.
///
/// Can be serialized into a vector of u8 using  `Vec<u8>::from()`.
/// Values can contain up to 32 bits (experience).
/// Certain values are fixed point and stored with integer and
/// fraction separately for precision and easier comparison.
#[derive(Default, PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: u32,
    pub energy: u32,
    pub dexterity: u32,
    pub vitality: u32,
    pub stat_points_left: u32,
    pub skill_points_left: u32,
    pub life_current: FixedPointStat,
    pub life_base: FixedPointStat,
    pub mana_current: FixedPointStat,
    pub mana_base: FixedPointStat,
    pub stamina_current: FixedPointStat,
    pub stamina_base: FixedPointStat,
    level: u8,
    experience: u32,
    gold_inventory: u32,
    gold_stash: u32,
}


impl Attributes {

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn set_level(&mut self, new_level : u8) -> Result<(), GameLogicError> {
        match new_level {
            1..=99 => {
                self.level = new_level;
                self.experience = EXPERIENCE_TABLE[self.level as usize - 1];
                Ok(())
            },
            _ => Err(GameLogicError { message: format!("Cannot set level {0}: value must be between 1 and 99.", new_level) })
        }
    }

    pub fn experience(&self) -> u32 {
        self.experience
    }

    pub fn set_experience(&mut self, new_experience: u32) -> Result<(), GameLogicError> {
        if new_experience <= MAX_XP {
            self.experience = new_experience;
            let new_level = get_level_from_experience(new_experience);
            if new_level != self.level {
                self.level = new_level
            }
            Ok(())
        } else {
            Err(GameLogicError { message: format!("Cannot set experience{0}: value must be between <= {1}.", new_experience, MAX_XP) })
        }
    }

    pub fn gold_inventory(&self) -> u32 {
        self.gold_inventory
    }

    pub fn set_gold_inventory(&mut self, new_gold_inventory : u32) -> Result<(), GameLogicError> {
        if new_gold_inventory <= self.level as u32  * GOLD_INVENTORY_PER_LEVEL {
            self.gold_inventory = new_gold_inventory;
            Ok(())
        } else {
            Err(GameLogicError { message: format!("Cannot set {0} gold in inventory: value must be <= {1} for level {2} character.", new_gold_inventory, self.level as u32 * GOLD_INVENTORY_PER_LEVEL, self.level) })
        }
    }

    pub fn gold_stash(&self) -> u32 {
        self.gold_stash
    }

    pub fn set_gold_stash(&mut self, new_gold_stash : u32) -> Result<(), GameLogicError> {
        if new_gold_stash <= MAX_GOLD_STASH {
            self.gold_stash = new_gold_stash;
            Ok(())
        } else {
            Err(GameLogicError { message: format!("Cannot set {0} gold in stash: value must be <= {1}.", new_gold_stash, MAX_GOLD_STASH) })
        }
    }

    pub fn default_class(class: Class) -> Self  {
        let amazon = (20, 25, 20, 15, 50, 84, 15);
        let assassin = (20, 20, 20, 25, 50, 95, 25);
        let barbarian = (30, 20, 25, 10, 55, 92, 10);
        let paladin = (25, 20, 25, 15, 55, 89, 15);
        let necromancer = (15, 25, 15, 25, 45, 79, 25);
        let sorceress = (10, 25, 10, 35, 40, 74, 35);
        let druid = (15, 20, 25, 20, 55, 84, 20);
    
        let stats = match class {
            Class::Amazon => amazon,
            Class::Assassin => assassin,
            Class::Barbarian => barbarian,
            Class::Paladin => paladin,
            Class::Necromancer => necromancer,
            Class::Sorceress => sorceress,
            Class::Druid => druid,
        };
    
        Attributes {
            strength: stats.0,
            dexterity: stats.1,
            vitality: stats.2,
            energy: stats.3,
            stat_points_left: 0,
            skill_points_left: 0,
            life_current: FixedPointStat {
                integer: stats.4,
                fraction: 0,
            },
            life_base: FixedPointStat {
                integer: stats.4,
                fraction: 0,
            },
            mana_current: FixedPointStat {
                integer: stats.6,
                fraction: 0,
            },
            mana_base: FixedPointStat {
                integer: stats.6,
                fraction: 0,
            },
            stamina_current: FixedPointStat {
                integer: stats.5,
                fraction: 0,
            },
            stamina_base: FixedPointStat {
                integer: stats.5,
                fraction: 0,
            },
            level: 1,
            experience: 0,
            gold_inventory: 0,
            gold_stash: 0,
        }
    }

}

fn get_level_from_experience(experience: u32) -> u8 {
    let mut level: u8 = 99;
    for (i, element) in EXPERIENCE_TABLE.iter().enumerate() {
        if *element > experience {
            level = i as u8;
            break;
        }
    }
    level
}



/// Write bits_count number of bits (LSB ordering) from bits_source into a vector of bytes.
pub fn write_u8(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: u8,
    bits_count: usize,
) {
    let mut bits_left_to_write: usize = bits_count;
    let mut bit_index = 0;
    loop {
        if bits_left_to_write == 0 {
            return;
        }
        if byte_vector.len() == byte_position.current_byte {
            byte_vector.push(0);
        }

        if byte_position.current_bit == 8 {
            byte_vector.push(0);
            byte_position.current_byte += 1;
            byte_position.current_bit = 0;
        }

        // println!("Length of byte vector: {0:?} current byte: {1:?}", byte_vector.len(), byte_position.current_byte);
        let bits_can_write_in_byte = cmp::min(bits_left_to_write, 8 - byte_position.current_bit);
        // println!("Writing {bits_can_write_in_byte:?} bits from position {0:?} of {1:#010b} into {2:#010b}", byte_position.current_bit, bits_source, byte_vector[byte_position.current_byte]);

        if bits_can_write_in_byte == 8 {
            // Special case because the bit library seems to fail when trying to set an entire byte using set_bit_range
            // e.g 0x00.set_bit_range(0..8, 0xFF)
            byte_vector[byte_position.current_byte] = bits_source;
        } else {
            byte_vector[byte_position.current_byte].set_bit_range(
                byte_position.current_bit..(byte_position.current_bit + bits_can_write_in_byte),
                bits_source.bit_range(bit_index..(bit_index + bits_can_write_in_byte)),
            );
            bit_index += bits_can_write_in_byte;
        }
        byte_position.current_bit += bits_can_write_in_byte;
        bits_left_to_write -= bits_can_write_in_byte;
    }
}

/// Write bits_count number of bits (LSB ordering) from bits_source into a vector of u8.
pub fn write_u32(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: u32,
    bits_count: usize,
) {
    let mut bits_left_to_write: usize = bits_count;
    // println!(
    //     "Writing {bits_left_to_write:?} bits of binary: {0:#034b}",
    //     bits_source
    // );
    let byte_source = bits_source.to_le_bytes();
    let mut byte_source_current = 0;
    loop {
        if bits_left_to_write == 0 {
            return;
        }
        let bits_can_write = cmp::min(bits_left_to_write, 8);
        write_u8(
            byte_vector,
            byte_position,
            byte_source[byte_source_current],
            bits_can_write,
        );
        bits_left_to_write -= bits_can_write;
        byte_source_current += 1;
    }
}

/// Get a byte-aligned vector of bytes representing a character's attribute.
pub fn generate(attributes: &Attributes) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::<u8>::new();
    let mut byte_position: BytePosition = BytePosition::default();
    result.append(&mut SECTION_HEADER.to_vec());
    byte_position.current_byte = 2;
    for header in 0..STAT_NUMBER {
        let stat = &STAT_KEY[header];
        let header_as_u32 = header as u32;

        write_u32(
            &mut result,
            &mut byte_position,
            header_as_u32,
            STAT_HEADER_LENGTH,
        );

        let value: u32 = match stat {
            Stat::Strength => attributes.strength,
            Stat::Energy => attributes.energy,
            Stat::Dexterity => attributes.dexterity,
            Stat::Vitality => attributes.vitality,
            Stat::StatPointsLeft => attributes.stat_points_left,
            Stat::SkillPointsLeft => attributes.skill_points_left,
            Stat::LifeCurrent => u32::from(&attributes.life_current),
            Stat::LifeBase => u32::from(&attributes.life_base),
            Stat::ManaCurrent => u32::from(&attributes.mana_current),
            Stat::ManaBase => u32::from(&attributes.mana_base),
            Stat::StaminaCurrent => u32::from(&attributes.stamina_current),
            Stat::StaminaBase => u32::from(&attributes.stamina_base),
            Stat::Level => attributes.level as u32,
            Stat::Experience => attributes.experience,
            Stat::GoldInventory => attributes.gold_inventory,
            Stat::GoldStash => attributes.gold_stash,
        };

        write_u32(
            &mut result,
            &mut byte_position,
            value,
            STAT_BITLENGTH[header],
        );
    }
    // add trailing 0x1FF to signal end of attributes section
    write_u32(&mut result, &mut byte_position, 0x1FF, STAT_HEADER_LENGTH);

    // If we end in the middle of a byte, add some padding so that the next section
    // starts on a new byte
    if byte_position.current_bit == 8 {
        byte_position.current_byte += 1;
        byte_position.current_bit = 0;
    } else if byte_position.current_bit != 0 {
        let bits_to_fill = 8 - byte_position.current_bit;
        write_u8(&mut result, &mut byte_position, 0, bits_to_fill);
        byte_position.current_byte += 1;
        byte_position.current_bit = 0;
    }

    result
}

/// Read a certain number of bits in a vector of bytes, starting at a given byte and bit index, and return a u32 with the value.
///
/// The attributes are stored in a packed struct with non-aligned bytes.
/// Headers for instance contain 9 bits, so they must be read over multiple bytes.
fn parse_bits(byte_vector: &[u8], byte_position: &mut BytePosition, bits_to_read: usize) -> u32 {
    let mut bits_left_to_read: usize = bits_to_read;
    let mut buffer: u32 = 0;
    let mut buffer_bit_position: usize = 0;
    loop {
        // println!("Bits left to read: {bits_left_to_read:?}");
        if bits_left_to_read == 0 {
            break;
        }
        if byte_position.current_bit > 7 {
            byte_position.current_byte += 1;
            byte_position.current_bit = 0;
        }
        let bits_parsing_count = cmp::min(8 - byte_position.current_bit, bits_left_to_read);
        let bits_parsed: u8 = byte_vector[byte_position.current_byte]
            .bit_range(byte_position.current_bit..(byte_position.current_bit + bits_parsing_count));

        buffer.set_bit_range(
            buffer_bit_position..(buffer_bit_position + bits_parsing_count),
            u32::from_le_bytes([bits_parsed, 0x00, 0x00, 0x00]),
        );
        buffer_bit_position += bits_parsing_count;
        bits_left_to_read -= bits_parsing_count;
        byte_position.current_bit += bits_parsing_count;

        // println!("Bits left to read: {bits_left_to_read:?},
        // Current byte index: {0:?},
        // Current bit index: {1:?},
        // {bits_parsing_count:?} bits parsed: {bits_parsed:#b}
        // ", byte_position.current_byte, byte_position.current_bit);
    }
    buffer
}

/// Parse vector of bytes containing attributes data while storing byte position and return an Attributes struct.
///
/// This function borrows a byte_position, which will therefore store the length in bytes of the
/// attributes section, and allow one to find the offset at which to start reading the next section.
/// If you don't need this information, use parse_attributes instead.
///
/// Attributes are stored in a pair format (header:value). Not all attributes are required to be
/// present. Headers are always 9 bits, and the STAT_KEY array contains the relevant Stat enum
/// for every header parsed. Values span different number of bits stored in STAT_BITLENGTH.
pub fn parse_with_position(
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
    let mut stats = Attributes::default();
    // println!("Parsed\n{0:?}", byte_vector);
    for _i in 0..STAT_NUMBER {
        let header = parse_bits(byte_vector, byte_position, STAT_HEADER_LENGTH);
        if header == SECTION_TRAILER {
            break;
        }

        let bits_to_parse = STAT_BITLENGTH[header as usize];
        // println!("Now parsing length: {bits_to_parse:?}, header : {header:?}");
        let value = parse_bits(byte_vector, byte_position, bits_to_parse);
        // println!(
        //     "Now parsed: {0:?}, length: {bits_to_parse:?}, value: {value:?}",
        //     STAT_KEY[header as usize]
        // );
        match STAT_KEY[header as usize] {
            Stat::Strength => stats.strength = value,
            Stat::Energy => stats.energy = value,
            Stat::Dexterity => stats.dexterity = value,
            Stat::Vitality => stats.vitality = value,
            Stat::StatPointsLeft => stats.stat_points_left = value,
            Stat::SkillPointsLeft => stats.skill_points_left = value,
            Stat::LifeCurrent => stats.life_current = FixedPointStat::from(value),
            Stat::LifeBase => stats.life_base = FixedPointStat::from(value),
            Stat::ManaCurrent => stats.mana_current = FixedPointStat::from(value),
            Stat::ManaBase => stats.mana_base = FixedPointStat::from(value),
            Stat::StaminaCurrent => stats.stamina_current = FixedPointStat::from(value),
            Stat::StaminaBase => stats.stamina_base = FixedPointStat::from(value),
            Stat::Level => stats.level = value as u8,
            Stat::Experience => stats.experience = value,
            Stat::GoldInventory => stats.gold_inventory = value,
            Stat::GoldStash => stats.gold_stash = value,
        }
    }
    Ok(stats)
}

/// Parse vector of bytes containing attributes data and return an Attributes struct.
///
/// Calls parse_attributes_with_position and discards the byte_position information.
pub fn parse(byte_vector: &Vec<u8>) -> Result<Attributes, ParseError> {
    let mut byte_position = BytePosition::default();
    parse_with_position(byte_vector, &mut byte_position)
}

