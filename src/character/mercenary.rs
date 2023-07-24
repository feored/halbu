use crate::Difficulty;
use crate::GameLogicError;
use crate::ParseError;

const VARIANTS: &'static [Variant; 39] = &[
    (Class::Rogue(Rogue::Fire), Difficulty::Normal),
    (Class::Rogue(Rogue::Cold), Difficulty::Normal),
    (Class::Rogue(Rogue::Fire), Difficulty::Nightmare),
    (Class::Rogue(Rogue::Cold), Difficulty::Nightmare),
    (Class::Rogue(Rogue::Fire), Difficulty::Hell),
    (Class::Rogue(Rogue::Cold), Difficulty::Hell),
    (
        Class::DesertMercenary(DesertMercenary::Prayer),
        Difficulty::Normal,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Defiance),
        Difficulty::Normal,
    ),
    (
        Class::DesertMercenary(DesertMercenary::BlessedAim),
        Difficulty::Normal,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Thorns),
        Difficulty::Nightmare,
    ),
    (
        Class::DesertMercenary(DesertMercenary::HolyFreeze),
        Difficulty::Nightmare,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Might),
        Difficulty::Nightmare,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Prayer),
        Difficulty::Hell,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Defiance),
        Difficulty::Hell,
    ),
    (
        Class::DesertMercenary(DesertMercenary::BlessedAim),
        Difficulty::Hell,
    ),
    (Class::IronWolf(IronWolf::Fire), Difficulty::Normal),
    (Class::IronWolf(IronWolf::Cold), Difficulty::Normal),
    (Class::IronWolf(IronWolf::Lightning), Difficulty::Normal),
    (Class::IronWolf(IronWolf::Fire), Difficulty::Nightmare),
    (Class::IronWolf(IronWolf::Cold), Difficulty::Nightmare),
    (Class::IronWolf(IronWolf::Lightning), Difficulty::Nightmare),
    (Class::IronWolf(IronWolf::Fire), Difficulty::Hell),
    (Class::IronWolf(IronWolf::Cold), Difficulty::Hell),
    (Class::IronWolf(IronWolf::Lightning), Difficulty::Hell),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Normal),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Normal),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Nightmare),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Nightmare),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Hell),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Hell),
    (
        Class::DesertMercenary(DesertMercenary::Prayer),
        Difficulty::Nightmare,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Defiance),
        Difficulty::Nightmare,
    ),
    (
        Class::DesertMercenary(DesertMercenary::BlessedAim),
        Difficulty::Nightmare,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Thorns),
        Difficulty::Hell,
    ),
    (
        Class::DesertMercenary(DesertMercenary::HolyFreeze),
        Difficulty::Hell,
    ),
    (
        Class::DesertMercenary(DesertMercenary::Might),
        Difficulty::Hell,
    ),
    (Class::Barbarian(Barbarian::Frenzy), Difficulty::Normal),
    (Class::Barbarian(Barbarian::Frenzy), Difficulty::Nightmare),
    (Class::Barbarian(Barbarian::Frenzy), Difficulty::Hell),
];

const ROGUE_NAMES: [&'static str; 41] = [
    "Aliza", "Ampliza", "Annor", "Abhaya", "Elly", "Paige", "Basanti", "Blaise", "Kyoko",
    "Klaudia", "Kundri", "Kyle", "Visala", "Elexa", "Floria", "Fiona", "Gwinni", "Gaile", "Hannah",
    "Heather", "Iantha", "Diane", "Isolde", "Divo", "Ithera", "Itonya", "Liene", "Maeko", "Mahala",
    "Liaza", "Meghan", "Olena", "Oriana", "Ryann", "Rozene", "Raissa", "Sharyn", "Shikha", "Debi",
    "Tylena", "Wendy",
];

const DESERTMERCENARY_NAMES: [&'static str; 21] = [
    "Hazade", "Alhizeer", "Azrael", "Ahsab", "Chalan", "Haseen", "Razan", "Emilio", "Pratham",
    "Fazel", "Jemali", "Kasim", "Gulzar", "Mizan", "Leharas", "Durga", "Neeraj", "Ilzan",
    "Zanarhi", "Waheed", "Vikhyat",
];

const IRONWOLF_NAMES: [&'static str; 20] = [
    "Jelani", "Barani", "Jabari", "Devak", "Raldin", "Telash", "Ajheed", "Narphet", "Khaleel",
    "Phaet", "Geshef", "Vanji", "Haphet", "Thadar", "Yatiraj", "Rhadge", "Yashied", "Jarulf",
    "Flux", "Scorch",
];

