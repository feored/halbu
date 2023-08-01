use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

#[allow(dead_code)]
const SECTION_HEADER: [u8; 2] = [0x4A, 0x4D];

const NO_ITEMS: [u8; 13] =
    [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];

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

pub fn generate(_placeholder: &Placeholder) -> Vec<u8> {
    let byte_vector: Vec<u8> = NO_ITEMS.to_vec();
    byte_vector
}
