//! Top-level save layout detection and orchestration.
//!
//! This module wires section codecs together and handles strict vs lax parsing.
//! Layout internals are public for advanced tooling/tests but are not considered
//! stable API surface.

use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::attributes::Attributes;
use crate::character::{
    decode_for_format as decode_character_for_format,
    encode_for_format as encode_character_for_format,
    expected_length_for_format as character_length_for_format,
};
use crate::items;
use crate::npcs::Placeholder as NPCs;
use crate::quests::Quests;
use crate::skills::{SkillPoints, SKILLS_SECTION_LENGTH};
use crate::utils::BytePosition;
use crate::waypoints::Waypoints;
use crate::{
    calc_checksum, CompatibilityCode, CompatibilityIssue, EncodeError, ExpansionType, GameEdition,
    IssueKind, IssueSeverity, ParseHardError, ParseIssue, ParsedSave, Save, SaveMeta, Strictness,
};

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];
const SIGNATURE_RANGE: Range<usize> = 0..4;
const VERSION_RANGE: Range<usize> = 4..8;
const FILE_SIZE_RANGE: Range<usize> = 8..12;
const CHECKSUM_RANGE: Range<usize> = 12..16;

const CHARACTER_SECTION_START: usize = 16;

const QUESTS_LENGTH: usize = 298;
const WAYPOINTS_LENGTH: usize = 80;
const NPCS_LENGTH: usize = 52;
const SKILLS_LENGTH: usize = SKILLS_SECTION_LENGTH;

const V99_QUESTS_START: usize = 335;
const V99_WAYPOINTS_START: usize = 633;
const V99_NPCS_START: usize = 713;
const V99_ATTRIBUTES_OFFSET: usize = 765;

const V105_QUESTS_START: usize = 403;
const V105_WAYPOINTS_START: usize = 701;
const V105_NPCS_START: usize = 781;
const V105_ATTRIBUTES_OFFSET: usize = 833;

#[derive(Debug, Clone, Copy)]
struct FormatCompatibilityEntry {
    format: FormatId,
    edition: GameEdition,
}

const FORMAT_COMPATIBILITY_TABLE: [FormatCompatibilityEntry; 2] = [
    FormatCompatibilityEntry { format: FormatId::V99, edition: GameEdition::D2RLegacy },
    FormatCompatibilityEntry { format: FormatId::V105, edition: GameEdition::RotW },
];

#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum FormatId {
    #[default]
    /// Legacy D2R save layout.
    V99,
    /// ROTW save layout.
    V105,
    /// Unknown layout version read from save bytes.
    Unknown(u32),
}

impl FormatId {
    /// Map numeric file version to a known format identifier.
    pub fn from_version(version: u32) -> Option<Self> {
        match version {
            99 => Some(Self::V99),
            105 => Some(Self::V105),
            _ => None,
        }
    }

    /// Formats this library can currently encode/write.
    pub const fn encodable_formats() -> [Self; 2] {
        [Self::V99, Self::V105]
    }

    /// Coarse edition family for known formats.
    pub const fn edition(self) -> Option<GameEdition> {
        match self {
            Self::V99 => Some(GameEdition::D2RLegacy),
            Self::V105 => Some(GameEdition::RotW),
            Self::Unknown(_) => None,
        }
    }

    /// Pick the best known fallback format for an unknown version.
    ///
    /// Selection order:
    /// 1. Prefer candidates matching `edition_hint` (legacy vs RotW family).
    /// 2. Within that set, pick smallest numeric distance to `version`.
    /// 3. If no hint match exists, fall back to closest across all known formats.
    pub fn fallback_for_unknown_version(version: u32, edition_hint: Option<GameEdition>) -> Self {
        let mut best_match_by_hint: Option<(u32, FormatId)> = None;
        let mut best_match_global: Option<(u32, FormatId)> = None;

        for entry in FORMAT_COMPATIBILITY_TABLE {
            let distance = entry.format.version().abs_diff(version);

            if best_match_global.is_none_or(|(best_distance, _)| distance < best_distance) {
                best_match_global = Some((distance, entry.format));
            }

            if edition_hint.is_some_and(|hint| !edition_hint_matches_format(hint, entry.edition)) {
                continue;
            }

            if best_match_by_hint.is_none_or(|(best_distance, _)| distance < best_distance) {
                best_match_by_hint = Some((distance, entry.format));
            }
        }

        best_match_by_hint
            .or(best_match_global)
            .map_or(Self::V99, |(_, format)| format)
    }

