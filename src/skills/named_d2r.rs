//! Default D2R skill-name mapping.
//!
//! For modded trees, use raw index APIs.

use std::fmt;

use crate::Class;

const AMAZON_SKILLS: [&str; 30] = [
    "Magic Arrow",
    "Fire Arrow",
    "Inner Sight",
    "Critical Strike",
    "Jab",
    "Cold Arrow",
    "Multiple Shot",
    "Dodge",
    "Power Strike",
    "Poison Javelin",
    "Exploding Arrow",
    "Slow Missiles",
    "Avoid",
    "Impale",
    "Lightning Bolt",
    "Ice Arrow",
    "Guided Arrow",
    "Penetrate",
    "Charged Strike",
    "Plague Javelin",
    "Strafe",
    "Immolation Arrow",
    "Decoy",
    "Evade",
    "Fend",
    "Freezing Arrow",
    "Valkyrie",
    "Pierce",
    "Lightning Strike",
    "Lightning Fury",
];

const SORCERESS_SKILLS: [&str; 30] = [
    "Fire Bolt",
    "Warmth",
    "Charged Bolt",
    "Ice Bolt",
    "Frozen Armor",
    "Inferno",
    "Static Field",
    "Telekinesis",
    "Frost Nova",
    "Ice Blast",
    "Blaze",
    "Fire Ball",
    "Nova",
    "Lightning",
    "Shiver Armor",
    "Fire Wall",
    "Enchant",
    "Chain Lightning",
    "Teleport",
    "Glacial Spike",
    "Meteor",
    "Thunder Storm",
    "Energy Shield",
    "Blizzard",
    "Chilling Armor",
    "Fire Mastery",
    "Hydra",
    "Lightning Mastery",
    "Frozen Orb",
    "Cold Mastery",
];

const NECROMANCER_SKILLS: [&str; 30] = [
    "Amplify Damage",
    "Teeth",
    "Bone Armor",
    "Skeleton Mastery",
    "Raise Skeleton",
    "Dim Vision",
    "Weaken",
    "Poison Dagger",
    "Corpse Explosion",
    "Clay Golem",
    "Iron Maiden",
    "Terror",
    "Bone Wall",
    "Golem Mastery",
    "Raise Skeletal Mage",
    "Confuse",
    "Life Tap",
    "Poison Explosion",
    "Bone Spear",
    "Blood Golem",
    "Attract",
    "Decrepify",
    "Bone Prison",
    "Summon Resist",
    "Iron Golem",
    "Lower Resist",
    "Poison Nova",
    "Bone Spirit",
    "Fire Golem",
    "Revive",
];

const PALADIN_SKILLS: [&str; 30] = [
    "Sacrifice",
    "Smite",
    "Might",
    "Prayer",
    "Resist Fire",
    "Holy Bolt",
    "Holy Fire",
    "Thorns",
    "Defiance",
    "Resist Cold",
    "Zeal",
    "Charge",
    "Blessed Aim",
    "Cleansing",
    "Resist Lightning",
    "Vengeance",
    "Blessed Hammer",
    "Concentration",
    "Holy Freeze",
    "Vigor",
    "Conversion",
    "Holy Shield",
    "Holy Shock",
    "Sanctuary",
    "Meditation",
    "Fist of the Heavens",
    "Fanaticism",
    "Conviction",
    "Redemption",
    "Salvation",
];

const BARBARIAN_SKILLS: [&str; 30] = [
    "Bash",
    "Sword Mastery",
    "Axe Mastery",
    "Mace Mastery",
    "Howl",
    "Find Potion",
    "Leap",
    "Double Swing",
    "Polearm Mastery",
    "Throwing Mastery",
    "Spear Mastery",
    "Taunt",
    "Shout",
    "Stun",
    "Double Throw",
    "Increased Stamina",
    "Find Item",
    "Leap Attack",
    "Concentrate",
    "Iron Skin",
    "Battle Cry",
    "Frenzy",
    "Increased Speed",
    "Battle Orders",
    "Grim Ward",
    "Whirlwind",
    "Berserk",
    "Natural Resistance",
    "War Cry",
    "Battle Command",
];

const DRUID_SKILLS: [&str; 30] = [
    "Raven",
    "Poison Creeper",
    "Werewolf",
    "Lycanthropy",
    "Firestorm",
    "Oak Sage",
    "Summon Spirit Wolf",
    "Werebear",
    "Molten Boulder",
    "Arctic Blast",
    "Carrion Vine",
    "Feral Rage",
    "Maul",
    "Fissure",
    "Cyclone Armor",
    "Heart of Wolverine",
    "Summon Dire Wolf",
    "Rabies",
    "Fire Claws",
    "Twister",
    "Solar Creeper",
    "Hunger",
    "Shock Wave",
    "Volcano",
    "Tornado",
    "Spirit of Barbs",
    "Summon Grizzly",
    "Fury",
    "Armageddon",
    "Hurricane",
];

