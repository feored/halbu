use super::*;
use crate::character::v105::{
    MODE_CLASSIC, MODE_EXPANSION, MODE_ROTW, OFFSET_STATUS,
};
use crate::{Class, ExpansionType, GameEdition, Save, Strictness};

fn encode_v105_with_mode(mode_marker: u8, mercenary_hired: bool) -> Vec<u8> {
    let mut save = Save::new(FormatId::V105, Class::Barbarian);
    let expansion_type =
        crate::character::v105::expansion_type_from_mode_marker(mode_marker).unwrap_or(ExpansionType::RotW);
    save.set_expansion_type(expansion_type);
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
        FormatId::fallback_for_unknown_version(96, Some(GameEdition::RotW)),
        FormatId::V105
    );
    assert_eq!(
        FormatId::fallback_for_unknown_version(200, Some(GameEdition::D2RLegacy)),
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
    assert_eq!(detect_edition_hint(&v105_like), Some(GameEdition::RotW));

    let mut v99_like =
        vec![0u8; CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_TWO + 1];
    v99_like
        [CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_ONE] =
        0x10;
    v99_like
        [CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_TWO] =
        0x1E;
    assert_eq!(detect_edition_hint(&v99_like), Some(GameEdition::D2RLegacy));
}

#[test]
fn decode_unknown_version_prefers_v99_when_markers_match_legacy() {
    let mut bytes = include_bytes!("../../assets/test/Joe.d2s").to_vec();
    bytes[4..8].copy_from_slice(&104u32.to_le_bytes());

    let parsed =
        decode_with_strictness(&bytes, Strictness::Lax).expect("unknown-version legacy save should parse");

    assert_eq!(parsed.save.format_id(), FormatId::Unknown(104));
    assert_eq!(parsed.save.character.name, "Joe");
}

#[test]
fn decode_unknown_version_prefers_v105_when_markers_match_rotw() {
    let mut bytes = include_bytes!("../../assets/test/Warlock_v105.d2s").to_vec();
    bytes[4..8].copy_from_slice(&96u32.to_le_bytes());

    let parsed =
        decode_with_strictness(&bytes, Strictness::Lax).expect("unknown-version rotw save should parse");

    assert_eq!(parsed.save.format_id(), FormatId::Unknown(96));
    assert_eq!(parsed.save.character.class, Class::Warlock);
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
fn encode_v105_preserves_legacy_expansion_status_bit() {
    let mut save = Save::new(FormatId::V105, Class::Barbarian);
    save.character.status = crate::character::Status::from(0b0010_0000);

    let encoded = encode(&save, FormatId::V105).expect("v105 save should encode");
    let encoded_status = encoded[CHARACTER_SECTION_START + OFFSET_STATUS];

    assert_eq!(
        encoded_status & 0b0010_0000,
        0b0010_0000,
        "v105 encode should preserve legacy expansion status bit"
    );
}

#[test]
fn encode_v99_rejects_rotw_expansion_type() {
    let mut save = Save::new(FormatId::V99, Class::Barbarian);
    save.set_expansion_type(ExpansionType::RotW);

    let error = encode(&save, FormatId::V99).expect_err("v99 should reject RotW expansion type");
    assert!(
        error
            .to_string()
            .contains("Cannot encode RotW expansion type as v99"),
        "unexpected error message: {error}"
    );
}

#[test]
fn decode_v99_maps_status_bit_to_expansion_type() {
    let mut classic = include_bytes!("../../assets/test/Joe.d2s").to_vec();
    classic[CHARACTER_SECTION_START + crate::character::v99::OFFSET_STATUS] &= !0b0010_0000;
    let parsed_classic =
        decode_with_strictness(&classic, Strictness::Strict).expect("classic v99 should parse");
    assert_eq!(parsed_classic.save.expansion_type(), ExpansionType::Classic);

    let mut expansion = include_bytes!("../../assets/test/Joe.d2s").to_vec();
    expansion[CHARACTER_SECTION_START + crate::character::v99::OFFSET_STATUS] |= 0b0010_0000;
    let parsed_expansion =
        decode_with_strictness(&expansion, Strictness::Strict).expect("expansion v99 should parse");
    assert_eq!(parsed_expansion.save.expansion_type(), ExpansionType::Expansion);
}

#[test]
fn decode_v105_maps_mode_marker_to_expansion_type() {
    let mut classic = include_bytes!("../../assets/test/barbrotw_v105.d2s").to_vec();
    classic[CHARACTER_SECTION_START + crate::character::v105::OFFSET_MODE_MARKER] = MODE_CLASSIC;
    let parsed_classic =
        decode_with_strictness(&classic, Strictness::Strict).expect("classic v105 should parse");
    assert_eq!(parsed_classic.save.expansion_type(), ExpansionType::Classic);

    let mut expansion = include_bytes!("../../assets/test/barbrotw_v105.d2s").to_vec();
    expansion[CHARACTER_SECTION_START + crate::character::v105::OFFSET_MODE_MARKER] =
        MODE_EXPANSION;
    let parsed_expansion =
        decode_with_strictness(&expansion, Strictness::Strict).expect("expansion v105 should parse");
    assert_eq!(parsed_expansion.save.expansion_type(), ExpansionType::Expansion);

    let mut rotw = include_bytes!("../../assets/test/barbrotw_v105.d2s").to_vec();
    rotw[CHARACTER_SECTION_START + crate::character::v105::OFFSET_MODE_MARKER] = MODE_ROTW;
    let parsed_rotw =
        decode_with_strictness(&rotw, Strictness::Strict).expect("rotw v105 should parse");
    assert_eq!(parsed_rotw.save.expansion_type(), ExpansionType::RotW);
}
