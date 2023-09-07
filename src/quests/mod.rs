use std::collections::HashSet;
use std::fmt;
use std::ops::Range;
use std::str;

use bit::BitIndex;
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::convert::u16_from;

const SECTION_LENGTH: usize = 298;
const SECTION_HEADER: [u8; 10] = [0x57, 0x6F, 0x6F, 0x21, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x01]; // Woo! + header

pub enum Section {
    Header,
    Normal,
    Nightmare,
    Hell,
    Act1,
    Act2,
    Act3,
    Act4,
    Act5,
}

impl Section {
    const fn range(self) -> Range<usize> {
        match self {
            Section::Header => 0..10,
            Section::Normal => 10..106,
            Section::Nightmare => 106..202,
            Section::Hell => 202..298,
            Section::Act1 => 0..16,
            Section::Act2 => 16..32,
            Section::Act3 => 32..48,
            Section::Act4 => 48..64,
            Section::Act5 => 64..84,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize, Hash)]
pub enum QuestFlag {
    RewardGranted = 0,
    RewardPending = 1,
    Started = 2,
    LeaveTown = 3,
    EnterArea = 4,
    Custom1 = 5,
    Custom2 = 6,
    Custom3 = 7,
    Custom4 = 8,
    Custom5 = 9,
    Custom6 = 10,
    Custom7 = 11,
    UpdateQuestLog = 12,
    PrimaryGoalDone = 13,
    CompletedNow = 14,
    CompletedBefore = 15,
}

const ALL_QUEST_FLAGS: [QuestFlag; 16] = [
    QuestFlag::RewardGranted,
    QuestFlag::RewardPending,
    QuestFlag::Started,
    QuestFlag::LeaveTown,
    QuestFlag::EnterArea,
    QuestFlag::Custom1,
    QuestFlag::Custom2,
    QuestFlag::Custom3,
    QuestFlag::Custom4,
    QuestFlag::Custom5,
    QuestFlag::Custom6,
    QuestFlag::Custom7,
    QuestFlag::UpdateQuestLog,
    QuestFlag::PrimaryGoalDone,
    QuestFlag::CompletedNow,
    QuestFlag::CompletedBefore,
];

/// Representation of the state of a quest. Stores a collection of `QuestFlag` values in a HashSet to indicate which flags are active.
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub state: HashSet<QuestFlag>,
}

impl From<u16> for Quest {
    fn from(value: u16) -> Self {
        let mut quest = Quest::default();
        for qf in ALL_QUEST_FLAGS {
            if value.bit(qf as usize) {
                quest.state.insert(qf);
            }
        }
        quest
    }
}

pub fn apply_flag(short: &mut u16, flag: QuestFlag, value: bool) {
    short.set_bit(flag as usize, value);
}

impl Quest {
    /// Computes and returns the `u16` value representing the state of the quest.
    pub fn value(&self) -> u16 {
        let mut value: u16 = 0;
        for flag in self.state.iter() {
            apply_flag(&mut value, *flag, true);
        }
        value
    }
}

impl fmt::Display for Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State: {0:?}", self.state)
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Act1 {
    pub prologue: Quest,
    pub q1: Quest,
    pub q2: Quest,
    pub q3: Quest,
    pub q4: Quest,
    pub q5: Quest,
    pub q6: Quest,
    pub completion: Quest,
}

impl fmt::Display for Act1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Prologue: {0}\nQuest 1: {1}\nQuest 2: {2}\nQuest 3: {3}\nQuest 4: {4}\nQuest 5: {5}\nQuest 6: {6}\nCompletion: {7}",
            self.prologue, self.q1, self.q2, self.q3, self.q4, self.q5, self.q6, self.completion
        )
    }
}

impl Act1 {
    fn to_bytes(&self) -> [u8; 16] {
        let mut quest_values: [u16; 8] = [0; 8];
        quest_values[0] = self.prologue.value();
        quest_values[1] = self.q1.value();
        quest_values[2] = self.q2.value();
        quest_values[3] = self.q3.value();
        quest_values[4] = self.q4.value();
        quest_values[5] = self.q5.value();
        quest_values[6] = self.q6.value();
        quest_values[7] = self.completion.value();
        let mut quest_bytes: [u8; 16] = [0; 16];
        for (index, val) in quest_values.iter().enumerate() {
            let start: usize = index * 2;
            quest_bytes[start..start + 2].copy_from_slice(&u16::to_le_bytes(*val));
        }
        quest_bytes
    }
}

