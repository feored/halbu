use crate::attributes::Attributes;
use crate::character::decode_for_format as decode_character_for_format;
use crate::items;
use crate::npcs::Placeholder as NPCs;
use crate::quests::Quests;
use crate::skills::{SkillPoints, SKILLS_SECTION_LENGTH};
use crate::utils::BytePosition;
use crate::waypoints::Waypoints;
use crate::{
    GameEdition, IssueKind, IssueSeverity, ParseHardError, ParseIssue, ParsedSave, Save, Strictness,
};

use super::edition_hint::detect_edition_hint;
use super::layout::{
    checksum_metadata, expansion_type_from_decoded_character, layout_for_decode, push_issue,
    range_readable, read_version, section_name_option, IssueContext, NPCS_LENGTH, QUESTS_LENGTH,
    SIGNATURE, SIGNATURE_RANGE, SKILLS_LENGTH, VERSION_RANGE, WAYPOINTS_LENGTH,
};
use super::FormatId;

/// Decode a save with configurable strictness.
pub(crate) fn decode(bytes: &[u8], strictness: Strictness) -> Result<ParsedSave, ParseHardError> {
    let mut parsed_save = Save::default();
    let mut issues: Vec<ParseIssue> = Vec::new();
    let (header_checksum, computed_checksum) = checksum_metadata(bytes);
    let mut detected_format = parsed_save.format();
    let mut decoded_layout = FormatId::V99;
    let mut edition_hint: Option<GameEdition> = None;
    let finalize = |save: Save,
                    issues: Vec<ParseIssue>,
                    detected_format: FormatId,
                    decoded_layout: FormatId,
                    edition_hint: Option<GameEdition>| ParsedSave {
        save,
        detected_format,
        decoded_layout,
        edition_hint,
        issues,
        header_checksum,
        computed_checksum,
    };

    if !range_readable(bytes, &SIGNATURE_RANGE, "signature", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: signature section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    if bytes[SIGNATURE_RANGE.start..SIGNATURE_RANGE.end] != SIGNATURE {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::InvalidSignature,
            IssueContext {
                section_name: section_name_option("signature"),
                message: format!(
                    "Invalid signature: expected {SIGNATURE:X?}, found {:X?}.",
                    &bytes[SIGNATURE_RANGE.start..SIGNATURE_RANGE.end]
                ),
                offset: Some(SIGNATURE_RANGE.start),
                expected: Some(SIGNATURE_RANGE.end - SIGNATURE_RANGE.start),
                found: Some(SIGNATURE_RANGE.end - SIGNATURE_RANGE.start),
            },
        );

        if strictness == Strictness::Strict {
            return Err(ParseHardError {
                message: "Cannot parse save: invalid signature in strict mode.".to_string(),
            });
        }
    }

    if !range_readable(bytes, &VERSION_RANGE, "version", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: version section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    let version = read_version(bytes);
    detected_format = FormatId::from_version(version).unwrap_or(FormatId::Unknown(version));
    if matches!(detected_format, FormatId::Unknown(_)) {
        edition_hint = detect_edition_hint(bytes);
    }
    parsed_save.set_format(detected_format);

    let selected_layout = layout_for_decode(detected_format, bytes, strictness, &mut issues)?;
    decoded_layout = selected_layout.format_id();

    if bytes.len() < selected_layout.minimum_decode_size() {
        push_issue(
            &mut issues,
            IssueSeverity::Warning,
            IssueKind::InconsistentLayout,
            IssueContext {
                section_name: None,
                message: format!(
                    "File length ({}) is shorter than minimum expected ({}) for layout {:?}.",
                    bytes.len(),
                    selected_layout.minimum_decode_size(),
                    selected_layout.format_id()
                ),
                offset: Some(0),
                expected: Some(selected_layout.minimum_decode_size()),
                found: Some(bytes.len()),
            },
        );
    }

    let character_range = selected_layout.character_range();
    if !range_readable(bytes, &character_range, "character", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: character section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    let character_slice = &bytes[character_range.start..character_range.end];
    let character_parse_result =
        decode_character_for_format(selected_layout.format_id(), character_slice);

    match character_parse_result {
        Ok(parsed_character) => {
            parsed_save.character = parsed_character;
            let expansion_type = expansion_type_from_decoded_character(
                selected_layout.format_id(),
                &parsed_save.character,
            );
            parsed_save.set_expansion_type_for_format(selected_layout.format_id(), expansion_type);
        }
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                IssueContext {
                    section_name: section_name_option("character"),
                    message: parse_error.to_string(),
                    offset: Some(character_range.start),
                    expected: Some(selected_layout.character_length()),
                    found: Some(selected_layout.character_length()),
                },
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: character section payload is invalid.".to_string(),
                });
            }
        }
    }

    let quests_range = selected_layout.quests_range();
    if !range_readable(bytes, &quests_range, "quests", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: quests section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }
    match Quests::parse(&bytes[quests_range.start..quests_range.end]) {
        Ok(parsed_quests) => parsed_save.quests = parsed_quests,
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                IssueContext {
                    section_name: section_name_option("quests"),
                    message: parse_error.to_string(),
                    offset: Some(quests_range.start),
                    expected: Some(QUESTS_LENGTH),
                    found: Some(QUESTS_LENGTH),
                },
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: quests section payload is invalid.".to_string(),
                });
            }
        }
    }

    let waypoints_range = selected_layout.waypoints_range();
    if !range_readable(bytes, &waypoints_range, "waypoints", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: waypoints section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }
    match Waypoints::parse(&bytes[waypoints_range.start..waypoints_range.end]) {
        Ok(parsed_waypoints) => parsed_save.waypoints = parsed_waypoints,
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                IssueContext {
                    section_name: section_name_option("waypoints"),
                    message: parse_error.to_string(),
                    offset: Some(waypoints_range.start),
                    expected: Some(WAYPOINTS_LENGTH),
                    found: Some(WAYPOINTS_LENGTH),
                },
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: waypoints section payload is invalid.".to_string(),
                });
            }
        }
    }

    let npcs_range = selected_layout.npcs_range();
    if !range_readable(bytes, &npcs_range, "npcs", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: NPC section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }
    match NPCs::parse(&bytes[npcs_range.start..npcs_range.end]) {
        Ok(parsed_npcs) => parsed_save.npcs = parsed_npcs,
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                IssueContext {
                    section_name: section_name_option("npcs"),
                    message: parse_error.to_string(),
                    offset: Some(npcs_range.start),
                    expected: Some(NPCS_LENGTH),
                    found: Some(NPCS_LENGTH),
                },
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: NPC section payload is invalid.".to_string(),
                });
            }
        }
    }

    if bytes.len() <= selected_layout.attributes_offset() {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::InconsistentLayout,
            IssueContext {
                section_name: section_name_option("attributes"),
                message: format!(
                    "Attributes offset {} is beyond file length {}.",
                    selected_layout.attributes_offset(),
                    bytes.len()
                ),
                offset: Some(selected_layout.attributes_offset()),
                expected: Some(1),
                found: Some(0),
            },
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: attributes offset is out of bounds.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    let attribute_bytes = &bytes[selected_layout.attributes_offset()..];
    if attribute_bytes.len() < 2 {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::TruncatedSection,
            IssueContext {
                section_name: section_name_option("attributes"),
                message: format!(
                    "Attributes section is too short: expected at least 2 bytes, found {}.",
                    attribute_bytes.len()
                ),
                offset: Some(selected_layout.attributes_offset()),
                expected: Some(2),
                found: Some(attribute_bytes.len()),
            },
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: attributes section header is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    let mut byte_position = BytePosition::default();
    match Attributes::parse(attribute_bytes, &mut byte_position) {
        Ok(parsed_attributes) => {
            parsed_save.attributes = parsed_attributes;
        }
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                IssueContext {
                    section_name: section_name_option("attributes"),
                    message: parse_error.to_string(),
                    offset: Some(selected_layout.attributes_offset()),
                    expected: None,
                    found: None,
                },
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: attributes section payload is invalid."
                        .to_string(),
                });
            }

            return Ok(finalize(
                parsed_save,
                issues,
                detected_format,
                decoded_layout,
                edition_hint,
            ));
        }
    }

    let skills_offset = selected_layout.attributes_offset() + byte_position.next_byte_offset();
    if (skills_offset + SKILLS_LENGTH) > bytes.len() {
        let expected_length = SKILLS_LENGTH;
        let found_length = bytes.len().saturating_sub(skills_offset);
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::TruncatedSection,
            IssueContext {
                section_name: section_name_option("skills"),
                message: format!(
                    "Skills section is truncated at offset {skills_offset}. Expected {expected_length}, found {found_length}."
                ),
                offset: Some(skills_offset),
                expected: Some(expected_length),
                found: Some(found_length),
            },
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: skills section is truncated.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    match SkillPoints::parse(&bytes[skills_offset..(skills_offset + SKILLS_LENGTH)]) {
        Ok(parsed_skills) => {
            parsed_save.skills = parsed_skills;
        }
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                IssueContext {
                    section_name: section_name_option("skills"),
                    message: parse_error.to_string(),
                    offset: Some(skills_offset),
                    expected: Some(SKILLS_LENGTH),
                    found: Some(SKILLS_LENGTH),
                },
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: skills section payload is invalid.".to_string(),
                });
            }
        }
    }

    let items_offset = skills_offset + SKILLS_SECTION_LENGTH;
    if items_offset > bytes.len() {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::TruncatedSection,
            IssueContext {
                section_name: section_name_option("items"),
                message: format!(
                    "Items section offset {items_offset} is beyond file length {}.",
                    bytes.len()
                ),
                offset: Some(items_offset),
                expected: Some(1),
                found: Some(0),
            },
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: items section is out of bounds.".to_string(),
            })
        } else {
            Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
        };
    }

    parsed_save.items = items::parse(&bytes[items_offset..]);

    Ok(finalize(parsed_save, issues, detected_format, decoded_layout, edition_hint))
}
