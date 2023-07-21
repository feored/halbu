use std::cmp;
use bit::BitIndex;

const OFFSET: usize = 765;
const TRAILER : u32 = 0x1FF;

const STAT_HEADER_LENGTH : usize = 9;
const STAT_NUMBER: usize = 16;
const STAT_KEY: [Stat; STAT_NUMBER] = [
    Stat::Strength,
    Stat::Energy,
    Stat::Dexterity,
    Stat::Vitality,
    Stat::StatPointsLeft,
    Stat::SkillPointsLeft,
    Stat::LifeCurrent,
    Stat::LifeMax,
    Stat::ManaCurrent,
    Stat::ManaMax,
    Stat::StaminaCurrent,
    Stat::StaminaMax,
    Stat::Level,
    Stat::Experience,
    Stat::GoldInventory,
    Stat::GoldStash,
];

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
    LifeMax,
    ManaCurrent,
    ManaMax,
    StaminaCurrent,
    StaminaMax,
    Level,
    Experience,
    GoldInventory,
    GoldStash,
}

#[derive(Default,PartialEq, Eq, Debug)]
pub struct Statistics {
    strength: u32,
    energy: u32,
    dexterity: u32,
    vitality: u32,
    stat_points_left: u32,
    skill_points_left: u32,
    life_current: u32,
    life_max: u32,
    mana_current: u32,
    mana_max: u32,
    stamina_current: u32,
    stamina_max: u32,
    level: u32,
    experience: u32,
    gold_inventory: u32,
    gold_stash: u32
}

#[derive(Default)]
struct BytePosition{
    current_byte : usize,
    current_bit : usize
}

fn parse_statistics_bits(byte_vector : &Vec<u8>, byte_position : &mut BytePosition, bits_to_read:usize) -> u32 {
    let mut bits_left_to_read : usize = bits_to_read;
    let mut buffer : u32 = 0;
    let mut buffer_bit_position: usize = 0;
    loop {
        println!("Bits left to read: {bits_left_to_read:?}");
        if bits_left_to_read == 0 {
            break
        }
        if byte_position.current_bit > 7 {
            byte_position.current_byte += 1;
            byte_position.current_bit = 0;
        }
        let bits_parsing_count = cmp::min(8-byte_position.current_bit, bits_left_to_read);
        let bits_parsed : u8 = byte_vector[byte_position.current_byte].bit_range(byte_position.current_bit..(byte_position.current_bit + bits_parsing_count));
        
        buffer.set_bit_range(buffer_bit_position..(buffer_bit_position + bits_parsing_count), u32::from_le_bytes([bits_parsed, 0x00, 0x00, 0x00]));
        buffer_bit_position += bits_parsing_count;
        bits_left_to_read = bits_left_to_read - bits_parsing_count;
        byte_position.current_bit += bits_parsing_count;

        println!("Bits left to read: {bits_left_to_read:?},
        Current byte index: {0:?},
        Current bit index: {1:?},
        {bits_parsing_count:?} bits parsed: {bits_parsed:#b}
        ", byte_position.current_byte, byte_position.current_bit);
    }
    buffer
}

fn parse_statistics(byte_vector : &Vec<u8>) -> Statistics{
    let mut byte_position = BytePosition::default();

    let mut stats = Statistics::default();

    for i in 0..STAT_NUMBER{
        println!("Parsing Run: {i:?}");
        let header = parse_statistics_bits(&byte_vector, &mut byte_position, STAT_HEADER_LENGTH);
        if header == TRAILER{
            break;
        }
        
        let bits_to_parse = STAT_BITLENGTH[header as usize];
        println!("Now parsing length: {bits_to_parse:?}, header : {header:?}");
        let value = parse_statistics_bits(&byte_vector, &mut byte_position, bits_to_parse);
        println!("Now parsed: {0:?}, length: {bits_to_parse:?}, value: {value:?}", STAT_KEY[header as usize]);
        match STAT_KEY[header as usize] {
            Stat::Strength => {stats.strength = value},
            Stat::Energy => {stats.energy = value},
            Stat::Dexterity => {stats.strength = value},
            Stat::Vitality => {stats.strength = value},
            Stat::StatPointsLeft => {stats.stat_points_left = value},
            Stat::SkillPointsLeft => {stats.skill_points_left = value},
            Stat::LifeCurrent => {stats.life_current = value},
            Stat::LifeMax => {stats.life_max = value},
            Stat::ManaCurrent => {stats.mana_current = value},
            Stat::ManaMax => {stats.mana_max = value},
            Stat::StaminaCurrent => {stats.stamina_current = value},
            Stat::StaminaMax => {stats.stamina_max = value},
            Stat::Level => {stats.level = value},
            Stat::Experience => {stats.experience = value},
            Stat::GoldInventory => {stats.gold_inventory = value},
            Stat::GoldStash => {stats.gold_stash = value},            
        }
    }

    stats
}


#[cfg(test)]
mod tests {
    use super::BytePosition;
    use super::Statistics;
    use super::parse_statistics_bits;
    use super::parse_statistics;
    
    #[test]
    fn test_parse_statistics_bit() {
        let bytes : Vec<u8> = vec!(0x00, 0x3C, 0x08, 0x0A0, 0x80, 0x00, 0x0A, 0x06);
        let mut byte_position = BytePosition::default();
        let header_result = parse_statistics_bits(&bytes, &mut byte_position, 9);
        assert_eq!(header_result, 0);

        let value_result = parse_statistics_bits(&bytes, &mut byte_position, 10);
        assert_eq!(value_result, 30);
    }

    #[test]
    fn test_parse_statistics() {
        // Level 1 newly-created barbarian
        let bytes : Vec<u8> = vec!(0x00, 0x3C, 0x08, 0xA0, 0x80, 0x00,
            0x0A, 0x06, 0x64, 0x60, 0x00, 0xE0, 0x06, 0x1C, 0x00, 0xB8,
            0x01, 0x08, 0x00, 0x14, 0x40, 0x02, 0x00, 0x05, 0xA0, 0x00,
            0x80, 0x0B, 0x2C, 0x00, 0xE0, 0x02, 0x0C, 0x02, 0xFF, 0x01,
            0x69, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x0);
        
        let expected_stats = Statistics {
            strength: 30,
            energy: 10,
            dexterity: 20,
            vitality: 25,
            stat_points_left: 0,
            skill_points_left: 0,
            life_current: 55,
            life_max: 55,
            mana_current: 10,
            mana_max: 10,
            stamina_current: 92,
            stamina_max: 92,
            level: 1,
            experience: 0,
            gold_inventory: 0,
            gold_stash: 0
        };

        let parsed_stats = parse_statistics(&bytes);

        assert_eq!(parsed_stats, expected_stats);
    }
}