impl From<&[u8]> for Act1 {
    fn from(value: &[u8]) -> Self {
        let mut complete_act = Act1::default();
        complete_act.prologue = Quest::from(u16_from(&value[0..2], "A1 Prologue"));
        complete_act.q1 = Quest::from(u16_from(&value[2..4], "A1 Q1"));
        complete_act.q2 = Quest::from(u16_from(&value[4..6], "A1 Q2"));
        complete_act.q3 = Quest::from(u16_from(&value[6..8], "A1 Q3"));
        complete_act.q4 = Quest::from(u16_from(&value[8..10], "A1 Q4"));
        complete_act.q5 = Quest::from(u16_from(&value[10..12], "A1 Q5"));
        complete_act.q6 = Quest::from(u16_from(&value[12..14], "A1 Q6"));
        complete_act.completion = Quest::from(u16_from(&value[14..16], "A1 Completion"));
        complete_act
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Act2 {
    pub prologue: Quest,
    pub q1: Quest,
    pub q2: Quest,
    pub q3: Quest,
    pub q4: Quest,
    pub q5: Quest,
    pub q6: Quest,
    pub completion: Quest,
}

impl fmt::Display for Act2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Prologue: {0}\nQuest 1: {1}\nQuest 2: {2}\nQuest 3: {3}\nQuest 4: {4}\nQuest 5: {5}\nQuest 6: {6}\nCompletion: {7}",
            self.prologue, self.q1, self.q2, self.q3, self.q4, self.q5, self.q6, self.completion
        )
    }
}

impl Act2 {
    fn to_bytes(&self) -> [u8; 16] {
        let mut quest_values: [u16; 8] = [0; 8];
        quest_values[0] = self.prologue.value();
        quest_values[1] = self.q1.value();
        quest_values[2] = self.q2.value();
        quest_values[3] = self.q3.value();
        quest_values[4] = self.q4.value();
        quest_values[5] = self.q5.value();
        quest_values[6] = self.q6.value();
        quest_values[7] = self.completion.value();
        let mut quest_bytes: [u8; 16] = [0; 16];
        for (index, val) in quest_values.iter().enumerate() {
            let start: usize = index * 2;
            quest_bytes[start..start + 2].copy_from_slice(&u16::to_le_bytes(*val));
        }
        quest_bytes
    }
}

impl From<&[u8]> for Act2 {
    fn from(value: &[u8]) -> Self {
        let mut complete_act = Act2::default();
        complete_act.prologue = Quest::from(u16_from(&value[0..2], "A2 Prologue"));
        complete_act.q1 = Quest::from(u16_from(&value[2..4], "A2 Q1"));
        complete_act.q2 = Quest::from(u16_from(&value[4..6], "A2 Q2"));
        complete_act.q3 = Quest::from(u16_from(&value[6..8], "A2 Q3"));
        complete_act.q4 = Quest::from(u16_from(&value[8..10], "A2 Q4"));
        complete_act.q5 = Quest::from(u16_from(&value[10..12], "A2 Q5"));
        complete_act.q6 = Quest::from(u16_from(&value[12..14], "A2 Q6"));
        complete_act.completion = Quest::from(u16_from(&value[14..16], "A2 Completion"));
        complete_act
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Act3 {
    pub prologue: Quest,
    pub q1: Quest,
    pub q2: Quest,
    pub q3: Quest,
    pub q4: Quest,
    pub q5: Quest,
    pub q6: Quest,
    pub completion: Quest,
}

impl fmt::Display for Act3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Prologue: {0}\nQuest 1: {1}\nQuest 2: {2}\nQuest 3: {3}\nQuest 4: {4}\nQuest 5: {5}\nQuest 6: {6}\nCompletion: {7}",
            self.prologue, self.q1, self.q2, self.q3, self.q4, self.q5, self.q6, self.completion
        )
    }
}

impl Act3 {
    fn to_bytes(&self) -> [u8; 16] {
        let mut quest_values: [u16; 8] = [0; 8];
        quest_values[0] = self.prologue.value();
        quest_values[1] = self.q1.value();
        quest_values[2] = self.q2.value();
        quest_values[3] = self.q3.value();
        quest_values[4] = self.q4.value();
        quest_values[5] = self.q5.value();
        quest_values[6] = self.q6.value();
        quest_values[7] = self.completion.value();
        let mut quest_bytes: [u8; 16] = [0; 16];
        for (index, val) in quest_values.iter().enumerate() {
            let start: usize = index * 2;
            quest_bytes[start..start + 2].copy_from_slice(&u16::to_le_bytes(*val));
        }
        quest_bytes
    }
}

