use std::fmt;

use crate::Class;
use log::warn;

use serde::{Deserialize, Serialize};

pub mod consts;
mod tests;
use consts::*;

const SECTION_HEADER: [u8; 2] = [0x69, 0x66];
const SKILLS_NUMBER: usize = 30;

/// Represents a single skill. The id values match the ones found in Skills.txt in the game's files.
#[derive(Default, PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub skilldesc: String,
    pub points: u8,
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}: {1}pts", self.id, self.points)
    }
}

/// Holds entire skill tree of a character.
#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Serialize, Deserialize)]
pub struct SkillSet([Skill; SKILLS_NUMBER]);

impl fmt::Display for SkillSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut final_string = String::default();
        for i in 0usize..SKILLS_NUMBER {
            final_string.push_str(&format!("{0}\n", self.0[i]));
        }
        write!(f, "{0}", final_string)
    }
}

impl Default for SkillSet {
    fn default() -> Self {
        SkillSet::default_class(Class::Amazon)
    }
}

impl SkillSet {
    /// Generates a byte vector from a given SkillSet
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut byte_vector: Vec<u8> = SECTION_HEADER.to_vec();
        for i in 0..SKILLS_NUMBER {
            byte_vector.push(self.0[i].points);
        }
        byte_vector
    }

    pub fn default_class(class: Class) -> Self {
        let mut default_skills: Vec<Skill> =
            vec![
                Skill { id: 0, name: String::from(""), skilldesc: String::from(""), points: 0u8 };
                SKILLS_NUMBER
            ];
        let skill_offset = SKILLS_OFFSET[match class {
            Class::Amazon => 0,
            Class::Sorceress => 1,
            Class::Necromancer => 2,
            Class::Paladin => 3,
            Class::Barbarian => 4,
            Class::Druid => 5,
            Class::Assassin => 6,
        }];
        for (i, skill) in default_skills.iter_mut().enumerate() {
            let id = i + skill_offset;
            skill.id = id as u32;
            skill.name = String::from(SKILLID[id]);
            skill.skilldesc = String::from(SKILLDESC[id]);
            skill.points = 0;
        }
        SkillSet(default_skills.try_into().unwrap())
    }

    /// Parse a vector of bytes containg a character's skill tree (starting with header 0x69 0x66) and returns a SkillSet on success.
    pub fn parse(byte_vector: &[u8], class: Class) -> SkillSet {
        let mut skills: SkillSet = SkillSet::default_class(class);
        if byte_vector.len() < 2 {
            warn!("Skills section too short to even read header (length: {0}), setting all skills to 0.", byte_vector.len())
        }
        if byte_vector[0..2] != SECTION_HEADER {
            warn!(
                "Found wrong header for skills section: expected {0:?}, found {1:?}",
                SECTION_HEADER,
                &byte_vector[0..2]
            );
        }
        for i in 0..SKILLS_NUMBER {
            if (i + 2) >= byte_vector.len() {
                warn!("Read skills up to {0}, rest is set to default.", i);
                return skills;
            }
            skills.0[i].points = byte_vector[i + 2];
        }
        skills
    }
}
