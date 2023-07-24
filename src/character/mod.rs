use crate::get_offset_from_position;
use crate::get_offset_range_from_position;
use crate::utils::get_sys_time_in_secs;
use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::GameLogicError;
use crate::OffsetID;
use crate::ParseError;
use bit::BitIndex;
use mercenary::Mercenary;
use std::str;

pub mod mercenary;

const SECTION_OFFSET: usize = 16;

const TITLES_CLASSIC_STANDARD_MALE: [&'static str; 4] = ["", "Sir", "Lord", "Baron"];
const TITLES_CLASSIC_STANDARD_FEMALE: [&'static str; 4] = ["", "Dame", "Lady", "Baroness"];
const TITLES_CLASSIC_HARDCORE_MALE: [&'static str; 4] = ["", "Count", "Duke", "King"];
const TITLES_CLASSIC_HARDCORE_FEMALE: [&'static str; 4] = ["", "Countess", "Duchess", "Queen"];
const TITLES_LOD_STANDARD_MALE: [&'static str; 4] = ["", "Slayer", "Champion", "Patriarch"];
const TITLES_LOD_STANDARD_FEMALE: [&'static str; 4] = ["", "Slayer", "Champion", "Matriarch"];
const TITLES_LOD_HARDCORE_MALE: [&'static str; 4] = ["", "Destroyer", "Conqueror", "Guardian"];
const TITLES_LOD_HARDCORE_FEMALE: [&'static str; 4] = ["", "Destroyer", "Conqueror", "Guardian"];

#[derive(PartialEq, Eq, Debug)]
pub struct Character {
    weapon_set: WeaponSet,
    pub status: Status,
    pub progression: u8,
    title: String,
    pub class: Class,
    level: u8,
    pub last_played: u32,
    assigned_skills: [u32; 16],
    left_mouse_skill: u32,
    right_mouse_skill: u32,
    left_mouse_switch_skill: u32,
    right_mouse_switch_skill: u32,
    pub menu_appearance: [u8; 32],
    pub difficulty: Difficulty,
    pub act: Act,
    pub map_seed: u32,
    pub mercenary: Mercenary,
    pub resurrected_menu_appearence: [u8; 48],
    name: String,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Status {
    ladder: bool,
    expansion: bool,
    hardcore: bool,
    died: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum WeaponSet {
    Main,
    Switch,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            weapon_set: WeaponSet::Main,
            status: Status::default(),
            progression: 0,
            title: String::default(),
            class: Class::Amazon,
            level: 1,
            last_played: get_sys_time_in_secs(),
            assigned_skills: [0x00; 16],
            left_mouse_skill: 0,
            right_mouse_skill: 0,
            left_mouse_switch_skill: 0,
            right_mouse_switch_skill: 0,
            menu_appearance: [0x00; 32],
            difficulty: Difficulty::Normal,
            act: Act::Act1,
            map_seed: 0,
            mercenary: Mercenary::default(),
            resurrected_menu_appearence: [0x00; 48],
            name: String::from("default"),
        }
    }
}

fn parse_character(bytes: &[u8; 319]) -> Result<Character, ParseError> {
    let mut character: Character = Character::default();

    let active_weapon = get_u32(bytes, OffsetID::WeaponSet);
    character.weapon_set = WeaponSet::try_from(active_weapon)?;
    character.status =
        Status::from(bytes[get_offset_from_position(OffsetID::Status, SECTION_OFFSET)]);
    character.progression = bytes[get_offset_from_position(OffsetID::Progression, SECTION_OFFSET)];

    let class = Class::try_from(bytes[get_offset_from_position(OffsetID::Class, SECTION_OFFSET)])?;

    character.class = match class {
        Class::Druid | Class::Assassin => {
            if character.status.expansion {
                class
            } else {
                return Err(ParseError {
                    message: format!(
                        "Found druid or assassin class ({0:?})set in non expansion character.",
                        class
                    ),
                });
            }
        }
        _ => class,
    };

    let level = bytes[get_offset_from_position(OffsetID::Level, SECTION_OFFSET)];
    character.level = match level {
        0u8 | 100u8..=255u8 => {
            return Err(ParseError {
                message: format!(
                    "Found character level outside of 1-99 range : {0:?}.",
                    level
                ),
            })
        }
        _ => level,
    };

    character.last_played = get_u32(bytes, OffsetID::LastPlayedDate);
    let assigned_skills =
        &bytes[get_offset_range_from_position(OffsetID::AssignedSkills, SECTION_OFFSET)];
    for i in 0..16 {
        let start = i * 4;
        let assigned_skill =
            u32::from_le_bytes(assigned_skills[start..start + 4].try_into().unwrap());
        character.assigned_skills[i] = assigned_skill;
    }

    character.left_mouse_skill = get_u32(bytes, OffsetID::LeftMouseSkill);
    character.right_mouse_skill = get_u32(bytes, OffsetID::RightMouseSkill);
    character.left_mouse_switch_skill = get_u32(bytes, OffsetID::LeftMouseSwitchSkill);
    character.right_mouse_switch_skill = get_u32(bytes, OffsetID::RightMouseSwitchSkill);
    let last_act = parse_last_act(
        &bytes[get_offset_range_from_position(OffsetID::Difficulty, SECTION_OFFSET)]
            .try_into()
            .unwrap(),
    );

    character.menu_appearance.clone_from_slice(
        &bytes[get_offset_range_from_position(OffsetID::MenuAppearance, SECTION_OFFSET)],
    );

    match last_act {
        Ok(last_act) => {
            character.difficulty = last_act.0;
            character.act = last_act.1;
        }
        Err(e) => return Err(e),
    };

    character.map_seed = get_u32(bytes, OffsetID::MapSeed);
    character.mercenary = mercenary::parse(
        &bytes[get_offset_range_from_position(OffsetID::Mercenary, SECTION_OFFSET)]
            .try_into()
            .unwrap(),
    )?;
    character.resurrected_menu_appearence.clone_from_slice(
        &bytes[get_offset_range_from_position(OffsetID::ResurrectedMenuAppearance, SECTION_OFFSET)],
    );

    let utf8name = match str::from_utf8(
        &bytes[get_offset_range_from_position(OffsetID::Name, SECTION_OFFSET)],
    ) {
        Ok(res) => res.trim_matches(char::from(0)),
        Err(e) => {
            return Err(ParseError {
                message: format!("Invalid utf-8 for character name: {0:?}", e),
            });
        }
    };
    character.name = String::from(utf8name);

    character.title = character.title();

    Ok(character)
}

fn parse_last_act(bytes: &[u8; 3]) -> Result<(Difficulty, Act), ParseError> {
    let mut last_act = (Difficulty::Normal, Act::Act1);
    let mut index = 0;
    if bytes[0] != 0x00 {
        last_act.0 = Difficulty::Normal;
    } else if bytes[1] != 0x00 {
        last_act.0 = Difficulty::Nightmare;
        index = 1;
    } else if bytes[2] != 0x00 {
        last_act.0 = Difficulty::Hell;
        index = 2;
    } else {
        return Err(ParseError {
            message: String::from("Couldn't read current difficulty, all 0."),
        });
    }

    last_act.1 = Act::try_from(bytes[index])?;

    Ok(last_act)
}

fn generate_last_act(difficulty: Difficulty, act: Act) -> [u8; 3] {
    let mut bytes: [u8; 3] = [0x00; 3];
    match difficulty {
        Difficulty::Normal => {
            bytes[0] = u8::from(act);
        }
        Difficulty::Nightmare => {
            bytes[1] = u8::from(act);
        }
        Difficulty::Hell => {
            bytes[2] = u8::from(act);
        }
    }
    bytes
}

impl Default for Status {
    fn default() -> Self {
        Self {
            expansion: (true),
            hardcore: (false),
            ladder: (false),
            died: (false),
        }
    }
}

impl From<u8> for Status {
    fn from(byte: u8) -> Status {
        let mut status = Status::default();
        status.hardcore = byte.bit(2);
        status.died = byte.bit(3);
        status.expansion = byte.bit(5);
        status.ladder = byte.bit(6);
        status
    }
}

impl From<Status> for u8 {
    fn from(status: Status) -> u8 {
        let mut result = 0u8;
        result.set_bit(2, status.hardcore);
        result.set_bit(3, status.died);
        result.set_bit(5, status.expansion);
        result.set_bit(6, status.ladder);
        result
    }
}

impl TryFrom<u32> for WeaponSet {
    type Error = ParseError;
    fn try_from(value: u32) -> Result<WeaponSet, ParseError> {
        match value {
            0u32 => Ok(WeaponSet::Main),
            1u32 => Ok(WeaponSet::Switch),
            _ => {
                return Err(ParseError {
                    message: format!(
                        "Found {0:?} instead of 0 or 1 in current active weapons.",
                        value
                    ),
                });
            }
        }
    }
}

impl From<WeaponSet> for u32 {
    fn from(weapon_set: WeaponSet) -> u32 {
        match weapon_set {
            WeaponSet::Main => 0u32,
            WeaponSet::Switch => 1u32,
        }
    }
}

impl Character {
    // Getters and setters for fields that need validation
    pub fn weapon_set(&self) -> &WeaponSet {
        &self.weapon_set
    }
    // Secondary weapon set is only available in expansion
    pub fn set_weapon_set(&mut self, new_weapon_set: WeaponSet) {
        if new_weapon_set == WeaponSet::Main || self.status.expansion {
            self.weapon_set = new_weapon_set;
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    // 16 bytes maximum, max one - or _
    pub fn set_name(&mut self, new_name: String) {
        if new_name.len() <= 16
            && new_name.matches("-").count() <= 1
            && new_name.matches("_").count() <= 1
        {
            self.name = new_name;
        }
    }

    pub fn level(&self) -> &u8 {
        &self.level
    }
    pub fn set_level(&mut self, new_level: u8) {
        if new_level > 0 && new_level < 100 {
            self.level = new_level
        }
    }

    // pub fn difficulty(&self) -> &(Difficulty, Act) {
    //     &self.difficulty
    // }
    // pub fn set_difficulty(&mut self, new_difficulty: (Difficulty, Act)) {
    //     if new_difficulty.1 == Act::Act5 && !self.status.expansion {
    //         return;
    //     }
    //     //TODO: set progression accordingly
    //     self.difficulty = new_difficulty
    // }
}

impl Character {
    // Return the appropriate title accounting for difficulties beaten
    pub fn title(&self) -> String {
        let male: bool = [Class::Barbarian, Class::Paladin, Class::Necromancer, Class::Druid]
            .contains(&self.class);
        if !self.status.expansion {
            let stage: usize = if self.progression < 4 {
                0
            } else if self.progression < 8 {
                1
            } else if self.progression < 12 {
                2
            } else {
                3
            };
            match (self.status.hardcore, male) {
                (false, false) => return String::from(TITLES_CLASSIC_STANDARD_FEMALE[stage]),
                (false, true) => return String::from(TITLES_CLASSIC_STANDARD_MALE[stage]),
                (true, false) => return String::from(TITLES_CLASSIC_HARDCORE_FEMALE[stage]),
                (true, true) => return String::from(TITLES_CLASSIC_HARDCORE_MALE[stage]),
            }
        } else {
            let stage: usize = if self.progression < 5 {
                0
            } else if self.progression < 9 {
                1
            } else if self.progression < 14 {
                2
            } else {
                3
            };
            match (self.status.hardcore, male) {
                (false, false) => return String::from(TITLES_LOD_STANDARD_FEMALE[stage]),
                (false, true) => return String::from(TITLES_LOD_STANDARD_MALE[stage]),
                (true, false) => return String::from(TITLES_LOD_HARDCORE_FEMALE[stage]),
                (true, true) => return String::from(TITLES_LOD_HARDCORE_MALE[stage]),
            }
        }
    }
}

fn get_u32(bytes: &[u8], id: OffsetID) -> u32 {
    u32::from_le_bytes(
        bytes[get_offset_range_from_position(id, SECTION_OFFSET)]
            .try_into()
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_character() -> () {
        let bytes: [u8; 319] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x0F, 0x00, 0x00, 0x01, 0x10, 0x1E, 0x5C,
            0x00, 0x00, 0x00, 0x00, 0xBB, 0x29, 0xBD, 0x64, 0xFF, 0xFF, 0xFF, 0xFF, 0x28, 0x00,
            0x00, 0x00, 0x3B, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00,
            0x2B, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x9B, 0x00,
            0x00, 0x00, 0x95, 0x00, 0x00, 0x00, 0x34, 0x00, 0x00, 0x00, 0xDC, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF,
            0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x37, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x39, 0x03, 0x02, 0x02, 0x02, 0x35,
            0xFF, 0x51, 0x02, 0x02, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x4D, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
            0x80, 0x43, 0x2D, 0x95, 0x53, 0x00, 0x00, 0x00, 0x00, 0x19, 0x50, 0x40, 0x5C, 0x07,
            0x00, 0x23, 0x00, 0xD6, 0x9B, 0x19, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6F, 0x62, 0x61, 0x20, 0xFF, 0x07, 0x1C,
            0x01, 0x04, 0x00, 0x00, 0x00, 0x75, 0x69, 0x74, 0x20, 0xFF, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x78, 0x70, 0x6C, 0x20, 0xFF, 0x07, 0xD9, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x75, 0x61, 0x70, 0x20, 0x4D, 0x07, 0xF8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4E,
            0x79, 0x61, 0x68, 0x61, 0x6C, 0x6C, 0x6F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_result = Character {
            weapon_set: WeaponSet::Main,
            status: Status {
                expansion: true,
                hardcore: false,
                ladder: false,
                died: false,
            },
            progression: 15,
            title: String::from("Matriarch"),
            class: Class::Sorceress,
            level: 92,
            last_played: get_sys_time_in_secs(),
            assigned_skills: [0x00; 16],
            left_mouse_skill: 0,
            right_mouse_skill: 0,
            left_mouse_switch_skill: 0,
            right_mouse_switch_skill: 0,
            menu_appearance: [0x00; 32],
            difficulty: Difficulty::Hell,
            act: Act::Act1,
            map_seed: 1402285379,
            mercenary: Mercenary::default(),
            resurrected_menu_appearence: [0x00; 48],
            name: String::from("Nyahallo"),
        };
        let parsed_result = match parse_character(&bytes) {
            Ok(result) => result,
            Err(e) => {
                println!("{e:?}");
                assert_eq!(false, true);
                return;
            }
        };
        // println!("{0:?}", parsed_result);
        assert_eq!(parsed_result.level, expected_result.level);
        assert_eq!(parsed_result.class, expected_result.class);
        assert_eq!(parsed_result.weapon_set, expected_result.weapon_set);
        assert_eq!(parsed_result.map_seed, expected_result.map_seed);
        assert_eq!(parsed_result.name, expected_result.name);
        assert_eq!(parsed_result.act, expected_result.act);
        assert_eq!(parsed_result.difficulty, expected_result.difficulty);
        assert_eq!(parsed_result.progression, expected_result.progression);
        assert_eq!(parsed_result.title, expected_result.title);
    }
}