impl From<&[u8]> for Act3 {
    fn from(value: &[u8]) -> Self {
        let mut complete_act = Act3::default();
        complete_act.prologue = Quest::from(u16_from(&value[0..2], "A3 Prologue"));
        complete_act.q1 = Quest::from(u16_from(&value[2..4], "A3 Q1"));
        complete_act.q2 = Quest::from(u16_from(&value[4..6], "A3 Q2"));
        complete_act.q3 = Quest::from(u16_from(&value[6..8], "A3 Q3"));
        complete_act.q4 = Quest::from(u16_from(&value[8..10], "A3 Q4"));
        complete_act.q5 = Quest::from(u16_from(&value[10..12], "A3 Q5"));
        complete_act.q6 = Quest::from(u16_from(&value[12..14], "A3 Q6"));
        complete_act.completion = Quest::from(u16_from(&value[14..16], "A3 Complete"));
        complete_act
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Act4 {
    pub prologue: Quest,
    pub q1: Quest,
    pub q2: Quest,
    pub q3: Quest,
    pub completion: Quest,
    pub unused_1: Quest,
    pub unused_2: Quest,
    pub unused_3: Quest,
}

impl fmt::Display for Act4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Prologue: {0}\nQuest 1: {1}\nQuest 2: {2}\nQuest 3: {3}\nCompletion: {4}\nUnused Quest 1: {5}\nUnused Quest 2: {6}\nUnused Quest 3: {7}\n",
            self.prologue, self.q1, self.q2, self.q3, self.completion, self.unused_1, self.unused_2, self.unused_3
        )
    }
}

impl Act4 {
    fn to_bytes(&self) -> [u8; 16] {
        let mut quest_values: [u16; 8] = [0; 8];
        quest_values[0] = self.prologue.value();
        quest_values[1] = self.q1.value();
        quest_values[2] = self.q2.value();
        quest_values[3] = self.q3.value();
        quest_values[4] = self.completion.value();
        quest_values[5] = self.unused_1.value();
        quest_values[6] = self.unused_2.value();
        quest_values[7] = self.unused_3.value();
        let mut quest_bytes: [u8; 16] = [0; 16];
        for (index, val) in quest_values.iter().enumerate() {
            let start: usize = index * 2;
            quest_bytes[start..start + 2].copy_from_slice(&u16::to_le_bytes(*val));
        }
        quest_bytes
    }
}

impl From<&[u8]> for Act4 {
    fn from(value: &[u8]) -> Self {
        let mut complete_act = Act4::default();
        complete_act.prologue = Quest::from(u16_from(&value[0..2], "A4 Prologue"));
        complete_act.q1 = Quest::from(u16_from(&value[2..4], "A4 Q1"));
        complete_act.q2 = Quest::from(u16_from(&value[4..6], "A4 Q2"));
        complete_act.q3 = Quest::from(u16_from(&value[6..8], "A4 Q3"));
        complete_act.completion = Quest::from(u16_from(&value[8..10], "A4 Completion"));
        complete_act.unused_1 = Quest::from(u16_from(&value[10..12], "A4 Unused1"));
        complete_act.unused_2 = Quest::from(u16_from(&value[12..14], "A4 Unused2"));
        complete_act.unused_3 = Quest::from(u16_from(&value[14..16], "A4 Unused3"));
        complete_act
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Act5 {
    pub prologue: Quest,
    pub unused_1: Quest,
    pub unused_2: Quest,
    pub q1: Quest,
    pub q2: Quest,
    pub q3: Quest,
    pub q4: Quest,
    pub q5: Quest,
    pub q6: Quest,
    pub completion: Quest,
}

impl fmt::Display for Act5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Prologue: {0}\nUnused Quest 1: {1}\nUnused Quest 2: {2}\nQuest 1: {3}\nQuest 2: {4}\nQuest 3: {5}\n\nQuest 4: {6}\nQuest 5: {7}\nQuest 6: {8}\nCompletion: {9}",
            self.prologue, self.unused_1, self.unused_2, self.q1, self.q2, self.q3, self.q4, self.q5, self.q6, self.completion
        )
    }
}

impl Act5 {
    fn to_bytes(&self) -> [u8; 20] {
        let mut quest_values: [u16; 10] = [0; 10];
        quest_values[0] = self.prologue.value();
        quest_values[1] = self.unused_1.value();
        quest_values[2] = self.unused_2.value();
        quest_values[3] = self.q1.value();
        quest_values[4] = self.q2.value();
        quest_values[5] = self.q3.value();
        quest_values[6] = self.q4.value();
        quest_values[7] = self.q5.value();
        quest_values[8] = self.q6.value();
        quest_values[9] = self.completion.value();
        let mut quest_bytes: [u8; 20] = [0; 20];
        for (index, val) in quest_values.iter().enumerate() {
            let start: usize = index * 2;
            quest_bytes[start..start + 2].copy_from_slice(&u16::to_le_bytes(*val));
        }
        quest_bytes
    }
}

