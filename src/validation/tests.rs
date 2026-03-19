use super::*;
use crate::quests::QuestFlag;
use crate::{Act, Class, Difficulty, ExpansionType, Save};

fn mercenary_experience_for_level(level: u8, xp_rate: u32) -> u32 {
    let level = u64::from(level);
    let value = u64::from(xp_rate) * level * level * (level + 1);
    value.min(u64::from(u32::MAX)) as u32
}

#[test]
fn validate_default_save_has_no_issues() {
    let save = Save::default();
    let report = build_validation_report(&save);

    assert!(report.issues.is_empty());
    assert!(report.is_valid());
}

#[test]
fn save_validate_matches_internal_helper() {
    let save = Save::default();
    assert_eq!(save.validate(), build_validation_report(&save));
}

#[test]
fn validate_reports_invalid_character_name() {
    let mut save = Save::default();
    save.character.name = "   ".to_string();

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::InvalidCharacterName));
}

#[test]
fn validate_reports_too_short_name() {
    let mut save = Save::default();
    save.character.name = "o".to_string();

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::InvalidCharacterName && issue.blocking));
}

#[test]
fn validate_reports_overlong_byte_name() {
    let mut save = Save::default();
    save.character.name = "😀😀😀😀😀😀😀😀😀😀😀😀😀".to_string();

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::InvalidCharacterName));
}

#[test]
fn validate_reports_mixed_script_name() {
    let mut save = Save::default();
    save.character.name = "あto".to_string();

    let report = build_validation_report(&save);
    let issue = report
        .issues
        .iter()
        .find(|issue| issue.code == ValidationCode::InvalidCharacterName)
        .expect("mixed script warning should be present");

    assert!(!issue.blocking);
    assert!(!report.has_blocking_issues());
}

#[test]
fn validate_reports_overlong_name() {
    let mut save = Save::default();
    save.character.name = "abcdefghijklmnop".to_string();

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::InvalidCharacterName));
}

#[test]
fn validate_reports_unknown_class_id() {
    let mut save = Save::default();
    save.character.class = Class::Unknown(0x7F);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::UnknownClassId));
}

#[test]
fn validate_reports_level_mismatch() {
    let mut save = Save::default();
    save.character.set_level(10);
    save.attributes.set_level(11);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::CharacterLevelMismatch));
}

#[test]
fn validate_reports_character_level_out_of_range() {
    let mut save = Save::default();
    save.set_level(120);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::CharacterLevelOutOfRange));
}

#[test]
fn validate_reports_progression_warning() {
    let mut save = Save::default();
    save.character.progression = 6;

    let report = build_validation_report(&save);
    let issue = report
        .issues
        .iter()
        .find(|issue| issue.code == ValidationCode::ProgressionNonCanonical)
        .expect("progression warning should be present");

    assert!(!issue.blocking);
    assert!(!report.has_blocking_issues());
}

#[test]
fn validate_reports_impossible_difficulty_selection() {
    let mut save = Save::default();
    save.character.difficulty = Difficulty::Hell;

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::ImpossibleDifficultySelection));
}

#[test]
fn validate_classic_saves_use_act_iv_for_difficulty_unlocks() {
    let mut save = Save::default();
    save.set_expansion_type(ExpansionType::Classic);
    save.character.difficulty = Difficulty::Nightmare;
    save.quests.normal.act4.completion.state.insert(QuestFlag::RewardGranted);

    let report = build_validation_report(&save);
    assert!(!report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::ImpossibleDifficultySelection));
}

#[test]
fn validate_reports_impossible_act_selection() {
    let mut save = Save::default();
    save.character.act = Act::Act5;

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::ImpossibleActSelection));
}

#[test]
fn validate_reports_impossible_quest_state() {
    let mut save = Save::default();
    save.quests.normal.act1.prologue.state.insert(QuestFlag::Started);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::QuestStateImpossible));
}

#[test]
fn validate_reports_act_iv_completion_without_terrors_end() {
    let mut save = Save::default();
    save.quests.normal.act4.completion.state.insert(QuestFlag::RewardGranted);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::QuestStateImpossible));
}

#[test]
fn validate_reports_mercenary_variant_unknown() {
    let mut save = Save::default();
    save.character.mercenary.id = 1;
    save.character.mercenary.variant_id = 99;
    save.character.mercenary.experience = 0;

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::MercenaryVariantUnknown));
}

#[test]
fn validate_reports_mercenary_name_id_out_of_range() {
    let mut save = Save::default();
    save.character.mercenary.id = 1;
    save.character.mercenary.variant_id = 13;
    save.character.mercenary.name_id = 99;
    save.character.mercenary.experience = mercenary_experience_for_level(1, 130);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::MercenaryNameIdOutOfRange));
}

#[test]
fn validate_reports_mercenary_level_above_player_level() {
    let mut save = Save::default();
    save.set_level(10);
    save.character.mercenary.id = 1;
    save.character.mercenary.variant_id = 13;
    save.character.mercenary.experience = mercenary_experience_for_level(20, 130);

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::MercenaryLevelImpossible));
}

#[test]
fn validate_reports_impossible_mercenary_experience() {
    let mut save = Save::default();
    save.set_level(99);
    save.character.mercenary.id = 1;
    save.character.mercenary.variant_id = 13;
    save.character.mercenary.experience = 0;

    let report = build_validation_report(&save);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == ValidationCode::MercenaryLevelImpossible));
}
