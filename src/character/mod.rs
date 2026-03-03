use std::fmt;

use bit::BitIndex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::utils::get_sys_time_in_secs;
use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::ParseHardError;

use mercenary::Mercenary;

pub mod codec;
pub mod common;
pub mod mercenary;
#[cfg(test)]
mod tests;
pub mod v105;
pub mod v99;

pub use codec::decode_for_format;
pub use codec::encode_for_format;
pub use codec::expected_length_for_format;
pub use codec::CharacterCodec;
pub use v105::CharacterCodecV105;
pub use v99::CharacterCodecV99;

pub const DEFAULT_CLASS: Class = Class::Amazon;
pub const DEFAULT_NAME: &str = "default";

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
    #[serde(default)]
    #[serde_as(as = "Bytes")]
    pub raw_section: Vec<u8>,
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
            raw_section: Vec::new(),
        }
    }
}

impl Character {
    pub fn default_class(class: Class) -> Self {
        Character { level: 1, class, ..Default::default() }
    }
}

pub(crate) fn parse_last_act(bytes: &[u8; 3]) -> Result<(Difficulty, Act), ParseHardError> {
    let mut last_act = (Difficulty::Normal, Act::Act1);
    let mut difficulty_index = 0;
    if bytes[0] != 0x00 {
        last_act.0 = Difficulty::Normal;
    } else if bytes[1] != 0x00 {
        last_act.0 = Difficulty::Nightmare;
        difficulty_index = 1;
    } else if bytes[2] != 0x00 {
        last_act.0 = Difficulty::Hell;
        difficulty_index = 2;
    } else {
        return Ok((Difficulty::Normal, Act::Act1));
    }

    last_act.1 = Act::try_from(bytes[difficulty_index])?;

    Ok(last_act)
}

pub(crate) fn write_last_act(difficulty: Difficulty, act: Act) -> [u8; 3] {
    let mut active_byte = u8::from(act);
    active_byte.set_bit(7, true);
    match difficulty {
        Difficulty::Normal => [active_byte, 0x00, 0x00],
        Difficulty::Nightmare => [0x00, active_byte, 0x00],
        Difficulty::Hell => [0x00, 0x00, active_byte],
    }
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
        result
    }
}
