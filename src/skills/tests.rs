use crate::skills::{
    d2r_skill_index, d2r_skill_name, NamedSkillError, SkillPoints, SKILLS_SECTION_LENGTH,
};
use crate::Class;

#[test]
fn test_parse_and_write() {
    let bytes = [
        0x69, 0x66, 0x00, 0x01, 0x00, 0x14, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x14, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x14,
    ];

    let skill_points = SkillPoints::parse(&bytes).expect("skills should parse");
    assert_eq!(skill_points.points.len(), SKILLS_SECTION_LENGTH - 2);
    assert_eq!(skill_points.points[0], 0x00);
    assert_eq!(skill_points.points[3], 0x14);
    assert_eq!(skill_points.to_bytes(), bytes);
}

#[test]
fn test_set_get() {
    let mut skill_points = SkillPoints::default();
    skill_points.set(5, 42);
    assert_eq!(skill_points.get(5), 42);
}

#[test]
fn test_named_lookup_d2r_roundtrip() {
    assert_eq!(d2r_skill_index(Class::Barbarian, "bash").expect("bash should exist"), 0);
    assert_eq!(d2r_skill_name(Class::Barbarian, 0).expect("index 0 should exist"), "Bash");
    assert_eq!(
        d2r_skill_index(Class::Warlock, "miasma bolt").expect("miasma bolt should exist"),
        22
    );
    assert_eq!(d2r_skill_name(Class::Warlock, 22).expect("index 22 should exist"), "Miasma Bolt");

    // Name lookup is normalized (case/spacing/punctuation-insensitive).
    assert_eq!(
        d2r_skill_index(Class::Paladin, "fist-of-the-heavens").expect("fotoh should exist"),
        25
    );
}

#[test]
fn test_named_skill_set_get() {
    let mut skill_points = SkillPoints::default();
    skill_points.set_by_name_d2r(Class::Sorceress, "Teleport", 20).expect("teleport should exist");
    assert_eq!(
        skill_points.get_by_name_d2r(Class::Sorceress, "teleport").expect("teleport should exist"),
        20
    );
}

#[test]
fn test_named_lookup_errors_for_unknown_name_and_class() {
    let unknown_skill = d2r_skill_index(Class::Sorceress, "Made Up Spell");
    assert!(matches!(unknown_skill, Err(NamedSkillError::UnknownSkillName { .. })));

    let unsupported_class = d2r_skill_index(Class::Unknown(0xFF), "Bash");
    assert!(matches!(
        unsupported_class,
        Err(NamedSkillError::UnsupportedClass(Class::Unknown(0xFF)))
    ));
}
