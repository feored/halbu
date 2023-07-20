use super::Difficulty;

//TOOD ADD ROGUE NAMES
// const ROGUE_NAMES : &'static [&'static str; 41] = [
//     "Aliza",
//     "Ampliza",
//     "Annor"
//     "Abhaya",
//     "Elly",
//     "Paige",
//     "Basanti",
//     "Blaise",
//     ""
// ]

const MERCENARY_VARIANTS: &'static [MercenaryType; 39] = &[
    MercenaryType {
        class: MercenaryClass::Rogue,
        variant: MercenaryVariant::Rogue(RogueVariant::Fire),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::Rogue,
        variant: MercenaryVariant::Rogue(RogueVariant::Cold),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::Rogue,
        variant: MercenaryVariant::Rogue(RogueVariant::Fire),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::Rogue,
        variant: MercenaryVariant::Rogue(RogueVariant::Cold),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::Rogue,
        variant: MercenaryVariant::Rogue(RogueVariant::Fire),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::Rogue,
        variant: MercenaryVariant::Rogue(RogueVariant::Cold),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Prayer),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Defiance),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::BlessedAim),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Thorns),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::HolyFreeze),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Might),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Prayer),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Defiance),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::BlessedAim),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Fire),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Cold),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Lightning),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Fire),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Cold),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Lightning),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Fire),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Cold),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::IronWolf,
        variant: MercenaryVariant::IronWolf(IronWolfVariant::Lightning),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Bash),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Bash),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Bash),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Bash),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Bash),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Bash),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Prayer),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Defiance),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::BlessedAim),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Thorns),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::HolyFreeze),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::DesertMercenary,
        variant: MercenaryVariant::DesertMercenary(DesertMercenaryVariant::Might),
        difficulty: Difficulty::Hell,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Frenzy),
        difficulty: Difficulty::Normal,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Frenzy),
        difficulty: Difficulty::Nightmare,
    },
    MercenaryType {
        class: MercenaryClass::Barbarian,
        variant: MercenaryVariant::Barbarian(BarbarianVariant::Frenzy),
        difficulty: Difficulty::Hell,
    },
];

#[derive(PartialEq, Eq)]
enum RogueVariant {
    Fire,
    Cold,
}

#[derive(PartialEq, Eq)]
enum DesertMercenaryVariant {
    Prayer,
    Defiance,
    BlessedAim,
    Thorns,
    HolyFreeze,
    Might,
}

#[derive(PartialEq, Eq)]
enum IronWolfVariant {
    Fire,
    Cold,
    Lightning,
}

#[derive(PartialEq, Eq)]
enum BarbarianVariant {
    Bash,
    Frenzy,
}

#[derive(PartialEq, Eq)]
enum MercenaryVariant {
    Rogue(RogueVariant),
    DesertMercenary(DesertMercenaryVariant),
    IronWolf(IronWolfVariant),
    Barbarian(BarbarianVariant),
}

impl Default for MercenaryType {
    fn default() -> Self {
        Self {
            class: MercenaryClass::Rogue,
            variant: MercenaryVariant::Rogue(RogueVariant::Cold),
            difficulty: Difficulty::Normal,
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct MercenaryType {
    class: MercenaryClass,
    variant: MercenaryVariant,
    difficulty: Difficulty,
}

impl From<u16> for &MercenaryType {
    fn from(id: u16) -> Self {
        let id_as_usize = usize::from(id);
        if id_as_usize > MERCENARY_VARIANTS.len() {
            return &MERCENARY_VARIANTS[0];
        } else {
            return &MERCENARY_VARIANTS[id_as_usize];
        }
    }
}

impl From<&MercenaryType> for u16 {
    fn from(mercenary_type: &MercenaryType) -> Self {
        for i in 0..MERCENARY_VARIANTS.len() {
            if MERCENARY_VARIANTS[i] == *mercenary_type {
                return i as u16;
            }
        }
        return 0;
    }
}

#[derive(PartialEq, Eq)]
pub enum MercenaryClass {
    Rogue,
    DesertMercenary,
    IronWolf,
    Barbarian,
}

pub struct Mercenary {
    dead: bool,
    id: u32,
    name_id: u16,
    mercenary_type: MercenaryType,
    experience: u32,
}

impl Default for Mercenary {
    fn default() -> Self {
        Self {
            dead: false,
            id: 0,
            name_id: 0,
            mercenary_type: MercenaryType::default(),
            experience: 0,
        }
    }
}
