use bit::BitIndex;
use std::cmp;
use std::fmt;

const OFFSET: usize = 765;
const TRAILER: u32 = 0x1FF;

const STAT_HEADER_LENGTH: usize = 9;
const STAT_NUMBER: usize = 16;

/// Array used to find the index of each stat
const STAT_KEY: [Stat; STAT_NUMBER] = [
    Stat::Strength,
    Stat::Energy,
    Stat::Dexterity,
    Stat::Vitality,
    Stat::StatPointsLeft,
    Stat::SkillPointsLeft,
    Stat::LifeCurrent,
    Stat::LifeBase,
    Stat::ManaCurrent,
    Stat::ManaBase,
    Stat::StaminaCurrent,
    Stat::StaminaBase,
    Stat::Level,
    Stat::Experience,
    Stat::GoldInventory,
    Stat::GoldStash,
];

/// Length in bits of each stat
const STAT_BITLENGTH: [usize; STAT_NUMBER] =
    [10, 10, 10, 10, 10, 8, 21, 21, 21, 21, 21, 21, 7, 32, 25, 25];

#[derive(PartialEq, Eq, Debug)]
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

/// Store integer and fraction parts of a fixed point number.
///
/// Life, mana and stamina are represented
/// as 21 bit fixed point numbers, 13 bit
/// for the integer and 8 for the fraction.
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct FixedPointStat {
    integer: u32,
    fraction: u32,
}

