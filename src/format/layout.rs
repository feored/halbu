use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::character::expected_length_for_format as character_length_for_format;
use crate::skills::SKILLS_SECTION_LENGTH;
use crate::{
    calc_checksum, ExpansionType, GameEdition, IssueKind, IssueSeverity, ParseHardError,
    ParseIssue, Strictness,
};

use super::edition_hint::detect_edition_hint;

pub(crate) const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];
pub(crate) const SIGNATURE_RANGE: Range<usize> = 0..4;
pub(crate) const VERSION_RANGE: Range<usize> = 4..8;
pub(crate) const FILE_SIZE_RANGE: Range<usize> = 8..12;
pub(crate) const CHECKSUM_RANGE: Range<usize> = 12..16;

pub(crate) const CHARACTER_SECTION_START: usize = 16;

pub(crate) const QUESTS_LENGTH: usize = 298;
pub(crate) const WAYPOINTS_LENGTH: usize = 80;
pub(crate) const NPCS_LENGTH: usize = 52;
pub(crate) const SKILLS_LENGTH: usize = SKILLS_SECTION_LENGTH;

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
    V99,
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

    /// Formats this library can encode.
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

            if edition_hint.is_some_and(|hint| hint != entry.edition) {
                continue;
            }

            if best_match_by_hint.is_none_or(|(best_distance, _)| distance < best_distance) {
                best_match_by_hint = Some((distance, entry.format));
            }
        }

        best_match_by_hint.or(best_match_global).map_or(Self::V99, |(_, format)| format)
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

/// Byte layout metadata used by top-level decode/encode paths.
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

pub(crate) fn section_name_option(section_name: &str) -> Option<String> {
    Some(section_name.to_string())
}

pub(crate) struct IssueContext {
    pub section_name: Option<String>,
    pub message: String,
    pub offset: Option<usize>,
    pub expected: Option<usize>,
    pub found: Option<usize>,
}

pub(crate) fn push_issue(
    issues: &mut Vec<ParseIssue>,
    severity: IssueSeverity,
    kind: IssueKind,
    context: IssueContext,
) {
    issues.push(ParseIssue {
        severity,
        kind,
        section: context.section_name,
        message: context.message,
        offset: context.offset,
        expected: context.expected,
        found: context.found,
    });
}

pub(crate) fn range_readable(
    bytes: &[u8],
    range: &Range<usize>,
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
        IssueContext {
            section_name: section_name_option(section_name),
            message: format!(
                "Section {section_name} is truncated. Expected {expected_length} bytes at offset {}, found {found_length}.",
                range.start
            ),
            offset: Some(range.start),
            expected: Some(expected_length),
            found: Some(found_length),
        },
    );

    false
}

pub(crate) fn layout_for_encode(target: FormatId) -> &'static dyn Layout {
    match target {
        FormatId::V99 => &V99_LAYOUT,
        FormatId::V105 | FormatId::Unknown(_) => &V105_LAYOUT,
    }
}

fn expansion_type_from_v99_status(character: &crate::character::Character) -> ExpansionType {
    if character.legacy_expansion_flag() {
        ExpansionType::Expansion
    } else {
        ExpansionType::Classic
    }
}

pub(crate) fn expansion_type_from_decoded_character(
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

pub(crate) fn apply_expansion_type_for_encode(
    character: &mut crate::character::Character,
    target: FormatId,
    expansion_type: ExpansionType,
) {
    match target {
        FormatId::V99 => {
            character.set_legacy_expansion_flag(!matches!(expansion_type, ExpansionType::Classic));
        }
        FormatId::V105 | FormatId::Unknown(_) => {
            // In v105, expansion mode is encoded via mode marker; keep status bit untouched.
            crate::character::v105::set_expansion_type(character, expansion_type);
        }
    }
}

pub(crate) fn layout_for_decode(
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
                IssueContext {
                    section_name: section_name_option("version"),
                    message: format!(
                        "Unsupported save version {version}. Falling back to {:?} layout in lax mode (edition hint: {:?}).",
                        fallback_layout.format_id(),
                        edition_hint
                    ),
                    offset: Some(VERSION_RANGE.start),
                    expected: Some(VERSION_RANGE.end - VERSION_RANGE.start),
                    found: Some(VERSION_RANGE.end - VERSION_RANGE.start),
                },
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

pub(crate) fn read_version(bytes: &[u8]) -> u32 {
    let mut version_bytes = [0u8; 4];
    version_bytes.copy_from_slice(&bytes[VERSION_RANGE.start..VERSION_RANGE.end]);
    u32::from_le_bytes(version_bytes)
}

pub(crate) fn checksum_metadata(bytes: &[u8]) -> (Option<u32>, Option<u32>) {
    if bytes.len() < CHECKSUM_RANGE.end {
        return (None, None);
    }

    let mut checksum_bytes = [0u8; 4];
    checksum_bytes.copy_from_slice(&bytes[CHECKSUM_RANGE.start..CHECKSUM_RANGE.end]);
    let header_checksum = Some(u32::from_le_bytes(checksum_bytes));
    let computed_checksum = header_checksum.map(|_| calc_checksum(bytes) as u32);
    (header_checksum, computed_checksum)
}

/// Validate signature/version bytes and return detected format id.
pub fn detect_format(bytes: &[u8]) -> Result<FormatId, ParseHardError> {
    if bytes.len() < SIGNATURE_RANGE.end {
        return Err(ParseHardError {
            message: "Cannot read save signature: file is truncated.".to_string(),
        });
    }

    if bytes[SIGNATURE_RANGE.start..SIGNATURE_RANGE.end] != SIGNATURE {
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
