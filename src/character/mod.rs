use std::ops::Range;
use std::str;

use bit::BitIndex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use unicode_segmentation::UnicodeSegmentation;

use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::ParseError;

use crate::attributes::Level;

use crate::utils::get_sys_time_in_secs;
use crate::utils::u32_from;
use crate::utils::u8_from;
use crate::utils::FileSection;

use mercenary::Mercenary;

pub mod consts;
pub mod mercenary;
mod tests;

use consts::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
    Name,
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
            Section::Mercenary => FileSection { offset: 161, bytes: 14 },
            Section::ResurrectedMenuAppearance => FileSection { offset: 203, bytes: 48 },
            Section::Name => FileSection { offset: 251, bytes: 48 },
        }
    }
}

#[serde_as]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub weapon_set: WeaponSet,
    pub status: Status,
    pub progression: Progression,
    pub title: String,
    pub class: Class,
    pub level: Level,
    pub last_played: u32,
    pub assigned_skills: [u32; 16],
    pub left_mouse_skill: u32,
    pub right_mouse_skill: u32,
    pub left_mouse_switch_skill: u32,
    pub right_mouse_switch_skill: u32,
    pub menu_appearance: [u8; 32],
    pub difficulty: Difficulty,
    pub act: Act,
    pub map_seed: u32,
    pub mercenary: Mercenary,
    #[serde_as(as = "Bytes")]
    pub resurrected_menu_appearance: [u8; 48],
    pub name: Name,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Status {
    ladder: bool,
    expansion: bool,
    hardcore: bool,
    died: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WeaponSet {
    Main,
    Switch,
}

#[derive(Default, PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Progression(u8);

impl Progression {
    pub fn default() -> Progression {
        Progression(0)
    }
    pub fn from(number: u8) -> Result<Progression, ParseError> {
        match number {
            0..=15 => Ok(Progression(number)),
            _ => Err(ParseError { message: format!("Progression must be between 0 and 15.") }),
        }
    }
    pub fn value(self) -> u8 {
        self.0
    }
}


impl Default for Character {
    fn default() -> Self {
        Self {
            weapon_set: WeaponSet::Main,
            status: Status::default(),
            progression: Progression::default(),
            title: String::default(),
            class: Class::Amazon,
            level: Level::default(),
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
            resurrected_menu_appearance: [0x00; 48],
            name: Name::default(),
        }
    }
}



pub fn parse(bytes: &[u8; 319]) -> Result<Character, ParseError> {
    let mut character: Character = Character::default();

    let active_weapon =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::WeaponSet))]);
    character.weapon_set = WeaponSet::try_from(active_weapon)?;

    character.status =
        Status::from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Status))]));

    character.progression =
        Progression::from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Progression))]))?;

    let class =
        Class::try_from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Class))]))?;

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

    let level_u8: u8 = u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Level))]);
    character.level = match Level::from(level_u8) {
        Err(_e) => {
            return Err(ParseError {
                message: format!("Found character level outside of 1-99 range : {0:?}.", level_u8),
            })
        }
        Ok(res) => res,
    };

    character.last_played =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LastPlayed))]);
    let assigned_skills = &bytes[Range::<usize>::from(FileSection::from(Section::AssignedSkills))];
    for i in 0..16 {
        let start = i * 4;
        let assigned_skill =
            u32::from_le_bytes(assigned_skills[start..start + 4].try_into().unwrap());
        character.assigned_skills[i] = assigned_skill;
    }

    character.left_mouse_skill =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSkill))]);
    character.right_mouse_skill =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSkill))]);
    character.left_mouse_switch_skill =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSwitchSkill))]);
    character.right_mouse_switch_skill =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSwitchSkill))]);

    let last_act = parse_last_act(
        &bytes[Range::<usize>::from(FileSection::from(Section::Difficulty))].try_into().unwrap(),
    );

    character
        .menu_appearance
        .clone_from_slice(&bytes[Range::<usize>::from(FileSection::from(Section::MenuAppearance))]);

    match last_act {
        Ok(last_act) => {
            character.difficulty = last_act.0;
            character.act = last_act.1;
        }
        Err(e) => return Err(e),
    };

    character.map_seed =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::MapSeed))]);
    character.mercenary = mercenary::parse(
        &bytes[Range::<usize>::from(FileSection::from(Section::Mercenary))].try_into().unwrap(),
    )?;

    character.resurrected_menu_appearance.clone_from_slice(
        &bytes[Range::<usize>::from(FileSection::from(Section::ResurrectedMenuAppearance))],
    );

    let utf8name =
        match str::from_utf8(&bytes[Range::<usize>::from(FileSection::from(Section::Name))]) {
            Ok(res) => res.trim_matches(char::from(0)),
            Err(e) => {
                return Err(ParseError {
                    message: format!("Invalid utf-8 for character name: {0:?}", e),
                });
            }
        };
    character.name = Name::from(&String::from(utf8name))?;

    character.title = character.title();

    Ok(character)
}

