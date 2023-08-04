use std::fmt;

use crate::Class;
use crate::ParseError;

use serde::{Deserialize, Serialize};

pub mod consts;
mod tests;

use consts::*;

/// Represents a single skill. The id values match the ones found in Skills.txt in the game's files.
#[derive(Default, PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: u8,
    pub name: String,
    pub description: String,
    pub level: u8,
    pub level_req: u8,
    pub prerequisites: Vec<u8>
}

impl fmt::Display for Skill{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} ({1}), {2} points invested (req. level {3})", self.name, self.id, self.level, self.level_req)
    }
}

/// Holds entire skill tree of a character.
#[derive(Default, PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Serialize, Deserialize)]
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


impl SkillSet{
    pub fn default(class: Class) -> Self {
        let class_offset: usize = class_offset(class);
        let mut skills: [Skill; 30] = std::array::from_fn(|_i| Skill::default());
        for i in 0usize..30usize{
            skills[i] = Skill{
                id: (i + class_offset) as u8,
                level: 0,
                name: String::from(SKILLS_REFERENCE[i + class_offset]),
                description: String::default(),
                prerequisites:  Vec::<u8>::new(),
                level_req: 1
            }
        }

        SkillSet(skills)
    }
}

/// Converts the value from 0-30 to the one found in the game's file by adding an offset specific to each class.
fn class_offset(class: Class) -> usize {
    match class {
        Class::Amazon => SKILL_OFFSET_AMAZON,
        Class::Assassin => SKILL_OFFSET_ASSASSIN,
        Class::Barbarian => SKILL_OFFSET_BARBARIAN,
        Class::Druid => SKILL_OFFSET_DRUID,
        Class::Necromancer => SKILL_OFFSET_NECROMANCER,
        Class::Paladin => SKILL_OFFSET_PALADIN,
        Class::Sorceress => SKILL_OFFSET_SORCERESS,
    }
}

/// Parse a vector of bytes containg a character's skill tree (starting with header 0x69 0x66) and returns a SkillSet on success.
pub fn parse(byte_vector: &[u8; 32], class: Class) -> Result<SkillSet, ParseError> {
    let mut skills: SkillSet = SkillSet::default(class);
    if byte_vector[0..2] != SECTION_HEADER {
        return Err(ParseError {
            message: format!(
                "Found wrong header for skills section: expected {0:?}, found {1:?}",
                SECTION_HEADER,
                &byte_vector[0..2]
            ),
        });
    }
    let offset = class_offset(class);
    for i in 0..30 {
        skills.0[i] = Skill {
            id: (i + offset) as u8,
            name: String::from(SKILLS_REFERENCE[i + offset]),
            level: byte_vector[i + 2],
            description: String::default(),
            level_req: 1,
            prerequisites: Vec::<u8>::new()
        };
    }
    Ok(skills)
}

/// Generates a byte vector from a given SkillSet
pub fn generate(skills: &SkillSet) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = SECTION_HEADER.to_vec();
    for i in 0..30 {
        byte_vector.push(skills.0[i].level);
    }
    byte_vector
}