    /// Numeric version value written in the save header.
    pub const fn version(self) -> u32 {
        match self {
            Self::V99 => 99,
            Self::V105 => 105,
            Self::Unknown(version) => version,
        }
    }
}

/// Layout metadata used by the top-level parser/encoder.
///
/// This trait is mostly an internal plumbing surface and may evolve.
pub trait Layout {
    fn format_id(&self) -> FormatId;
    fn character_length(&self) -> usize;
    fn quests_start(&self) -> usize;
    fn waypoints_start(&self) -> usize;
    fn npcs_start(&self) -> usize;
    fn attributes_offset(&self) -> usize;

    fn character_range(&self) -> Range<usize> {
        CHARACTER_SECTION_START..(CHARACTER_SECTION_START + self.character_length())
    }

    fn quests_range(&self) -> Range<usize> {
        self.quests_start()..(self.quests_start() + QUESTS_LENGTH)
    }

    fn waypoints_range(&self) -> Range<usize> {
        self.waypoints_start()..(self.waypoints_start() + WAYPOINTS_LENGTH)
    }

    fn npcs_range(&self) -> Range<usize> {
        self.npcs_start()..(self.npcs_start() + NPCS_LENGTH)
    }

    fn minimum_decode_size(&self) -> usize {
        self.attributes_offset() + SKILLS_LENGTH + 16
    }
}

/// Concrete layout metadata for [`FormatId::V99`].
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutV99;

impl Layout for LayoutV99 {
    fn format_id(&self) -> FormatId {
        FormatId::V99
    }

    fn character_length(&self) -> usize {
        character_length_for_format(FormatId::V99)
    }

    fn quests_start(&self) -> usize {
        V99_QUESTS_START
    }

    fn waypoints_start(&self) -> usize {
        V99_WAYPOINTS_START
    }

    fn npcs_start(&self) -> usize {
        V99_NPCS_START
    }

    fn attributes_offset(&self) -> usize {
        V99_ATTRIBUTES_OFFSET
    }
}

/// Concrete layout metadata for [`FormatId::V105`].
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutV105;

impl Layout for LayoutV105 {
    fn format_id(&self) -> FormatId {
        FormatId::V105
    }

    fn character_length(&self) -> usize {
        character_length_for_format(FormatId::V105)
    }

    fn quests_start(&self) -> usize {
        V105_QUESTS_START
    }

    fn waypoints_start(&self) -> usize {
        V105_WAYPOINTS_START
    }

    fn npcs_start(&self) -> usize {
        V105_NPCS_START
    }

    fn attributes_offset(&self) -> usize {
        V105_ATTRIBUTES_OFFSET
    }
}

static V99_LAYOUT: LayoutV99 = LayoutV99;
static V105_LAYOUT: LayoutV105 = LayoutV105;

fn section_name_option(section_name: &str) -> Option<String> {
    Some(section_name.to_string())
}

fn push_issue(
    issues: &mut Vec<ParseIssue>,
    severity: IssueSeverity,
    kind: IssueKind,
    section_name: Option<String>,
    message: String,
    offset: Option<usize>,
    expected: Option<usize>,
    found: Option<usize>,
) {
    issues.push(ParseIssue {
        severity,
        kind,
        section: section_name,
        message,
        offset,
        expected,
        found,
    });
}

fn range_readable(
    bytes: &[u8],
    range: Range<usize>,
    section_name: &str,
    issues: &mut Vec<ParseIssue>,
) -> bool {
    if bytes.len() >= range.end {
        return true;
    }

    let expected_length = range.end - range.start;
    let found_length = bytes.len().saturating_sub(range.start);
    push_issue(
        issues,
        IssueSeverity::Error,
        IssueKind::TruncatedSection,
        section_name_option(section_name),
        format!(
            "Section {section_name} is truncated. Expected {expected_length} bytes at offset {}, found {found_length}.",
            range.start
        ),
        Some(range.start),
        Some(expected_length),
        Some(found_length),
    );

    false
}

