use std::fmt;
use std::ops::Range;

use bit::BitIndex;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::Act;
use crate::ParseError;

pub mod consts;
use consts::*;
enum Section {
    Header,
    Normal,
    Nightmare,
    Hell,
    DifficultyHeader,
    DifficultyWaypointsValue,
}

impl Section {
    const fn range(self) -> Range<usize> {
        match self {
            Section::Header => 0..8,
            Section::Normal => 8..32,
            Section::Nightmare => 32..56,
            Section::Hell => 56..80,
            Section::DifficultyHeader => 0..2,
            Section::DifficultyWaypointsValue => 2..10,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaypointInfo {
    pub id: Waypoint,
    pub name: String,
    pub act: Act,
    pub acquired: bool,
}

impl fmt::Display for WaypointInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} - {1}: {2}", self.act, self.name, self.acquired)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Waypoints {
    pub normal: DifficultyWaypoints,
    pub nightmare: DifficultyWaypoints,
    pub hell: DifficultyWaypoints,
}

impl fmt::Display for Waypoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Normal:\n{0}\nNightmare:{1}\nHell:\n {2}",
            self.normal, self.nightmare, self.hell
        )
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]

pub struct ActWaypoints(pub [WaypointInfo; 9]);
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]

pub struct Act4Waypoints(pub [WaypointInfo; 3]);

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyWaypoints {
    pub act1: ActWaypoints,
    pub act2: ActWaypoints,
    pub act3: ActWaypoints,
    pub act4: Act4Waypoints,
    pub act5: ActWaypoints,
}

impl Act4Waypoints {
    /// Set whether single waypoint is acquired using WaypointID
    pub fn set(&mut self, wp_id: Waypoint, acquired: bool) {
        for wp in self.0.iter_mut() {
            if wp.id == wp_id {
                wp.acquired = acquired;
            }
        }
    }
    /// Get whether a single waypoint is acquired using WaypointID
    pub fn get(&self, wp_id: Waypoint) -> bool {
        for wp in self.0.clone().into_iter() {
            if wp.id == wp_id {
                return wp.acquired;
            }
        }
        false
    }
    /// Set whether all waypoints in this act are acquired
    pub fn set_all(&mut self, acquired: bool) {
        for wp in self.0.iter_mut() {
            wp.acquired = acquired;
        }
    }
    /// Set whether single waypoint is acquired
    pub fn set_num(&mut self, index: usize, acquired: bool) {
        if index >= self.0.len() {
            panic!(
                "Current act has only {0} waypoints but tried to set {1} on waypoint {2}",
                self.0.len(),
                acquired,
                index
            );
        }
        self.0[index].acquired = acquired;
    }
    /// Get whether a single waypoint is acquired
    pub fn get_num(&self, index: usize) -> bool {
        if index >= self.0.len() {
            panic!(
                "Current act has only {0} waypoints but tried to get value of waypoint {1}",
                self.0.len(),
                index
            );
        }
        self.0[index].acquired
    }
}

impl ActWaypoints {
    /// Set whether all waypoints in this act are acquired
    pub fn set_all(&mut self, acquired: bool) {
        for wp in self.0.iter_mut() {
            wp.acquired = acquired;
        }
    }
    /// Set whether single waypoint is acquired using WaypointID
    pub fn set(&mut self, wp_id: Waypoint, acquired: bool) {
        for wp in self.0.iter_mut() {
            if wp.id == wp_id {
                wp.acquired = acquired;
            }
        }
    }
    /// Get whether a single waypoint is acquired using WaypointID
    pub fn get(&self, wp_id: Waypoint) -> bool {
        for wp in self.0.clone().into_iter() {
            if wp.id == wp_id {
                return wp.acquired;
            }
        }
        false
    }

    /// Set whether single waypoint is acquired
    pub fn set_num(&mut self, index: usize, acquired: bool) {
        if index >= self.0.len() {
            panic!(
                "Current act has only {0} waypoints but tried to set {1} on waypoint {2}",
                self.0.len(),
                acquired,
                index
            );
        }
        self.0[index].acquired = acquired;
    }
    /// Get whether a single waypoint is acquired
    pub fn get_num(&self, index: usize) -> bool {
        if index >= self.0.len() {
            panic!(
                "Current act has only {0} waypoints but tried to get value of waypoint {1}",
                self.0.len(),
                index
            );
        }
        self.0[index].acquired
    }
}

