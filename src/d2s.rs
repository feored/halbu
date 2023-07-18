const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

const VERSION_v100: u32 = 71;
const VERSION_v107: u32 = 87;
const VERSION_v108: u32 = 89;
const VERSION_v109: u32 = 92;
const VERSION_v110: u32 = 96;

const CLASS_AMAZON: u8 = 0;
const CLASS_SORCERESS: u8 = 1;
const CLASS_NECROMANCER: u8 = 2;
const CLASS_PALADIN: u8 = 3;
const CLASS_BARBARIAN: u8 = 4;
const CLASS_DRUID: u8 = 5;
const CLASS_ASSASSIN: u8 = 6;

const TITLES_CLASSIC_STANDARD_MALE : [&'static str; 4]= ["", "Sir", "Lord", "Baron"];
const TITLES_CLASSIC_STANDARD_FEMALE : [&'static str; 4]= ["", "Dame", "Lady", "Baroness"];
const TITLES_CLASSIC_HARDCORE_MALE : [&'static str; 4]= ["", "Count", "Duke", "King"];
const TITLES_CLASSIC_HARDCORE_FEMALE : [&'static str; 4]= ["", "Countess", "Duchess", "Queen"];
const TITLES_LOD_STANDARD_MALE : [&'static str; 4]= ["", "Slayer", "Champion", "Patriarch"];
const TITLES_LOD_STANDARD_FEMALE : [&'static str; 4]= ["", "Slayer", "Champion", "Matriarch"];
const TITLES_LOD_HARDCORE_MALE : [&'static str; 4]= ["", "Destroyer", "Conqueror", "Guardian"];
const TITLES_LOD_HARDCORE_FEMALE : [&'static str; 4]= ["", "Destroyer", "Conqueror", "Guardian"];

pub struct Character {
    pub version: Version,
    weapon_set: WeaponSet,
    name: String,
    pub status: Status,
    pub progression: u8,
    pub class: CharacterClass,
    level: u8,
    difficulty: (Difficulty, Act),
    pub map: u32,
}

pub struct Status{
    ladder: bool,
    expansion : bool,
    hardcore : bool,
    died : bool
}

impl Default for Status {
    fn default() -> Self{
        Self { expansion: (true), hardcore: (false), ladder: (false), died: (false) }
    }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            version: Version::v110,
            weapon_set: WeaponSet::Main,
            name: String::from(""),
            status: Status::default(),
            progression: 0,
            class: CharacterClass::Amazon,
            level: 1,
            difficulty: (Difficulty::Normal, Act::Act1),
            map: 0,
        }
    }
}

impl Character {
    // Getters and setters for fields that need validation
    pub fn weapon_set(&self) -> &Version {
        &self.version
    }
    // Secondary weapon set is only available in expansion
    pub fn set_weapon_set(&self, new_weapon_set: WeaponSet) {
        if new_weapon_set == WeaponSet::Main || self.status.contains(&CharacterStatus::Expansion) {
            self.weapon_set = new_weapon_set
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    // 16 bytes maximum, max one - or _
    pub fn set_name(&self, new_name: String) {
        if new_name.len() <= 16
            && new_name.matches("-").count() <= 1
            && new_name.matches("_").count() <= 1
        {
            self.name = new_name;
        }
    }

    pub fn level(&self) -> &u8{
        &self.level
    }
    pub fn set_level(&self, new_level:u8){
        if new_level > 0 && new_level < 100{
            self.level = new_level
        }
    }

    pub fn difficulty(&self) -> &(Difficulty, Act){
        &self.difficulty
    }
    pub fn set_difficulty(&self, new_difficulty : (Difficulty, Act)){
        let (new_diff, new_act) = new_difficulty;
        if new_act == Act::Act5 && !self.status.contains
    }
}

pub enum Difficulty {
    Normal,
    Nightmare,
    Hell,
}
pub enum Act {
    Act1,
    Act2,
    Act3,
    Act4,
    Act5,
}
#[derive(PartialEq, Eq)]
pub enum WeaponSet {
    Main,
    Secondary,
}

#[derive(PartialEq, Eq)]
pub enum CharacterClass {
    Amazon,
    Sorceress,
    Necromancer,
    Paladin,
    Barbarian,
    Druid,
    Assassin,
}

struct HeaderSection {
    pub offset: usize,
    pub bytes: usize,
}
#[derive(Debug)]
pub enum HeaderID {
    Signature,
    VersionID,
    FileSize,
    Checksum,
    WeaponSet,
    CharacterName,
    CharacterStatus,
    CharacterProgression,
}

fn get_header_bytes_range(id: HeaderID) -> (usize, usize) {
    let header_data: HeaderSection = get_header_data(id);
    (header_data.offset, header_data.offset + header_data.bytes)
}

fn get_header_data(id: HeaderID) -> HeaderSection {
    match id {
        HeaderID::Signature => HeaderSection {
            offset: (0),
            bytes: (4),
        },
        HeaderID::VersionID => HeaderSection {
            offset: (4),
            bytes: (4),
        },
        HeaderID::FileSize => HeaderSection {
            offset: (8),
            bytes: (4),
        },
        HeaderID::Checksum => HeaderSection {
            offset: (12),
            bytes: (4),
        },
        HeaderID::WeaponSet => HeaderSection {
            offset: (16),
            bytes: (4),
        },
        HeaderID::CharacterName => HeaderSection {
            offset: (20),
            bytes: (16),
        },
        HeaderID::CharacterStatus => HeaderSection {
            offset: (36),
            bytes: (1),
        },
        HeaderID::CharacterProgression => HeaderSection {
            offset: (37),
            bytes: (1),
        },
    }
}

#[derive(Debug)]
pub enum Version {
    v100,
    v107,
    v108,
    v109,
    v110,
}

pub fn get_version(version_bytes: &[u8; 4]) -> Result<Version, &'static str> {
    let version_number: u32 = u32::from_le_bytes(*version_bytes);
    match version_number {
        VERSION_v100 => Ok(Version::v100),
        VERSION_v107 => Ok(Version::v107),
        VERSION_v108 => Ok(Version::v108),
        VERSION_v109 => Ok(Version::v109),
        VERSION_v110 => Ok(Version::v110),
        _ => Err("version ID does not match any known version of the game."),
    }
}

pub fn check_valid_signature(bytes: &Vec<u8>) -> bool {
    let (header_start, header_end) = get_header_bytes_range(HeaderID::Signature);
    bytes[header_start..header_end] == SIGNATURE
}

pub fn calc_checksum(bytes: &Vec<u8>) -> i32 {
    let mut checksum: i32 = 0;
    let (checksum_start, checksum_end) = get_header_bytes_range(HeaderID::Checksum);
    for i in 0..bytes.len() {
        let mut ch: i32 = bytes[i] as i32;
        if i >= checksum_start && i < checksum_end {
            ch = 0;
        }
        checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
    }
    checksum
}

impl Character {
    // Return the appropriate title accounting for difficulties beaten
    pub fn title(&self) -> String {
        let male: bool = [
            CharacterClass::Barbarian,
            CharacterClass::Paladin,
            CharacterClass::Necromancer,
            CharacterClass::Druid,
        ]
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
                (_, _) => return Default::default(),
                (false, false) => return String::from(TITLES_CLASSIC_STANDARD_FEMALE[stage]),
                (false, true) => return String::from(TITLES_CLASSIC_STANDARD_MALE[stage]),
                (true, false) => return String::from(TITLES_CLASSIC_HARDCORE_FEMALE[stage]),
                (true, true) => return String::from(TITLES_CLASSIC_HARDCORE_MALE[stage])
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
                (_, _) => return Default::default(),
                (false, false) => return String::from(TITLES_LOD_STANDARD_FEMALE[stage]),
                (false, true) => return String::from(TITLES_LOD_STANDARD_MALE[stage]),
                (true, false) => return String::from(TITLES_LOD_HARDCORE_FEMALE[stage]),
                (true, true) => return String::from(TITLES_LOD_HARDCORE_MALE[stage])
            }
        }
    }
}