const BARBARIAN_NAMES: [&'static str; 67] = [
    "Varaya",
    "Khan",
    "Klisk",
    "Bors",
    "Brom",
    "Wiglaf",
    "Hrothgar",
    "Scyld",
    "Healfdane",
    "Heorogar",
    "Halgaunt",
    "Hygelac",
    "Egtheow",
    "Bohdan",
    "Wulfgar",
    "Hild",
    "Heatholaf",
    "Weder",
    "Vikhyat",
    "Unferth",
    "Sigemund",
    "Heremod",
    "Hengest",
    "Folcwald",
    "Frisian",
    "Hnaef",
    "Guthlaf",
    "Oslaf",
    "Yrmenlaf",
    "Garmund",
    "Freawaru",
    "Eadgils",
    "Onela",
    "Damien",
    "Erfor",
    "Weohstan",
    "Wulf",
    "Bulwye",
    "Lief",
    "Magnus",
    "Klatu",
    "Drus",
    "Hoku",
    "Kord",
    "Uther",
    "Ip",
    "Ulf",
    "Tharr",
    "Kaelim",
    "Ulric",
    "Alaric",
    "Ethelred",
    "Caden",
    "Elgifu",
    "Tostig",
    "Alcuin",
    "Emund",
    "Sigurd",
    "Gorm",
    "Hollis",
    "Ragnar",
    "Torkel",
    "Wulfstan",
    "Alban",
    "Barloc",
    "Bill",
    "Theodoric",
];

pub type Variant = (Class, Difficulty);

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Class {
    Rogue(Rogue),
    DesertMercenary(DesertMercenary),
    IronWolf(IronWolf),
    Barbarian(Barbarian),
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Rogue {
    Fire,
    Cold,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum DesertMercenary {
    Prayer,
    Defiance,
    BlessedAim,
    Thorns,
    HolyFreeze,
    Might,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum IronWolf {
    Fire,
    Cold,
    Lightning,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Barbarian {
    Bash,
    Frenzy,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Mercenary {
    dead: bool,
    id: u32,
    name_id: u16,
    name: &'static str,
    variant: Variant,
    experience: u32,
}

impl Default for Mercenary {
    fn default() -> Self {
        Self {
            dead: false,
            id: 0,
            name_id: 0,
            name: ROGUE_NAMES[0],
            variant: VARIANTS[0],
            experience: 0,
        }
    }
}

fn variant_id(variant: &Variant) -> Result<u16, GameLogicError> {
    let mut variant_id: u16 = 99;

    for i in 0..VARIANTS.len() {
        if *variant == VARIANTS[i] {
            variant_id = i as u16;
            break;
        }
    }
    if (variant_id as usize) > VARIANTS.len() {
        Err(GameLogicError {
            message: format!(
                "There is no mercenary ID for type {0:?} recruited in {1:?}",
                variant.0, variant.1
            ),
        })
    } else {
        Ok(variant_id)
    }
}

fn names_list(class: Class) -> &'static [&'static str] {
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
    mercenary.name = names_list[name_id as usize];

    mercenary.experience = u32::from_le_bytes(<[u8; 4]>::try_from(&data[10..14]).unwrap());

    Ok(mercenary)
}

pub fn generate_mercenary(mercenary: &Mercenary) -> Result<[u8; 14], GameLogicError> {
    let mut bytes: [u8; 14] = [0x00; 14];
    bytes[0..2].clone_from_slice(match mercenary.dead {
        true => &[0x01, 0x00],
        false => &[0x00, 0x00],
    });

    bytes[2..6].clone_from_slice(&mercenary.id.to_le_bytes());
    bytes[6..8].clone_from_slice(&mercenary.name_id.to_le_bytes());
    let variant_id = match variant_id(&mercenary.variant) {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    bytes[8..10].clone_from_slice(&variant_id.to_le_bytes());
    bytes[10..14].clone_from_slice(&mercenary.experience.to_le_bytes());

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let expected_result = Mercenary {
            dead: false,
            id: 3461679u32,
            name_id: 3,
            name: "Abhaya",
            variant: (Class::Rogue(Rogue::Cold), Difficulty::Normal),
            experience: 63722u32,
        };
        let bytes =
            [0x00, 0x00, 0x2F, 0xD2, 0x34, 0x00, 0x03, 0x00, 0x01, 0x00, 0xEA, 0xF8, 0x00, 0x00];
        let mut parsed_result: Mercenary = Mercenary::default();
        match parse(&bytes) {
            Ok(res) => parsed_result = res,
            Err(e) => {
                println! {"Test failed: {e:?}"}
            }
        };
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn generate_mercenary_test() {
        let expected_result =
            [0x00, 0x00, 0x2F, 0xD2, 0x34, 0x00, 0x03, 0x00, 0x01, 0x00, 0xEA, 0xF8, 0x00, 0x00];
        let merc = Mercenary {
            dead: false,
            id: 3461679u32,
            name_id: 3,
            name: "Abhaya",
            variant: (Class::Rogue(Rogue::Cold), Difficulty::Normal),
            experience: 63722u32,
        };
        let mut parsed_result: [u8; 14] = [0x00; 14];
        match generate_mercenary(&merc) {
            Ok(res) => parsed_result = res,
            Err(e) => {
                println! {"Test failed: {e:?}"}
            }
        };
        assert_eq!(parsed_result, expected_result);
    }
}
