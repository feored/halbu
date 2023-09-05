use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

#[allow(dead_code)]
const SECTION_HEADER: [u8; 2] = [0x4A, 0x4D];
#[allow(dead_code)]
const NO_ITEMS: [u8; 13] =
    [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];

const NO_ITEMS_MERC: [u8; 17] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00,
];

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

pub fn generate(placeholder: &Placeholder, mercenary_hired: bool) -> Vec<u8> {
    match (mercenary_hired, placeholder.data.len()) {
        (false, 0) => NO_ITEMS.to_vec(),
        (true, 0) => NO_ITEMS_MERC.to_vec(),
        _ => placeholder.data.clone(),
    }
}
