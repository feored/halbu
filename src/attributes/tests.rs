#[cfg(test)]
mod tests {
    use crate::attributes::*;

    #[test]
    fn test_write_and_read_attributes() {
        let expected_attributes = Attributes {
            strength: Attribute(156),
            energy: Attribute(35),
            dexterity: Attribute(35),
            vitality: Attribute(324),
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
            level: Level(92),
            experience: Experience(2036912623),
            gold_inventory: 0,
            gold_stash: 45964,
        };
        let result: Vec<u8> = generate(&expected_attributes);
        let parsed_attributes = match parse(&result) {
            Ok(res) => res,
            Err(e) => panic!("Failed test_write_and_read_attributes: {e}"),
        };

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
            0x67, 0x66, 0x00, 0x3C, 0x08, 0xA0, 0x80, 0x00, 0x0A, 0x06, 0x64, 0x60, 0x00, 0xE0,
            0x06, 0x1C, 0x00, 0xB8, 0x01, 0x08, 0x00, 0x14, 0x40, 0x02, 0x00, 0x05, 0xA0, 0x00,
            0x80, 0x0B, 0x2C, 0x00, 0xE0, 0x02, 0x0C, 0x02, 0xFF, 0x01,
        ];

        let expected_stats = Attributes {
            strength: Attribute(30),
            energy: Attribute(10),
            dexterity: Attribute(20),
            vitality: Attribute(25),
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
            level: Level(1),
            experience: Experience(0),
            gold_inventory: 0,
            gold_stash: 0,
        };

        let parsed_stats = match parse(&bytes) {
            Ok(res) => res,
            Err(e) => panic!("Failed test_parse_attributes_1: {e}"),
        };

        //println!("Parsed stats:");
        //println!("{parsed_stats:?}");

        assert_eq!(parsed_stats, expected_stats);
    }

    #[test]
    fn test_parse_attributes_2() {
        // Level 92 sorceress
        let bytes: Vec<u8> = vec![
            0x67, 0x66, 0x00, 0x38, 0x09, 0x30, 0x82, 0x80, 0x11, 0x06, 0x10, 0x65, 0x00, 0x80,
            0x9D, 0x1C, 0x00, 0x98, 0x19, 0x08, 0x98, 0x2A, 0x45, 0x02, 0x80, 0x6C, 0xA0, 0x00,
            0xA0, 0x44, 0x2C, 0x00, 0xF8, 0x0E, 0x0C, 0xB8, 0x0D, 0xDE, 0xA3, 0xD1, 0xF2, 0x1E,
            0x30, 0xCE, 0x02, 0xF8, 0x0F,
        ];

        let expected_stats = Attributes {
            strength: Attribute(156),
            energy: Attribute(35),
            dexterity: Attribute(35),
            vitality: Attribute(324),
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
            level: Level(92),
            experience: Experience(2036912623),
            gold_inventory: 0,
            gold_stash: 45964,
        };

        let parsed_stats = match parse(&bytes) {
            Ok(res) => res,
            Err(e) => panic!("Failed test_parse_attributes_2: {e}"),
        };
        // println!("Expected stats:");
        // println!("{expected_stats:?}");

        // println!("Parsed stats:");
        // println!("{parsed_stats:?}");
        assert_eq!(parsed_stats, expected_stats);
    }

    #[test]
    fn test_level_from_xp() {
        assert_eq!(get_level_from_experience(Experience(0)), Level(1u8));
        assert_eq!(get_level_from_experience(Experience(2749)), Level(3u8));
        assert_eq!(get_level_from_experience(Experience(3_520_485_254)), Level(99u8));
    }
}
