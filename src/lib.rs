pub mod Header;
pub mod Quests;
pub mod Waypoints;
pub mod Npcs;
pub mod Statistics;

use Header::Character as Character;
use Header::Mercenary as Mercenary;


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