impl From<u32> for FixedPointStat {
    fn from(fixed_point_number: u32) -> FixedPointStat {
        let integer: u32 = fixed_point_number.bit_range(8..21);
        let fraction: u32 = fixed_point_number.bit_range(0..8);
        FixedPointStat {
            integer: integer,
            fraction: fraction,
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
/// Can be serialized into a vector of u8 using  Vec<u8>::from().
/// Values can contain up to 32 bits (experience).
/// Certain values are fixed point and stored with integer and
/// fraction separately for precision and easier comparison.
#[derive(Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Attributes {
    strength: u32,
    energy: u32,
    dexterity: u32,
    vitality: u32,
    stat_points_left: u32,
    skill_points_left: u32,
    life_current: FixedPointStat,
    life_base: FixedPointStat,
    mana_current: FixedPointStat,
    mana_base: FixedPointStat,
    stamina_current: FixedPointStat,
    stamina_base: FixedPointStat,
    level: u32,
    experience: u32,
    gold_inventory: u32,
    gold_stash: u32,
}

/// Keep track of current byte and bit index in the attributes byte vector.
#[derive(Default)]
pub struct BytePosition {
    pub current_byte: usize,
    pub current_bit: usize,
}

/// Write bits_count number of bits (LSB ordering) from bits_source into a vector of bytes.
pub fn write_u8(
    byte_vector: &mut Vec<u8>,
    byte_position: &mut BytePosition,
    bits_source: u8,
    bits_count: usize,
) -> () {
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
) -> () {
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
impl From<Attributes> for Vec<u8> {
    fn from(attributes: Attributes) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        let mut byte_position: BytePosition = BytePosition::default();
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
                Stat::Level => attributes.level,
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
}

/// Read a certain number of bits in a vector of bytes, starting at a given byte and bit index, and return a u32 with the value.
///
/// The attributes are stored in a packed struct with non-aligned bytes.
/// Headers for instance contain 9 bits, so they must be read over multiple bytes.
fn parse_bits(byte_vector: &Vec<u8>, byte_position: &mut BytePosition, bits_to_read: usize) -> u32 {
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
        bits_left_to_read = bits_left_to_read - bits_parsing_count;
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
pub fn parse_attributes_with_position(
    byte_vector: &Vec<u8>,
    byte_position: &mut BytePosition,
) -> Attributes {
    let mut stats = Attributes::default();

    for _i in 0..STAT_NUMBER {
        let header = parse_bits(&byte_vector, byte_position, STAT_HEADER_LENGTH);
        if header == TRAILER {
            break;
        }

        let bits_to_parse = STAT_BITLENGTH[header as usize];
        // println!("Now parsing length: {bits_to_parse:?}, header : {header:?}");
        let value = parse_bits(&byte_vector, byte_position, bits_to_parse);
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
            Stat::Level => stats.level = value,
            Stat::Experience => stats.experience = value,
            Stat::GoldInventory => stats.gold_inventory = value,
            Stat::GoldStash => stats.gold_stash = value,
        }
    }
    stats
}

/// Parse vector of bytes containing attributes data and return an Attributes struct.
///
/// Calls parse_attributes_with_position and discards the byte_position information.
pub fn parse_attributes(byte_vector: &Vec<u8>) -> Attributes {
    let mut byte_position = BytePosition::default();
    parse_attributes_with_position(byte_vector, &mut byte_position)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bit::BitIndex;

    #[test]
    fn test_write_and_read_attributes() {
        let expected_attributes = Attributes {
            strength: 156,
            energy: 35,
            dexterity: 35,
            vitality: 324,
            stat_points_left: 0,
            skill_points_left: 0,
            life_current: FixedPointStat {
                integer: 1260,
                fraction: 0,
            },
            life_base: FixedPointStat {
                integer: 819,
                fraction: 0,
            },
            mana_current: FixedPointStat {
                integer: 661,
                fraction: 76,
            },
            mana_base: FixedPointStat {
                integer: 217,
                fraction: 0,
            },
            stamina_current: FixedPointStat {
                integer: 549,
                fraction: 0,
            },
            stamina_base: FixedPointStat {
                integer: 479,
                fraction: 0,
            },
            level: 92,
            experience: 2036912623,
            gold_inventory: 0,
            gold_stash: 45964,
        };
        let result: Vec<u8> = Vec::<u8>::from(expected_attributes);
        let parsed_attributes = parse_attributes(&result);

        assert_eq!(parsed_attributes, expected_attributes);
    }

    #[test]
    fn test_write_some_stat() {
        let mut result: Vec<u8> = Vec::<u8>::default();
        let mut byte_position: BytePosition = BytePosition::default();
        let header: u32 = 0; // strength
        let value: u32 = 30;

        // let expected_result = vec![
        //     0x00, 0x38, 0x09, 0x30, 0x82, 0x80, 0x11, 0x06, 0x10, 0x65, 0x00, 0x80, 0x9D, 0x1C,
        //     0x00, 0x98, 0x19, 0x08, 0x98, 0x2A, 0x45, 0x02, 0x80, 0x6C, 0xA0, 0x00, 0xA0, 0x44,
        //     0x2C, 0x00, 0xF8, 0x0E, 0x0C, 0xB8, 0x0D, 0xDE, 0xA3, 0xD1, 0xF2, 0x1E, 0x30, 0xCE,
        //     0x02, 0xF8, 0x0F,
        // ];

        //write_u8(&mut result, &mut byte_position, 7, 8);
        write_u32(&mut result, &mut byte_position, header, 9);
        write_u32(&mut result, &mut byte_position, value, 10);
        write_u32(&mut result, &mut byte_position, 1, 9);
        write_u32(&mut result, &mut byte_position, 10, 10);

        // println!("WRITE TEST:");
        // for i in 0..result.len(){
        //     println!("Bit {i:?}: {0:#010b}", result[i]);
        // }
        // println!("{result:?}");
        assert_eq!([0x00, 0x3C, 0x08, 0xA0, 0x00], result[0..5]);
    }

    #[test]
    fn test_parse_attributes_bit() {
        let bytes: Vec<u8> = vec![0x00, 0x3C, 0x08, 0x0A0, 0x80, 0x00, 0x0A, 0x06];
        let mut byte_position = BytePosition::default();
        let header_result = parse_bits(&bytes, &mut byte_position, 9);
        assert_eq!(header_result, 0);

        let value_result = parse_bits(&bytes, &mut byte_position, 10);
        assert_eq!(value_result, 30);
    }

    #[test]
    fn test_parse_attributes_1() {
        // Level 1 newly-created barbarian
        let bytes: Vec<u8> = vec![
            0x00, 0x3C, 0x08, 0xA0, 0x80, 0x00, 0x0A, 0x06, 0x64, 0x60, 0x00, 0xE0, 0x06, 0x1C,
            0x00, 0xB8, 0x01, 0x08, 0x00, 0x14, 0x40, 0x02, 0x00, 0x05, 0xA0, 0x00, 0x80, 0x0B,
            0x2C, 0x00, 0xE0, 0x02, 0x0C, 0x02, 0xFF, 0x01,
        ];

        let expected_stats = Attributes {
            strength: 30,
            energy: 10,
            dexterity: 20,
            vitality: 25,
            stat_points_left: 0,
            skill_points_left: 0,
            life_current: FixedPointStat {
                integer: 55,
                fraction: 0,
            },
            life_base: FixedPointStat {
                integer: 55,
                fraction: 0,
            },
            mana_current: FixedPointStat {
                integer: 10,
                fraction: 0,
            },
            mana_base: FixedPointStat {
                integer: 10,
                fraction: 0,
            },
            stamina_current: FixedPointStat {
                integer: 92,
                fraction: 0,
            },
            stamina_base: FixedPointStat {
                integer: 92,
                fraction: 0,
            },
            level: 1,
            experience: 0,
            gold_inventory: 0,
            gold_stash: 0,
        };

        let parsed_stats = parse_attributes(&bytes);

        //println!("Parsed stats:");
        //println!("{parsed_stats:?}");

        assert_eq!(parsed_stats, expected_stats);
    }

    #[test]
    fn test_parse_attributes_2() {
        // Level 92 sorceress
        let bytes: Vec<u8> = vec![
            0x00, 0x38, 0x09, 0x30, 0x82, 0x80, 0x11, 0x06, 0x10, 0x65, 0x00, 0x80, 0x9D, 0x1C,
            0x00, 0x98, 0x19, 0x08, 0x98, 0x2A, 0x45, 0x02, 0x80, 0x6C, 0xA0, 0x00, 0xA0, 0x44,
            0x2C, 0x00, 0xF8, 0x0E, 0x0C, 0xB8, 0x0D, 0xDE, 0xA3, 0xD1, 0xF2, 0x1E, 0x30, 0xCE,
            0x02, 0xF8, 0x0F,
        ];

        let expected_stats = Attributes {
            strength: 156,
            energy: 35,
            dexterity: 35,
            vitality: 324,
            stat_points_left: 0,
            skill_points_left: 0,
            life_current: FixedPointStat {
                integer: 1260,
                fraction: 0,
            },
            life_base: FixedPointStat {
                integer: 819,
                fraction: 0,
            },
            mana_current: FixedPointStat {
                integer: 661,
                fraction: 76,
            },
            mana_base: FixedPointStat {
                integer: 217,
                fraction: 0,
            },
            stamina_current: FixedPointStat {
                integer: 549,
                fraction: 0,
            },
            stamina_base: FixedPointStat {
                integer: 479,
                fraction: 0,
            },
            level: 92,
            experience: 2036912623,
            gold_inventory: 0,
            gold_stash: 45964,
        };

        let parsed_stats = parse_attributes(&bytes);
        // println!("Expected stats:");
        // println!("{expected_stats:?}");

        // println!("Parsed stats:");
        // println!("{parsed_stats:?}");
        assert_eq!(parsed_stats, expected_stats);
    }
}
