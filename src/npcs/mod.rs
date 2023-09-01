use log::warn;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

const SECTION_HEADER: [u8; 4] = [0x01, 0x77, 0x34, 0x00];

#[serde_as]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    #[serde_as(as = "Bytes")]
    data: [u8; 52],
}

impl Default for Placeholder {
    fn default() -> Self {
        let mut placeholder = Placeholder { data: [0x00; 52] };
        placeholder.data[0..4].copy_from_slice(&SECTION_HEADER);
        placeholder
    }
}

impl Placeholder {
    pub fn parse(bytes: &[u8]) -> Placeholder {
        if bytes[0..4] != SECTION_HEADER {
            warn!(
                "Found wrong header for NPC section, expected {0:X?} but found {1:X?}",
                SECTION_HEADER,
                &bytes[0..4]
            );
        }
        let mut placeholder: Placeholder = Placeholder { data: [0x00; 52] };
        placeholder.data.copy_from_slice(bytes);

        placeholder
    }

    pub fn to_bytes(&self) -> [u8; 52] {
        let mut bytes: [u8; 52] = [0x00; 52];
        bytes.copy_from_slice(&self.data[0..52]);

        bytes
    }
}
