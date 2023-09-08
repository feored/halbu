use std::fmt;
use std::ops::Range;
use std::str;
use std::time::SystemTime;

use bit::BitIndex;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::convert::u32_from;
use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::ParseError;

use mercenary::Mercenary;

pub mod mercenary;
mod tests;

pub(crate) const DEFAULT_CLASS: Class = Class::Amazon;
pub(crate) const DEFAULT_NAME: &'static str = "default";

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

impl Section {
    const fn range(self) -> Range<usize> {
        match self {
            Section::WeaponSet => 0..4,
            Section::Status => 20..21,
            Section::Progression => 21..22,
            Section::Class => 24..25,
            Section::Level => 27..28,
            Section::LastPlayed => 32..36,
            Section::AssignedSkills => 40..104,
            Section::LeftMouseSkill => 104..108,
            Section::RightMouseSkill => 108..112,
            Section::LeftMouseSwitchSkill => 112..116,
            Section::RightMouseSwitchSkill => 116..120,
            Section::MenuAppearance => 120..152,
            Section::Difficulty => 152..155,
            Section::MapSeed => 155..159,
            Section::Mercenary => 161..175,
            Section::ResurrectedMenuAppearance => 203..251,
            Section::Name => 251..299,
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
            class: DEFAULT_CLASS,
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
            name: String::from(DEFAULT_NAME),
        }
    }
}

impl Character {
    pub fn parse(bytes: &[u8]) -> Character {
        let mut character: Character = Character::default();
        character.weapon_switch =
            u32_from(&bytes[Section::WeaponSet.range()], "character.weapon_switch") != 0;

        character.status = Status::from(bytes[Section::Status.range()][0]);

        character.progression = bytes[Section::Progression.range()][0];

        character.class = match Class::try_from(bytes[Section::Class.range()][0]) {
            Ok(res) => res,
            Err(e) => {
                warn!(
                    "{0}\nFailed to get class, using default: {1}.",
                    e.to_string(),
                    DEFAULT_CLASS
                );
                DEFAULT_CLASS
            }
        };

        character.level = bytes[Section::Level.range()][0];

        character.last_played =
            u32_from(&bytes[Section::LastPlayed.range()], "character.last_played");
        let assigned_skills: &[u8] = &bytes[Section::AssignedSkills.range()];
        for i in 0..16 {
            let start = i * 4;
            let assigned_skill =
                u32_from(&assigned_skills[start..start + 4], "character/assigned_skill");
            character.assigned_skills[i] = assigned_skill;
        }

        character.left_mouse_skill =
            u32_from(&bytes[Section::LeftMouseSkill.range()], "character.left_mouse_skill");
        character.right_mouse_skill =
            u32_from(&bytes[Section::RightMouseSkill.range()], "character.right_mouse_skill");
        character.left_mouse_switch_skill = u32_from(
            &bytes[Section::LeftMouseSwitchSkill.range()],
            "character.left_mouse_switch_skill",
        );
        character.right_mouse_switch_skill = u32_from(
            &bytes[Section::RightMouseSwitchSkill.range()],
            "character.right_mouse_switch_skill",
        );

        character.menu_appearance.clone_from_slice(&bytes[Section::MenuAppearance.range()]);

        let last_act = parse_last_act(&bytes[Section::Difficulty.range()].try_into().unwrap());

        match last_act {
            Ok(last_act) => {
                character.difficulty = last_act.0;
                character.act = last_act.1;
            }
            Err(e) => {
                {
                    warn!(
                        "{0}\nFailed to get last difficulty and act, using default: {1} {2}.",
                        e.to_string(),
                        Act::Act1,
                        Difficulty::Normal
                    );
                }
                character.difficulty = Difficulty::Normal;
                character.act = Act::Act1;
            }
        };

        character.map_seed = u32_from(&bytes[Section::MapSeed.range()], "Character Map Seed");
        // Mercenary size is contained within character size, so no need to check length
        character.mercenary =
            Mercenary::parse(&bytes[Section::Mercenary.range()].try_into().unwrap());

        character
            .resurrected_menu_appearance
            .clone_from_slice(&bytes[Section::ResurrectedMenuAppearance.range()]);

        let utf8name = match str::from_utf8(&bytes[Section::Name.range()]) {
            Ok(res) => res.trim_matches(char::from(0)),
            Err(e) => {
                warn!(
                    "Found invalid utf-8 for character name: {0}, using default: {1}",
                    e.to_string(),
                    DEFAULT_NAME
                );
                DEFAULT_NAME
            }
        };
        character.name = String::from(utf8name);

        character
    }

    pub fn to_bytes(&self) -> [u8; 319] {
        let mut bytes: [u8; 319] = [0x00; 319];

        bytes[Section::WeaponSet.range()]
            .copy_from_slice(&u32::to_le_bytes(u32::from(self.weapon_switch)));
        bytes[Section::Status.range().start] = u8::from(self.status);
        bytes[Section::Progression.range().start] = self.progression;
        bytes[Section::Class.range().start] = u8::from(self.class);
        bytes[Section::Level.range().start] = self.level;
        bytes[Section::LastPlayed.range()].copy_from_slice(&u32::to_le_bytes(self.last_played));

        let mut assigned_skills: [u8; 64] = [0x00; 64];
        for i in 0..16 {
            assigned_skills[(i * 4)..((i * 4) + 4)]
                .copy_from_slice(&u32::to_le_bytes(self.assigned_skills[i]));
        }
        bytes[Section::AssignedSkills.range()].copy_from_slice(&assigned_skills);
        bytes[Section::LeftMouseSkill.range()]
            .copy_from_slice(&u32::to_le_bytes(self.left_mouse_skill));
        bytes[Section::RightMouseSkill.range()]
            .copy_from_slice(&u32::to_le_bytes(self.right_mouse_skill));
        bytes[Section::LeftMouseSwitchSkill.range()]
            .copy_from_slice(&u32::to_le_bytes(self.left_mouse_switch_skill));
        bytes[Section::RightMouseSwitchSkill.range()]
            .copy_from_slice(&u32::to_le_bytes(self.right_mouse_switch_skill));
        bytes[Section::MenuAppearance.range()].copy_from_slice(&self.menu_appearance);
        bytes[Section::Difficulty.range()]
            .copy_from_slice(&write_last_act(self.difficulty, self.act));
        bytes[Section::MapSeed.range()].copy_from_slice(&u32::to_le_bytes(self.map_seed));
        bytes[Section::Mercenary.range()].copy_from_slice(&self.mercenary.write());
        bytes[Section::ResurrectedMenuAppearance.range()]
            .copy_from_slice(&self.resurrected_menu_appearance);
        let mut name: [u8; 48] = [0x00; 48];
        let name_as_bytes = self.name.as_bytes();
        name[0..name_as_bytes.len()].clone_from_slice(name_as_bytes);
        bytes[Section::Name.range()].copy_from_slice(&name);

        // Add padding, unknown bytes, etc
        bytes[25] = 0x10;
        bytes[26] = 0x1E;
        bytes[36..40].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

        bytes
    }
    pub fn default_class(class: Class) -> Self {
        let default_character: Character =
            Character { level: 1, class: class, ..Default::default() };
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

    last_act.1 = match Act::try_from(bytes[index]) {
        Ok(res) => res,
        Err(e) => return Err(ParseError { message: format!("{0}", e.to_string()) }),
    };

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

fn get_sys_time_in_secs() -> u32 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as u32,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}
