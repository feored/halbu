#[test]
fn mercenary_parse_test() {
    let expected_result = crate::character::mercenary::Mercenary {
        is_dead: false,
        id: 3461679u32,
        name_id: 3,
        variant_id: 1u16,
        experience: 63722u32,
    };
    let bytes: [u8; 14] =
        [0x00, 0x00, 0x2F, 0xD2, 0x34, 0x00, 0x03, 0x00, 0x01, 0x00, 0xEA, 0xF8, 0x00, 0x00];
    let parsed_result: crate::character::mercenary::Mercenary =
        crate::character::mercenary::Mercenary::parse(&bytes).expect("mercenary should parse");
    assert_eq!(parsed_result, expected_result);
}

#[test]
fn mercenary_write_test() {
    let expected_result: [u8; 14] =
        [0x00, 0x00, 0x2F, 0xD2, 0x34, 0x00, 0x03, 0x00, 0x01, 0x00, 0xEA, 0xF8, 0x00, 0x00];
    let merc: crate::character::mercenary::Mercenary = crate::character::mercenary::Mercenary {
        is_dead: false,
        id: 3461679u32,
        name_id: 3,
        variant_id: 1u16,
        experience: 63722u32,
    };
    let parsed_result: [u8; 14] = merc.write();
    assert_eq!(parsed_result, expected_result);
}

#[test]
fn mercenary_write_zeroes_unhired_payload() {
    let merc: crate::character::mercenary::Mercenary = crate::character::mercenary::Mercenary {
        is_dead: true,
        id: 0,
        name_id: 20,
        variant_id: 7,
        experience: 220,
    };

    assert_eq!(merc.write(), [0x00; 14]);
}
