pub mod Header;
pub mod Quests;
pub mod Waypoints;
pub mod Npcs;
pub mod Statistics;

use Header::Character as Character;
use Header::Mercenary as Mercenary;

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

const VERSION_100: u32 = 71;
const VERSION_107: u32 = 87;
const VERSION_108: u32 = 89;
const VERSION_109: u32 = 92;
const VERSION_110: u32 = 96;

pub struct Save {
    version: Header::Version,
    character: Character::Character,
    mercenary: Mercenary::Mercenary,
}

#[derive(PartialEq, Eq)]
pub enum Difficulty {
    Normal,
    Nightmare,
    Hell,
}
#[derive(PartialEq, Eq)]
pub enum Act {
    Act1,
    Act2,
    Act3,
    Act4,
    Act5,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
