use crate::character::encode_for_format as encode_character_for_format;
use crate::items;
use crate::{calc_checksum, CompatibilityChecks, EncodeError, ExpansionType, Save};

use super::compatibility::validate_encode_compatibility;
use super::layout::{
    apply_expansion_type_for_encode, layout_for_encode, CHECKSUM_RANGE, FILE_SIZE_RANGE, SIGNATURE,
    SIGNATURE_RANGE, VERSION_RANGE,
};
use super::FormatId;

fn empty_items_layout_for_encode(
    target: FormatId,
    expansion_type: ExpansionType,
) -> items::EmptyLayout {
    match target {
        FormatId::V99 => {
            if expansion_type == ExpansionType::Classic {
                items::EmptyLayout::LegacyClassic
            } else {
                items::EmptyLayout::LegacyExpansion
            }
        }
        FormatId::V105 | FormatId::Unknown(_) => match expansion_type {
            ExpansionType::Classic => items::EmptyLayout::V105Classic,
            ExpansionType::RotW => items::EmptyLayout::V105RotW,
            ExpansionType::Expansion => items::EmptyLayout::V105Expansion,
        },
    }
}

/// Encode a [`Save`] into bytes for a target layout.
///
/// Placeholder sections keep raw bytes when available; otherwise section-specific defaults
/// are generated (for example empty-item trailers).
pub(crate) fn encode(
    save: &Save,
    target: FormatId,
    compatibility_checks: CompatibilityChecks,
) -> Result<Vec<u8>, EncodeError> {
    if let FormatId::Unknown(version) = target {
        return Err(EncodeError::new(format!(
            "Cannot encode to unknown format version {version}. Choose a known target format."
        )));
    }

    if compatibility_checks == CompatibilityChecks::Enforce {
        validate_encode_compatibility(save, target)?;
    }

    let selected_layout = layout_for_encode(target);
    let mut encoded_bytes = vec![0x00; selected_layout.attributes_offset()];

    encoded_bytes[SIGNATURE_RANGE.start..SIGNATURE_RANGE.end].copy_from_slice(&SIGNATURE);
    encoded_bytes[VERSION_RANGE.start..VERSION_RANGE.end]
        .copy_from_slice(&target.version().to_le_bytes());

    let mut character_for_encode = save.character.clone();
    apply_expansion_type_for_encode(
        &mut character_for_encode,
        selected_layout.format_id(),
        save.expansion_type(),
    );
    let character_bytes =
        encode_character_for_format(selected_layout.format_id(), &character_for_encode)
            .map_err(|error| EncodeError::new(error.to_string()))?;
    let character_range = selected_layout.character_range();
    encoded_bytes[character_range.start..character_range.end]
        .copy_from_slice(&character_bytes[..selected_layout.character_length()]);

    encoded_bytes[selected_layout.quests_range()].copy_from_slice(&save.quests.to_bytes());
    encoded_bytes[selected_layout.waypoints_range()].copy_from_slice(&save.waypoints.to_bytes());
    encoded_bytes[selected_layout.npcs_range()].copy_from_slice(&save.npcs.to_bytes());

    let mut attribute_bytes =
        save.attributes.to_bytes().map_err(|error| EncodeError::new(error.to_string()))?;
    encoded_bytes.append(&mut attribute_bytes);

    let mut skill_bytes = save.skills.to_bytes();
    encoded_bytes.append(&mut skill_bytes);

    let items_layout = empty_items_layout_for_encode(target, save.expansion_type());
    let mut item_bytes =
        items::generate(&save.items, items_layout, character_for_encode.mercenary.is_hired());
    encoded_bytes.append(&mut item_bytes);

    let file_size = encoded_bytes.len() as u32;
    encoded_bytes[FILE_SIZE_RANGE].copy_from_slice(&file_size.to_le_bytes());

    let checksum = calc_checksum(&encoded_bytes);
    encoded_bytes[CHECKSUM_RANGE].copy_from_slice(&checksum.to_le_bytes());

    Ok(encoded_bytes)
}
