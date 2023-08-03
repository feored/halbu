#[cfg(test)]

mod tests {
    use crate::character::mercenary::*;
    #[test]
    fn parse_test() {
        let expected_result = Mercenary {
            dead: false,
            id: 3461679u32,
            name_id: 3,
            name: String::from("Abhaya"),
            variant: Variant(Class::Rogue(Rogue::Cold), Difficulty::Normal),
            experience: 63722u32,
        };
        let bytes =
            [0x00, 0x00, 0x2F, 0xD2, 0x34, 0x00, 0x03, 0x00, 0x01, 0x00, 0xEA, 0xF8, 0x00, 0x00];
        let mut parsed_result: Mercenary = Mercenary::default();
        match parse(&bytes) {
            Ok(res) => parsed_result = res,
            Err(e) => {
                println! {"Test failed: {e:?}"}
            }
        };
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn generate_mercenary_test() {
        let expected_result =
            [0x00, 0x00, 0x2F, 0xD2, 0x34, 0x00, 0x03, 0x00, 0x01, 0x00, 0xEA, 0xF8, 0x00, 0x00];
        let merc = Mercenary {
            dead: false,
            id: 3461679u32,
            name_id: 3,
            name: String::from("Abhaya"),
            variant: Variant(Class::Rogue(Rogue::Cold), Difficulty::Normal),
            experience: 63722u32,
        };
        let parsed_result: [u8; 14] = generate(&merc);
        assert_eq!(parsed_result, expected_result);
    }
}