fn section_readable(
    bytes: &[u8],
    section_name: &str,
    section_range: Range<usize>,
    issues: &mut Vec<ParseIssue>,
) -> bool {
    range_readable(bytes, section_range, section_name, issues)
}

fn layout_for_encode(target: FormatId) -> &'static dyn Layout {
    match target {
        FormatId::V99 => &V99_LAYOUT,
        FormatId::V105 | FormatId::Unknown(_) => &V105_LAYOUT,
    }
}

fn edition_hint_matches_format(edition_hint: GameEdition, format_edition: GameEdition) -> bool {
    edition_hint == format_edition
}

fn detect_edition_hint(bytes: &[u8]) -> Option<GameEdition> {
    let v105_marker_one =
        CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_ONE;
    let v105_marker_two =
        CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_TWO;
    if bytes.get(v105_marker_one) == Some(&0x10) && bytes.get(v105_marker_two) == Some(&0x1E) {
        return Some(GameEdition::RotW);
    }

    let v99_marker_one =
        CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_ONE;
    let v99_marker_two =
        CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_TWO;
    if bytes.get(v99_marker_one) == Some(&0x10) && bytes.get(v99_marker_two) == Some(&0x1E) {
        return Some(GameEdition::D2RLegacy);
    }

    None
}

fn expansion_type_from_v99_status(character: &crate::character::Character) -> ExpansionType {
    if character.status.is_expansion() {
        ExpansionType::Expansion
    } else {
        ExpansionType::Classic
    }
}

fn expansion_type_from_decoded_character(
    format_id: FormatId,
    character: &crate::character::Character,
) -> ExpansionType {
    match format_id {
        FormatId::V99 => expansion_type_from_v99_status(character),
        FormatId::V105 | FormatId::Unknown(_) => {
            crate::character::v105::expansion_type(character).unwrap_or(ExpansionType::RotW)
        }
    }
}

fn apply_expansion_type_for_encode(
    character: &mut crate::character::Character,
    target: FormatId,
    expansion_type: ExpansionType,
) {
    match target {
        FormatId::V99 => {
            character.status.set_expansion(!matches!(expansion_type, ExpansionType::Classic));
        }
        FormatId::V105 | FormatId::Unknown(_) => {
            // In v105, expansion mode is encoded via mode marker; keep status bit untouched.
            crate::character::v105::set_expansion_type(character, expansion_type);
        }
    }
}

fn layout_for_decode(
    detected_format: FormatId,
    bytes: &[u8],
    strictness: Strictness,
    issues: &mut Vec<ParseIssue>,
) -> Result<&'static dyn Layout, ParseHardError> {
    match detected_format {
        FormatId::V99 => Ok(&V99_LAYOUT),
        FormatId::V105 => Ok(&V105_LAYOUT),
        FormatId::Unknown(version) => {
            let edition_hint = detect_edition_hint(bytes);
            let fallback_format = FormatId::fallback_for_unknown_version(version, edition_hint);
            let fallback_layout = layout_for_encode(fallback_format);
            push_issue(
                issues,
                IssueSeverity::Warning,
                IssueKind::UnsupportedVersion,
                section_name_option("version"),
                format!(
                    "Unsupported save version {version}. Falling back to {:?} layout in lax mode (edition hint: {:?}).",
                    fallback_layout.format_id(),
                    edition_hint
                ),
                Some(VERSION_RANGE.start),
                Some(VERSION_RANGE.end - VERSION_RANGE.start),
                Some(VERSION_RANGE.end - VERSION_RANGE.start),
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: format!("Unsupported save version {version} in strict mode."),
                });
            }

            Ok(fallback_layout)
        }
    }
}

fn read_version(bytes: &[u8]) -> u32 {
    let mut version_bytes = [0u8; 4];
    version_bytes.copy_from_slice(&bytes[VERSION_RANGE.clone()]);
    u32::from_le_bytes(version_bytes)
}