const ASSASSIN_SKILLS: [&str; 30] = [
    "Fire Blast",
    "Claw Mastery",
    "Psychic Hammer",
    "Tiger Strike",
    "Dragon Talon",
    "Shock Web",
    "Blade Sentinel",
    "Burst of Speed",
    "Fists of Fire",
    "Dragon Claw",
    "Charged Bolt Sentry",
    "Wake of Fire",
    "Weapon Block",
    "Cloak of Shadows",
    "Cobra Strike",
    "Blade Fury",
    "Fade",
    "Shadow Warrior",
    "Claws of Thunder",
    "Dragon Tail",
    "Lightning Sentry",
    "Wake of Inferno",
    "Mind Blast",
    "Blades of Ice",
    "Dragon Flight",
    "Death Sentry",
    "Blade Shield",
    "Venom",
    "Shadow Master",
    "Phoenix Strike",
];

const WARLOCK_SKILLS: [&str; 30] = [
    "Summon Goatman",
    "Demonic Mastery",
    "Death Mark",
    "Summon Tainted",
    "Summon Defiler",
    "Blood Oath",
    "Engorge",
    "Blood Boil",
    "Consume",
    "Bind Demon",
    "Levitate",
    "Eldritch Blast",
    "Hex Bane",
    "Hex Siphon",
    "Psychic Ward",
    "Echoing Strike",
    "Hex Purge",
    "Blade Warp",
    "Cleave",
    "Mirrored Blades",
    "Sigil Lethargy",
    "Ring of Fire",
    "Miasma Bolt",
    "Sigil Rancor",
    "Enhanced Entropy",
    "Flame Wave",
    "Miasma Chains",
    "Sigil Death",
    "Apocalypse",
    "Abyss",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamedSkillError {
    /// Class has no built-in D2R name table in this mapping.
    UnsupportedClass(Class),
    /// Name was not found in the class table after normalization.
    UnknownSkillName { class: Class, skill_name: String },
    /// Index is out of bounds for the class table.
    InvalidSkillIndex { class: Class, skill_index: usize },
}

impl fmt::Display for NamedSkillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedClass(class) => {
                write!(f, "No default D2R named-skill mapping for class {class}")
            }
            Self::UnknownSkillName { class, skill_name } => {
                write!(f, "Unknown D2R skill '{skill_name}' for class {class}")
            }
            Self::InvalidSkillIndex { class, skill_index } => {
                write!(f, "Invalid skill index {skill_index} for class {class}")
            }
        }
    }
}

impl std::error::Error for NamedSkillError {}

/// Resolve raw skill index to default D2R skill name for a class.
pub fn d2r_skill_name(class: Class, skill_index: usize) -> Result<&'static str, NamedSkillError> {
    let class_skills = d2r_skills_for_class(class)?;
    class_skills
        .get(skill_index)
        .copied()
        .ok_or(NamedSkillError::InvalidSkillIndex { class, skill_index })
}

/// Resolve default D2R skill name to class-local raw skill index.
///
/// Name matching is normalized to ignore case and non-alphanumeric characters.
pub fn d2r_skill_index(class: Class, skill_name: &str) -> Result<usize, NamedSkillError> {
    if has_no_ascii_alnum(skill_name) {
        return Err(NamedSkillError::UnknownSkillName {
            class,
            skill_name: skill_name.to_string(),
        });
    }

    let class_skills = d2r_skills_for_class(class)?;
    class_skills
        .iter()
        .position(|candidate| eq_normalized(candidate, skill_name))
        .ok_or(NamedSkillError::UnknownSkillName { class, skill_name: skill_name.to_string() })
}

fn d2r_skills_for_class(class: Class) -> Result<&'static [&'static str; 30], NamedSkillError> {
    match class {
        Class::Amazon => Ok(&AMAZON_SKILLS),
        Class::Sorceress => Ok(&SORCERESS_SKILLS),
        Class::Necromancer => Ok(&NECROMANCER_SKILLS),
        Class::Paladin => Ok(&PALADIN_SKILLS),
        Class::Barbarian => Ok(&BARBARIAN_SKILLS),
        Class::Druid => Ok(&DRUID_SKILLS),
        Class::Assassin => Ok(&ASSASSIN_SKILLS),
        Class::Warlock => Ok(&WARLOCK_SKILLS),
        Class::Unknown(_) => Err(NamedSkillError::UnsupportedClass(class)),
    }
}

fn normalized_ascii_alnum_byte(byte: u8) -> Option<u8> {
    byte.is_ascii_alphanumeric().then_some(byte.to_ascii_lowercase())
}

fn has_no_ascii_alnum(skill_name: &str) -> bool {
    !skill_name.bytes().any(|byte| byte.is_ascii_alphanumeric())
}

fn eq_normalized(left: &str, right: &str) -> bool {
    let mut left_iter = left.bytes().filter_map(normalized_ascii_alnum_byte);
    let mut right_iter = right.bytes().filter_map(normalized_ascii_alnum_byte);

    loop {
        match (left_iter.next(), right_iter.next()) {
            (Some(left_byte), Some(right_byte)) if left_byte == right_byte => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}
