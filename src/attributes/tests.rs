#[test]
fn test_write_and_read_attributes() {
    let expected_attributes = crate::attributes::Attributes {
        strength: crate::attributes::Stat {
            id: 0,
            name: String::from("strength"),
            value: 156,
            bit_length: 10,
        },
        energy: crate::attributes::Stat {
            id: 1,
            name: String::from("energy"),
            value: 35,
            bit_length: 10,
        },
        dexterity: crate::attributes::Stat {
            id: 2,
            name: String::from("dexterity"),
            value: 35,
            bit_length: 10,
        },
        vitality: crate::attributes::Stat {
            id: 3,
            name: String::from("vitality"),
            value: 324,
            bit_length: 10,
        },
        statpts: crate::attributes::Stat {
            id: 4,
            name: String::from("statpts"),
            value: 0,
            bit_length: 10,
        },
        newskills: crate::attributes::Stat {
            id: 5,
            name: String::from("newskills"),
            value: 0,
            bit_length: 8,
        },
        hitpoints: crate::attributes::Stat {
            id: 6,
            name: String::from("hitpoints"),
            value: 322560,
            bit_length: 21,
        },
        maxhp: crate::attributes::Stat {
            id: 7,
            name: String::from("maxhp"),
            value: 209664,
            bit_length: 21,
        },
        mana: crate::attributes::Stat {
            id: 8,
            name: String::from("mana"),
            value: 156,
            bit_length: 21,
        },
        maxmana: crate::attributes::Stat {
            id: 9,
            name: String::from("maxmana"),
            value: 55552,
            bit_length: 21,
        },
        stamina: crate::attributes::Stat {
            id: 10,
            name: String::from("stamina"),
            value: 140544,
            bit_length: 21,
        },
        maxstamina: crate::attributes::Stat {
            id: 11,
            name: String::from("maxstamina"),
            value: 122624,
            bit_length: 21,
        },
        level: crate::attributes::Stat {
            id: 12,
            name: String::from("level"),
            value: 92,
            bit_length: 7,
        },
        experience: crate::attributes::Stat {
            id: 13,
            name: String::from("experience"),
            value: 2036912623,
            bit_length: 32,
        },
        gold: crate::attributes::Stat {
            id: 14,
            name: String::from("gold"),
            value: 0,
            bit_length: 25,
        },
        goldbank: crate::attributes::Stat {
            id: 15,
            name: String::from("goldbank"),
            value: 45964,
            bit_length: 25,
        },
    };
    let result: Vec<u8> = expected_attributes.to_bytes().expect("attributes should serialize");

    let mut byte_position = crate::utils::BytePosition::default();
    let parsed_attributes = crate::attributes::Attributes::parse(&result, &mut byte_position)
        .expect("attributes should parse");

    assert_eq!(parsed_attributes, expected_attributes);
}

#[test]
fn test_write_stats() {
    let mut result: Vec<u8> = Vec::<u8>::default();
    let mut byte_position = crate::utils::BytePosition::default();
    let header: u32 = 0; // strength
    let value: u32 = 30;

    crate::utils::write_bits(&mut result, &mut byte_position, header, 9)
        .expect("header should write");
    crate::utils::write_bits(&mut result, &mut byte_position, value, 10)
        .expect("value should write");
    crate::utils::write_bits(&mut result, &mut byte_position, 1u32, 9)
        .expect("header should write");
    crate::utils::write_bits(&mut result, &mut byte_position, 10u32, 10)
        .expect("value should write");

    assert_eq!([0x00, 0x3C, 0x08, 0xA0, 0x00], result[0..5]);
}

#[test]
fn test_read_stats() {
    let bytes: Vec<u8> = vec![0x00, 0x3C, 0x08, 0x0A0, 0x80, 0x00, 0x0A, 0x06];
    let mut byte_position = crate::utils::BytePosition::default();
    let header_result = crate::utils::read_bits(&bytes, &mut byte_position, 9).unwrap();
    assert_eq!(header_result, 0);

    let value_result = crate::utils::read_bits(&bytes, &mut byte_position, 10).unwrap();
    assert_eq!(value_result, 30);
}

#[test]
fn test_resource_setters_use_q8() {
    let mut attributes = crate::attributes::Attributes::default();

    attributes.set_hp(1234);
    attributes.set_max_hp(2345);
    attributes.set_mana(345);
    attributes.set_max_mana(456);
    attributes.set_stamina(567);
    attributes.set_max_stamina(678);

    assert_eq!(attributes.hitpoints.value, 1234 * 256);
    assert_eq!(attributes.maxhp.value, 2345 * 256);
    assert_eq!(attributes.mana.value, 345 * 256);
    assert_eq!(attributes.maxmana.value, 456 * 256);
    assert_eq!(attributes.stamina.value, 567 * 256);
    assert_eq!(attributes.maxstamina.value, 678 * 256);

    assert_eq!(attributes.get_hp(), 1234);
    assert_eq!(attributes.get_max_hp(), 2345);
    assert_eq!(attributes.get_mana(), 345);
    assert_eq!(attributes.get_max_mana(), 456);
    assert_eq!(attributes.get_stamina(), 567);
    assert_eq!(attributes.get_max_stamina(), 678);
}

#[test]
fn test_typed_stat_access() {
    let attributes = crate::attributes::Attributes::default();

    let strength = attributes.stat(crate::attributes::AttributeId::Strength);
    assert_eq!(strength.id, crate::attributes::AttributeId::Strength.id());
    assert_eq!(strength.name, crate::attributes::AttributeId::Strength.name());
    assert_eq!(strength.bit_length, crate::attributes::AttributeId::Strength.bit_length());
}
