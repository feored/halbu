use std::fmt;
use std::ops::Range;

use bit::BitIndex;
use serde::{Deserialize, Serialize};

use crate::Act;
use crate::ParseHardError;

pub mod consts;
use consts::*;
#[cfg(test)]
mod tests;

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

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum WaypointError {
    WrongAct { waypoint: Waypoint, expected: Act, actual: Act },
    IndexOutOfRange { act: Act, index: u8, max_index: u8 },
}

impl fmt::Display for WaypointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongAct { waypoint, expected, actual } => write!(
                f,
                "Waypoint {:?} belongs to {:?}; expected {:?}.",
                waypoint, actual, expected
            ),
            Self::IndexOutOfRange { act, index, max_index } => write!(
                f,
                "Waypoint index {index} is out of range for {:?}; expected 0..={max_index}.",
                act
            ),
        }
    }
}

impl std::error::Error for WaypointError {}

mod waypoint_state_array {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::WaypointState;

    pub(super) fn serialize<S, const WAYPOINT_COUNT: usize>(
        waypoints: &[WaypointState; WAYPOINT_COUNT],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        waypoints.as_slice().serialize(serializer)
    }

    pub(super) fn deserialize<'de, D, const WAYPOINT_COUNT: usize>(
        deserializer: D,
    ) -> Result<[WaypointState; WAYPOINT_COUNT], D::Error>
    where
        D: Deserializer<'de>,
    {
        let waypoints = Vec::<WaypointState>::deserialize(deserializer)?;
        let found = waypoints.len();
        waypoints.try_into().map_err(|_| {
            D::Error::custom(format!("Expected {WAYPOINT_COUNT} waypoints, found {found}."))
        })
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WaypointState {
    pub id: Waypoint,
    pub acquired: bool,
}

impl fmt::Display for WaypointState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}: {}", self.id.act(), self.id.name(), self.acquired)
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
        write!(f, "Normal:\n{}\nNightmare:\n{}\nHell:\n{}", self.normal, self.nightmare, self.hell)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct ActWaypoints<const WAYPOINT_COUNT: usize> {
    act: Act,
    #[serde(with = "waypoint_state_array")]
    waypoints: [WaypointState; WAYPOINT_COUNT],
}

impl<const WAYPOINT_COUNT: usize> ActWaypoints<WAYPOINT_COUNT> {
    fn new_for_act(act: Act) -> Self {
        let expected_waypoint_count = Waypoint::count_for_act(act);
        assert!(
            WAYPOINT_COUNT == expected_waypoint_count,
            "Act {act:?} expects {expected_waypoint_count} waypoints, got {WAYPOINT_COUNT}."
        );

        let mut waypoints = [WaypointState::default(); WAYPOINT_COUNT];
        for (index, waypoint_state) in waypoints.iter_mut().enumerate() {
            let waypoint = Waypoint::from_act_index(act, index)
                .expect("Act waypoint defaults must map to valid waypoint IDs.");
            *waypoint_state = WaypointState { id: waypoint, acquired: false };
        }

        Self { act, waypoints }
    }

    pub const fn act(&self) -> Act {
        self.act
    }

    pub const fn len(&self) -> usize {
        WAYPOINT_COUNT
    }

    pub const fn is_empty(&self) -> bool {
        WAYPOINT_COUNT == 0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, WaypointState> {
        self.waypoints.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, WaypointState> {
        self.waypoints.iter_mut()
    }

    /// Set all waypoints in this act.
    pub fn set_all(&mut self, acquired: bool) {
        for waypoint in self.iter_mut() {
            waypoint.acquired = acquired;
        }
    }

    /// Set one waypoint by id.
    pub fn set(&mut self, waypoint: Waypoint, acquired: bool) -> Result<(), WaypointError> {
        if waypoint.act() != self.act {
            return Err(WaypointError::WrongAct {
                waypoint,
                expected: self.act,
                actual: waypoint.act(),
            });
        }

        self.waypoints[waypoint.index_within_act()].acquired = acquired;
        Ok(())
    }

    /// Read one waypoint by id.
    pub fn get(&self, waypoint: Waypoint) -> Result<bool, WaypointError> {
        if waypoint.act() != self.act {
            return Err(WaypointError::WrongAct {
                waypoint,
                expected: self.act,
                actual: waypoint.act(),
            });
        }

        Ok(self.waypoints[waypoint.index_within_act()].acquired)
    }

    /// Set one waypoint by index.
    pub fn set_by_index(&mut self, index: usize, acquired: bool) -> Result<(), WaypointError> {
        if index >= self.len() {
            return Err(WaypointError::IndexOutOfRange {
                act: self.act,
                index: index as u8,
                max_index: (self.len() - 1) as u8,
            });
        }

        self.waypoints[index].acquired = acquired;
        Ok(())
    }

    /// Read one waypoint by index.
    pub fn get_by_index(&self, index: usize) -> Result<bool, WaypointError> {
        if index >= self.len() {
            return Err(WaypointError::IndexOutOfRange {
                act: self.act,
                index: index as u8,
                max_index: (self.len() - 1) as u8,
            });
        }

        Ok(self.waypoints[index].acquired)
    }
}

impl<const WAYPOINT_COUNT: usize> fmt::Display for ActWaypoints<WAYPOINT_COUNT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, waypoint) in self.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{waypoint}")?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyWaypoints {
    pub act1: ActWaypoints<9>,
    pub act2: ActWaypoints<9>,
    pub act3: ActWaypoints<9>,
    pub act4: ActWaypoints<3>,
    pub act5: ActWaypoints<9>,
}

