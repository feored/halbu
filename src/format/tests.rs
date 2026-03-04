use super::*;
use crate::character::v105::{MODE_CLASSIC, MODE_EXPANSION, MODE_ROTW, OFFSET_MODE_MARKER};
use crate::{Class, Save};

fn encode_v105_with_mode(mode_marker: u8, mercenary_hired: bool) -> Vec<u8> {
    let mut save = Save::new(FormatId::V105, Class::Barbarian);
    let mut raw_section = vec![0u8; crate::character::expected_length_for_format(FormatId::V105)];
    raw_section[OFFSET_MODE_MARKER] = mode_marker;
    save.character.raw_section = raw_section;
    if mercenary_hired {
        save.character.mercenary.id = 1;
    }

    encode(&save, FormatId::V105).expect("v105 save should encode")
}

#[test]
fn encode_v105_empty_items_layout() {
    const CLASSIC_NO_ITEMS: [u8; 4] = [0x4A, 0x4D, 0x00, 0x00];
    const EXPANSION_NO_ITEMS: [u8; 13] =
        [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];
    const EXPANSION_NO_ITEMS_MERC: [u8; 17] = [
        0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B,
        0x66, 0x00,
    ];
    const ROTW_NO_ITEMS: [u8; 19] = [
        0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00, 0x01, 0x00,
        0x6C, 0x66, 0x00, 0x00,
    ];
    const ROTW_NO_ITEMS_MERC: [u8; 23] = [
        0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B,
        0x66, 0x00, 0x01, 0x00, 0x6C, 0x66, 0x00, 0x00,
    ];

    let cases: [(u8, bool, &[u8]); 6] = [
        (MODE_CLASSIC, false, &CLASSIC_NO_ITEMS),
        (MODE_CLASSIC, true, &CLASSIC_NO_ITEMS),
        (MODE_EXPANSION, false, &EXPANSION_NO_ITEMS),
        (MODE_EXPANSION, true, &EXPANSION_NO_ITEMS_MERC),
        (MODE_ROTW, false, &ROTW_NO_ITEMS),
        (MODE_ROTW, true, &ROTW_NO_ITEMS_MERC),
    ];

    for (mode_marker, mercenary_hired, expected_suffix) in cases {
        let encoded = encode_v105_with_mode(mode_marker, mercenary_hired);
        assert!(
            encoded.ends_with(expected_suffix),
            "unexpected items trailer for mode {mode_marker} (merc hired = {mercenary_hired})"
        );
    }
}
