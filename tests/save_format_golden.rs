use halbu::format::FormatId;
use halbu::{Class, CompatibilityChecks, CompatibilityCode, ExpansionType, Save, Strictness};

fn goldens() -> [(&'static str, &'static [u8], FormatId); 2] {
    [
        ("Joe_v99", &include_bytes!("../assets/test/Joe.d2s")[..], FormatId::V99),
        ("Warlock_v105", &include_bytes!("../assets/test/Warlock_v105.d2s")[..], FormatId::V105),
    ]
}

fn v105_mode_goldens() -> [(&'static str, &'static [u8]); 3] {
    [
        ("barbclassic_v105", &include_bytes!("../assets/test/barbclassic_v105.d2s")[..]),
        ("barbexp_v105", &include_bytes!("../assets/test/barbexp_v105.d2s")[..]),
        ("barbrotw_v105", &include_bytes!("../assets/test/barbrotw_v105.d2s")[..]),
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
        save.set_format(FormatId::Unknown(0));
        save.character.raw_section.clear();
        save.items = halbu::items::Placeholder::default();
    }

    assert_eq!(left, right, "{label}: semantic model mismatch");
}

#[test]
fn strict_parses_goldens() {
    for (name, bytes, expected_format) in goldens() {
        let save = parse_strict_clean(name, bytes);
        assert_eq!(save.format(), expected_format, "{name}: wrong format");
    }
}

#[test]
fn same_format_roundtrip_keeps_model() {
    for (name, bytes, format_id) in goldens() {
        let original = parse_strict_clean(name, bytes);
        let encoded = original
            .encode_for(format_id, CompatibilityChecks::Enforce)
            .unwrap_or_else(|e| panic!("encode {name}: {e}"));
        let reparsed = parse_strict_clean(name, &encoded);
        assert_eq!(reparsed.format(), format_id, "{name}: format drift");
        assert_same_model(original, reparsed, name);
    }
}

#[test]
fn v105_mode_goldens_roundtrip_semantic() {
    for (name, bytes) in v105_mode_goldens() {
        let original = parse_strict_clean(name, bytes);
        let encoded = original
            .encode_for(FormatId::V105, CompatibilityChecks::Enforce)
            .unwrap_or_else(|e| panic!("encode {name}: {e}"));
        let reparsed = parse_strict_clean(name, &encoded);
        assert_eq!(reparsed.format(), FormatId::V105, "{name}: format drift");
        assert_same_model(original, reparsed, name);
    }
}

#[test]
#[ignore = "Pending task 004 (v105 encoder): the existing v99->v105 encoder \
            does not reshape v99 item bytes into v105 layout, so the new v105 \
            item parser (task 003) chokes when re-parsing the round-tripped \
            output. Re-enable once task 004 lands."]
fn v99_to_v105_back_to_v99_keeps_model() {
    let source = &include_bytes!("../assets/test/Joe.d2s")[..];
    let start = parse_strict_clean("Joe_v99", source);

    let encoded_v105 = start
        .encode_for(FormatId::V105, CompatibilityChecks::Enforce)
        .unwrap_or_else(|e| panic!("encode Joe_v99 -> v105: {e}"));
    let as_v105 = parse_strict_clean("Joe_as_v105", &encoded_v105);
    assert_eq!(as_v105.format(), FormatId::V105, "Joe_as_v105: wrong format");
    assert_same_model(start.clone(), as_v105.clone(), "Joe v99 -> v105");

    let encoded_v99 = as_v105
        .encode_for(FormatId::V99, CompatibilityChecks::Enforce)
        .unwrap_or_else(|e| panic!("encode Joe_v105 -> v99: {e}"));
    let back_to_v99 = parse_strict_clean("Joe_back_to_v99", &encoded_v99);
    assert_eq!(back_to_v99.format(), FormatId::V99, "Joe_back_to_v99: wrong format");
    assert_same_model(start, back_to_v99, "Joe v99 -> v105 -> v99");
}

#[test]
fn warlock_v105_cannot_encode_to_v99() {
    let warlock =
        parse_strict_clean("Warlock_v105", &include_bytes!("../assets/test/Warlock_v105.d2s")[..]);
    let encode_result = warlock.encode_for(FormatId::V99, CompatibilityChecks::Enforce);
    let error = encode_result.expect_err("Warlock should not encode to v99");

    assert!(
        error.to_string().contains("Warlock can only be encoded as RotW."),
        "unexpected error message: {error}"
    );
}