impl DifficultyWaypoints {
    pub fn set_all(&mut self, acquired: bool) {
        self.act1.set_all(acquired);
        self.act2.set_all(acquired);
        self.act3.set_all(acquired);
        self.act4.set_all(acquired);
        self.act5.set_all(acquired);
    }

    fn set(&mut self, waypoint: Waypoint, acquired: bool) -> Result<(), WaypointError> {
        match waypoint.act() {
            Act::Act1 => self.act1.set(waypoint, acquired),
            Act::Act2 => self.act2.set(waypoint, acquired),
            Act::Act3 => self.act3.set(waypoint, acquired),
            Act::Act4 => self.act4.set(waypoint, acquired),
            Act::Act5 => self.act5.set(waypoint, acquired),
        }
    }

    fn get(&self, waypoint: Waypoint) -> Result<bool, WaypointError> {
        match waypoint.act() {
            Act::Act1 => self.act1.get(waypoint),
            Act::Act2 => self.act2.get(waypoint),
            Act::Act3 => self.act3.get(waypoint),
            Act::Act4 => self.act4.get(waypoint),
            Act::Act5 => self.act5.get(waypoint),
        }
    }
}

impl fmt::Display for DifficultyWaypoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Act I:\n{}\nAct II:\n{}\nAct III:\n{}\nAct IV:\n{}\nAct V:\n{}",
            self.act1, self.act2, self.act3, self.act4, self.act5
        )
    }
}

