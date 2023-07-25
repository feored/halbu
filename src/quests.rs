use std::fmt;
use std::ops::Range;
use std::str;

use bit::BitIndex;

use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::GameLogicError;
use crate::ParseError;

use crate::utils::u16_from;
use crate::utils::u32_from;
use crate::utils::u8_from;
use crate::utils::FileSection;

const SECTION_HEADER: [u8; 10] = [0x57, 0x6F, 0x6F, 0x21, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x01];
const ACT_1_QUESTS: [&'static str; 6] = [
    "Den of Evil",
    "Sisters' Burial Ground",
    "Search For Cain",
    "The Forgotten Tower",
    "Tools of the Trade",
    "Sisters to the Slaughter",
];
const ACT_2_QUESTS: [&'static str; 6] = [
    "Radament's Lair",
    "The Horadric Staff",
    "Tainted Sun",
    "Arcane Sanctuary",
    "The Summoner",
    "The Seven Tombs",
];
const ACT_3_QUESTS: [&'static str; 6] = [
    "The Golden Bird",
    "Blade of the Old Religion",
    "Khalim's Will",
    "Lam Esen's Tome",
    "The Blackened Temple",
    "The Guardian",
];
const ACT_4_QUESTS: [&'static str; 3] = ["Fallen Angel", "Hell's Forge", "Terror's End"];
const ACT_5_QUESTS: [&'static str; 6] = [
    "Siege on Harrogath",
    "Rescue on Mount Arreat",
    "Prison of Ice",
    "Betrayal of Harrogath",
    "Rite of Passage",
    "Eve of Destruction",
];

#[derive(PartialEq, Eq, Debug)]
enum Section {
    Act1Introduction,
    Act1Quests,
    Act2Travel,
    Act2Introduction,
    Act2Quests,
    Act3Travel,
    Act3Introduction,
    Act3Quests,
    Act4Travel,
    Act4Introduction,
    Act4Quests,
    Act5Travel,
    BaseGameComplete,
    Act5Quests,
    ResetStats,
    DifficultyComplete,
}

impl From<Section> for FileSection {
    fn from(section: Section) -> FileSection {
        match section {
            Section::Act1Introduction => FileSection {
                offset: 0,
                bytes: 2,
            },
            Section::Act1Quests => FileSection {
                offset: 2,
                bytes: 12,
            },
            Section::Act2Travel => FileSection {
                offset: 14,
                bytes: 2,
            },
            Section::Act2Introduction => FileSection {
                offset: 16,
                bytes: 2,
            },
            Section::Act2Quests => FileSection {
                offset: 18,
                bytes: 12,
            },
            Section::Act3Travel => FileSection {
                offset: 30,
                bytes: 2,
            },
            Section::Act3Introduction => FileSection {
                offset: 32,
                bytes: 2,
            },
            Section::Act3Quests => FileSection {
                offset: 34,
                bytes: 12,
            },
            Section::Act4Travel => FileSection {
                offset: 46,
                bytes: 2,
            },
            Section::Act4Introduction => FileSection {
                offset: 48,
                bytes: 2,
            },
            Section::Act4Quests => FileSection {
                offset: 50,
                bytes: 12,
            },
            Section::Act5Travel => FileSection {
                offset: 62,
                bytes: 2,
            },
            Section::BaseGameComplete => FileSection {
                offset: 64,
                bytes: 2,
            },
            Section::Act5Quests => FileSection {
                offset: 70,
                bytes: 12,
            },
            Section::ResetStats => FileSection {
                offset: 82,
                bytes: 1,
            },
            Section::DifficultyComplete => FileSection {
                offset: 83,
                bytes: 1,
            },
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Stage {
    Completed,
    RequirementsMet,
    Started,
    Closed,
    CompletedInGame,
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub struct Quest {
    id: usize,
    name: &'static str,
    flags: u16,
    act: Act,
    difficulty: Difficulty,
}

impl fmt::Display for Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Quest {0}:\t{1}\t({2} {3}):\t{4:#018b}\t{4:?}\t {4:X?}",
            self.id, self.name, self.act, self.difficulty, self.flags
        )
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Quests {
    pub normal: DifficultyQuests,
    pub nightmare: DifficultyQuests,
    pub hell: DifficultyQuests,
}

impl fmt::Display for Quests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Normal: {0}\nNightmare: {1}\nHell: {2}",
            self.normal, self.nightmare, self.hell
        )
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct DifficultyQuests {
    pub flags: QuestFlags,
    pub quests: QuestSet,
}

pub type QuestSet = [Quest; 27];

impl fmt::Display for DifficultyQuests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut final_string = format!("Flags:\n {0}\nQuests:\n", self.flags);
        for i in 0..self.quests.len() {
            final_string.push_str(&format!("{0}\n", self.quests[i]));
        }
        write!(f, "{0}", final_string)
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct QuestFlags {
    pub act_1_introduction: bool,
    pub act_2_travel: bool,
    pub act_2_introduction: bool,
    pub act_3_travel: bool,
    pub act_3_introduction: bool,
    pub act_4_travel: bool,
    pub act_4_introduction: bool,
    pub act_5_travel: bool,
    pub completed_base_game: bool,
    pub reset_stats: bool,
    pub completed_difficulty: bool,
}

impl fmt::Display for QuestFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Completed difficulty: {0:?}",
            self.completed_difficulty
        )
    }
}

impl Quest {
    fn set_stage(&mut self, stage: Stage, value: bool) {
        self.flags.set_bit(usize::from(stage), value);
    }
    fn finish(&mut self) {
        self.set_stage(Stage::Completed, true);
        self.set_stage(Stage::Closed, true);
    }
    fn clear(&mut self){
        self.flags = 0;
    }
}

impl From<Stage> for usize {
    fn from(stage: Stage) -> usize {
        match stage {
            Stage::Completed => 0,
            Stage::RequirementsMet => 1,
            Stage::Started => 2,
            Stage::Closed => 12,
            Stage::CompletedInGame => 13,
        }
    }
}

fn write_flags(bytes: &mut Vec<u8>, flags: &QuestFlags) {
    bytes[Range::<usize>::from(FileSection::from(Section::Act1Introduction))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_1_introduction as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act2Travel))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_2_travel as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act2Introduction))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_2_introduction as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act3Travel))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_3_travel as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act3Introduction))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_3_introduction as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act4Travel))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_4_travel as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act4Introduction))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_4_introduction as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::Act5Travel))]
        .copy_from_slice(&u16::to_le_bytes(flags.act_5_travel as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::BaseGameComplete))]
        .copy_from_slice(&u16::to_le_bytes(flags.completed_base_game as u16));
    bytes[Range::<usize>::from(FileSection::from(Section::ResetStats))]
        .copy_from_slice(&u8::to_le_bytes(flags.reset_stats as u8));
    bytes[Range::<usize>::from(FileSection::from(Section::DifficultyComplete))]
        .copy_from_slice(match flags.completed_difficulty { true => &[0x80], false => &[0x00]});
}

