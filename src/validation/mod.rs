use crate::character::mercenary::{
    level_from_experience as mercenary_level_from_experience, mercenary_name_count_for_variant_id,
    xp_rate_for_variant_id,
};
use crate::quests::{Quest, QuestFlag};
use crate::{Act, Difficulty, ExpansionType, Save};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use unicode_script::{Script, ScriptExtension};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationCode {
    InvalidCharacterName,
    UnknownClassId,
    CharacterLevelMismatch,
    CharacterLevelOutOfRange,
    ProgressionNonCanonical,
    ImpossibleDifficultySelection,
    ImpossibleActSelection,
    MercenaryDataWithoutHire,
    MercenaryVariantUnknown,
    MercenaryNameIdOutOfRange,
    MercenaryLevelImpossible,
    QuestStateImpossible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub code: ValidationCode,
    pub blocking: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        !self.has_blocking_issues()
    }

    pub fn has_blocking_issues(&self) -> bool {
        self.issues.iter().any(|issue| issue.blocking)
    }
}

fn issue(code: ValidationCode, message: impl Into<String>) -> ValidationIssue {
    ValidationIssue { code, blocking: true, message: message.into() }
}

fn warning(code: ValidationCode, message: impl Into<String>) -> ValidationIssue {
    ValidationIssue { code, blocking: false, message: message.into() }
}

fn has_mixed_name_scripts(name: &str) -> bool {
    let mut scripts = HashSet::new();

    for c in name.chars() {
        if !c.is_alphabetic() {
            continue;
        }

        for script in ScriptExtension::from(c).iter() {
            if matches!(script, Script::Common | Script::Inherited) {
                continue;
            }

            scripts.insert(script);
        }

        if scripts.len() > 1 {
            return true;
        }
    }

    false
}

fn validate_character_name(save: &Save, issues: &mut Vec<ValidationIssue>) {
    let name = save.character.name.as_str();
    let grapheme_count = UnicodeSegmentation::graphemes(name, true).count();
    let byte_count = name.len();

    if name.trim().is_empty() {
        issues.push(issue(ValidationCode::InvalidCharacterName, "Character name is empty."));
        return;
    }

    if name.chars().any(char::is_control) {
        issues.push(issue(
            ValidationCode::InvalidCharacterName,
            "Character name contains control characters.",
        ));
    }

    if grapheme_count < 2 {
        issues.push(issue(
            ValidationCode::InvalidCharacterName,
            "Character name must be at least 2 graphemes long.",
        ));
    }

    if grapheme_count > 15 {
        issues.push(issue(
            ValidationCode::InvalidCharacterName,
            "Character name exceeds the game's 15-grapheme limit.",
        ));
    }

    if byte_count > 48 {
        issues.push(issue(
            ValidationCode::InvalidCharacterName,
            "Character name exceeds the game's 48-byte name field.",
        ));
    }

    if has_mixed_name_scripts(name) {
        issues.push(warning(
            ValidationCode::InvalidCharacterName,
            "Character name uses mixed scripts.",
        ));
    }
}

fn validate_class(save: &Save, issues: &mut Vec<ValidationIssue>) {
    if let crate::Class::Unknown(class_id) = save.character.class {
        issues.push(issue(
            ValidationCode::UnknownClassId,
            format!("Unknown class id {class_id} is not recognized by the game."),
        ));
    }
}

fn validate_level_sync(save: &Save, issues: &mut Vec<ValidationIssue>) {
    let character_level = save.character.level();
    let attribute_level = save.attributes.level();

    if !(1..=99).contains(&character_level) || !(1..=99).contains(&attribute_level) {
        issues.push(issue(
            ValidationCode::CharacterLevelOutOfRange,
            format!(
                "Character level ({character_level}) and attributes level ({attribute_level}) must both be between 1 and 99."
            ),
        ));
    }

    if character_level != attribute_level {
        issues.push(issue(
            ValidationCode::CharacterLevelMismatch,
            format!(
                "Character level ({character_level}) does not match attributes level ({attribute_level})."
            ),
        ));
    }
}