pub fn generate(character: &Character) -> [u8; 319] {
    let mut bytes: [u8; 319] = [0x00; 319];

    bytes[Range::<usize>::from(FileSection::from(Section::WeaponSet))]
        .copy_from_slice(&u32::to_le_bytes(u32::from(character.weapon_set)));
    bytes[Range::<usize>::from(FileSection::from(Section::Status)).start] =
        u8::from(character.status);
    bytes[Range::<usize>::from(FileSection::from(Section::Progression)).start] =
        character.progression.value();
    bytes[Range::<usize>::from(FileSection::from(Section::Class)).start] =
        u8::from(character.class);
    bytes[Range::<usize>::from(FileSection::from(Section::Level)).start] = character.level.value();
    bytes[Range::<usize>::from(FileSection::from(Section::LastPlayed))]
        .copy_from_slice(&u32::to_le_bytes(character.last_played));

    let mut assigned_skills: [u8; 64] = [0x00; 64];
    for i in 0..16 {
        assigned_skills[(i * 4)..((i * 4) + 4)]
            .copy_from_slice(&u32::to_le_bytes(character.assigned_skills[i]));
    }
    bytes[Range::<usize>::from(FileSection::from(Section::AssignedSkills))]
        .copy_from_slice(&assigned_skills);
    bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSkill))]
        .copy_from_slice(&u32::to_le_bytes(character.left_mouse_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSkill))]
        .copy_from_slice(&u32::to_le_bytes(character.right_mouse_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSwitchSkill))]
        .copy_from_slice(&u32::to_le_bytes(character.left_mouse_switch_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSwitchSkill))]
        .copy_from_slice(&u32::to_le_bytes(character.right_mouse_switch_skill));
    bytes[Range::<usize>::from(FileSection::from(Section::MenuAppearance))]
        .copy_from_slice(&character.menu_appearance);
    bytes[Range::<usize>::from(FileSection::from(Section::Difficulty))]
        .copy_from_slice(&generate_last_act(character.difficulty, character.act));
    bytes[Range::<usize>::from(FileSection::from(Section::MapSeed))]
        .copy_from_slice(&u32::to_le_bytes(character.map_seed));
    bytes[Range::<usize>::from(FileSection::from(Section::Mercenary))]
        .copy_from_slice(&mercenary::generate(&character.mercenary));
    bytes[Range::<usize>::from(FileSection::from(Section::ResurrectedMenuAppearance))]
        .copy_from_slice(&character.resurrected_menu_appearance);
    let mut name: [u8; 48] = [0x00; 48];
    let name_as_bytes = character.name.0.as_bytes();
    name[0..name_as_bytes.len()].clone_from_slice(name_as_bytes);
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
        Difficulty::Normal => [active_byte, 0x00, 0x00],
        Difficulty::Nightmare => [0x00, active_byte, 0x00],
        Difficulty::Hell => [0x00, 0x00, active_byte],
    }
}

impl Default for Status {
    fn default() -> Self {
        Self { expansion: true, hardcore: false, ladder: false, died: false }
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
        //println!("Converted status: {0:#010b}", result);
        result
    }
}

impl TryFrom<u32> for WeaponSet {
    type Error = ParseError;
    fn try_from(value: u32) -> Result<WeaponSet, ParseError> {
        match value {
            0u32 => Ok(WeaponSet::Main),
            1u32 => Ok(WeaponSet::Switch),
            _ => Err(ParseError {
                message: format!("Found {0:?} instead of 0 or 1 in current active weapons.", value),
            }),
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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Name(String);

impl Name {
    pub fn default() -> Name {
        Name(String::from("default"))
    }

    pub fn from(name: &String) -> Result<Name, ParseError> {
        fn count_occurences(character: char, string: String) -> usize {
            let mut chars = string.chars();
            let mut count: usize = 0;
            loop {
                match chars.next() {
                    None => break,
                    Some(ch) => {
                        if ch == character {
                            count += 1
                        }
                    }
                }
            }
            count
        }

        let len = name.graphemes(true).collect::<Vec<&str>>().len();

        if len < 2 {
            return Err(ParseError {
                message: format!(
                    "Name {0} is invalid, names cannot have less than 2 characters.",
                    name
                ),
            });
        }

        if len > 15 {
            return Err(ParseError {
                message: format!(
                    "Name {0} is invalid, names cannot have more than 15 characters.",
                    name
                ),
            });
        }

        if name.starts_with(|c: char| !c.is_alphabetic()) {
            return Err(ParseError {
                message: format!("Name {0} is invalid, names must start with a letter", name),
            });
        }

        if count_occurences('_', name.clone()) > 1 || count_occurences('-', name.clone()) > 1 {
            return Err(ParseError {
                message: format!(
                    "Name {0} is invalid, names cannot have more than 15 characters.",
                    name
                ),
            });
        }

        let mut chars = name.chars();
        loop {
            match chars.next() {
                None => break,
                Some(res) => {
                    if !res.is_alphabetic() && res != '_' && res != '-' {
                        return Err(ParseError { message: format!("Name {0} is invalid, names can only contain letters and one underscore and dash.", name) });
                    }
                }
            }
        }

        Ok(Name(String::from(name)))
    }
}

impl Character {
    pub fn default_class(class: Class) -> Self {
        Character { class: class, ..Default::default() }
    }

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
        let acts_per_difficulty: usize = 4 + self.status.expansion as usize;
        // Progression always within 0..=15
        let difficulty_beaten: usize = (self.progression.value() as usize) / acts_per_difficulty;
        match (self.status.expansion, self.status.hardcore, male) {
            (false, false, false) => String::from(TITLES_CLASSIC_STANDARD_FEMALE[difficulty_beaten]),
            (false, false, true) => String::from(TITLES_CLASSIC_STANDARD_MALE[difficulty_beaten]),
            (false, true, false) => String::from(TITLES_CLASSIC_HARDCORE_FEMALE[difficulty_beaten]),
            (false, true, true) => String::from(TITLES_CLASSIC_HARDCORE_MALE[difficulty_beaten]),
            (true, false, false) => String::from(TITLES_LOD_STANDARD_FEMALE[difficulty_beaten]),
            (true, false, true) => String::from(TITLES_LOD_STANDARD_MALE[difficulty_beaten]),
            (true, true, false) => String::from(TITLES_LOD_HARDCORE_FEMALE[difficulty_beaten]),
            (true, true, true) => String::from(TITLES_LOD_HARDCORE_MALE[difficulty_beaten]),
        }
    }
}
