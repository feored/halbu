use std::ops::Range;
use std::str;

use bit::BitIndex;

use crate::Act;
use crate::Class;
use crate::Difficulty;
use crate::GameLogicError;
use crate::ParseError;

use crate::utils::FileSection;
use crate::utils::u32_from;
use crate::utils::u16_from;
use crate::utils::u8_from;

const SECTION_HEADER: [u8; 10] = [0x57, 0x6F, 0x6F, 0x21, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x01];


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
    DifficultyComplete
}

impl From<Section> for FileSection {
    fn from(section: Section) -> FileSection {
        match section {
            Section::Act1Introduction   =>  FileSection {offset: 0, bytes: 2},
            Section::Act1Quests         =>  FileSection {offset: 2, bytes: 12},
            Section::Act2Travel         =>  FileSection {offset: 14, bytes: 2},
            Section::Act2Introduction   =>  FileSection {offset: 16, bytes: 2},
            Section::Act2Quests         =>  FileSection {offset: 18, bytes: 12},
            Section::Act3Travel         =>  FileSection {offset: 30, bytes: 2},
            Section::Act3Introduction   =>  FileSection {offset: 32, bytes: 2},
            Section::Act3Quests         =>  FileSection {offset: 34, bytes: 12},
            Section::Act4Travel         =>  FileSection {offset: 46, bytes: 2},
            Section::Act4Introduction   =>  FileSection {offset: 48, bytes: 2},
            Section::Act4Quests         =>  FileSection {offset: 50, bytes: 12},
            Section::Act5Travel         =>  FileSection {offset: 62, bytes: 2},
            Section::BaseGameComplete   =>  FileSection {offset: 64, bytes: 2},
            Section::Act5Quests         =>  FileSection {offset: 70, bytes: 12},
            Section::ResetStats         =>  FileSection {offset: 82, bytes: 1},
            Section::DifficultyComplete =>  FileSection {offset: 83, bytes: 1}
        }
    }
} 

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Stage {
    Completed,
    RequirementsMet,
    Started,
    Closed,
    CompletedInGame
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub struct Quest {
    id: usize,
    flags: u16,
    act: Act,
    difficulty: Difficulty,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Quests{
    pub normal: QuestSet,
    pub nightmare: QuestSet,
    pub hell: QuestSet
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct QuestSet{
    pub flags: QuestFlags,
    pub quests: [Quest; 27]
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct QuestFlags {
    pub act_1_introduction: u16,
    pub act_2_travel: u16,
    pub act_2_introduction: u16,
    pub act_3_travel: u16,
    pub act_3_introduction: u16,
    pub act_4_travel: u16,
    pub act_4_introduction: u16,
    pub act_5_travel: u16,
    pub completed_base_game: u16,
    pub reset_stats: u8,
    pub completed_difficulty : u8
}

impl Quest {
    fn set_stage(&mut self, stage: Stage, value: bool){
        self.flags.set_bit(usize::from(stage), value);
    }
    fn finish(&mut self) {
        self.set_stage(Stage::Completed, true);
        self.set_stage(Stage::Closed, true);
    }
}

impl From<Stage> for usize {
    fn from(stage:Stage) -> usize {
        match stage{
            Stage::Completed => 0,
            Stage::RequirementsMet => 1,
            Stage::Started => 2,
            Stage::Closed => 12,
            Stage::CompletedInGame => 13
        }
    }
}

fn parse_flags(bytes: &[u8;96]) -> Result<QuestFlags, ParseError>{
    let mut flags : QuestFlags = QuestFlags::default();
    flags.act_1_introduction =   u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act1Introduction))]);
    flags.act_2_travel =         u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act2Travel))]);
    flags.act_2_introduction =   u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act2Introduction))]);
    flags.act_3_travel =         u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act3Travel))]);
    flags.act_3_introduction =   u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act3Introduction))]);
    flags.act_4_travel =         u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act4Travel))]);
    flags.act_4_introduction =   u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act4Introduction))]);
    flags.act_5_travel =         u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::Act5Travel))]);
    flags.completed_base_game =  u16_from(&bytes[Range::<usize>::from(FileSection::from(Section::BaseGameComplete))]);
    flags.reset_stats =          u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::ResetStats))]);
    flags.completed_difficulty = u8_from(&bytes[Range::<usize>::from(FileSection::from(Section::DifficultyComplete))]);
    Ok(flags)
}

fn parse_quests(bytes: &[u8;96]) -> Result<[Quest; 27], ParseError>{
    let mut quests : [Quest; 27] = [Quest::default();27];

    

    Ok(quests)
}

pub fn parse(bytes: &[u8;298]) -> Result<Quests, ParseError>{
    if bytes[0..10] != SECTION_HEADER{
        return Err(ParseError{message: format!{"Found wrong header for quests: {:02X?}", &bytes[0..10]}})
    }
    let mut quests = Quests::default();

    Ok(quests)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse(){
        let bytes : [u8;298] = [0x00; 298];
        let result = match parse(&bytes) {
            Ok(res) => res,
            Err(e) => {
                panic!("Failed test_parse in quests: {0:?}", e);
            }
        };

    }
}