fn minimum_progression_for(save: &Save) -> u8 {
    let completed_act_count = if save.expansion_type() == ExpansionType::Classic { 4 } else { 5 };

    match save.character.difficulty {
        Difficulty::Normal => 0,
        Difficulty::Nightmare => completed_act_count,
        Difficulty::Hell => completed_act_count * 2,
    }
}

fn validate_progression_floor(save: &Save, issues: &mut Vec<ValidationIssue>) {
    let minimum_progression = minimum_progression_for(save);

    if save.character.progression >= minimum_progression {
        return;
    }

    issues.push(warning(
        ValidationCode::ProgressionNonCanonical,
        format!(
            "Progression value {} is below the minimum expected for {:?} difficulty in {:?} mode.",
            save.character.progression,
            save.character.difficulty,
            save.expansion_type()
        ),
    ));
}

fn quest_reward_granted(quest: &Quest) -> bool {
    quest.state.contains(&QuestFlag::RewardGranted)
}

fn quest_completed(quest: &Quest) -> bool {
    quest_reward_granted(quest) || quest.state.contains(&QuestFlag::CompletedBefore)
}

fn quest_has_only_reward_granted(quest: &Quest) -> bool {
    quest.value() & !0x0001 == 0
}

fn current_difficulty_quests(save: &Save) -> &crate::quests::DifficultyQuests {
    match save.character.difficulty {
        Difficulty::Normal => &save.quests.normal,
        Difficulty::Nightmare => &save.quests.nightmare,
        Difficulty::Hell => &save.quests.hell,
    }
}

fn difficulty_unlocked(save: &Save, difficulty: Difficulty) -> bool {
    let unlock_act = if save.expansion_type() == ExpansionType::Classic {
        &save.quests.normal.act4.completion
    } else {
        &save.quests.normal.act5.completion
    };
    let nightmare_unlock_act = if save.expansion_type() == ExpansionType::Classic {
        &save.quests.nightmare.act4.completion
    } else {
        &save.quests.nightmare.act5.completion
    };

    match difficulty {
        Difficulty::Normal => true,
        // Act V completions on real saves often retain completion through
        // `CompletedBefore` rather than `RewardGranted`, especially when reset
        // state is present. Accept either marker here.
        Difficulty::Nightmare => quest_completed(unlock_act),
        Difficulty::Hell => quest_completed(nightmare_unlock_act),
    }
}

fn act_unlocked(save: &Save, act: Act) -> bool {
    let quests = current_difficulty_quests(save);

    match act {
        Act::Act1 => true,
        Act::Act2 => quest_reward_granted(&quests.act1.completion),
        Act::Act3 => quest_reward_granted(&quests.act2.completion),
        Act::Act4 => quest_reward_granted(&quests.act3.completion),
        Act::Act5 => quest_reward_granted(&quests.act4.q2),
    }
}

fn validate_progression(save: &Save, issues: &mut Vec<ValidationIssue>) {
    validate_progression_floor(save, issues);

    if !difficulty_unlocked(save, save.character.difficulty) {
        issues.push(issue(
            ValidationCode::ImpossibleDifficultySelection,
            format!(
                "Difficulty {:?} is not unlocked by the quest state.",
                save.character.difficulty
            ),
        ));
    }

    if !act_unlocked(save, save.character.act) {
        issues.push(issue(
            ValidationCode::ImpossibleActSelection,
            format!(
                "Act {:?} is not unlocked for {:?}.",
                save.character.act, save.character.difficulty
            ),
        ));
    }
}

