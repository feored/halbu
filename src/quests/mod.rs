use std::collections::HashSet;
use std::fmt;
use std::ops::Range;
use std::str;

use bit::BitIndex;
use serde::{Deserialize, Serialize};

use crate::ParseError;

use crate::utils::u16_from;

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
        write!(f, "Quest State: {0:?}:\t{1:#018b}\t{1:X?}", self.state, self.value())
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
        complete_act.prologue = Quest::from(u16_from(&value[0..2]));
        complete_act.q1 = Quest::from(u16_from(&value[2..4]));
        complete_act.q2 = Quest::from(u16_from(&value[4..6]));
        complete_act.q3 = Quest::from(u16_from(&value[6..8]));
        complete_act.q4 = Quest::from(u16_from(&value[8..10]));
        complete_act.q5 = Quest::from(u16_from(&value[10..12]));
        complete_act.q6 = Quest::from(u16_from(&value[12..14]));
        complete_act.completion = Quest::from(u16_from(&value[14..16]));
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
        complete_act.prologue = Quest::from(u16_from(&value[0..2]));
        complete_act.q1 = Quest::from(u16_from(&value[2..4]));
        complete_act.q2 = Quest::from(u16_from(&value[4..6]));
        complete_act.q3 = Quest::from(u16_from(&value[6..8]));
        complete_act.q4 = Quest::from(u16_from(&value[8..10]));
        complete_act.q5 = Quest::from(u16_from(&value[10..12]));
        complete_act.q6 = Quest::from(u16_from(&value[12..14]));
        complete_act.completion = Quest::from(u16_from(&value[14..16]));
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
        complete_act.prologue = Quest::from(u16_from(&value[0..2]));
        complete_act.q1 = Quest::from(u16_from(&value[2..4]));
        complete_act.q2 = Quest::from(u16_from(&value[4..6]));
        complete_act.q3 = Quest::from(u16_from(&value[6..8]));
        complete_act.q4 = Quest::from(u16_from(&value[8..10]));
        complete_act.q5 = Quest::from(u16_from(&value[10..12]));
        complete_act.q6 = Quest::from(u16_from(&value[12..14]));
        complete_act.completion = Quest::from(u16_from(&value[14..16]));
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
        complete_act.prologue = Quest::from(u16_from(&value[0..2]));
        complete_act.q1 = Quest::from(u16_from(&value[2..4]));
        complete_act.q2 = Quest::from(u16_from(&value[4..6]));
        complete_act.q3 = Quest::from(u16_from(&value[6..8]));
        complete_act.completion = Quest::from(u16_from(&value[8..10]));
        complete_act.unused_1 = Quest::from(u16_from(&value[10..12]));
        complete_act.unused_2 = Quest::from(u16_from(&value[12..14]));
        complete_act.unused_3 = Quest::from(u16_from(&value[14..16]));
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
        complete_act.prologue = Quest::from(u16_from(&value[0..2]));
        complete_act.unused_1 = Quest::from(u16_from(&value[2..4]));
        complete_act.unused_2 = Quest::from(u16_from(&value[4..6]));
        complete_act.q1 = Quest::from(u16_from(&value[6..8]));
        complete_act.q2 = Quest::from(u16_from(&value[8..10]));
        complete_act.q3 = Quest::from(u16_from(&value[10..12]));
        complete_act.q4 = Quest::from(u16_from(&value[12..14]));
        complete_act.q5 = Quest::from(u16_from(&value[14..16]));
        complete_act.q6 = Quest::from(u16_from(&value[16..18]));
        complete_act.completion = Quest::from(u16_from(&value[18..20]));
        complete_act
    }
}

