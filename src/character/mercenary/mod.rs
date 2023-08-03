use serde::{Deserialize, Serialize};

use crate::Difficulty;
use crate::ParseError;

pub mod consts;
mod tests;

use consts::*;
use std::cmp;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Variant(pub Class, pub Difficulty);

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Class {
    Rogue(Rogue),
    DesertMercenary(DesertMercenary),
    IronWolf(IronWolf),
    Barbarian(Barbarian),
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Rogue {
    Fire,
    Cold,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DesertMercenary {
    Prayer,
    Defiance,
    BlessedAim,
    Thorns,
    HolyFreeze,
    Might,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum IronWolf {
    Fire,
    Cold,
    Lightning,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Barbarian {
    Bash,
    Frenzy,
}

/// TODO: Make private, add getters and setters that throw GameLogicError
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Mercenary {
    pub dead: bool,
    pub id: u32,
    pub name_id: u16,
    pub name: String,
    pub variant: Variant,
    pub experience: u32,
}

impl Default for Mercenary {
    fn default() -> Self {
        Self {
            dead: false,
            id: 0,
            name_id: 0,
            name: String::from(ROGUE_NAMES[0]),
            variant: VARIANTS[0],
            experience: 0,
        }
    }
}

impl Mercenary {
    pub fn level_from_xp(&self) -> u8 {
        let xp_constant: f64 = self.experience as f64 / xp_rate(&self.variant) as f64;
        let s: f64 = xp_constant.powf(1f64/3f64).floor();
        let mut result = if xp_constant < ( s.powf(3.0) + s.powf(2.0)) {
            (s as u8) - 1
        } else {
            s as u8
        };
        if result == 0 {
            result = 1;
        } else if result > 98 {
            result = 98;
        }
        result
    }

    pub fn xp_from_level(&self, level: u8) -> u32 {
        let level = level as usize;
        let max_xp = (xp_rate(&self.variant) * (98 + 1) * (98 * 98)) as u32;
        let result = (xp_rate(&self.variant) * (level + 1) * (level * level)) as u32;
        cmp::min(result, max_xp)
    }
}

pub fn xp_rate(variant: &Variant) -> usize{
    let add =  match variant.1 {
        Difficulty::Normal => 0,
        Difficulty::Nightmare => 10,
        Difficulty::Hell => 20
    };
    add + match variant.0 {
        Class::Rogue(Rogue::Fire) => 100,
        Class::Rogue(Rogue::Cold) => 105,
        Class::DesertMercenary(_) => 110,
        Class::IronWolf(IronWolf::Fire) => 110,
        Class::IronWolf(IronWolf::Lightning) => 110,
        Class::IronWolf(IronWolf::Cold) => 120,
        Class::Barbarian(_) => 120
    }
}

fn variant_id(variant: &Variant) -> u16 {
    let mut variant_id: u16 = 99;

    for i in 0..VARIANTS.len() {
        if *variant == VARIANTS[i] {
            variant_id = i as u16;
            break;
        }
    }
    if (variant_id as usize) > VARIANTS.len() {
        panic!("There is no mercenary ID for type {0:?} recruited in {1:?}", variant.0, variant.1);
    }
    variant_id
}

pub fn names_list(class: Class) -> &'static [&'static str] {
    match class {
        Class::Rogue(_) => &ROGUE_NAMES,
        Class::DesertMercenary(_) => &DESERTMERCENARY_NAMES,
        Class::IronWolf(_) => &IRONWOLF_NAMES,
        Class::Barbarian(_) => &BARBARIAN_NAMES,
    }
}

pub fn parse(data: &[u8; 14]) -> Result<Mercenary, ParseError> {
    let mut mercenary: Mercenary = Mercenary::default();
    if data[0..2] != [0x00, 0x00] {
        mercenary.dead = true;
    }

    mercenary.id = u32::from_le_bytes(<[u8; 4]>::try_from(&data[2..6]).unwrap());
    let variant_id: u16 = u16::from_le_bytes(<[u8; 2]>::try_from(&data[8..10]).unwrap());
    mercenary.variant = VARIANTS[variant_id as usize];

    let name_id: u16 = u16::from_le_bytes(<[u8; 2]>::try_from(&data[6..8]).unwrap());
    let names_list: &[&str] = names_list(mercenary.variant.0);
    if name_id as usize > names_list.len() {
        return Err(ParseError {
            message: format!("Found invalid name ID {} for mercenary", name_id),
        });
    }
    mercenary.name_id = name_id;
    mercenary.name = String::from(names_list[name_id as usize]);

    mercenary.experience = u32::from_le_bytes(<[u8; 4]>::try_from(&data[10..14]).unwrap());

    Ok(mercenary)
}

pub fn generate(mercenary: &Mercenary) -> [u8; 14] {
    let mut bytes: [u8; 14] = [0x00; 14];
    bytes[0..2].copy_from_slice(match mercenary.dead {
        true => &[0x01, 0x00],
        false => &[0x00, 0x00],
    });

    bytes[2..6].copy_from_slice(&mercenary.id.to_le_bytes());
    bytes[6..8].copy_from_slice(&mercenary.name_id.to_le_bytes());
    let variant_id = variant_id(&mercenary.variant);

    bytes[8..10].copy_from_slice(&variant_id.to_le_bytes());
    bytes[10..14].copy_from_slice(&mercenary.experience.to_le_bytes());

    bytes
}
