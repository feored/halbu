use std::fmt;
use std::ops::Range;
use std::str;

use bit::BitIndex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::ParseError;

use crate::utils::get_sys_time_in_secs;
use crate::utils::u32_from;
use crate::utils::u8_from;
use crate::utils::FileSection;

use mercenary::Mercenary;

pub mod mercenary;
mod tests;

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
    pub weapon_switch: bool,
    pub status: Status,
    pub progression: u8,
    pub class: Class,
    pub level: u8,
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
    pub name: String,
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_string = format!("Name: {0}\n", self.name);
        final_string.push_str(&format!("Weapon Switch: {0}\n", self.weapon_switch));
        final_string.push_str(&format!("Status:\n {0}\n", self.status));
        final_string.push_str(&format!("Progression:\n {0:?}\n", self.progression));
        final_string.push_str(&format!("Class:\n {0}\n", self.class));
        final_string.push_str(&format!("Level:\n {0}\n", self.level));
        final_string.push_str(&format!("Difficulty:\n {0}\n", self.difficulty));
        final_string.push_str(&format!("Act:\n {0}\n", self.act));
        final_string.push_str(&format!("Map seed:\n {0}\n", self.map_seed));
        final_string.push_str(&format!("Mercenary:\n {0}\n", self.mercenary));
        write!(f, "{0}", final_string)
    }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            weapon_switch: false,
            status: Status::default(),
            progression: 0,
            class: Class::Amazon,
            level: 1,
            last_played: get_sys_time_in_secs(),
            assigned_skills: [0x0000FFFF; 16],
            left_mouse_skill: 0,
            right_mouse_skill: 0,
            left_mouse_switch_skill: 0,
            right_mouse_switch_skill: 0,
            menu_appearance: [0xFF; 32],
            difficulty: Difficulty::Normal,
            act: Act::Act1,
            map_seed: 0,
            mercenary: Mercenary::default(),
            resurrected_menu_appearance: [0x00; 48],
            name: String::from("default"),
        }
    }
}

pub fn parse(bytes: &[u8; 319]) -> Result<Character, ParseError> {
    let mut character: Character = Character::default();
    character.weapon_switch = u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::WeaponSet))]) != 0;

    character.status =
        Status::from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Status))]));

    character.progression = u8_from(
        &bytes[Range::<usize>::from(FileSection::from(Section::Progression))],
    );

    character.class =
        Class::try_from(u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Class))]))?;

    character.level = u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::Level))]);

    character.last_played =
        u32_from(&bytes[Range::<usize>::from(FileSection::from(Section::LastPlayed))]);
    let assigned_skills: &[u8] = &bytes[Range::<usize>::from(FileSection::from(Section::AssignedSkills))];
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

    character
        .menu_appearance
        .clone_from_slice(&bytes[Range::<usize>::from(FileSection::from(Section::MenuAppearance))]);

    let last_act = parse_last_act(
        &bytes[Range::<usize>::from(FileSection::from(Section::Difficulty))].try_into().unwrap(),
    );

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
    character.name = String::from(utf8name);

    Ok(character)
}

impl Character {
    pub fn write(&self) -> [u8; 319] {
        let mut bytes: [u8; 319] = [0x00; 319];
    
        bytes[Range::<usize>::from(FileSection::from(Section::WeaponSet))]
            .copy_from_slice(&u32::to_le_bytes(u32::from(self.weapon_switch)));
        bytes[Range::<usize>::from(FileSection::from(Section::Status)).start] =
            u8::from(self.status);
        bytes[Range::<usize>::from(FileSection::from(Section::Progression)).start] =
            self.progression;
        bytes[Range::<usize>::from(FileSection::from(Section::Class)).start] =
            u8::from(self.class);
        bytes[Range::<usize>::from(FileSection::from(Section::Level)).start] = self.level;
        bytes[Range::<usize>::from(FileSection::from(Section::LastPlayed))]
            .copy_from_slice(&u32::to_le_bytes(self.last_played));
    
        let mut assigned_skills: [u8; 64] = [0x00; 64];
        for i in 0..16 {
            assigned_skills[(i * 4)..((i * 4) + 4)]
                .copy_from_slice(&u32::to_le_bytes(self.assigned_skills[i]));
        }
        bytes[Range::<usize>::from(FileSection::from(Section::AssignedSkills))]
            .copy_from_slice(&assigned_skills);
        bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSkill))]
            .copy_from_slice(&u32::to_le_bytes(self.left_mouse_skill));
        bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSkill))]
            .copy_from_slice(&u32::to_le_bytes(self.right_mouse_skill));
        bytes[Range::<usize>::from(FileSection::from(Section::LeftMouseSwitchSkill))]
            .copy_from_slice(&u32::to_le_bytes(self.left_mouse_switch_skill));
        bytes[Range::<usize>::from(FileSection::from(Section::RightMouseSwitchSkill))]
            .copy_from_slice(&u32::to_le_bytes(self.right_mouse_switch_skill));
        bytes[Range::<usize>::from(FileSection::from(Section::MenuAppearance))]
            .copy_from_slice(&self.menu_appearance);
        bytes[Range::<usize>::from(FileSection::from(Section::Difficulty))]
            .copy_from_slice(&write_last_act(self.difficulty, self.act));
        bytes[Range::<usize>::from(FileSection::from(Section::MapSeed))]
            .copy_from_slice(&u32::to_le_bytes(self.map_seed));
        bytes[Range::<usize>::from(FileSection::from(Section::Mercenary))]
            .copy_from_slice(&self.mercenary.write());
        bytes[Range::<usize>::from(FileSection::from(Section::ResurrectedMenuAppearance))]
            .copy_from_slice(&self.resurrected_menu_appearance);
        let mut name: [u8; 48] = [0x00; 48];
        let name_as_bytes = self.name.as_bytes();
        name[0..name_as_bytes.len()].clone_from_slice(name_as_bytes);
        bytes[Range::<usize>::from(FileSection::from(Section::Name))].copy_from_slice(&name);
    
        // Add padding, unknown bytes, etc
        bytes[25] = 0x10;
        bytes[26] = 0x1E;
        bytes[36..40].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
    
        bytes
    }
    pub fn default_class(class: Class) -> Self {
        let default_character: Character = Character { level: 1, class: class, ..Default::default()};
        default_character
    }
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

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Status {
    ladder: bool,
    expansion: bool,
    hardcore: bool,
    died: bool,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ladder: {0}, Expansion: {1}, Hardcore: {2}, Has died: {3}",
            self.ladder, self.expansion, self.hardcore, self.died
        )
    }
}


fn write_last_act(difficulty: Difficulty, act: Act) -> [u8; 3] {
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
