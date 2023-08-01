use super::*;

pub const VARIANTS: &[Variant; 39] = &[
    (Class::Rogue(Rogue::Fire), Difficulty::Normal),
    (Class::Rogue(Rogue::Cold), Difficulty::Normal),
    (Class::Rogue(Rogue::Fire), Difficulty::Nightmare),
    (Class::Rogue(Rogue::Cold), Difficulty::Nightmare),
    (Class::Rogue(Rogue::Fire), Difficulty::Hell),
    (Class::Rogue(Rogue::Cold), Difficulty::Hell),
    (Class::DesertMercenary(DesertMercenary::Prayer), Difficulty::Normal),
    (Class::DesertMercenary(DesertMercenary::Defiance), Difficulty::Normal),
    (Class::DesertMercenary(DesertMercenary::BlessedAim), Difficulty::Normal),
    (Class::DesertMercenary(DesertMercenary::Thorns), Difficulty::Nightmare),
    (Class::DesertMercenary(DesertMercenary::HolyFreeze), Difficulty::Nightmare),
    (Class::DesertMercenary(DesertMercenary::Might), Difficulty::Nightmare),
    (Class::DesertMercenary(DesertMercenary::Prayer), Difficulty::Hell),
    (Class::DesertMercenary(DesertMercenary::Defiance), Difficulty::Hell),
    (Class::DesertMercenary(DesertMercenary::BlessedAim), Difficulty::Hell),
    (Class::IronWolf(IronWolf::Fire), Difficulty::Normal),
    (Class::IronWolf(IronWolf::Cold), Difficulty::Normal),
    (Class::IronWolf(IronWolf::Lightning), Difficulty::Normal),
    (Class::IronWolf(IronWolf::Fire), Difficulty::Nightmare),
    (Class::IronWolf(IronWolf::Cold), Difficulty::Nightmare),
    (Class::IronWolf(IronWolf::Lightning), Difficulty::Nightmare),
    (Class::IronWolf(IronWolf::Fire), Difficulty::Hell),
    (Class::IronWolf(IronWolf::Cold), Difficulty::Hell),
    (Class::IronWolf(IronWolf::Lightning), Difficulty::Hell),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Normal),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Normal),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Nightmare),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Nightmare),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Hell),
    (Class::Barbarian(Barbarian::Bash), Difficulty::Hell),
    (Class::DesertMercenary(DesertMercenary::Prayer), Difficulty::Nightmare),
    (Class::DesertMercenary(DesertMercenary::Defiance), Difficulty::Nightmare),
    (Class::DesertMercenary(DesertMercenary::BlessedAim), Difficulty::Nightmare),
    (Class::DesertMercenary(DesertMercenary::Thorns), Difficulty::Hell),
    (Class::DesertMercenary(DesertMercenary::HolyFreeze), Difficulty::Hell),
    (Class::DesertMercenary(DesertMercenary::Might), Difficulty::Hell),
    (Class::Barbarian(Barbarian::Frenzy), Difficulty::Normal),
    (Class::Barbarian(Barbarian::Frenzy), Difficulty::Nightmare),
    (Class::Barbarian(Barbarian::Frenzy), Difficulty::Hell),
];

pub const ROGUE_NAMES: [&str; 41] = [
    "Aliza", "Ampliza", "Annor", "Abhaya", "Elly", "Paige", "Basanti", "Blaise", "Kyoko",
    "Klaudia", "Kundri", "Kyle", "Visala", "Elexa", "Floria", "Fiona", "Gwinni", "Gaile", "Hannah",
    "Heather", "Iantha", "Diane", "Isolde", "Divo", "Ithera", "Itonya", "Liene", "Maeko", "Mahala",
    "Liaza", "Meghan", "Olena", "Oriana", "Ryann", "Rozene", "Raissa", "Sharyn", "Shikha", "Debi",
    "Tylena", "Wendy",
];

pub const DESERTMERCENARY_NAMES: [&str; 21] = [
    "Hazade", "Alhizeer", "Azrael", "Ahsab", "Chalan", "Haseen", "Razan", "Emilio", "Pratham",
    "Fazel", "Jemali", "Kasim", "Gulzar", "Mizan", "Leharas", "Durga", "Neeraj", "Ilzan",
    "Zanarhi", "Waheed", "Vikhyat",
];

pub const IRONWOLF_NAMES: [&str; 20] = [
    "Jelani", "Barani", "Jabari", "Devak", "Raldin", "Telash", "Ajheed", "Narphet", "Khaleel",
    "Phaet", "Geshef", "Vanji", "Haphet", "Thadar", "Yatiraj", "Rhadge", "Yashied", "Jarulf",
    "Flux", "Scorch",
];

pub const BARBARIAN_NAMES: [&str; 67] = [
    "Varaya",
    "Khan",
    "Klisk",
    "Bors",
    "Brom",
    "Wiglaf",
    "Hrothgar",
    "Scyld",
    "Healfdane",
    "Heorogar",
    "Halgaunt",
    "Hygelac",
    "Egtheow",
    "Bohdan",
    "Wulfgar",
    "Hild",
    "Heatholaf",
    "Weder",
    "Vikhyat",
    "Unferth",
    "Sigemund",
    "Heremod",
    "Hengest",
    "Folcwald",
    "Frisian",
    "Hnaef",
    "Guthlaf",
    "Oslaf",
    "Yrmenlaf",
    "Garmund",
    "Freawaru",
    "Eadgils",
    "Onela",
    "Damien",
    "Erfor",
    "Weohstan",
    "Wulf",
    "Bulwye",
    "Lief",
    "Magnus",
    "Klatu",
    "Drus",
    "Hoku",
    "Kord",
    "Uther",
    "Ip",
    "Ulf",
    "Tharr",
    "Kaelim",
    "Ulric",
    "Alaric",
    "Ethelred",
    "Caden",
    "Elgifu",
    "Tostig",
    "Alcuin",
    "Emund",
    "Sigurd",
    "Gorm",
    "Hollis",
    "Ragnar",
    "Torkel",
    "Wulfstan",
    "Alban",
    "Barloc",
    "Bill",
    "Theodoric",
];