impl fmt::Display for DifficultyQuests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Act I:\n {0:?}\nAct II:\n{1:?}\nAct III:\n{2:?}\nAct IV:\n{3:?}\nAct V:\n{4:?}",
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
        write!(f, "Normal: {0}\nNightmare: {1}\nHell: {2}", self.normal, self.nightmare, self.hell)
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

    pub fn parse(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() < SECTION_LENGTH {
            return Err(ParseError {
                message: format!(
                    "Quests section should be {0} bytes but found {1} instead.",
                    SECTION_LENGTH,
                    bytes.len()
                ),
            });
        }
        if bytes[Section::Header.range()] != SECTION_HEADER {
            return Err(ParseError {
                message: format!(
                    "Found wrong header for quests, expected {0:X?} but found {1:X?}",
                    SECTION_HEADER,
                    &bytes[Section::Header.range()]
                ),
            });
        }

        let mut quests: Quests = Quests::default();

        quests.normal = DifficultyQuests::from(&bytes[Section::Normal.range()]);
        quests.nightmare = DifficultyQuests::from(&bytes[Section::Nightmare.range()]);
        quests.hell = DifficultyQuests::from(&bytes[Section::Hell.range()]);

        Ok(quests)
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_generate_and_parse() {
    //     let bytes: [u8; 298] = [
    //         0x57, 0x6F, 0x6F, 0x21, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x01, 0x01, 0x00, 0x01, 0x10,
    //         0x1D, 0x10, 0x4E, 0x80, 0x1D, 0x10, 0x00, 0x00, 0x1D, 0x00, 0x01, 0x00, 0x01, 0x00,
    //         0x1D, 0x10, 0x79, 0x1C, 0x05, 0x10, 0x81, 0x11, 0x05, 0x10, 0x65, 0x1F, 0x01, 0x00,
    //         0x01, 0x00, 0x01, 0x10, 0x7D, 0x10, 0xF5, 0x13, 0x01, 0x10, 0x0D, 0x10, 0x61, 0x10,
    //         0x01, 0x00, 0x01, 0x00, 0x01, 0x10, 0x01, 0x13, 0x01, 0x10, 0x01, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x80, 0x08, 0x00,
    //         0x8D, 0x17, 0x0C, 0x00, 0x19, 0x13, 0xCD, 0x15, 0x01, 0x80, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x10, 0x0C, 0x00,
    //         0x4A, 0x80, 0x19, 0x10, 0x00, 0x00, 0x19, 0x10, 0x01, 0x00, 0x01, 0x00, 0x11, 0x10,
    //         0x79, 0x18, 0x05, 0x10, 0x81, 0x11, 0x05, 0x10, 0x25, 0x18, 0x01, 0x00, 0x01, 0x00,
    //         0x01, 0x10, 0x7D, 0x10, 0xF5, 0x13, 0x01, 0x10, 0x0D, 0x10, 0x61, 0x10, 0x01, 0x00,
    //         0x01, 0x00, 0x01, 0x10, 0x01, 0x13, 0x01, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x80, 0x08, 0x00, 0x89, 0x17,
    //         0x0C, 0x00, 0x19, 0x13, 0xCD, 0x15, 0x02, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x10, 0x1D, 0x10, 0x0C, 0x00,
    //         0x19, 0x14, 0x15, 0x10, 0x19, 0x10, 0x01, 0x00, 0x01, 0x00, 0x11, 0x90, 0x79, 0x1C,
    //         0x05, 0x90, 0x81, 0x11, 0x05, 0x10, 0x25, 0x1A, 0x01, 0x00, 0x01, 0x00, 0x05, 0x10,
    //         0x7D, 0x10, 0x00, 0x00, 0x01, 0x10, 0x09, 0x10, 0x71, 0x10, 0x01, 0x00, 0x01, 0x00,
    //         0x01, 0x10, 0x01, 0x13, 0x01, 0x80, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x90, 0x01, 0x10, 0x89, 0x17, 0x1E, 0x80,
    //         0x19, 0x13, 0xDD, 0x17, 0x01, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00,
    //     ];
    //     let parsed_result = match parse(&bytes) {
    //         Ok(res) => res,
    //         Err(e) => {
    //             println!("#### FAILED TO PARSE QUESTS");
    //             panic!("{e:?}")
    //         }
    //     };
    //     //println!("{0}", parsed_result);
    //     assert_eq!(parsed_result.hell.flags.completed_difficulty, true);
    //     assert_eq!(parsed_result.hell.quests[26].name, "Eve of Destruction");

    //     let mut new_bytes : [u8;298] = [0x00; 298];
    //     new_bytes.copy_from_slice(&generate(&parsed_result));

    //     assert_eq!(bytes, new_bytes);
    // }
}
