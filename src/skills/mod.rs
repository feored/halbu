//! Skills section model.
//!
//! Core API is raw index-based (`set`/`get`) so modded skill trees remain supported.
//! D2R name mapping is available via [`SkillPoints::set_by_name_d2r`]
//! and [`SkillPoints::get_by_name_d2r`].

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Class;
use crate::ParseHardError;

mod named_d2r;
#[cfg(test)]
mod tests;

pub use named_d2r::d2r_skill_index;
pub use named_d2r::d2r_skill_name;
pub use named_d2r::NamedSkillError;

const SECTION_HEADER: [u8; 2] = [0x69, 0x66];
/// Number of skill slots stored in this section.
pub const SKILL_POINTS_COUNT: usize = 30;
/// Total encoded section length including 2-byte section header.
pub const SKILLS_SECTION_LENGTH: usize = 2 + SKILL_POINTS_COUNT;

/// Skill point values indexed by class-local slot id (`0..30`).
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct SkillPoints {
    pub points: [u8; SKILL_POINTS_COUNT],
}

impl Default for SkillPoints {
    fn default() -> Self {
        Self { points: [0; SKILL_POINTS_COUNT] }
    }
}

impl fmt::Display for SkillPoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.points)
    }
}

impl SkillPoints {
    /// Parse skill section bytes (`0x69 0x66` + 30 values).
    pub fn parse(skills_section_bytes: &[u8]) -> Result<SkillPoints, ParseHardError> {
        if skills_section_bytes.len() < SKILLS_SECTION_LENGTH {
            return Err(ParseHardError {
                message: format!(
                    "Skills section too short: expected at least {SKILLS_SECTION_LENGTH} bytes, found {}.",
                    skills_section_bytes.len()
                ),
            });
        }

        if skills_section_bytes[0..2] != SECTION_HEADER {
            return Err(ParseHardError {
                message: format!(
                    "Invalid skills header: expected {SECTION_HEADER:X?}, found {:X?}.",
                    &skills_section_bytes[0..2]
                ),
            });
        }

        let mut points = [0u8; SKILL_POINTS_COUNT];
        points.copy_from_slice(&skills_section_bytes[2..SKILLS_SECTION_LENGTH]);

        Ok(SkillPoints { points })
    }

    /// Serialize to skill section bytes (`0x69 0x66` + 30 values).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut skills_section_bytes = Vec::with_capacity(SKILLS_SECTION_LENGTH);
        skills_section_bytes.extend_from_slice(&SECTION_HEADER);
        skills_section_bytes.extend_from_slice(&self.points);
        skills_section_bytes
    }

    /// Set every skill slot to the same value.
    pub fn set_all(&mut self, value: u8) {
        self.points.fill(value);
    }

    /// Set skill value by raw class-local index.
    pub fn set(&mut self, skill_index: usize, value: u8) {
        self.points[skill_index] = value;
    }

    /// Get skill value by raw class-local index.
    pub fn get(&self, skill_index: usize) -> u8 {
        self.points[skill_index]
    }

    /// Resolve a default D2R skill name to a class-local slot and set its value.
    ///
    /// For modded trees, use raw indices.
    pub fn set_by_name_d2r(
        &mut self,
        class: Class,
        skill_name: &str,
        value: u8,
    ) -> Result<(), NamedSkillError> {
        let index = d2r_skill_index(class, skill_name)?;
        self.set(index, value);
        Ok(())
    }

    /// Resolve a default D2R skill name to a class-local slot and return its value.
    ///
    /// For modded trees, use raw indices.
    pub fn get_by_name_d2r(&self, class: Class, skill_name: &str) -> Result<u8, NamedSkillError> {
        let index = d2r_skill_index(class, skill_name)?;
        Ok(self.get(index))
    }
}