fn parse_flags(bytes: &[u8; 96]) -> Result<QuestFlags, ParseError> {
    let mut flags: QuestFlags = QuestFlags::default();
    flags.act_1_introduction =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act1Introduction))]);
    // Any non-zero is considered true
    flags.act_2_travel =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act2Travel))]);
    flags.act_2_introduction =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act2Introduction))]);
    flags.act_3_travel =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act3Travel))]);
    flags.act_3_introduction =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act3Introduction))]);
    flags.act_4_travel =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act4Travel))]);
    flags.act_4_introduction =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act4Introduction))]);
    flags.act_5_travel =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act5Travel))]);
    flags.completed_base_game =
        0 != u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::BaseGameComplete))]);
    flags.reset_stats =
        0 != u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::ResetStats))]);
    flags.completed_difficulty =
        0 != u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::DifficultyComplete))]);
    Ok(flags)
}

fn write_quests(byte_vector: &mut Vec<u8>, quests: &QuestSet) {
    for act in 0..4 {
        let mut act_quests: [u8; 12] = [0x00; 12];
        let quests_number = match act {
            0..=2 | 4 => 6,
            3 => 3,
            _ => unreachable!()
        };
        for i in 0..quests_number {
            let quest_index = i + match act {
                0 => 0,
                1 => 6,
                2 => 12,
                3 => 18,
                4 => 21,
                _ => unreachable!()
            };
            let quest_value = u16::to_le_bytes(quests[quest_index].flags);
            // println!{"@@@@@ Quest: {0}", quests[quest_index]};
            // println!("%%%%% Flags: {0:?} Quest value: {quest_value:X?} ", quests[quest_index].flags);
            act_quests[i * 2] = quest_value[0];
            act_quests[(i * 2) + 1] = quest_value[1];
            // println!("Putting them at {0}..{1}", (i * 2), (i * 2) + 1);
            // println!{"Current quests: {0:X?}", act_quests};
        }
        let section = match act {
            0 => Section::Act1Quests,
            1 => Section::Act2Quests,
            2 => Section::Act3Quests,
            3 => Section::Act4Quests,
            4 => Section::Act5Quests,
            _ => unreachable!()
        };
        // println!{"############# Writing quests: {0:X?}", act_quests};
        byte_vector[Range::<usize>::from(FileSection::from(section))].copy_from_slice(&act_quests);
    }
}