impl Default for DifficultyWaypoints {
    fn default() -> Self {
        let mut act1 = ActWaypoints::<9>::new_for_act(Act::Act1);
        act1.set_by_index(0, true).expect("Act I index 0 must always exist.");

        Self {
            act1,
            act2: ActWaypoints::<9>::new_for_act(Act::Act2),
            act3: ActWaypoints::<9>::new_for_act(Act::Act3),
            act4: ActWaypoints::<3>::new_for_act(Act::Act4),
            act5: ActWaypoints::<9>::new_for_act(Act::Act5),
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

impl Waypoint {
    pub const ALL: [Self; 39] = [
        Self::RogueEncampment,
        Self::ColdPlains,
        Self::StonyField,
        Self::DarkWood,
        Self::BlackMarsh,
        Self::OuterCloister,
        Self::Jail,
        Self::InnerCloister,
        Self::Catacombs,
        Self::LutGholein,
        Self::Sewers,
        Self::DryHills,
        Self::HallsOfTheDead,
        Self::FarOasis,
        Self::LostCity,
        Self::PalaceCellar,
        Self::ArcaneSanctuary,
        Self::CanyonOfTheMagi,
        Self::KurastDocks,
        Self::SpiderForest,
        Self::GreatMarsh,
        Self::FlayerJungle,
        Self::LowerKurast,
        Self::KurastBazaar,
        Self::UpperKurast,
        Self::Travincal,
        Self::DuranceOfHate,
        Self::PandemoniumFortress,
        Self::CityOfTheDamned,
        Self::RiverOfFlames,
        Self::Harrogath,
        Self::FrigidHighlands,
        Self::ArreatPlateau,
        Self::CrystallinePassage,
        Self::HallsOfPain,
        Self::GlacialTrail,
        Self::FrozenTundra,
        Self::TheAncientsWay,
        Self::WorldstoneKeep,
    ];

    pub const fn absolute_index(self) -> usize {
        self as usize
    }

    pub const fn act(self) -> Act {
        match self.absolute_index() {
            0..=8 => Act::Act1,
            9..=17 => Act::Act2,
            18..=26 => Act::Act3,
            27..=29 => Act::Act4,
            30..=38 => Act::Act5,
            _ => unreachable!(),
        }
    }

    pub const fn count_for_act(act: Act) -> usize {
        match act {
            Act::Act1 | Act::Act2 | Act::Act3 | Act::Act5 => 9,
            Act::Act4 => 3,
        }
    }

    const fn act_start_index(act: Act) -> usize {
        match act {
            Act::Act1 => 0,
            Act::Act2 => 9,
            Act::Act3 => 18,
            Act::Act4 => 27,
            Act::Act5 => 30,
        }
    }

    pub const fn index_within_act(self) -> usize {
        self.absolute_index() - Self::act_start_index(self.act())
    }

    pub fn name(self) -> &'static str {
        let index_within_act = self.index_within_act();
        match self.act() {
            Act::Act1 => NAMES_ACT1[index_within_act],
            Act::Act2 => NAMES_ACT2[index_within_act],
            Act::Act3 => NAMES_ACT3[index_within_act],
            Act::Act4 => NAMES_ACT4[index_within_act],
            Act::Act5 => NAMES_ACT5[index_within_act],
        }
    }

    pub fn from_act_index(act: Act, index_within_act: usize) -> Result<Self, ParseHardError> {
        if index_within_act >= Self::count_for_act(act) {
            return Err(ParseHardError {
                message: format!(
                    "Invalid waypoint index {index_within_act} for {:?}; expected 0..={}.",
                    act,
                    Self::count_for_act(act) - 1
                ),
            });
        }

        Self::try_from(Self::act_start_index(act) + index_within_act)
    }
}

impl From<Waypoint> for Act {
    fn from(waypoint: Waypoint) -> Act {
        waypoint.act()
    }
}

impl TryFrom<usize> for Waypoint {
    type Error = ParseHardError;

    fn try_from(id: usize) -> Result<Waypoint, ParseHardError> {
        Waypoint::ALL.get(id).copied().ok_or_else(|| ParseHardError {
            message: format!("Invalid waypoint id {id}; expected 0..={}.", Waypoint::ALL.len() - 1),
        })
    }
}

impl Waypoints {
    pub fn parse(bytes: &[u8]) -> Result<Waypoints, ParseHardError> {
        if bytes.len() < Section::Hell.range().end {
            return Err(ParseHardError {
                message: format!(
                    "Waypoints section is truncated: expected {} bytes, found {}.",
                    Section::Hell.range().end,
                    bytes.len()
                ),
            });
        }

        if bytes[Section::Header.range()] != SECTION_HEADER {
            return Err(ParseHardError {
                message: format!(
                    "Found wrong waypoints header: {:X?} (Expected: {SECTION_HEADER:X?})",
                    &bytes[Section::Header.range()]
                ),
            });
        }

        Ok(Self {
            normal: Waypoints::parse_difficulty(&bytes[Section::Normal.range()])?,
            nightmare: Waypoints::parse_difficulty(&bytes[Section::Nightmare.range()])?,
            hell: Waypoints::parse_difficulty(&bytes[Section::Hell.range()])?,
        })
    }

    fn parse_difficulty(bytes: &[u8]) -> Result<DifficultyWaypoints, ParseHardError> {
        if bytes.len() < 24 {
            return Err(ParseHardError {
                message: format!(
                    "Waypoint difficulty section is truncated: expected 24 bytes, found {}.",
                    bytes.len()
                ),
            });
        }

        if bytes[Section::DifficultyHeader.range()] != DIFFICULTY_HEADER {
            return Err(ParseHardError {
                message: format!(
                    "Found wrong waypoint difficulty header: {:X?} (Expected: {DIFFICULTY_HEADER:X?})",
                    &bytes[Section::DifficultyHeader.range()]
                ),
            });
        }

        let mut difficulty_waypoints = DifficultyWaypoints::default();

        for waypoint in Waypoint::ALL {
            let absolute_index = waypoint.absolute_index();
            let current_byte = bytes[2 + absolute_index / 8];
            let acquired = current_byte.bit(absolute_index % 8);

            difficulty_waypoints
                .set(waypoint, acquired)
                .expect("Waypoint-to-act mapping must stay internally consistent.");
        }

        Ok(difficulty_waypoints)
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
        bytes[Section::DifficultyHeader.range()].copy_from_slice(&DIFFICULTY_HEADER);

        let mut flags: u64 = 0;
        for waypoint in Waypoint::ALL {
            let acquired = waypoints
                .get(waypoint)
                .expect("Waypoint-to-act mapping must stay internally consistent.");
            flags.set_bit(waypoint.absolute_index(), acquired);
        }

        bytes[Section::DifficultyWaypointsValue.range()].copy_from_slice(&u64::to_le_bytes(flags));
        bytes
    }
}