fn validate_quest_state(save: &Save, issues: &mut Vec<ValidationIssue>) {
    let true_false_quests = [
        ("normal act1 prologue", &save.quests.normal.act1.prologue),
        ("normal act1 completion", &save.quests.normal.act1.completion),
        ("normal act2 prologue", &save.quests.normal.act2.prologue),
        ("normal act2 completion", &save.quests.normal.act2.completion),
        ("normal act3 prologue", &save.quests.normal.act3.prologue),
        ("normal act3 completion", &save.quests.normal.act3.completion),
        ("normal act4 prologue", &save.quests.normal.act4.prologue),
        ("normal act4 completion", &save.quests.normal.act4.completion),
        ("normal act5 prologue", &save.quests.normal.act5.prologue),
        ("nightmare act1 prologue", &save.quests.nightmare.act1.prologue),
        ("nightmare act1 completion", &save.quests.nightmare.act1.completion),
        ("nightmare act2 prologue", &save.quests.nightmare.act2.prologue),
        ("nightmare act2 completion", &save.quests.nightmare.act2.completion),
        ("nightmare act3 prologue", &save.quests.nightmare.act3.prologue),
        ("nightmare act3 completion", &save.quests.nightmare.act3.completion),
        ("nightmare act4 prologue", &save.quests.nightmare.act4.prologue),
        ("nightmare act4 completion", &save.quests.nightmare.act4.completion),
        ("nightmare act5 prologue", &save.quests.nightmare.act5.prologue),
        ("hell act1 prologue", &save.quests.hell.act1.prologue),
        ("hell act1 completion", &save.quests.hell.act1.completion),
        ("hell act2 prologue", &save.quests.hell.act2.prologue),
        ("hell act2 completion", &save.quests.hell.act2.completion),
        ("hell act3 prologue", &save.quests.hell.act3.prologue),
        ("hell act3 completion", &save.quests.hell.act3.completion),
        ("hell act4 prologue", &save.quests.hell.act4.prologue),
        ("hell act4 completion", &save.quests.hell.act4.completion),
        ("hell act5 prologue", &save.quests.hell.act5.prologue),
    ];

    for (label, quest) in true_false_quests {
        if !quest_has_only_reward_granted(quest) {
            issues.push(issue(
                ValidationCode::QuestStateImpossible,
                format!("{label} should only use RewardGranted."),
            ));
        }
    }

    for (difficulty_label, quests) in [
        ("normal", &save.quests.normal),
        ("nightmare", &save.quests.nightmare),
        ("hell", &save.quests.hell),
    ] {
        if quest_reward_granted(&quests.act4.completion) && !quest_reward_granted(&quests.act4.q2) {
            issues.push(issue(
                ValidationCode::QuestStateImpossible,
                format!("{difficulty_label} Act IV completion requires Terror's End."),
            ));
        }
    }
}

fn validate_mercenary_level(save: &Save, issues: &mut Vec<ValidationIssue>) {
    let mercenary = save.character.mercenary;
    if !mercenary.is_hired() {
        if mercenary.has_data_without_hire() {
            issues.push(issue(
                ValidationCode::MercenaryDataWithoutHire,
                "Mercenary name, type, experience, and death state must be zero when no mercenary is hired.",
            ));
        }
        return;
    }

    let Some(xp_rate) = xp_rate_for_variant_id(mercenary.variant_id) else {
        issues.push(issue(
            ValidationCode::MercenaryVariantUnknown,
            format!("Mercenary variant id {} is not recognized.", mercenary.variant_id),
        ));
        return;
    };

    let mercenary_level = mercenary_level_from_experience(mercenary.experience, xp_rate);
    let player_level = save.attributes.level();

    if let Some(name_count) = mercenary_name_count_for_variant_id(mercenary.variant_id) {
        if mercenary.name_id as usize >= name_count {
            issues.push(issue(
                ValidationCode::MercenaryNameIdOutOfRange,
                format!(
                    "Mercenary name id {} is out of range for this mercenary type.",
                    mercenary.name_id
                ),
            ));
        }
    }

    if mercenary_level == 0 {
        issues.push(issue(
            ValidationCode::MercenaryLevelImpossible,
            format!(
                "Mercenary experience ({}) does not map to a valid level for variant {}.",
                mercenary.experience, mercenary.variant_id
            ),
        ));
    } else if mercenary_level > player_level {
        issues.push(issue(
            ValidationCode::MercenaryLevelImpossible,
            format!(
                "Mercenary level ({mercenary_level}) is higher than player level ({player_level})."
            ),
        ));
    }
}

/// Build a validation report for a save model.
pub(crate) fn build_validation_report(save: &Save) -> ValidationReport {
    let mut report = ValidationReport::default();

    validate_class(save, &mut report.issues);
    validate_character_name(save, &mut report.issues);
    validate_level_sync(save, &mut report.issues);
    validate_progression(save, &mut report.issues);
    validate_quest_state(save, &mut report.issues);
    validate_mercenary_level(save, &mut report.issues);

    report
}

#[cfg(test)]
mod tests;
