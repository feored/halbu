//! NPC section placeholder support.
//!
//! This section is currently preserved as fixed raw bytes with header validation.

use crate::ParseHardError;
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
    /// Parse and validate NPC section bytes.
    pub fn parse(bytes: &[u8]) -> Result<Placeholder, ParseHardError> {
        if bytes.len() < 52 {
            return Err(ParseHardError {
                message: format!(
                    "NPC section is truncated: expected 52 bytes, found {}.",
                    bytes.len()
                ),
            });
        }

        if bytes[0..4] != SECTION_HEADER {
            return Err(ParseHardError {
                message: format!(
                    "Found wrong header for NPC section, expected {SECTION_HEADER:X?} but found {:X?}.",
                    &bytes[0..4]
                ),
            });
        }
        let mut placeholder: Placeholder = Placeholder { data: [0x00; 52] };
        placeholder.data.copy_from_slice(&bytes[0..52]);

        Ok(placeholder)
    }

    /// Serialize the NPC placeholder bytes back to the save.
    pub fn to_bytes(&self) -> [u8; 52] {
        let mut bytes: [u8; 52] = [0x00; 52];
        bytes.copy_from_slice(&self.data[0..52]);

        bytes
    }
}
