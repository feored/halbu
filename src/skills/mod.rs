use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ParseHardError;

#[cfg(test)]
mod tests;

const SECTION_HEADER: [u8; 2] = [0x69, 0x66];
pub const SKILL_POINTS_COUNT: usize = 30;
pub const SKILLS_SECTION_LENGTH: usize = 2 + SKILL_POINTS_COUNT;

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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut skills_section_bytes = Vec::with_capacity(SKILLS_SECTION_LENGTH);
        skills_section_bytes.extend_from_slice(&SECTION_HEADER);
        skills_section_bytes.extend_from_slice(&self.points);
        skills_section_bytes
    }

    pub fn set_all(&mut self, value: u8) {
        self.points.fill(value);
    }

    pub fn set(&mut self, skill_index: usize, value: u8) {
        self.points[skill_index] = value;
    }

    pub fn get(&self, skill_index: usize) -> u8 {
        self.points[skill_index]
    }
}
