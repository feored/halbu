use crate::utils::u16_from;
use crate::utils::u32_from;
use crate::ParseHardError;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MercenaryType {
    Rogue,
    DesertMercenary,
    IronWolf,
    Barbarian,
}

fn mercenary_type_for_variant_id(variant_id: u16) -> Option<MercenaryType> {
    match variant_id {
        0..=5 => Some(MercenaryType::Rogue),
        6..=14 | 30..=35 => Some(MercenaryType::DesertMercenary),
        15..=23 => Some(MercenaryType::IronWolf),
        24..=29 | 36..=38 => Some(MercenaryType::Barbarian),
        _ => None,
    }
}

fn mercenary_name_count_for_type(mercenary_type: MercenaryType) -> usize {
    match mercenary_type {
        MercenaryType::Rogue => 41,
        MercenaryType::DesertMercenary => 21,
        MercenaryType::IronWolf => 20,
        MercenaryType::Barbarian => 67,
    }
}

/// Return the mercenary name count for a known variant id.
pub(crate) fn mercenary_name_count_for_variant_id(variant_id: u16) -> Option<usize> {
    mercenary_type_for_variant_id(variant_id).map(mercenary_name_count_for_type)
}

/// Return the XP rate for a known mercenary variant id.
pub(crate) fn xp_rate_for_variant_id(variant_id: u16) -> Option<u32> {
    match variant_id {
        0 => Some(100),
        1 => Some(105),
        2 => Some(110),
        3 => Some(115),
        4 => Some(120),
        5 => Some(125),
        6..=8 => Some(110),
        9..=11 => Some(120),
        12..=14 => Some(130),
        15 | 17 => Some(110),
        16 => Some(120),
        18 | 20 => Some(120),
        19 => Some(130),
        21 | 23 => Some(130),
        22 => Some(140),
        24 | 25 => Some(120),
        26 | 27 => Some(130),
        28 | 29 => Some(140),
        30..=32 => Some(120),
        33..=35 => Some(130),
        36 => Some(120),
        37 => Some(130),
        38 => Some(140),
        _ => None,
    }
}

/// Resolve a mercenary level from current experience.
///
/// This returns `0` when the experience is below level 1.
pub(crate) fn level_from_experience(experience: u32, xp_rate: u32) -> u8 {
    let scaled_experience = experience / xp_rate;
    let guess = (scaled_experience as f64).cbrt().floor() as u8;

    if scaled_experience < u32::from(guess) * u32::from(guess) * u32::from(guess + 1) {
        guess.saturating_sub(1)
    } else {
        guess
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
    pub(crate) fn has_data_without_hire(&self) -> bool {
        !self.is_hired()
            && (self.is_dead || self.name_id != 0 || self.variant_id != 0 || self.experience != 0)
    }

    pub fn write(&self) -> [u8; 14] {
        let mut bytes: [u8; 14] = [0x00; 14];
        if !self.is_hired() {
            return bytes;
        }

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

    pub fn parse(data: &[u8]) -> Result<Mercenary, ParseHardError> {
        if data.len() < 14 {
            return Err(ParseHardError {
                message: format!(
                    "Mercenary section is truncated: expected 14 bytes, found {}.",
                    data.len()
                ),
            });
        }

        let mut mercenary: Mercenary = Mercenary::default();
        if u16_from(&data[Section::IsDead.range()], "mercenary.is_dead")? != 0 {
            mercenary.is_dead = true;
        }

        mercenary.id = u32_from(&data[Section::Id.range()], "mercenary.id")?;
        mercenary.variant_id = u16_from(&data[Section::VariantId.range()], "mercenary.variant_id")?;
        mercenary.name_id = u16_from(&data[Section::NameId.range()], "mercenary.name_id")?;
        mercenary.experience =
            u32_from(&data[Section::Experience.range()], "mercenary.experience")?;

        Ok(mercenary)
    }

    pub fn is_hired(&self) -> bool {
        self.id != 0u32
    }
}