impl fmt::Display for ActWaypoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut final_string = String::default();
        for wp in &self.0 {
            final_string.push_str(format!("{0}\n", wp).as_str());
        }
        write!(f, "{0}", final_string)
    }
}

impl fmt::Display for Act4Waypoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut final_string = String::default();
        for wp in &self.0 {
            final_string.push_str(format!("{0}\n", wp).as_str());
        }
        write!(f, "{0}", final_string)
    }
}

impl DifficultyWaypoints {
    pub fn set_all(&mut self, acquired: bool) {
        self.act1.set_all(acquired);
        self.act2.set_all(acquired);
        self.act3.set_all(acquired);
        self.act4.set_all(acquired);
        self.act5.set_all(acquired);
    }
}

impl fmt::Display for DifficultyWaypoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}\n{1}\n{2}\n{3}\n{4}", self.act1, self.act2, self.act3, self.act4, self.act5)
    }
}

impl Default for DifficultyWaypoints {
    fn default() -> Self {
        fn default_waypoints(act: Act) -> ActWaypoints {
            let mut default_waypoints: [WaypointInfo; 9] = <[WaypointInfo; 9]>::default();
            for i in 0..9 {
                default_waypoints[i].act = act;
                default_waypoints[i].name = match act {
                    Act::Act1 => String::from(NAMES_ACT1[i]),
                    Act::Act2 => String::from(NAMES_ACT2[i]),
                    Act::Act3 => String::from(NAMES_ACT3[i]),
                    Act::Act4 => String::from(NAMES_ACT4[i]),
                    Act::Act5 => String::from(NAMES_ACT5[i]),
                };
                let absolute_id: usize = i + match act {
                    Act::Act1 => 0,
                    Act::Act2 => 9,
                    Act::Act3 => 18,
                    Act::Act4 => 27,
                    Act::Act5 => 30,
                };
                default_waypoints[i].id = match Waypoint::try_from(absolute_id) {
                    Ok(res) => res,
                    Err(e) => panic!("Error getting default difficulty waypoint: {e}"),
                };
                default_waypoints[i].acquired = false
            }
            if act == Act::Act1 {
                default_waypoints[0].acquired = true;
            }
            ActWaypoints(default_waypoints)
        }
        Self {
            act1: default_waypoints(Act::Act1),
            act2: default_waypoints(Act::Act2),
            act3: default_waypoints(Act::Act3),
            act4: {
                let mut default_waypoints: [WaypointInfo; 3] = <[WaypointInfo; 3]>::default();
                for i in 0..3 {
                    default_waypoints[i].act = Act::Act4;
                    default_waypoints[i].name = String::from(NAMES_ACT4[i]);
                    default_waypoints[i].id = match Waypoint::try_from(27 + i) {
                        Ok(res) => res,
                        Err(e) => panic!("Error getting default difficulty waypoint: {e}"),
                    };
                    default_waypoints[i].acquired = false;
                }
                Act4Waypoints(default_waypoints)
            },
            act5: default_waypoints(Act::Act5),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub enum Waypoint {
    #[default]
    RogueEncampment = 0,
    ColdPlains = 1,
    StonyField = 2,
    DarkWood = 3,
    BlackMarsh = 4,
    OuterCloister = 5,
    Jail = 6,
    InnerCloister = 7,
    Catacombs = 8,
    LutGholein = 9,
    Sewers = 10,
    DryHills = 11,
    HallsOfTheDead = 12,
    FarOasis = 13,
    LostCity = 14,
    PalaceCellar = 15,
    ArcaneSanctuary = 16,
    CanyonOfTheMagi = 17,
    KurastDocks = 18,
    SpiderForest = 19,
    GreatMarsh = 20,
    FlayerJungle = 21,
    LowerKurast = 22,
    KurastBazaar = 23,
    UpperKurast = 24,
    Travincal = 25,
    DuranceOfHate = 26,
    PandemoniumFortress = 27,
    CityOfTheDamned = 28,
    RiverOfFlames = 29,
    Harrogath = 30,
    FrigidHighlands = 31,
    ArreatPlateau = 32,
    CrystallinePassage = 33,
    HallsOfPain = 34,
    GlacialTrail = 35,
    FrozenTundra = 36,
    TheAncientsWay = 37,
    WorldstoneKeep = 38,
}

impl From<Waypoint> for Act {
    fn from(id: Waypoint) -> Act {
        match id as usize {
            0..=8 => Act::Act1,
            9..=17 => Act::Act2,
            18..=26 => Act::Act3,
            27..=29 => Act::Act4,
            30..=38 => Act::Act5,
            _ => unreachable!(),
        }
    }
}

impl TryFrom<usize> for Waypoint {
    type Error = ParseError;
    fn try_from(id: usize) -> Result<Waypoint, ParseError> {
        match id {
            0 => Ok(Waypoint::RogueEncampment),
            1 => Ok(Waypoint::ColdPlains),
            2 => Ok(Waypoint::StonyField),
            3 => Ok(Waypoint::DarkWood),
            4 => Ok(Waypoint::BlackMarsh),
            5 => Ok(Waypoint::OuterCloister),
            6 => Ok(Waypoint::Jail),
            7 => Ok(Waypoint::InnerCloister),
            8 => Ok(Waypoint::Catacombs),
            9 => Ok(Waypoint::LutGholein),
            10 => Ok(Waypoint::Sewers),
            11 => Ok(Waypoint::DryHills),
            12 => Ok(Waypoint::HallsOfTheDead),
            13 => Ok(Waypoint::FarOasis),
            14 => Ok(Waypoint::LostCity),
            15 => Ok(Waypoint::PalaceCellar),
            16 => Ok(Waypoint::ArcaneSanctuary),
            17 => Ok(Waypoint::CanyonOfTheMagi),
            18 => Ok(Waypoint::KurastDocks),
            19 => Ok(Waypoint::SpiderForest),
            20 => Ok(Waypoint::GreatMarsh),
            21 => Ok(Waypoint::FlayerJungle),
            22 => Ok(Waypoint::LowerKurast),
            23 => Ok(Waypoint::KurastBazaar),
            24 => Ok(Waypoint::UpperKurast),
            25 => Ok(Waypoint::Travincal),
            26 => Ok(Waypoint::DuranceOfHate),
            27 => Ok(Waypoint::PandemoniumFortress),
            28 => Ok(Waypoint::CityOfTheDamned),
            29 => Ok(Waypoint::RiverOfFlames),
            30 => Ok(Waypoint::Harrogath),
            31 => Ok(Waypoint::FrigidHighlands),
            32 => Ok(Waypoint::ArreatPlateau),
            33 => Ok(Waypoint::CrystallinePassage),
            34 => Ok(Waypoint::HallsOfPain),
            35 => Ok(Waypoint::GlacialTrail),
            36 => Ok(Waypoint::FrozenTundra),
            37 => Ok(Waypoint::TheAncientsWay),
            38 => Ok(Waypoint::WorldstoneKeep),
            _ => Err(ParseError { message: format!("Cannot convert ID > 8 to waypoint: {id:?}") }),
        }
    }
}

impl Waypoints {
    pub fn parse(bytes: &[u8]) -> Waypoints {
        let mut waypoints = Waypoints::default();

        if bytes[Section::Header.range()] != SECTION_HEADER {
            warn!(
                "Found wrong waypoints header: {0:X?} (Expected: {1:X?}",
                &bytes[Section::Header.range()],
                SECTION_HEADER
            );
        }
        waypoints.normal =
            Waypoints::parse_difficulty(&bytes[Section::Normal.range()].try_into().unwrap());
        waypoints.nightmare =
            Waypoints::parse_difficulty(&bytes[Section::Nightmare.range()].try_into().unwrap());
        waypoints.hell =
            Waypoints::parse_difficulty(&bytes[Section::Hell.range()].try_into().unwrap());
        waypoints
    }

    fn parse_difficulty(bytes: &[u8; 24]) -> DifficultyWaypoints {
        let mut waypoints: DifficultyWaypoints = DifficultyWaypoints::default();
        if bytes[Section::DifficultyHeader.range()] != DIFFICULTY_HEADER {
            warn!(
                "Found wrong waypoint difficulty header: {0:X?} (Expected: {1:X?}",
                &bytes[0..2],
                DIFFICULTY_HEADER
            );
        }
        for id in 0..39 {
            let current_byte = bytes[2 + id / 8];
            let waypoint = Waypoint::try_from(id).unwrap();
            match Act::from(waypoint) {
                Act::Act1 => {
                    waypoints.act1.0[id] = WaypointInfo {
                        id: waypoint,
                        name: String::from(NAMES_ACT1[id]),
                        act: Act::Act1,
                        acquired: current_byte.bit(id % 8),
                    }
                }
                Act::Act2 => {
                    waypoints.act2.0[id - 9] = WaypointInfo {
                        id: waypoint,
                        name: String::from(NAMES_ACT2[id - 9]),
                        act: Act::Act2,
                        acquired: current_byte.bit(id % 8),
                    }
                }
                Act::Act3 => {
                    waypoints.act3.0[id - 18] = WaypointInfo {
                        id: waypoint,
                        name: String::from(NAMES_ACT3[id - 18]),
                        act: Act::Act3,
                        acquired: current_byte.bit(id % 8),
                    }
                }
                Act::Act4 => {
                    waypoints.act4.0[id - 27] = WaypointInfo {
                        id: waypoint,
                        name: String::from(NAMES_ACT4[id - 27]),
                        act: Act::Act4,
                        acquired: current_byte.bit(id % 8),
                    }
                }
                Act::Act5 => {
                    waypoints.act5.0[id - 30] = WaypointInfo {
                        id: waypoint,
                        name: String::from(NAMES_ACT5[id - 30]),
                        act: Act::Act5,
                        acquired: current_byte.bit(id % 8),
                    }
                }
            }
        }
        waypoints
    }

    pub fn to_bytes(&self) -> [u8; 80] {
        let mut bytes: [u8; 80] = [0x00; 80];
        bytes[Section::Header.range()].copy_from_slice(&SECTION_HEADER);
        bytes[Section::Normal.range()]
            .copy_from_slice(&Waypoints::difficulty_to_bytes(&self.normal));
        bytes[Section::Nightmare.range()]
            .copy_from_slice(&Waypoints::difficulty_to_bytes(&self.nightmare));
        bytes[Section::Hell.range()].copy_from_slice(&Waypoints::difficulty_to_bytes(&self.hell));
        bytes
    }

    fn difficulty_to_bytes(waypoints: &DifficultyWaypoints) -> [u8; 24] {
        let mut bytes: [u8; 24] = [0x00; 24];
        bytes[0..2].copy_from_slice(&DIFFICULTY_HEADER);
        fn fill_flags(waypoints: &[WaypointInfo], length: usize) -> u64 {
            let mut flags: u64 = 0;
            for i in 0..length {
                flags.set_bit(i, waypoints[i].acquired);
            }
            flags
        }
        let mut flags: u64 = 0;
        flags.set_bit_range(0..9, fill_flags(&waypoints.act1.0, 9).bit_range(0..9));
        flags.set_bit_range(9..18, fill_flags(&waypoints.act2.0, 9).bit_range(0..9));
        flags.set_bit_range(18..27, fill_flags(&waypoints.act3.0, 9).bit_range(0..9));
        flags.set_bit_range(27..30, fill_flags(&waypoints.act4.0, 3).bit_range(0..3));
        flags.set_bit_range(30..39, fill_flags(&waypoints.act5.0, 9).bit_range(0..9));
        bytes[Section::DifficultyWaypointsValue.range()].copy_from_slice(&u64::to_le_bytes(flags));
        bytes
    }
}
