use halbu::format::FormatId;
use halbu::{Save, Strictness};

fn goldens() -> [(&'static str, &'static [u8], FormatId); 2] {
    [
        ("Joe_v99", &include_bytes!("../assets/test/Joe.d2s")[..], FormatId::V99),
        (
            "Warlock_v105",
            &include_bytes!("../assets/test/Warlock_v105.d2s")[..],
            FormatId::V105,
        ),
    ]
}

fn v105_mode_goldens() -> [(&'static str, &'static [u8]); 3] {
    [
        (
            "barbclassic_v105",
            &include_bytes!("../assets/test/barbclassic_v105.d2s")[..],
        ),
        (
            "barbexp_v105",
            &include_bytes!("../assets/test/barbexp_v105.d2s")[..],
        ),
        (
            "barbrotw_v105",
            &include_bytes!("../assets/test/barbrotw_v105.d2s")[..],
        ),
    ]
}

fn parse_strict_clean(name: &str, bytes: &[u8]) -> Save {
    let parsed =
        Save::parse(bytes, Strictness::Strict).unwrap_or_else(|e| panic!("parse {name}: {e}"));
    assert!(parsed.issues.is_empty(), "{name}: unexpected parse issues: {:?}", parsed.issues);
    parsed.save
}

fn assert_same_model(left: Save, right: Save, label: &str) {
    let mut left = left;
    let mut right = right;

    for save in [&mut left, &mut right] {
        // Ignore format bookkeeping and raw/opaque payload preservation.
        save.version = 0;
        save.meta.format = FormatId::Unknown(0);
        save.character.raw_section.clear();
        save.items = halbu::items::Placeholder::default();
    }

    assert_eq!(left, right, "{label}: semantic model mismatch");
}

#[test]
fn strict_parses_goldens() {
    for (name, bytes, expected_format) in goldens() {
        let save = parse_strict_clean(name, bytes);
        assert_eq!(save.format_id(), expected_format, "{name}: wrong format");
    }
}

#[test]
fn same_format_roundtrip_keeps_model() {
    for (name, bytes, format_id) in goldens() {
        let original = parse_strict_clean(name, bytes);
        let encoded = original
            .to_bytes_for(format_id)
            .unwrap_or_else(|e| panic!("encode {name}: {e}"));
        let reparsed = parse_strict_clean(name, &encoded);
        assert_eq!(reparsed.format_id(), format_id, "{name}: format drift");
        assert_same_model(original, reparsed, name);
    }
}

#[test]
fn v105_mode_goldens_roundtrip_semantic() {
    for (name, bytes) in v105_mode_goldens() {
        let original = parse_strict_clean(name, bytes);
        let encoded = original
            .to_bytes_for(FormatId::V105)
            .unwrap_or_else(|e| panic!("encode {name}: {e}"));
        let reparsed = parse_strict_clean(name, &encoded);
        assert_eq!(reparsed.format_id(), FormatId::V105, "{name}: format drift");
        assert_same_model(original, reparsed, name);
    }
}

#[test]
fn v99_to_v105_back_to_v99_keeps_model() {
    let source = &include_bytes!("../assets/test/Joe.d2s")[..];
    let start = parse_strict_clean("Joe_v99", source);

    let encoded_v105 = start
        .to_bytes_for(FormatId::V105)
        .unwrap_or_else(|e| panic!("encode Joe_v99 -> v105: {e}"));
    let as_v105 = parse_strict_clean("Joe_as_v105", &encoded_v105);
    assert_eq!(as_v105.format_id(), FormatId::V105, "Joe_as_v105: wrong format");
    assert_same_model(start.clone(), as_v105.clone(), "Joe v99 -> v105");

    let encoded_v99 = as_v105
        .to_bytes_for(FormatId::V99)
        .unwrap_or_else(|e| panic!("encode Joe_v105 -> v99: {e}"));
    let back_to_v99 = parse_strict_clean("Joe_back_to_v99", &encoded_v99);
    assert_eq!(back_to_v99.format_id(), FormatId::V99, "Joe_back_to_v99: wrong format");
    assert_same_model(start, back_to_v99, "Joe v99 -> v105 -> v99");
}
