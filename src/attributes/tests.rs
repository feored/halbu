#[cfg(test)]
mod tests {
    use crate::attributes::*;
    use crate::utils::*;
    use crate::Class;

    #[test]
    fn test_write_and_read_attributes() {
        let expected_attributes = Attributes {
            strength: Stat{
                id: 0,
                name:String::from("strength"),
                value: 156,
                bit_length: 10
            },
            energy: Stat{
                id: 1,
                name:String::from("energy"),
                value: 35,
                bit_length: 10
            },
            dexterity: Stat{
                id: 2,
                name:String::from("dexterity"),
                value: 35,
                bit_length: 10
            },
            vitality: Stat{
                id: 3,
                name:String::from("vitality"),
                value: 324,
                bit_length: 10
            },
            statpts:Stat{
                id: 4,
                name:String::from("statpts"),
                value: 0,
                bit_length: 10
            },
            newskills: Stat{
                id: 5,
                name:String::from("newskills"),
                value: 0,
                bit_length: 8
            },
            hitpoints: Stat{
                id: 6,
                name:String::from("hitpoints"),
                value: 322560,
                bit_length: 21
            },
            maxhp: Stat{
                        id: 7,
                        name:String::from("maxhp"),
                        value: 209664,
                        bit_length: 21
            },
            mana: Stat{
                        id: 8,
                        name:String::from("mana"),
                        value: 156,
                        bit_length: 21
            },
            maxmana: Stat{
                        id: 9,
                        name:String::from("maxmana"),
                        value: 55552,
                        bit_length: 21
            },
            stamina: Stat{
                        id: 10,
                        name:String::from("stamina"),
                        value: 140544,
                        bit_length: 21
            },
            maxstamina: Stat{
                        id: 11,
                        name: String::from("maxstamina"),
                        value: 122624,
                        bit_length: 21
            },
            level: Stat{
                        id: 12,
                        name: String::from("level"),
                        value: 92,
                        bit_length: 7
            },
            experience: Stat{
                        id: 13,
                        name: String::from("experience"),
                        value: 2036912623,
                        bit_length: 32
            },
            gold: Stat{
                        id: 14,
                        name: String::from("gold"),
                        value: 0,
                        bit_length: 25
            },
            goldbank: Stat{
                        id: 15,
                        name: String::from("goldbank"),
                        value: 45964,
                        bit_length: 25
                    }
        };
        let result: Vec<u8> = expected_attributes.write();

        let mut byte_position : BytePosition = BytePosition::default();
        let parsed_attributes = match parse(&result, &mut byte_position) {
            Ok(res) => res,
            Err(e) => panic!("Failed test_write_and_read_attributes: {e}"),
        };

        assert_eq!(parsed_attributes, expected_attributes);
    }

    #[test]
    fn test_attributes_class_default(){
        let mut expected_attributes = Attributes::default();

        expected_attributes.level.value = 1;

        expected_attributes.strength.value = 15;
        expected_attributes.dexterity.value = 25;
        expected_attributes.vitality.value = 15;
        expected_attributes.energy.value = 25;
        
        expected_attributes.maxhp.value = 45 * 256;
        expected_attributes.hitpoints.value = expected_attributes.maxhp.value;

        expected_attributes.maxmana.value = 25 * 256;
        expected_attributes.mana.value = expected_attributes.maxmana.value;

        expected_attributes.maxstamina.value = 79 * 256;
        expected_attributes.stamina.value = expected_attributes.maxstamina.value;

        let generated_result = Attributes::default_class(Class::Necromancer);

        assert_eq!(generated_result, expected_attributes);
    }

    #[test]
    fn test_write_stats() {
        let mut result: Vec<u8> = Vec::<u8>::default();
        let mut byte_position: BytePosition = BytePosition::default();
        let header: u32 = 0; // strength
        let value: u32 = 30;

        //write_u8(&mut result, &mut byte_position, 7, 8);
        write_bits(&mut result, &mut byte_position, header, 9);
        write_bits(&mut result, &mut byte_position, value, 10);
        write_bits(&mut result, &mut byte_position, 1u32, 9);
        write_bits(&mut result, &mut byte_position, 10u32, 10);

        assert_eq!([0x00, 0x3C, 0x08, 0xA0, 0x00], result[0..5]);
    }

    #[test]
    fn test_read_stats() {
        let bytes: Vec<u8> = vec![0x00, 0x3C, 0x08, 0x0A0, 0x80, 0x00, 0x0A, 0x06];
        let mut byte_position = BytePosition::default();
        let header_result = read_bits(&bytes, &mut byte_position, 9);
        assert_eq!(header_result, 0);

        let value_result = read_bits(&bytes, &mut byte_position, 10);
        assert_eq!(value_result, 30);
    }

}
