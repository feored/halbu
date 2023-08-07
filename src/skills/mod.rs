use std::fmt;

use crate::ParseError;
use crate::Class;
use crate::utils::read_csv;
use crate::utils::Record;

use serde::{Deserialize, Serialize};

mod tests;

pub const SECTION_HEADER: [u8; 2] = [0x69, 0x66];
pub const SKILLS_NUMBER : usize = 30;
pub const SKILLS_OFFSET : [usize; 7] = [6, 36, 66, 96, 126, 221, 251];

/// Represents a single skill. The id values match the ones found in Skills.txt in the game's files.
#[derive(Default, PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: u8,
    pub name: String,
    pub skilldesc: String,
    pub points: u8
}

impl fmt::Display for Skill{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}: {1}pts",self.id, self.points)
    }
}

/// Holds entire skill tree of a character.
#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Serialize, Deserialize)]
pub struct SkillSet([Skill; SKILLS_NUMBER]);

impl fmt::Display for SkillSet{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut final_string = String::default();
        for i in 0usize..SKILLS_NUMBER{
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
    pub fn write(&self) -> Vec<u8> {
        let mut byte_vector: Vec<u8> = SECTION_HEADER.to_vec();
        for i in 0..SKILLS_NUMBER {
            byte_vector.push(self.0[i].points);
        }
        byte_vector
    }

    pub fn default_class(class: Class) -> Self {
        let mut default_skills : Vec<Skill> = vec![Skill{id: 0, name: String::from(""), skilldesc: String::from(""), points: 0u8}; SKILLS_NUMBER];
        let csv_skills : Vec<Record> = read_csv(include_bytes!("../../assets/data/skills.txt")).unwrap();
        let skill_offset = SKILLS_OFFSET[match class {
            Class::Amazon => 0,
            Class::Sorceress => 1,
            Class::Necromancer => 2,
            Class::Paladin => 3,
            Class::Barbarian => 4,
            Class::Druid => 5,
            Class::Assassin => 6
        }];
        for (i, skill) in default_skills.iter_mut().enumerate() {
            let id = i + skill_offset;
            skill.id = id as u8;
            skill.name = csv_skills[id]["skill"].clone();
            skill.skilldesc = csv_skills[id]["skilldesc"].clone();
            skill.points = 0;
        }
        SkillSet(default_skills.try_into().unwrap())
    }
}


/// Parse a vector of bytes containg a character's skill tree (starting with header 0x69 0x66) and returns a SkillSet on success.
pub fn parse(byte_vector: &[u8; 32], class: Class) -> Result<SkillSet, ParseError> {
    let mut skills: SkillSet = SkillSet::default_class(class);
    if byte_vector[0..2] != SECTION_HEADER {
        return Err(ParseError {
            message: format!(
                "Found wrong header for skills section: expected {0:?}, found {1:?}",
                SECTION_HEADER,
                &byte_vector[0..2]
            ),
        });
    }
    for i in 0..SKILLS_NUMBER {
        skills.0[i].points = byte_vector[i + 2];
    }
    Ok(skills)
}


