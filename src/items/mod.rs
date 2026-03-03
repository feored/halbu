use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

#[allow(dead_code)]
const SECTION_HEADER: [u8; 2] = [0x4A, 0x4D];
#[allow(dead_code)]
const NO_ITEMS_EXPANSION: [u8; 13] =
    [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];

const NO_ITEMS_EXPANSION_MERC: [u8; 17] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00,
];

const NO_ITEMS_CLASSIC: [u8; 4] = [0x4A, 0x4D, 0x00, 0x00];

const NO_ITEMS_ROTW: [u8; 19] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00, 0x01, 0x00, 0x6C,
    0x66, 0x00, 0x00,
];

const NO_ITEMS_ROTW_MERC: [u8; 23] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00, 0x01, 0x00, 0x6C, 0x66, 0x00, 0x00,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyLayout {
    LegacyExpansion,
    V105Classic,
    V105Expansion,
    V105Rotw,
}

#[serde_as]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    #[serde_as(as = "Bytes")]
    data: Vec<u8>,
}

pub fn parse(byte_vector: &[u8]) -> Placeholder {
    let mut placeholder: Placeholder = Placeholder { data: Vec::<u8>::new() };
    placeholder.data.extend_from_slice(byte_vector);

    placeholder
}

pub fn generate(
    placeholder: &Placeholder,
    empty_layout: EmptyLayout,
    mercenary_hired: bool,
) -> Vec<u8> {
    if !placeholder.data.is_empty() {
        return placeholder.data.clone();
    }

    match (empty_layout, mercenary_hired) {
        (EmptyLayout::V105Classic, _) => NO_ITEMS_CLASSIC.to_vec(),
        (EmptyLayout::V105Rotw, false) => NO_ITEMS_ROTW.to_vec(),
        (EmptyLayout::V105Rotw, true) => NO_ITEMS_ROTW_MERC.to_vec(),
        (EmptyLayout::LegacyExpansion | EmptyLayout::V105Expansion, false) => {
            NO_ITEMS_EXPANSION.to_vec()
        }
        (EmptyLayout::LegacyExpansion | EmptyLayout::V105Expansion, true) => {
            NO_ITEMS_EXPANSION_MERC.to_vec()
        }
    }
}
