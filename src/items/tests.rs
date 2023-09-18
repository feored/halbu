#[cfg(test)]
mod tests {
    use crate::items::*;

    #[test]
    fn items_header_test() {
        let bytes: [u8; 10] = [0x10, 0x0, 0x80, 0x0, 0x4D, 0x04, 0x40, 0xBC, 0x19, 0xF2];
        let mut byte_data = ByteIO::new(&bytes);
        // Header { identified: true, broken: false, socketed: false, ear: false, starter_gear: false, compact: false, ethereal: false,
        // personalized: false, runeword: false, status: Equipped, slot: Helmet, column: 1, row: 0, storage: None, base: "cap ", socketed_count: 0 }
        let parsed_result = Header::parse(&mut byte_data).unwrap();
        let expected_result = Header {
            identified: true,
            status: Status::Equipped,
            slot: Slot::Helmet,
            column: 1,
            base: String::from("cap "),
            ..Default::default()
        };
        assert_eq!(parsed_result, expected_result);
    }
}
