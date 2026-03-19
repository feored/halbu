use crate::character::decode_for_format as decode_character_for_format;
use crate::{IssueKind, IssueSeverity, ParseHardError, ParseIssue, SaveSummary, Strictness};

use super::layout::{
    expansion_type_from_decoded_character, layout_for_decode, push_issue, range_readable,
    read_version, section_name_option, SIGNATURE, SIGNATURE_RANGE, VERSION_RANGE,
};
use super::FormatId;

/// Summarize top-level header and character fields for list views.
pub(crate) fn summarize(
    bytes: &[u8],
    strictness: Strictness,
) -> Result<SaveSummary, ParseHardError> {
    let mut summary = SaveSummary::default();
    let mut issues: Vec<ParseIssue> = Vec::new();

    if !range_readable(bytes, &SIGNATURE_RANGE, "signature", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot summarize save: signature section is truncated.".to_string(),
            })
        } else {
            summary.issues = issues;
            Ok(summary)
        };
    }

    if bytes[SIGNATURE_RANGE.start..SIGNATURE_RANGE.end] != SIGNATURE {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::InvalidSignature,
            section_name_option("signature"),
            format!(
                "Invalid signature: expected {SIGNATURE:X?}, found {:X?}.",
                &bytes[SIGNATURE_RANGE.start..SIGNATURE_RANGE.end]
            ),
            Some(SIGNATURE_RANGE.start),
            Some(SIGNATURE_RANGE.end - SIGNATURE_RANGE.start),
            Some(SIGNATURE_RANGE.end - SIGNATURE_RANGE.start),
        );

        if strictness == Strictness::Strict {
            return Err(ParseHardError {
                message: "Cannot summarize save: invalid signature in strict mode.".to_string(),
            });
        }
    }

    if !range_readable(bytes, &VERSION_RANGE, "version", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot summarize save: version section is truncated.".to_string(),
            })
        } else {
            summary.issues = issues;
            Ok(summary)
        };
    }

    let version = read_version(bytes);
    let detected_format = FormatId::from_version(version).unwrap_or(FormatId::Unknown(version));
    summary.version = Some(version);
    summary.format = Some(detected_format);
    summary.edition = detected_format.edition();

    let selected_layout = layout_for_decode(detected_format, bytes, strictness, &mut issues)?;
    let character_range = selected_layout.character_range();
    if !range_readable(bytes, &character_range, "character", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot summarize save: character section is truncated.".to_string(),
            })
        } else {
            summary.issues = issues;
            Ok(summary)
        };
    }

    match decode_character_for_format(
        selected_layout.format_id(),
        &bytes[character_range.start..character_range.end],
    ) {
        Ok(character) => {
            let expansion_type =
                expansion_type_from_decoded_character(selected_layout.format_id(), &character);
            let title = character.title_d2r(expansion_type).map(str::to_string);
            let class = character.class;
            let level = character.level();
            let name = character.name;
            summary.expansion_type = Some(expansion_type);
            summary.name = Some(name);
            summary.class = Some(class);
            summary.level = Some(level);
            summary.title = title;
        }
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                section_name_option("character"),
                parse_error.to_string(),
                Some(character_range.start),
                Some(selected_layout.character_length()),
                Some(selected_layout.character_length()),
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot summarize save: character section payload is invalid."
                        .to_string(),
                });
            }
        }
    }

    summary.issues = issues;
    Ok(summary)
}
