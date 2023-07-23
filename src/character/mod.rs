use crate::Act;
use crate::Difficulty;

pub mod mercenary;

const AMAZON: u8 = 0;
const SORCERESS: u8 = 1;
const NECROMANCER: u8 = 2;
const PALADIN: u8 = 3;
const BARBARIAN: u8 = 4;
const DRUID: u8 = 5;
const ASSASSIN: u8 = 6;

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
    name: String,
    pub status: Status,
    pub progression: u8,
    pub class: Class,
    level: u8,
    difficulty: (Difficulty, Act),
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
    Secondary,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Class {
    Amazon,
    Sorceress,
    Necromancer,
    Paladin,
    Barbarian,
    Druid,
    Assassin,
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

impl Default for Character {
    fn default() -> Self {
        Self {
            weapon_set: WeaponSet::Main,
            name: String::from(""),
            status: Status::default(),
            progression: 0,
            class: Class::Amazon,
            level: 1,
            difficulty: (Difficulty::Normal, Act::Act1),
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

    pub fn difficulty(&self) -> &(Difficulty, Act) {
        &self.difficulty
    }
    pub fn set_difficulty(&mut self, new_difficulty: (Difficulty, Act)) {
        if new_difficulty.1 == Act::Act5 && !self.status.expansion {
            return;
        }
        //TODO: set progression accordingly
        self.difficulty = new_difficulty
    }
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