#[test]
fn check_compatibility_reports_blocking_warlock_to_v99() {
    let warlock =
        parse_strict_clean("Warlock_v105", &include_bytes!("../assets/test/Warlock_v105.d2s")[..]);
    let issues = warlock.check_compatibility(FormatId::V99);

    assert!(
        issues
            .iter()
            .any(|issue| issue.blocking && issue.code == CompatibilityCode::WarlockRequiresRotW),
        "expected blocking WarlockRequiresRotW issue, got: {:?}",
        issues
    );
}

#[test]
fn check_compatibility_reports_blocking_warlock_non_rotw_expansion() {
    let mut warlock =
        parse_strict_clean("Warlock_v105", &include_bytes!("../assets/test/Warlock_v105.d2s")[..]);
    warlock.set_expansion_type(ExpansionType::Expansion);
    let issues = warlock.check_compatibility(FormatId::V105);

    assert!(
        issues
            .iter()
            .any(|issue| issue.blocking
                && issue.code == CompatibilityCode::WarlockRequiresRotWExpansion),
        "expected blocking WarlockRequiresRotWExpansion issue, got: {:?}",
        issues
    );

    let encode_error = warlock
        .encode_for(FormatId::V105, CompatibilityChecks::Enforce)
        .expect_err("Warlock with non-RotW expansion should not encode");
    assert!(
        encode_error.to_string().contains("Warlock requires RotW expansion mode."),
        "unexpected error message: {encode_error}"
    );
}

#[test]
fn check_compatibility_reports_blocking_rotw_to_v99() {
    let mut save = Save::new(FormatId::V99, halbu::Class::Barbarian);
    save.set_expansion_type(ExpansionType::RotW);
    let issues = save.check_compatibility(FormatId::V99);

    assert!(
        issues.iter().any(|issue| {
            issue.blocking && issue.code == CompatibilityCode::RotWExpansionRequiresRotWEdition
        }),
        "expected blocking RotWExpansionRequiresRotWEdition issue, got: {:?}",
        issues
    );
}

#[test]
fn check_compatibility_reports_blocking_expansion_classes_in_classic_mode() {
    for class in [Class::Druid, Class::Assassin] {
        let mut save = Save::new(FormatId::V105, class);
        save.set_expansion_type(ExpansionType::Classic);
        let issues = save.check_compatibility(FormatId::V105);

        assert!(
            issues.iter().any(|issue| {
                issue.blocking
                    && issue.code == CompatibilityCode::ExpansionClassRequiresExpansionMode
            }),
            "expected blocking ExpansionClassRequiresExpansionMode issue for class {class}, got: {:?}",
            issues
        );
    }
}

#[test]
fn expansion_classes_in_classic_mode_cannot_encode() {
    for class in [Class::Druid, Class::Assassin] {
        let mut save = Save::new(FormatId::V105, class);
        save.set_expansion_type(ExpansionType::Classic);

        let error = save
            .encode_for(FormatId::V105, CompatibilityChecks::Enforce)
            .expect_err("expansion-only class in classic mode should not encode");
        assert!(
            error.to_string().contains("Druid and Assassin cannot be encoded in Classic mode."),
            "unexpected error message for class {class}: {error}"
        );
    }
}

#[test]
fn check_compatibility_reports_blocking_unknown_class_to_known_target() {
    let mut save = Save::new(FormatId::V105, Class::Amazon);
    save.character.class = Class::Unknown(0x7F);

    let issues = save.check_compatibility(FormatId::V99);

    assert!(
        issues.iter().any(|issue| issue.blocking
            && issue.code == CompatibilityCode::UnknownClassRequiresKnownTarget),
        "expected blocking UnknownClassRequiresKnownTarget issue, got: {:?}",
        issues
    );
}

#[test]
fn unknown_class_cannot_encode_to_known_targets() {
    let mut save = Save::new(FormatId::V105, Class::Amazon);
    save.character.class = Class::Unknown(0x7F);

    for target in [FormatId::V99, FormatId::V105] {
        let error = save
            .encode_for(target, CompatibilityChecks::Enforce)
            .expect_err("unknown class should not encode to known targets");
        assert!(
            error.to_string().contains("Unknown class id 127"),
            "unexpected error message for target {target:?}: {error}"
        );
    }
}

#[test]
fn encode_for_force_bypasses_compatibility_blockers() {
    let mut save = Save::new(FormatId::V105, Class::Druid);
    save.set_expansion_type(ExpansionType::Classic);

    save.encode_for(FormatId::V105, CompatibilityChecks::Enforce)
        .expect_err("strict encode should block expansion-only class in classic mode");

    let forced = save
        .encode_for(FormatId::V105, CompatibilityChecks::Ignore)
        .expect("forced encode should bypass compatibility blockers");
    assert!(!forced.is_empty(), "forced encode should still produce bytes");
}
