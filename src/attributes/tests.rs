#[cfg(test)]
mod tests {
    use crate::attributes::*;
    use crate::Class;

    #[test]
    fn test_write_and_read_attributes() {
        let expected_attributes = Attributes {
            strength: Stat { value: 156, bit_length: 10 },
            energy: Stat { value: 35, bit_length: 10 },
            dexterity: Stat { value: 35, bit_length: 10 },
            vitality: Stat { value: 324, bit_length: 10 },
            stat_points_left: Stat { value: 0, bit_length: 10 },
            skill_points_left: Stat { value: 0, bit_length: 8 },
            life_current: Stat { value: 322560, bit_length: 21 },
            life_base: Stat { value: 209664, bit_length: 21 },
            mana_current: Stat { value: 156, bit_length: 21 },
            mana_base: Stat { value: 55552, bit_length: 21 },
            stamina_current: Stat { value: 140544, bit_length: 21 },
            stamina_base: Stat { value: 122624, bit_length: 21 },
            level: Stat { value: 92, bit_length: 7 },
            experience: Stat { value: 2036912623, bit_length: 32 },
            gold_inventory: Stat { value: 0, bit_length: 25 },
            gold_stash: Stat { value: 45964, bit_length: 25 },
        };
        let result: Vec<u8> = expected_attributes.to_bytes();

        let mut reader: ByteIO = ByteIO::new(&result, false);
        let parsed_attributes = Attributes::parse(&mut reader);

        assert_eq!(parsed_attributes, expected_attributes);
    }

    #[test]
    fn test_attributes_class_default() {
        let mut expected_attributes = Attributes::default();

        expected_attributes.level.value = 1;

        expected_attributes.strength.value = 15;
        expected_attributes.dexterity.value = 25;
        expected_attributes.vitality.value = 15;
        expected_attributes.energy.value = 25;

        expected_attributes.life_base.value = 45 * 256;
        expected_attributes.life_current.value = expected_attributes.life_base.value;

        expected_attributes.mana_base.value = 25 * 256;
        expected_attributes.mana_current.value = expected_attributes.mana_base.value;

        expected_attributes.stamina_base.value = 79 * 256;
        expected_attributes.stamina_current.value = expected_attributes.stamina_base.value;

        let generated_result = Attributes::default_class(Class::Necromancer);

        assert_eq!(generated_result, expected_attributes);
    }

    #[test]
    fn test_write_stats() {
        let mut io = ByteIO::default();
        let header: u32 = 0; // strength
        let value: u32 = 30;

        //write_u8(&mut result, &mut byte_position, 7, 8);
        io.write_bits(header, 9);
        io.write_bits(value, 10);
        io.write_bits(1u32, 9);
        io.write_bits(10u32, 10);

        assert_eq!([0x00, 0x3C, 0x08, 0xA0, 0x00], io.data[0..5]);
    }

    #[test]
    fn test_read_stats() {
        let bytes: [u8; 8] = [0x00, 0x3C, 0x08, 0x0A0, 0x80, 0x00, 0x0A, 0x06];
        let mut io = ByteIO::new(&bytes, false);
        let header_result = io.read_bits(9).unwrap();
        assert_eq!(header_result, 0);

        let value_result = io.read_bits(10).unwrap();
        assert_eq!(value_result, 30);
    }
}
