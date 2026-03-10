use super::*;
use crate::character::v105::{
    MODE_CLASSIC, MODE_EXPANSION, MODE_ROTW, OFFSET_MODE_MARKER, OFFSET_STATUS,
};
use crate::{Class, ExpansionType, Save};

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
fn encodable_formats_lists_supported_versions() {
    let formats = FormatId::encodable_formats();

    assert_eq!(formats, [FormatId::V99, FormatId::V105]);
    assert_eq!(formats.map(FormatId::version), [99, 105]);
    assert_eq!(Save::supported_output_formats(), formats);
}

#[test]
fn fallback_for_unknown_version_prefers_edition_hint() {
    assert_eq!(
        FormatId::fallback_for_unknown_version(96, Some(ExpansionType::RotW)),
        FormatId::V105
    );
    assert_eq!(
        FormatId::fallback_for_unknown_version(200, Some(ExpansionType::Classic)),
        FormatId::V99
    );
}

#[test]
fn fallback_for_unknown_version_uses_closest_when_hint_missing() {
    assert_eq!(FormatId::fallback_for_unknown_version(96, None), FormatId::V99);
    assert_eq!(FormatId::fallback_for_unknown_version(104, None), FormatId::V105);
}

#[test]
fn detect_edition_hint_from_reserved_markers() {
    let mut v105_like =
        vec![0u8; CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_TWO + 1];
    v105_like
        [CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_ONE] =
        0x10;
    v105_like
        [CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_TWO] =
        0x1E;
    assert_eq!(detect_edition_hint(&v105_like), Some(ExpansionType::RotW));

    let mut v99_like =
        vec![0u8; CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_TWO + 1];
    v99_like
        [CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_ONE] =
        0x10;
    v99_like
        [CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_TWO] =
        0x1E;
    assert_eq!(detect_edition_hint(&v99_like), Some(ExpansionType::Classic));
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

#[test]
fn encode_v105_clears_legacy_expansion_status_bit() {
    let mut save = Save::new(FormatId::V105, Class::Barbarian);
    save.character.status = crate::character::Status::from(0b0010_0000);

    let encoded = encode(&save, FormatId::V105).expect("v105 save should encode");
    let encoded_status = encoded[CHARACTER_SECTION_START + OFFSET_STATUS];

    assert_eq!(
        encoded_status & 0b0010_0000,
        0,
        "v105 encode should clear legacy expansion status bit"
    );
}