/// Validate signature/version bytes and return detected format id.
pub fn detect_format(bytes: &[u8]) -> Result<FormatId, ParseHardError> {
    if bytes.len() < SIGNATURE_RANGE.end {
        return Err(ParseHardError {
            message: "Cannot read save signature: file is truncated.".to_string(),
        });
    }

    if bytes[SIGNATURE_RANGE.clone()] != SIGNATURE {
        return Err(ParseHardError { message: "Invalid save signature.".to_string() });
    }

    if bytes.len() < VERSION_RANGE.end {
        return Err(ParseHardError {
            message: "Cannot read save version: file is truncated.".to_string(),
        });
    }

    let version = read_version(bytes);
    Ok(FormatId::from_version(version).unwrap_or(FormatId::Unknown(version)))
}

/// Decode a save in lax mode.
///
/// Equivalent to [`decode_with_strictness`] with [`Strictness::Lax`].
pub fn decode(bytes: &[u8]) -> Result<ParsedSave, ParseHardError> {
    decode_with_strictness(bytes, Strictness::Lax)
}

/// Decode a save with configurable strictness.
pub fn decode_with_strictness(
    bytes: &[u8],
    strictness: Strictness,
) -> Result<ParsedSave, ParseHardError> {
    let mut parsed_save = Save::default();
    let mut issues: Vec<ParseIssue> = Vec::new();

    if !section_readable(bytes, "signature", SIGNATURE_RANGE.clone(), &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: signature section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }

    if bytes[SIGNATURE_RANGE.clone()] != SIGNATURE {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::InvalidSignature,
            section_name_option("signature"),
            format!(
                "Invalid signature: expected {SIGNATURE:X?}, found {:X?}.",
                &bytes[SIGNATURE_RANGE.clone()]
            ),
            Some(SIGNATURE_RANGE.start),
            Some(SIGNATURE_RANGE.end - SIGNATURE_RANGE.start),
            Some(SIGNATURE_RANGE.end - SIGNATURE_RANGE.start),
        );

        if strictness == Strictness::Strict {
            return Err(ParseHardError {
                message: "Cannot parse save: invalid signature in strict mode.".to_string(),
            });
        }
    }

    if !section_readable(bytes, "version", VERSION_RANGE.clone(), &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: version section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }

    parsed_save.version = read_version(bytes);
    let detected_format = FormatId::from_version(parsed_save.version)
        .unwrap_or(FormatId::Unknown(parsed_save.version));
    parsed_save.meta = SaveMeta { format: detected_format };

    let selected_layout = layout_for_decode(detected_format, bytes, strictness, &mut issues)?;

    if bytes.len() < selected_layout.minimum_decode_size() {
        push_issue(
            &mut issues,
            IssueSeverity::Warning,
            IssueKind::InconsistentLayout,
            None,
            format!(
                "File length ({}) is shorter than minimum expected ({}) for layout {:?}.",
                bytes.len(),
                selected_layout.minimum_decode_size(),
                selected_layout.format_id()
            ),
            Some(0),
            Some(selected_layout.minimum_decode_size()),
            Some(bytes.len()),
        );
    }

    let character_range = selected_layout.character_range();
    if !range_readable(bytes, character_range.clone(), "character", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: character section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }

    let character_slice = &bytes[character_range.clone()];
    let character_parse_result =
        decode_character_for_format(selected_layout.format_id(), character_slice);

    match character_parse_result {
        Ok(parsed_character) => {
            parsed_save.expansion_type =
                expansion_type_from_decoded_character(selected_layout.format_id(), &parsed_character);
            parsed_save.character = parsed_character;
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
                    message: "Cannot parse save: character section payload is invalid.".to_string(),
                });
            }
        }
    }

    let quests_range = selected_layout.quests_range();
    if !range_readable(bytes, quests_range.clone(), "quests", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: quests section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }
    match Quests::parse(&bytes[quests_range.clone()]) {
        Ok(parsed_quests) => parsed_save.quests = parsed_quests,
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                section_name_option("quests"),
                parse_error.to_string(),
                Some(quests_range.start),
                Some(QUESTS_LENGTH),
                Some(QUESTS_LENGTH),
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: quests section payload is invalid.".to_string(),
                });
            }
        }
    }

    let waypoints_range = selected_layout.waypoints_range();
    if !range_readable(bytes, waypoints_range.clone(), "waypoints", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: waypoints section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }
    match Waypoints::parse(&bytes[waypoints_range.clone()]) {
        Ok(parsed_waypoints) => parsed_save.waypoints = parsed_waypoints,
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                section_name_option("waypoints"),
                parse_error.to_string(),
                Some(waypoints_range.start),
                Some(WAYPOINTS_LENGTH),
                Some(WAYPOINTS_LENGTH),
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: waypoints section payload is invalid.".to_string(),
                });
            }
        }
    }

    let npcs_range = selected_layout.npcs_range();
    if !range_readable(bytes, npcs_range.clone(), "npcs", &mut issues) {
        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: NPC section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }
    match NPCs::parse(&bytes[npcs_range.clone()]) {
        Ok(parsed_npcs) => parsed_save.npcs = parsed_npcs,
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                section_name_option("npcs"),
                parse_error.to_string(),
                Some(npcs_range.start),
                Some(NPCS_LENGTH),
                Some(NPCS_LENGTH),
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
            section_name_option("attributes"),
            format!(
                "Attributes offset {} is beyond file length {}.",
                selected_layout.attributes_offset(),
                bytes.len()
            ),
            Some(selected_layout.attributes_offset()),
            Some(1),
            Some(0),
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: attributes offset is out of bounds.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }

    let attribute_bytes = bytes[selected_layout.attributes_offset()..].to_vec();
    if attribute_bytes.len() < 2 {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::TruncatedSection,
            section_name_option("attributes"),
            format!(
                "Attributes section is too short: expected at least 2 bytes, found {}.",
                attribute_bytes.len()
            ),
            Some(selected_layout.attributes_offset()),
            Some(2),
            Some(attribute_bytes.len()),
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: attributes section header is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }

    let mut byte_position = BytePosition::default();
    match Attributes::parse(&attribute_bytes, &mut byte_position) {
        Ok(parsed_attributes) => {
            parsed_save.attributes = parsed_attributes;
        }
        Err(parse_error) => {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                IssueKind::InvalidValue,
                section_name_option("attributes"),
                parse_error.to_string(),
                Some(selected_layout.attributes_offset()),
                None,
                None,
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: attributes section payload is invalid."
                        .to_string(),
                });
            }

            return Ok(ParsedSave { save: parsed_save, issues });
        }
    }

    let skills_offset = selected_layout.attributes_offset() + byte_position.current_byte + 1;
    if (skills_offset + SKILLS_LENGTH) > bytes.len() {
        let expected_length = SKILLS_LENGTH;
        let found_length = bytes.len().saturating_sub(skills_offset);
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::TruncatedSection,
            section_name_option("skills"),
            format!(
                "Skills section is truncated at offset {skills_offset}. Expected {expected_length}, found {found_length}."
            ),
            Some(skills_offset),
            Some(expected_length),
            Some(found_length),
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: skills section is truncated.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
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
                section_name_option("skills"),
                parse_error.to_string(),
                Some(skills_offset),
                Some(SKILLS_LENGTH),
                Some(SKILLS_LENGTH),
            );

            if strictness == Strictness::Strict {
                return Err(ParseHardError {
                    message: "Cannot parse save: skills section payload is invalid.".to_string(),
                });
            }
        }
    }

    let items_offset = skills_offset + SKILLS_LENGTH;
    if items_offset > bytes.len() {
        push_issue(
            &mut issues,
            IssueSeverity::Error,
            IssueKind::TruncatedSection,
            section_name_option("items"),
            format!("Items section offset {items_offset} is beyond file length {}.", bytes.len()),
            Some(items_offset),
            Some(1),
            Some(0),
        );

        return if strictness == Strictness::Strict {
            Err(ParseHardError {
                message: "Cannot parse save: items section is out of bounds.".to_string(),
            })
        } else {
            Ok(ParsedSave { save: parsed_save, issues })
        };
    }

    parsed_save.items = items::parse(&bytes[items_offset..]);

    Ok(ParsedSave { save: parsed_save, issues })
}

