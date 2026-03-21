use crate::{CompatibilityCode, CompatibilityIssue, EncodeError, ExpansionType, GameEdition, Save};

use super::FormatId;

fn class_compatibility_issues(
    class: crate::Class,
    expansion_type: ExpansionType,
    target: FormatId,
) -> Vec<CompatibilityIssue> {
    let mut issues = Vec::new();

    match class {
        crate::Class::Unknown(class_id) => {
            if matches!(target, FormatId::V99 | FormatId::V105) {
                issues.push(CompatibilityIssue {
                    code: CompatibilityCode::UnknownClassRequiresKnownTarget,
                    blocking: true,
                    message: format!(
                        "Unknown class id {class_id} cannot be converted to {target:?}."
                    ),
                });
            }
        }
        crate::Class::Warlock => {
            if target.edition().is_some_and(|edition| edition != GameEdition::RotW) {
                issues.push(CompatibilityIssue {
                    code: CompatibilityCode::WarlockRequiresRotW,
                    blocking: true,
                    message: "Warlock can only be encoded as RotW.".to_string(),
                });
            }

            if expansion_type != ExpansionType::RotW {
                issues.push(CompatibilityIssue {
                    code: CompatibilityCode::WarlockRequiresRotWExpansion,
                    blocking: true,
                    message: "Warlock requires RotW expansion mode.".to_string(),
                });
            }
        }
        crate::Class::Druid | crate::Class::Assassin => {
            if expansion_type == ExpansionType::Classic {
                issues.push(CompatibilityIssue {
                    code: CompatibilityCode::ExpansionClassRequiresExpansionMode,
                    blocking: true,
                    message: "Druid and Assassin cannot be encoded in Classic mode.".to_string(),
                });
            }
        }
        _ => {}
    }

    issues
}

pub(crate) fn compatibility_issues(save: &Save, target: FormatId) -> Vec<CompatibilityIssue> {
    let mut issues =
        class_compatibility_issues(save.character.class, save.expansion_type(), target);

    if save
        .items
        .mercenary_hire_state_changed(save.character.mercenary.is_hired())
    {
        issues.push(CompatibilityIssue {
            code: CompatibilityCode::MercenaryHireStateToggleUnsupported,
            blocking: true,
            message: "Changing mercenary.id between 0 (no mercenary hired) and nonzero (mercenary hired) is not supported by this version of halbu, because the mercenary item subsection in the raw item tail is not rewritten yet.".to_string(),
        });
    }

    if target.edition().is_some_and(|edition| edition != GameEdition::RotW)
        && save.expansion_type() == ExpansionType::RotW
    {
        issues.push(CompatibilityIssue {
            code: CompatibilityCode::RotWExpansionRequiresRotWEdition,
            blocking: true,
            message: "RotW expansion mode cannot be encoded to non-RotW formats.".to_string(),
        });
    }

    issues
}

pub(crate) fn validate_encode_compatibility(
    save: &Save,
    target: FormatId,
) -> Result<(), EncodeError> {
    let errors: Vec<CompatibilityIssue> =
        compatibility_issues(save, target).into_iter().filter(|issue| issue.blocking).collect();
    if errors.is_empty() {
        return Ok(());
    }

    let message = errors.iter().map(|issue| issue.message.as_str()).collect::<Vec<_>>().join(" ");
    Err(EncodeError::new(message))
}
