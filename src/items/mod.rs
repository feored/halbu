//! Items section placeholder support.
//!
//! Item payload is preserved as raw bytes.
//! If raw bytes are empty, encoding emits a known empty-inventory trailer for the target layout.

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

const V99_EMPTY_ITEMS_CLASSIC: [u8; 4] = [0x4A, 0x4D, 0x00, 0x00];
const V99_EMPTY_ITEMS_EXPANSION: [u8; 13] =
    [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];
const V99_EMPTY_ITEMS_EXPANSION_MERC: [u8; 17] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00,
];

const V105_EMPTY_ITEMS_CLASSIC: [u8; 4] = [0x4A, 0x4D, 0x00, 0x00];
const V105_EMPTY_ITEMS_EXPANSION: [u8; 13] =
    [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];
const V105_EMPTY_ITEMS_EXPANSION_MERC: [u8; 17] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00,
];
const V105_EMPTY_ITEMS_ROTW: [u8; 19] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00, 0x01, 0x00, 0x6C,
    0x66, 0x00, 0x00,
];

const V105_EMPTY_ITEMS_ROTW_MERC: [u8; 23] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00, 0x01, 0x00, 0x6C, 0x66, 0x00, 0x00,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyLayout {
    /// Legacy D2R classic empty-item trailer.
    LegacyClassic,
    /// Legacy D2R expansion empty-item trailer.
    LegacyExpansion,
    /// V105 classic empty-item trailer.
    V105Classic,
    /// V105 expansion empty-item trailer.
    V105Expansion,
    /// V105 RotW empty-item trailer.
    V105RotW,
}

/// Raw items-section payload placeholder.
#[serde_as]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    #[serde_as(as = "Bytes")]
    data: Vec<u8>,
}

/// Store item bytes without decoding.
pub fn parse(byte_vector: &[u8]) -> Placeholder {
    Placeholder { data: byte_vector.to_vec() }
}

/// Generate item bytes.
///
/// If `placeholder` contains raw bytes, they are returned unchanged.
/// Otherwise, a known empty-item layout trailer is emitted.
pub fn generate(
    placeholder: &Placeholder,
    empty_layout: EmptyLayout,
    mercenary_hired: bool,
) -> Vec<u8> {
    if !placeholder.data.is_empty() {
        return placeholder.data.clone();
    }

    match (empty_layout, mercenary_hired) {
        (EmptyLayout::LegacyClassic, _) => V99_EMPTY_ITEMS_CLASSIC.to_vec(),
        (EmptyLayout::LegacyExpansion, false) => V99_EMPTY_ITEMS_EXPANSION.to_vec(),
        (EmptyLayout::LegacyExpansion, true) => V99_EMPTY_ITEMS_EXPANSION_MERC.to_vec(),
        (EmptyLayout::V105Classic, _) => V105_EMPTY_ITEMS_CLASSIC.to_vec(),
        (EmptyLayout::V105Expansion, false) => V105_EMPTY_ITEMS_EXPANSION.to_vec(),
        (EmptyLayout::V105Expansion, true) => V105_EMPTY_ITEMS_EXPANSION_MERC.to_vec(),
        (EmptyLayout::V105RotW, false) => V105_EMPTY_ITEMS_ROTW.to_vec(),
        (EmptyLayout::V105RotW, true) => V105_EMPTY_ITEMS_ROTW_MERC.to_vec(),
    }
}