fn empty_items_layout_for_encode(target: FormatId, mode_marker: u8) -> items::EmptyLayout {
    match target {
        FormatId::V99 => items::EmptyLayout::LegacyExpansion,
        FormatId::V105 | FormatId::Unknown(_) => match mode_marker {
            crate::character::v105::MODE_CLASSIC => items::EmptyLayout::V105Classic,
            crate::character::v105::MODE_ROTW => items::EmptyLayout::V105Rotw,
            _ => items::EmptyLayout::V105Expansion,
        },
    }
}

fn v105_mode_marker_for_encode(save: &Save) -> u8 {
    crate::character::v105::mode_marker_from_expansion_type(save.expansion_type)
}

pub(crate) fn compatibility_issues(save: &Save, target: FormatId) -> Vec<CompatibilityIssue> {
    let mut issues = Vec::new();

    if save.character.class == crate::Class::Warlock
        && target.edition().is_some_and(|edition| edition != GameEdition::RotW)
    {
        issues.push(CompatibilityIssue {
            code: CompatibilityCode::WarlockRequiresRotw,
            blocking: true,
            message: "Warlock class is only supported in RotW editions.".to_string(),
        });
    }

    if save.character.class == crate::Class::Warlock && save.expansion_type != ExpansionType::RotW {
        issues.push(CompatibilityIssue {
            code: CompatibilityCode::WarlockRequiresRotwExpansion,
            blocking: true,
            message: "Warlock class requires RotW expansion type.".to_string(),
        });
    }

    if target == FormatId::V99 && save.expansion_type == ExpansionType::RotW {
        issues.push(CompatibilityIssue {
            code: CompatibilityCode::RotwExpansionRequiresV105,
            blocking: true,
            message: "Cannot encode RotW expansion type as v99. Use v105 format for RotW saves."
                .to_string(),
        });
    }

    issues
}