fn parse_quests(bytes: &[u8; 96], difficulty: Difficulty) -> Result<QuestSet, ParseError> {
    let mut quests: QuestSet = QuestSet::default();
    let act_1_quests = &bytes[Range::<usize>::from(FileSection::from(Section::Act1Quests))];
    // println!("{0:X?}", act_1_quests);
    for i in 0..6 {
        // println!("{0:X?}", &act_1_quests[(i*2)..((i*2)+ 2)]);
        quests[i] = Quest {
            id: i,
            name: ACT_1_QUESTS[i],
            act: Act::Act1,
            difficulty: difficulty,
            flags: u16_from(&act_1_quests[(i*2)..((i*2) + 2)])
        };
    }

    let act_2_quests = &bytes[Range::<usize>::from(FileSection::from(Section::Act2Quests))];
    for i in 0..6 {
        quests[i + 6] = Quest {
            id: i + 6,
            name: ACT_2_QUESTS[i],
            act: Act::Act2,
            difficulty: difficulty,
            flags: u16_from(&act_2_quests[(i*2)..((i*2) + 2)])
        };
    }

    let act_3_quests = &bytes[Range::<usize>::from(FileSection::from(Section::Act3Quests))];
    for i in 0..6 {
        quests[i + 12] = Quest {
            id: i + 12,
            name: ACT_3_QUESTS[i],
            act: Act::Act3,
            difficulty: difficulty,
            flags: u16_from(&act_3_quests[(i*2)..((i*2) + 2)])
        };
    }

    let act_4_quests = &bytes[Range::<usize>::from(FileSection::from(Section::Act4Quests))];
    for i in 0..3 {
        quests[i + 18] = Quest {
            id: i + 18,
            name: ACT_4_QUESTS[i],
            act: Act::Act4,
            difficulty: difficulty,
            flags: u16_from(&act_4_quests[(i*2)..((i*2) + 2)])
        };
    }

    let act_5_quests = &bytes[Range::<usize>::from(FileSection::from(Section::Act5Quests))];
    for i in 0..6 {
        quests[i + 21] = Quest {
            id: i + 21,
            name: ACT_5_QUESTS[i],
            act: Act::Act5,
            difficulty: difficulty,
            flags: u16_from(&act_5_quests[(i*2)..((i*2) + 2)])
        };
    }
    
    Ok(quests)
}

pub fn parse(bytes: &[u8; 298]) -> Result<Quests, ParseError> {
    if bytes[0..10] != SECTION_HEADER {
        return Err(ParseError {
            message: format! {"Found wrong header for quests: {:02X?}", &bytes[0..10]},
        });
    }
    let mut quests = Quests::default();

    quests.normal.quests = parse_quests(&bytes[10..106].try_into().unwrap(), Difficulty::Normal)?;
    quests.nightmare.quests =
        parse_quests(&bytes[106..202].try_into().unwrap(), Difficulty::Nightmare)?;
    quests.hell.quests = parse_quests(&bytes[202..298].try_into().unwrap(), Difficulty::Hell)?;

    quests.normal.flags = parse_flags(&bytes[10..106].try_into().unwrap())?;
    quests.nightmare.flags = parse_flags(&bytes[106..202].try_into().unwrap())?;
    quests.hell.flags = parse_flags(&bytes[202..298].try_into().unwrap())?;

    Ok(quests)
}

pub fn generate(all_quests: &Quests) -> Vec<u8> {
    let mut byte_vector = SECTION_HEADER.to_vec();
    byte_vector.resize(298, 0x00);

    let mut normal = Vec::<u8>::new();
    normal.resize(96, 0x00);
    write_quests(&mut normal, &all_quests.normal.quests);
    write_flags(&mut normal, &all_quests.normal.flags);
    byte_vector[10..106].copy_from_slice(&normal);

    let mut nightmare = Vec::<u8>::new();
    nightmare.resize(96, 0x00);
    write_quests(&mut nightmare, &all_quests.nightmare.quests);
    write_flags(&mut nightmare, &all_quests.nightmare.flags);
    byte_vector[106..202].copy_from_slice(&nightmare);

    let mut hell = Vec::<u8>::new();
    hell.resize(96, 0x00);
    write_quests(&mut hell, &all_quests.hell.quests);
    write_flags(&mut hell, &all_quests.hell.flags);
    byte_vector[202..298].copy_from_slice(&hell);    

    byte_vector
}

#[cfg(test)]
mod tests {
    use super::*;

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
    //     println!("{0}", parsed_result);
    //     assert_eq!(parsed_result.hell.flags.completed_difficulty, true);
    //     assert_eq!(parsed_result.hell.quests[26].name, "Eve of Destruction");

    //     let mut new_bytes : [u8;298] = [0x00; 298];
    //     new_bytes.copy_from_slice(&generate(&parsed_result));

    //     assert_eq!(bytes, new_bytes);
    // }
}
