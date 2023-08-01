pub const SECTION_HEADER: [u8; 8] = [0x57, 0x53, 0x01, 0x00, 0x00, 0x00, 0x50, 0x00];
pub const DIFFICULTY_HEADER: [u8; 2] = [0x02, 0x01];
pub const SECTION_TRAILER: u8 = 0x01;

pub const NAMES_ACT1: [&str; 9] = [
    "Rogue Encampment",
    "Cold Plains",
    "Stony Field",
    "Dark Wood",
    "Black Marsh",
    "Outer Cloister",
    "Jail",
    "Inner Cloister",
    "Catacombs",
];

pub const NAMES_ACT2: [&str; 9] = [
    "Lut Gholein",
    "Sewers",
    "Dry Hills",
    "Halls of the Dead",
    "Far Oasis",
    "Lost City",
    "Palace Cellar",
    "Arcane Sanctuary",
    "Canyon of the Magi",
];

pub const NAMES_ACT3: [&str; 9] = [
    "Kurast Docks",
    "Spider Forest",
    "Great Marsh",
    "Flayer Jungle",
    "Lower Kurast",
    "Kurast Bazaar",
    "Upper Kurast",
    "Travincal",
    "Durance of Hate",
];

pub const NAMES_ACT4: [&str; 3] = ["Pandemonium Fortress", "City of the Damned", "River of Flames"];

pub const NAMES_ACT5: [&str; 9] = [
    "Harrogath",
    "Frigid Highlands",
    "Arreat Plateau",
    "Crystalline Passage",
    "Halls of Pain",
    "Glacial Trail",
    "Frozen Tundra",
    "The Ancients' Way",
    "Worldstone Keep",
];
