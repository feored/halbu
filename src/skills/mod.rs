use std::fmt;

use crate::ParseError;

use serde::{Deserialize, Serialize};

mod tests;

pub const SECTION_HEADER: [u8; 2] = [0x69, 0x66];
pub const SECTION_BYTES: usize = 32;

/// Represents a single skill. The id values match the ones found in Skills.txt in the game's files.
#[derive(Default, PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Copy, Serialize, Deserialize)]
pub struct Skill {
    pub id: u8,
    pub points: u8
}

impl fmt::Display for Skill{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}: {1}pts",self.id, self.points)
    }
}

/// Holds entire skill tree of a character.
#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Copy, Serialize, Deserialize)]
pub struct SkillSet([Skill; 30]);

impl fmt::Display for SkillSet{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut final_string = String::default();
        for i in 0usize..30usize{
            final_string.push_str(&format!("{0}\n", self.0[i]));
        }
        write!(f, "{0}", final_string)
    }
}

impl Default for SkillSet{
    fn default() -> Self {
        let mut skills : [Skill; 30] = [Skill{id: 0, points: 0};30];
        for i in 0..30{
            skills[i].id = i as u8;
        }
        SkillSet(skills)
    }
}


impl SkillSet {
    /// Generates a byte vector from a given SkillSet
    pub fn write(&self) -> Vec<u8> {
        let mut byte_vector: Vec<u8> = SECTION_HEADER.to_vec();
        for i in 0..30 {
            byte_vector.push(self.0[i].points);
        }
        byte_vector
    }
}


/// Parse a vector of bytes containg a character's skill tree (starting with header 0x69 0x66) and returns a SkillSet on success.
pub fn parse(byte_vector: &[u8; 32]) -> Result<SkillSet, ParseError> {
    let mut skills: SkillSet = SkillSet::default();
    if byte_vector[0..2] != SECTION_HEADER {
        return Err(ParseError {
            message: format!(
                "Found wrong header for skills section: expected {0:?}, found {1:?}",
                SECTION_HEADER,
                &byte_vector[0..2]
            ),
        });
    }
    for i in 0..30 {
        skills.0[i] = Skill {
            id: i as u8,
            points: byte_vector[i + 2],
        };
    }
    Ok(skills)
}