fn validate_encode_compatibility(save: &Save, target: FormatId) -> Result<(), EncodeError> {
    let errors: Vec<CompatibilityIssue> =
        compatibility_issues(save, target).into_iter().filter(|issue| issue.blocking).collect();
    if errors.is_empty() {
        return Ok(());
    }

    let message = errors
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    Err(EncodeError::new(message))
}

/// Encode a [`Save`] into bytes for a target layout.
///
/// Placeholder sections keep raw bytes when available; otherwise section-specific defaults
/// are generated (for example empty-item trailers).
pub fn encode(save: &Save, target: FormatId) -> Result<Vec<u8>, EncodeError> {
    validate_encode_compatibility(save, target)?;

    let selected_layout = layout_for_encode(target);
    let mut encoded_bytes = vec![0x00; selected_layout.attributes_offset()];

    encoded_bytes[SIGNATURE_RANGE.clone()].copy_from_slice(&SIGNATURE);
    encoded_bytes[VERSION_RANGE.clone()].copy_from_slice(&target.version().to_le_bytes());

    let mut character_for_encode = save.character.clone();
    apply_expansion_type_for_encode(
        &mut character_for_encode,
        selected_layout.format_id(),
        save.expansion_type,
    );
    let character_bytes = encode_character_for_format(selected_layout.format_id(), &character_for_encode)
        .map_err(|error| EncodeError::new(error.to_string()))?;
    let character_range = selected_layout.character_range();
    encoded_bytes[character_range.clone()]
        .copy_from_slice(&character_bytes[..selected_layout.character_length()]);

    encoded_bytes[selected_layout.quests_range()].copy_from_slice(&save.quests.to_bytes());
    encoded_bytes[selected_layout.waypoints_range()].copy_from_slice(&save.waypoints.to_bytes());
    encoded_bytes[selected_layout.npcs_range()].copy_from_slice(&save.npcs.to_bytes());

    let mut attribute_bytes =
        save.attributes.to_bytes().map_err(|error| EncodeError::new(error.to_string()))?;
    encoded_bytes.append(&mut attribute_bytes);

    let mut skill_bytes = save.skills.to_bytes();
    encoded_bytes.append(&mut skill_bytes);

    let items_layout = empty_items_layout_for_encode(target, v105_mode_marker_for_encode(save));
    let mut item_bytes =
        items::generate(&save.items, items_layout, character_for_encode.mercenary.is_hired());
    encoded_bytes.append(&mut item_bytes);

    let file_size = encoded_bytes.len() as u32;
    encoded_bytes[FILE_SIZE_RANGE].copy_from_slice(&file_size.to_le_bytes());

    let checksum = calc_checksum(&encoded_bytes);
    encoded_bytes[CHECKSUM_RANGE].copy_from_slice(&checksum.to_le_bytes());

    Ok(encoded_bytes)
}

#[cfg(test)]
mod tests;
