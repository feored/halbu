use std::ops::Range;
use std::str;

use bit::BitIndex;

use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::GameLogicError;
use crate::ParseError;

use crate::utils::get_sys_time_in_secs;
use crate::utils::FileSection;
use crate::utils::u32_from;
use crate::utils::u8_from;

use mercenary::Mercenary;


pub mod mercenary;


const TITLES_CLASSIC_STANDARD_MALE: [&'static str; 4] = ["", "Sir", "Lord", "Baron"];
const TITLES_CLASSIC_STANDARD_FEMALE: [&'static str; 4] = ["", "Dame", "Lady", "Baroness"];
const TITLES_CLASSIC_HARDCORE_MALE: [&'static str; 4] = ["", "Count", "Duke", "King"];
const TITLES_CLASSIC_HARDCORE_FEMALE: [&'static str; 4] = ["", "Countess", "Duchess", "Queen"];
const TITLES_LOD_STANDARD_MALE: [&'static str; 4] = ["", "Slayer", "Champion", "Patriarch"];
const TITLES_LOD_STANDARD_FEMALE: [&'static str; 4] = ["", "Slayer", "Champion", "Matriarch"];
const TITLES_LOD_HARDCORE_MALE: [&'static str; 4] = ["", "Destroyer", "Conqueror", "Guardian"];
const TITLES_LOD_HARDCORE_FEMALE: [&'static str; 4] = ["", "Destroyer", "Conqueror", "Guardian"];

#[derive(PartialEq, Eq, Debug)]
enum Section {
    WeaponSet,
    Status,
    Progression,
    Class,
    Level,
    LastPlayed,
    AssignedSkills,
    LeftMouseSkill,
    RightMouseSkill,
    LeftMouseSwitchSkill,
    RightMouseSwitchSkill,
    MenuAppearance,
    Difficulty,
    MapSeed,
    Mercenary,
    ResurrectedMenuAppearance,
    Name
}

impl From<Section> for FileSection {
    fn from(section: Section) -> FileSection {
        match section {
            Section::WeaponSet => FileSection { offset: 0, bytes: 4 },
            Section::Status => FileSection { offset: 20, bytes: 1 },
            Section::Progression => FileSection { offset: 21, bytes: 1 },
            Section::Class => FileSection { offset: 24, bytes: 1 },
            Section::Level => FileSection { offset: 27, bytes: 1 },
            Section::LastPlayed => FileSection { offset: 32, bytes: 4 },
            Section::AssignedSkills => FileSection { offset: 40, bytes: 64 },
            Section::LeftMouseSkill => FileSection { offset: 104, bytes: 4 },
            Section::RightMouseSkill => FileSection { offset: 108, bytes: 4 },
            Section::LeftMouseSwitchSkill => FileSection { offset: 112, bytes: 4 },
            Section::RightMouseSwitchSkill => FileSection { offset: 116, bytes: 4 },
            Section::MenuAppearance => FileSection { offset: 120, bytes: 32 },
            Section::Difficulty => FileSection { offset: 152, bytes: 3 },
            Section::MapSeed => FileSection { offset: 155, bytes: 4 },
            Section::Mercenary=> FileSection { offset: 161, bytes: 14 },
            Section::ResurrectedMenuAppearance => FileSection { offset: 203, bytes: 48 },
            Section::Name => FileSection { offset: 251, bytes: 16 },
        }
    }
} 

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
    died: bool
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

fn parse(bytes: &[u8; 319]) -> Result<Character, ParseError> {
    let mut character: Character = Character::default();

    let active_weapon = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::WeaponSet))]);
    character.weapon_set = WeaponSet::try_from(active_weapon)?;

    character.status = Status::from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Status))]));

    character.progression = u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Progression))]);

    let class = Class::try_from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Class))]))?;

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

    let level = u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Level))]);
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

    character.last_played = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LastPlayed))]);
    let assigned_skills =  &bytes[Range::<usize>::from(FileSection::from(Section::AssignedSkills))];
    for i in 0..16 {
        let start = i * 4;
        let assigned_skill =
            u32::from_le_bytes(assigned_skills[start..start + 4].try_into().unwrap());
        character.assigned_skills[i] = assigned_skill;
    }

    character.left_mouse_skill = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSkill))]);
    character.right_mouse_skill = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSkill))]);
    character.left_mouse_switch_skill = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSwitchSkill))]);
    character.right_mouse_switch_skill = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSwitchSkill))]);

    let last_act = parse_last_act(
        &bytes[Range::<usize>::from(FileSection::from(Section::Difficulty))]
            .try_into()
            .unwrap(),
    );

    character.menu_appearance.clone_from_slice(
        &bytes[Range::<usize>::from(FileSection::from(Section::MenuAppearance))]
    );

    match last_act {
        Ok(last_act) => {
            character.difficulty = last_act.0;
            character.act = last_act.1;
        }
        Err(e) => return Err(e),
    };

    character.map_seed = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::MapSeed))]);
    character.mercenary = mercenary::parse(
        &bytes[Range::<usize>::from(FileSection::from(Section::Mercenary))]
            .try_into()
            .unwrap(),
    )?;
    
    character.resurrected_menu_appearence.clone_from_slice(
        &bytes[Range::<usize>::from(FileSection::from(Section::ResurrectedMenuAppearance))]
    );

    let utf8name = match str::from_utf8(
        &bytes[Range::<usize>::from(FileSection::from(Section::Name))]
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

pub fn generate(character: &Character) -> [u8; 319] {
    let mut bytes : [u8;319] = [0x00; 319];

    bytes[Range::<usize>::from(FileSection::from(Section::WeaponSet))].copy_from_slice(&u32::to_le_bytes(u32::from(character.weapon_set)));
    bytes[Range::<usize>::from(FileSection::from(Section::Status))][0] = u8::from(character.status);
    bytes[Range::<usize>::from(FileSection::from(Section::Progression))][0] = u8::from(character.progression);
    bytes[Range::<usize>::from(FileSection::from(Section::Class))][0] = u8::from(character.class);
    bytes[Range::<usize>::from(FileSection::from(Section::Level))][0] = u8::from(character.level);
    bytes[Range::<usize>::from(FileSection::from(Section::LastPlayed))].copy_from_slice(&u32::to_le_bytes(u32::from(character.last_played)));
    

    let mut assigned_skills : [u8; 64] = [0x00; 64];
    for i in 0..16 {
        assigned_skills[(i*4)..((i*4)+4)].copy_from_slice(&u32::to_le_bytes(character.assigned_skills[i]));
    }
    bytes[Range::<usize>::from(FileSection::from(Section::AssignedSkills))].copy_from_slice(&assigned_skills);
    bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSkill))].copy_from_slice(&u32::to_le_bytes(character.left_mouse_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSkill))].copy_from_slice(&u32::to_le_bytes(character.right_mouse_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSwitchSkill))].copy_from_slice(&u32::to_le_bytes(character.left_mouse_switch_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSwitchSkill))].copy_from_slice(&u32::to_le_bytes(character.right_mouse_switch_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::MenuAppearance))].copy_from_slice(&character.menu_appearance);
    bytes[Range::<usize>::from(FileSection::from(Section::Difficulty))].copy_from_slice(&generate_last_act(character.difficulty, character.act));
    bytes[Range::<usize>::from(FileSection::from(Section::MapSeed))].copy_from_slice(&u32::to_le_bytes(character.map_seed));
    bytes[Range::<usize>::from(FileSection::from(Section::Mercenary))].copy_from_slice(&mercenary::generate(&character.mercenary));
    bytes[Range::<usize>::from(FileSection::from(Section::ResurrectedMenuAppearance))].copy_from_slice(&character.resurrected_menu_appearence);
    let mut name : [u8; 16] = [0x00;16];
    let name_as_bytes = character.name.as_bytes();
    name[0..name_as_bytes.len()].clone_from_slice(&name_as_bytes);
    bytes[Range::<usize>::from(FileSection::from(Section::Name))].copy_from_slice(&name);


    // Add padding, unknown bytes, etc
    bytes[25] = 0x10;
    bytes[26] = 0x1E;
    bytes[36..40].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

    bytes
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
    let mut active_byte = u8::from(act);
    active_byte.set_bit(7, true);
    match difficulty {
        Difficulty::Normal => {
            [active_byte, 0x00, 0x00]
        }
        Difficulty::Nightmare => {
            [0x00, active_byte, 0x00]
        }
        Difficulty::Hell => {
            [0x00, 0x00, active_byte]
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self {
            expansion: true,
            hardcore: false,
            ladder: false,
            died: false,
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
        println!("Converted status: {0:#010b}", result);
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
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

        let expected_result = Character{
            weapon_set: WeaponSet::Main,
            status: Status {
                expansion: true,
                hardcore: false,
                ladder: false,
                died: true,
            },
            progression: 15,
            title: String::from("Matriarch"),
            class: Class::Sorceress,
            level: 92,
            last_played: 1690118587,
            assigned_skills: [40, 59, 54, 42, 43, 65535, 65535, 155, 149, 52, 220, 65535, 65535, 65535, 65535, 65535],
            left_mouse_skill: 55,
            right_mouse_skill: 54,
            left_mouse_switch_skill: 0,
            right_mouse_switch_skill: 54,
            menu_appearance: [57, 3, 2, 2, 2, 53, 255, 81, 2, 2, 255, 255, 255, 255, 255, 255, 77, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
            difficulty: Difficulty::Hell,
            act: Act::Act1,
            map_seed: 1402285379,
            mercenary: Mercenary { dead: false, id: 1547718681, name_id: 7, name: "Emilio", variant: (mercenary::Class::DesertMercenary(mercenary::DesertMercenary::Might), Difficulty::Hell), experience: 102341590 },
            resurrected_menu_appearence: [111, 98, 97, 32, 255, 7, 28, 1, 4, 0, 0, 0, 117, 105, 116, 32, 255, 2, 0, 0, 0, 0, 0, 0, 120, 112, 108, 32, 255, 7, 217, 0, 0, 0, 0, 0, 117, 97, 112, 32, 77, 7, 248, 0, 0, 0, 0, 0],
            name: String::from("Nyahallo"),
        };
        let parsed_result = match parse(&bytes) {
            Ok(result) => result,
            Err(e) => {
                println!("{e:?}");
                assert_eq!(false, true);
                return;
            }
        };
        //println!("{0:?}", parsed_result);
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn test_generate(){
        let expected_result: [u8; 319] = [
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

        let character = Character{
            weapon_set: WeaponSet::Main,
            status: Status {
                expansion: true,
                hardcore: false,
                ladder: false,
                died: true,
            },
            progression: 15,
            title: String::from("Matriarch"),
            class: Class::Sorceress,
            level: 92,
            last_played: 1690118587,
            assigned_skills: [40, 59, 54, 42, 43, 65535, 65535, 155, 149, 52, 220, 65535, 65535, 65535, 65535, 65535],
            left_mouse_skill: 55,
            right_mouse_skill: 54,
            left_mouse_switch_skill: 0,
            right_mouse_switch_skill: 54,
            menu_appearance: [57, 3, 2, 2, 2, 53, 255, 81, 2, 2, 255, 255, 255, 255, 255, 255, 77, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
            difficulty: Difficulty::Hell,
            act: Act::Act1,
            map_seed: 1402285379,
            mercenary: Mercenary { dead: false, id: 1547718681, name_id: 7, name: "Emilio", variant: (mercenary::Class::DesertMercenary(mercenary::DesertMercenary::Might), Difficulty::Hell), experience: 102341590 },
            resurrected_menu_appearence: [111, 98, 97, 32, 255, 7, 28, 1, 4, 0, 0, 0, 117, 105, 116, 32, 255, 2, 0, 0, 0, 0, 0, 0, 120, 112, 108, 32, 255, 7, 217, 0, 0, 0, 0, 0, 117, 97, 112, 32, 77, 7, 248, 0, 0, 0, 0, 0],
            name: String::from("Nyahallo"),
        };
        let generated_result = generate(&character);


        assert_eq!(expected_result, generated_result);

    }
}
