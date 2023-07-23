#![warn(
    anonymous_parameters,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

pub mod attributes;
pub mod header;
pub mod npcs;
pub mod quests;
pub mod skills;
pub mod waypoints;

use header::character;
use header::mercenary;

pub struct Save {
    version: header::Version,
    character: character::Character,
    mercenary: mercenary::Mercenary,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Difficulty {
    Normal,
    Nightmare,
    Hell,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Act {
    Act1,
    Act2,
    Act3,
    Act4,
    Act5,
}
