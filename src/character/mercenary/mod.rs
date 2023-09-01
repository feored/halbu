use crate::utils::u16_from;
use crate::utils::u32_from;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Range;

mod tests;

enum Section {
    IsDead,
    Id,
    NameId,
    VariantId,
    Experience,
}

impl Section {
    const fn range(self) -> Range<usize> {
        match self {
            Section::IsDead => 0..2,
            Section::Id => 2..6,
            Section::NameId => 6..8,
            Section::VariantId => 8..10,
            Section::Experience => 10..14,
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Mercenary {
    pub is_dead: bool,
    pub id: u32,
    pub name_id: u16,
    pub variant_id: u16,
    pub experience: u32,
}

impl fmt::Display for Mercenary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Dead: {0}, ID: {1}, Name_ID: {2}, Variant: {3}, XP: {4}",
            self.is_dead, self.id, self.name_id, self.variant_id, self.experience
        )
    }
}

impl Mercenary {
    pub fn write(&self) -> [u8; 14] {
        let mut bytes: [u8; 14] = [0x00; 14];
        bytes[Section::IsDead.range()].copy_from_slice(match self.is_dead {
            true => &[0x01, 0x00],
            false => &[0x00, 0x00],
        });

        bytes[Section::Id.range()].copy_from_slice(&self.id.to_le_bytes());
        bytes[Section::NameId.range()].copy_from_slice(&self.name_id.to_le_bytes());
        bytes[Section::VariantId.range()].copy_from_slice(&self.variant_id.to_le_bytes());
        bytes[Section::Experience.range()].copy_from_slice(&self.experience.to_le_bytes());
        bytes
    }

    pub fn parse(data: &[u8; 14]) -> Mercenary {
        let mut mercenary: Mercenary = Mercenary::default();
        if u16_from(&data[Section::IsDead.range()], "mercenary.is_dead") != 0 {
            mercenary.is_dead = true;
        }

        mercenary.id = u32_from(&data[Section::Id.range()], "mercenary.id");
        mercenary.variant_id = u16_from(&data[Section::VariantId.range()], "mercenary.variant_id");
        mercenary.name_id = u16_from(&data[Section::NameId.range()], "mercenary.name_id");
        mercenary.experience = u32_from(&data[Section::Experience.range()], "mercenary.experience");

        mercenary
    }

    pub fn is_hired(&self) -> bool {
        self.id != 0u32
    }
}
