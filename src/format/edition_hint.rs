use crate::attributes::Attributes;
use crate::skills::SKILLS_SECTION_LENGTH;
use crate::utils::BytePosition;
use crate::GameEdition;

use super::layout::{layout_for_encode, CHARACTER_SECTION_START, VERSION_RANGE};
use super::FormatId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StructureCoherence {
    Truncated,
    AttributesHeaderMissing,
    AttributesParseFailed,
    SkillsHeaderMismatch,
    ItemsHeaderMismatch,
    Full,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct EditionEvidence {
    structure: StructureCoherence,
    character_decodes: bool,
    marker_matches: bool,
}

fn format_for_edition(edition: GameEdition) -> FormatId {
    match edition {
        GameEdition::D2RLegacy => FormatId::V99,
        GameEdition::RotW => FormatId::V105,
    }
}

fn reserved_marker_offsets(edition: GameEdition) -> (usize, usize) {
    match edition {
        GameEdition::D2RLegacy => (
            CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_ONE,
            CHARACTER_SECTION_START + crate::character::v99::OFFSET_RESERVED_VERSION_MARKER_TWO,
        ),
        GameEdition::RotW => (
            CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_ONE,
            CHARACTER_SECTION_START + crate::character::v105::OFFSET_RESERVED_VERSION_MARKER_TWO,
        ),
    }
}

fn detect_structure_coherence(bytes: &[u8], edition: GameEdition) -> StructureCoherence {
    let format = format_for_edition(edition);
    let layout = layout_for_encode(format);
    let attributes_offset = layout.attributes_offset();

    if bytes.len() < attributes_offset + 2 {
        return StructureCoherence::Truncated;
    }

    if bytes[attributes_offset..attributes_offset + 2] != [0x67, 0x66] {
        return StructureCoherence::AttributesHeaderMissing;
    }

    let mut position = BytePosition::default();
    let attributes_bytes = &bytes[attributes_offset..];
    if Attributes::parse(attributes_bytes, &mut position).is_err() {
        return StructureCoherence::AttributesParseFailed;
    }

    let skills_offset = attributes_offset + position.next_byte_offset();
    if bytes.len() < skills_offset + 2 {
        return StructureCoherence::Truncated;
    }

    if bytes[skills_offset..skills_offset + 2] != [0x69, 0x66] {
        return StructureCoherence::SkillsHeaderMismatch;
    }

    let items_offset = skills_offset + SKILLS_SECTION_LENGTH;
    if bytes.len() < items_offset + 2 {
        return StructureCoherence::Truncated;
    }

    if bytes[items_offset..items_offset + 2] == [0x4A, 0x4D] {
        StructureCoherence::Full
    } else {
        StructureCoherence::ItemsHeaderMismatch
    }
}

fn collect_edition_evidence(bytes: &[u8], edition: GameEdition) -> EditionEvidence {
    let format = format_for_edition(edition);
    let layout = layout_for_encode(format);

    let character_range = layout.character_range();
    let character_decodes = if bytes.len() < character_range.end {
        false
    } else {
        let character_bytes = &bytes[character_range.start..character_range.end];
        crate::character::decode_for_format(format, character_bytes).is_ok()
    };

    let (marker_one, marker_two) = reserved_marker_offsets(edition);
    let marker_matches =
        bytes.get(marker_one) == Some(&0x10) && bytes.get(marker_two) == Some(&0x1E);

    EditionEvidence {
        structure: detect_structure_coherence(bytes, edition),
        character_decodes,
        marker_matches,
    }
}

fn compare_evidence(legacy: EditionEvidence, rotw: EditionEvidence) -> Option<GameEdition> {
    if !legacy.character_decodes
        && !rotw.character_decodes
        && !legacy.marker_matches
        && !rotw.marker_matches
    {
        return None;
    }

    if legacy.structure > rotw.structure {
        return Some(GameEdition::D2RLegacy);
    }
    if rotw.structure > legacy.structure {
        return Some(GameEdition::RotW);
    }

    match (legacy.character_decodes, rotw.character_decodes) {
        (true, false) => return Some(GameEdition::D2RLegacy),
        (false, true) => return Some(GameEdition::RotW),
        _ => {}
    }

    match (legacy.marker_matches, rotw.marker_matches) {
        (true, false) => Some(GameEdition::D2RLegacy),
        (false, true) => Some(GameEdition::RotW),
        _ => None,
    }
}

/// Infer the most likely game edition for bytes whose version is unknown or unsupported.
///
/// This compares two edition hypotheses:
/// - `GameEdition::D2RLegacy` using the v99 layout
/// - `GameEdition::RotW` using the v105 layout
///
/// It returns `None` when the evidence is ambiguous.
pub fn detect_edition_hint(bytes: &[u8]) -> Option<GameEdition> {
    if bytes.len() >= VERSION_RANGE.end {
        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&bytes[VERSION_RANGE.start..VERSION_RANGE.end]);
        let version = u32::from_le_bytes(version_bytes);
        if let Some(format) = FormatId::from_version(version) {
            return format.edition();
        }
    }

    let legacy = collect_edition_evidence(bytes, GameEdition::D2RLegacy);
    let rotw = collect_edition_evidence(bytes, GameEdition::RotW);
    compare_evidence(legacy, rotw)
}
