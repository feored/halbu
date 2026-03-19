//! Character section model and format-specific codecs.
//!
//! [`title_for_progression_d2r`] and [`Character::title_d2r`] implement the default D2R
//! title rules. `Save::title_d2r()` is the canonical entry point. Modded title systems
//! are out of scope.

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

/// Default class used when constructing a blank [`Character`].
pub const DEFAULT_CLASS: Class = Class::Amazon;
/// Default placeholder name used when constructing a blank [`Character`].
pub const DEFAULT_NAME: &str = "default";

/// In-memory representation of the character section.
#[serde_as]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// Current weapon tab selection in the UI.
    pub weapon_switch: bool,
    /// Character status flags stored as bit fields in save bytes.
    status: Status,
    /// Character progression score.
    ///
    /// In default D2R this roughly tracks act-boss completion across difficulties.
    pub progression: u8,
    /// Character class id.
    pub class: Class,
    /// Character level mirrored by `attributes.level`.
    level: u8,
    /// Last-played timestamp as stored in the save.
    pub last_played: u32,
    /// Assigned skill ids in the skill bar/hotkeys area.
    pub assigned_skills: [u32; 16],
    pub left_mouse_skill: u32,
    pub right_mouse_skill: u32,
    pub left_mouse_switch_skill: u32,
    pub right_mouse_switch_skill: u32,
    /// Legacy menu appearance bytes.
    pub menu_appearance: [u8; 32],
    pub difficulty: Difficulty,
    pub act: Act,
    pub map_seed: u32,
    pub mercenary: Mercenary,
    /// Resurrected preview appearance bytes.
    #[serde_as(as = "Bytes")]
    pub resurrected_menu_appearance: [u8; 48],
    /// Character name string encoded in the character section.
    pub name: String,
    /// Raw encoded character section bytes used to preserve unknown regions on write.
    #[serde(default)]
    #[serde_as(as = "Bytes")]
    pub raw_section: Vec<u8>,
}

/// Return the default D2R title for the given progression tuple.
///
/// Modded title systems are out of scope for this mapping.
pub fn title_for_progression_d2r(
    progression: u8,
    class: Class,
    is_expansion: bool,
    is_hardcore: bool,
) -> Option<&'static str> {
    if is_expansion {
        return match progression {
            0..=3 => None,
            5..=8 => Some(if is_hardcore { "Destroyer" } else { "Slayer" }),
            10..=13 => Some(if is_hardcore { "Conqueror" } else { "Champion" }),
            15 => Some(if is_hardcore {
                "Guardian"
            } else {
                gendered_title_d2r(class, "Patriarch", "Matriarch")
            }),
            _ => None,
        };
    }

    match progression {
        0..=3 => None,
        4..=7 => Some(if is_hardcore {
            gendered_title_d2r(class, "Count", "Countess")
        } else {
            gendered_title_d2r(class, "Sir", "Dame")
        }),
        8..=11 => Some(if is_hardcore {
            gendered_title_d2r(class, "Duke", "Duchess")
        } else {
            gendered_title_d2r(class, "Lord", "Lady")
        }),
        12 => Some(if is_hardcore {
            gendered_title_d2r(class, "King", "Queen")
        } else {
            gendered_title_d2r(class, "Baron", "Baroness")
        }),
        _ => None,
    }
}

fn gendered_title_d2r(
    class: Class,
    male_title: &'static str,
    female_title: &'static str,
) -> &'static str {
    match class {
        Class::Amazon | Class::Sorceress | Class::Assassin => female_title,
        Class::Necromancer
        | Class::Paladin
        | Class::Barbarian
        | Class::Druid
        | Class::Warlock
        | Class::Unknown(_) => male_title,
    }
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
    /// Build a default character for a chosen class.
    pub fn default_class(class: Class) -> Self {
        Character { level: 1, class, ..Default::default() }
    }

    /// Character level as stored in the character section.
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Full status bitfield as decoded from save bytes.
    ///
    /// The expansion bit here is legacy/v99-only. Canonical expansion mode lives on
    /// [`crate::Save::expansion_type`].
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Set character level in the character section.
    ///
    /// Keep this synchronized with attributes through [`crate::Save::set_level`].
    pub(crate) fn set_level(&mut self, level: u8) {
        self.level = level;
    }

    /// Hardcore status bit.
    pub const fn is_hardcore(&self) -> bool {
        self.status.is_hardcore()
    }

    /// Ladder status bit.
    pub const fn is_ladder(&self) -> bool {
        self.status.is_ladder()
    }

    /// "Has died" status bit.
    pub const fn has_died(&self) -> bool {
        self.status.has_died()
    }

    /// Set hardcore status bit.
    pub fn set_hardcore(&mut self, hardcore: bool) {
        self.status.set_hardcore(hardcore);
    }

    /// Set ladder status bit.
    pub fn set_ladder(&mut self, ladder: bool) {
        self.status.set_ladder(ladder);
    }

    /// Set "has died" status bit.
    pub fn set_died(&mut self, died: bool) {
        self.status.set_died(died);
    }

    /// Legacy expansion status flag stored in the v99 status byte.
    ///
    /// This is not canonical for expansion mode. Use [`crate::Save::set_expansion_type`].
    pub(crate) const fn legacy_expansion_flag(&self) -> bool {
        self.status.is_expansion()
    }

    /// Set legacy v99 expansion status flag.
    ///
    /// This is internal glue for v99 encode/decode only.
    pub(crate) fn set_legacy_expansion_flag(&mut self, expansion: bool) {
        self.status.set_expansion(expansion);
    }

    /// Return the default D2R title for this character state.
    ///
    /// Pass the canonical [`crate::ExpansionType`] from [`crate::Save::expansion_type`].
    /// Mods may use different title rules.
    pub fn title_d2r(&self, expansion_type: crate::ExpansionType) -> Option<&'static str> {
        title_for_progression_d2r(
            self.progression,
            self.class,
            !matches!(expansion_type, crate::ExpansionType::Classic),
            self.status.hardcore,
        )
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

/// Status bitfield for the character section.
///
/// Bit mapping in save bytes:
/// - `bit 2`: hardcore
/// - `bit 3`: has died
/// - `bit 5`: legacy expansion flag (v99 mapping; not canonical in [`crate::Save`])
/// - `bit 6`: ladder
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Status {
    /// Ladder status bit.
    ladder: bool,
    /// Expansion status bit.
    expansion: bool,
    /// Hardcore status bit.
    hardcore: bool,
    /// "Has died" status bit.
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

impl Status {
    pub const fn is_expansion(&self) -> bool {
        self.expansion
    }

    pub const fn is_hardcore(&self) -> bool {
        self.hardcore
    }

    pub const fn is_ladder(&self) -> bool {
        self.ladder
    }

    pub const fn has_died(&self) -> bool {
        self.died
    }

    pub(crate) fn set_expansion(&mut self, expansion: bool) {
        self.expansion = expansion;
    }

    pub(crate) fn set_hardcore(&mut self, hardcore: bool) {
        self.hardcore = hardcore;
    }

    pub(crate) fn set_ladder(&mut self, ladder: bool) {
        self.ladder = ladder;
    }

    pub(crate) fn set_died(&mut self, died: bool) {
        self.died = died;
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