impl From<&[u8]> for Act5 {
    fn from(value: &[u8]) -> Self {
        let mut complete_act = Act5::default();
        complete_act.prologue = Quest::from(u16_from(&value[0..2], "A5 Prologue"));
        complete_act.unused_1 = Quest::from(u16_from(&value[2..4], "A5 Unused1"));
        complete_act.unused_2 = Quest::from(u16_from(&value[4..6], "A1 Unused2"));
        complete_act.q1 = Quest::from(u16_from(&value[6..8], "A5 Q1"));
        complete_act.q2 = Quest::from(u16_from(&value[8..10], "A5 Q2"));
        complete_act.q3 = Quest::from(u16_from(&value[10..12], "A5 Q3"));
        complete_act.q4 = Quest::from(u16_from(&value[12..14], "A5 Q4"));
        complete_act.q5 = Quest::from(u16_from(&value[14..16], "A5 Q5"));
        complete_act.q6 = Quest::from(u16_from(&value[16..18], "A5 Q6"));
        complete_act.completion = Quest::from(u16_from(&value[18..20], "A5 Completion"));
        complete_act
    }
}

impl fmt::Display for DifficultyQuests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Act I:\n{0}\nAct II:\n{1}\nAct III:\n{2}\nAct IV:\n{3}\nAct V:\n{4}",
            self.act1, self.act2, self.act3, self.act4, self.act5
        )
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct DifficultyQuests {
    pub act1: Act1,
    pub act2: Act2,
    pub act3: Act3,
    pub act4: Act4,
    pub act5: Act5,
}

// section length = 298bytes -> header 10 bytes -> 288bytes for all quests -> 96 * 3 difficulties
impl DifficultyQuests {
    pub fn to_bytes(&self) -> [u8; 96] {
        let mut byte_vector: [u8; 96] = [0; 96];
        byte_vector[Section::Act1.range()].copy_from_slice(&self.act1.to_bytes());
        byte_vector[Section::Act2.range()].copy_from_slice(&self.act2.to_bytes());
        byte_vector[Section::Act3.range()].copy_from_slice(&self.act3.to_bytes());
        byte_vector[Section::Act4.range()].copy_from_slice(&self.act4.to_bytes());
        byte_vector[Section::Act5.range()].copy_from_slice(&self.act5.to_bytes());
        byte_vector
    }
}

impl From<&[u8]> for DifficultyQuests {
    fn from(value: &[u8]) -> Self {
        let mut diff_quests: DifficultyQuests = DifficultyQuests::default();
        diff_quests.act1 = Act1::from(&value[Section::Act1.range()]);
        diff_quests.act2 = Act2::from(&value[Section::Act2.range()]);
        diff_quests.act3 = Act3::from(&value[Section::Act3.range()]);
        diff_quests.act4 = Act4::from(&value[Section::Act4.range()]);
        diff_quests.act5 = Act5::from(&value[Section::Act5.range()]);
        diff_quests
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Quests {
    pub normal: DifficultyQuests,
    pub nightmare: DifficultyQuests,
    pub hell: DifficultyQuests,
}

impl fmt::Display for Quests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Normal:\n{0}\nNightmare:\n{1}\nHell:\n{2}",
            self.normal, self.nightmare, self.hell
        )
    }
}

impl Quests {
    pub fn to_bytes(&self) -> [u8; 298] {
        let mut byte_vector: [u8; 298] = [0; 298];
        byte_vector[Section::Header.range()].copy_from_slice(&SECTION_HEADER);
        byte_vector[Section::Normal.range()].copy_from_slice(&self.normal.to_bytes());
        byte_vector[Section::Nightmare.range()].copy_from_slice(&self.nightmare.to_bytes());
        byte_vector[Section::Hell.range()].copy_from_slice(&self.hell.to_bytes());
        byte_vector
    }

    pub fn parse(bytes: &[u8]) -> Self {
        let mut quests: Quests = Quests::default();

        if bytes.len() < SECTION_LENGTH {
            error!(
                "Quests section should be {0} bytes but found {1} instead.",
                SECTION_LENGTH,
                bytes.len()
            );
            return quests;
        }

        if bytes[Section::Header.range()] != SECTION_HEADER {
            warn!(
                "Found wrong header for quests, expected {0:X?} but found {1:X?}",
                SECTION_HEADER,
                &bytes[Section::Header.range()]
            );
        }

        quests.normal = DifficultyQuests::from(&bytes[Section::Normal.range()]);
        quests.nightmare = DifficultyQuests::from(&bytes[Section::Nightmare.range()]);
        quests.hell = DifficultyQuests::from(&bytes[Section::Hell.range()]);

        quests
    }
}

#[cfg(test)]
mod tests {}
