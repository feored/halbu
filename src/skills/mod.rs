use crate::Class;
use crate::ParseError;

pub mod consts;
mod tests;
/// Represents a single skill. The id values match the ones found in Skills.txt in the game's files.
#[derive(Default, PartialEq, Eq, Debug, Ord, PartialOrd, Clone)]
pub struct Skill {
    id: u8,
    name: String,
    level: u8,
}

/// Holds entire skill tree of a character.
pub type SkillSet = [Skill; 30];

/// Converts the value from 0-30 to the one found in the game's file by adding an offset specific to each class.
fn get_offset(class: Class) -> usize {
    match class {
        Class::Amazon => consts::SKILL_OFFSET_AMAZON,
        Class::Assassin => consts::SKILL_OFFSET_ASSASSIN,
        Class::Barbarian => consts::SKILL_OFFSET_BARBARIAN,
        Class::Druid => consts::SKILL_OFFSET_DRUID,
        Class::Necromancer => consts::SKILL_OFFSET_NECROMANCER,
        Class::Paladin => consts::SKILL_OFFSET_PALADIN,
        Class::Sorceress => consts::SKILL_OFFSET_SORCERESS,
    }
}

/// Parse a vector of bytes containg a character's skill tree (starting with header 0x69 0x66) and returns a SkillSet on success.
pub fn parse(byte_vector: &[u8; 32], class: Class) -> Result<SkillSet, ParseError> {
    let mut skills: SkillSet = SkillSet::default();
    if byte_vector[0..2] != consts::SECTION_HEADER {
        return Err(ParseError {
            message: format!(
                "Found wrong header for skills section: expected {0:?}, found {1:?}",
                consts::SECTION_HEADER,
                &byte_vector[0..2]
            ),
        });
    }
    let offset = get_offset(class);
    for i in 0..30 {
        skills[i] = Skill {
            id: (i + offset) as u8,
            name: String::from(consts::SKILLS_REFERENCE[i + offset]),
            level: byte_vector[i + 2],
        };
    }
    Ok(skills)
}

/// Generates a byte vector from a given SkillSet
pub fn generate(skills: &SkillSet) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = consts::SECTION_HEADER.to_vec();
    for i in 0..30 {
        byte_vector.push(skills[i].level);
    }
    byte_vector
}

