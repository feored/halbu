
pub const SECTION_HEADER: [u8; 2] = [0x69, 0x66];
pub const SECTION_BYTES: usize = 32;

pub const SKILL_OFFSET_AMAZON: usize = 6;
pub const SKILL_OFFSET_SORCERESS: usize = 36;
pub const SKILL_OFFSET_NECROMANCER: usize = 66;
pub const SKILL_OFFSET_PALADIN: usize = 96;
pub const SKILL_OFFSET_BARBARIAN: usize = 126;
pub const SKILL_OFFSET_DRUID: usize = 221;
pub const SKILL_OFFSET_ASSASSIN: usize = 251;

pub const SKILLS_REFERENCE: [&str; 357] = [
    "Attack",
    "Kick",
    "Throw",
    "Unsummon",
    "Left Hand Throw",
    "Left Hand Swing",
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
    "Dopplezon",
    "Evade",
    "Fend",
    "Freezing Arrow",
    "Valkyrie",
    "Pierce",
    "Lightning Strike",
    "Lightning Fury",
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
    "BloodGolem",
    "Attract",
    "Decrepify",
    "Bone Prison",
    "Summon Resist",
    "IronGolem",
    "Lower Resist",
    "Poison Nova",
    "Bone Spirit",
    "FireGolem",
    "Revive",
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
    "Bash",
    "Sword Mastery",
    "Axe Mastery",
    "Mace Mastery",
    "Howl",
    "Find Potion",
    "Leap",
    "Double Swing",
    "Pole Arm Mastery",
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
    "Fire Hit",
    "UnHolyBolt",
    "SkeletonRaise",
    "MaggotEgg",
    "ShamanFire",
    "MagottUp",
    "MagottDown",
    "MagottLay",
    "AndrialSpray",
    "Jump",
    "Swarm Move",
    "Nest",
    "Quick Strike",
    "VampireFireball",
    "VampireFirewall",
    "VampireMeteor",
    "GargoyleTrap",
    "SpiderLay",
    "VampireHeal",
    "VampireRaise",
    "Submerge",
    "FetishAura",
    "FetishInferno",
    "ZakarumHeal",
    "Emerge",
    "Resurrect",
    "Bestow",
    "MissileSkill1",
    "MonTeleport",
    "PrimeLightning",
    "PrimeBolt",
    "PrimeBlaze",
    "PrimeFirewall",
    "PrimeSpike",
    "PrimeIceNova",
    "PrimePoisonball",
    "PrimePoisonNova",
    "DiabLight",
    "DiabCold",
    "DiabFire",
    "FingerMageSpider",
    "DiabWall",
    "DiabRun",
    "DiabPrison",
    "PoisonBallTrap",
    "AndyPoisonBolt",
    "HireableMissile",
    "DesertTurret",
    "ArcaneTower",
    "MonBlizzard",
    "Mosquito",
    "CursedBallTrapRight",
    "CursedBallTrapLeft",
    "MonFrozenArmor",
    "MonBoneArmor",
    "MonBoneSpirit",
    "MonCurseCast",
    "HellMeteor",
    "RegurgitatorEat",
    "MonFrenzy",
    "QueenDeath",
    "Scroll of Identify",
    "Book of Identify",
    "Scroll of Townportal",
    "Book of Townportal",
    "Raven",
    "Plague Poppy",
    "Wearwolf",
    "Shape Shifting",
    "Firestorm",
    "Oak Sage",
    "Summon Spirit Wolf",
    "Wearbear",
    "Molten Boulder",
    "Arctic Blast",
    "Cycle of Life",
    "Feral Rage",
    "Maul",
    "Eruption",
    "Cyclone Armor",
    "Heart of Wolverine",
    "Summon Fenris",
    "Rabies",
    "Fire Claws",
    "Twister",
    "Vines",
    "Hunger",
    "Shock Wave",
    "Volcano",
    "Tornado",
    "Spirit of Barbs",
    "Summon Grizzly",
    "Fury",
    "Armageddon",
    "Hurricane",
    "Fire Trauma",
    "Claw Mastery",
    "Psychic Hammer",
    "Tiger Strike",
    "Dragon Talon",
    "Shock Field",
    "Blade Sentinel",
    "Quickness",
    "Fists of Fire",
    "Dragon Claw",
    "Charged Bolt Sentry",
    "Wake of Fire Sentry",
    "Weapon Block",
    "Cloak of Shadows",
    "Cobra Strike",
    "Blade Fury",
    "Fade",
    "Shadow Warrior",
    "Claws of Thunder",
    "Dragon Tail",
    "Lightning Sentry",
    "Inferno Sentry",
    "Mind Blast",
    "Blades of Ice",
    "Dragon Flight",
    "Death Sentry",
    "Blade Shield",
    "Venom",
    "Shadow Master",
    "Royal Strike",
    "Wake Of Destruction Sentry",
    "Imp Inferno",
    "Imp Fireball",
    "Baal Taunt",
    "Baal Corpse Explode",
    "Baal Monster Spawn",
    "Catapult Charged Ball",
    "Catapult Spike Ball",
    "Suck Blood",
    "Cry Help",
    "Healing Vortex",
    "Teleport 2",
    "Self-resurrect",
    "Vine Attack",
    "Overseer Whip",
    "Barbs Aura",
    "Wolverine Aura",
    "Oak Sage Aura",
    "Imp Fire Missile",
    "Impregnate",
    "Siege Beast Stomp",
    "MinionSpawner",
    "CatapultBlizzard",
    "CatapultPlague",
    "CatapultMeteor",
    "BoltSentry",
    "CorpseCycler",
    "DeathMaul",
    "Defense Curse",
    "Blood Mana",
    "mon inferno sentry",
    "mon death sentry",
    "sentry lightning",
    "fenris rage",
    "Baal Tentacle",
    "Baal Nova",
    "Baal Inferno",
    "Baal Cold Missiles",
    "MegademonInferno",
    "EvilHutSpawner",
    "CountessFirewall",
    "ImpBolt",
    "Horror Arctic Blast",
    "death sentry ltng",
    "VineCycler",
    "BearSmite",
    "Resurrect2",
    "BloodLordFrenzy",
    "Baal Teleport",
    "Imp Teleport",
    "Baal Clone Teleport",
    "ZakarumLightning",
    "VampireMissile",
    "MephistoMissile",
    "DoomKnightMissile",
    "RogueMissile",
    "HydraMissile",
    "NecromageMissile",
    "MonBow",
    "MonFireArrow",
    "MonColdArrow",
    "MonExplodingArrow",
    "MonFreezingArrow",
    "MonPowerStrike",
    "SuccubusBolt",
    "MephFrostNova",
    "MonIceSpear",
    "ShamanIce",
    "Diablogeddon",
    "Delerium Change",
    "NihlathakCorpseExplosion",
    "SerpentCharge",
    "Trap Nova",
    "UnHolyBoltEx",
    "ShamanFireEx",
    "Imp Fire Missile Ex",
];
