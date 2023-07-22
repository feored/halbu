#![ warn
(
   anonymous_parameters          ,
   nonstandard_style             ,
   rust_2018_idioms              ,
   single_use_lifetimes          ,
   trivial_casts                 ,
   trivial_numeric_casts         ,
   unreachable_pub               ,
   unused_extern_crates          ,
   unused_qualifications         ,
   variant_size_differences      ,
)]

pub mod header;
pub mod quests;
pub mod waypoints;
pub mod npcs;
pub mod attributes;
pub mod skills;

use header::character as character;
use header::mercenary as mercenary;


pub struct Save {
    version: header::Version,
    character: character::Character,
    mercenary: mercenary::Mercenary,
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
