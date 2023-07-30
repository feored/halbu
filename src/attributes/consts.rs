use super::Stat;

pub const SECTION_TRAILER: u32 = 0x1FF;

pub const SECTION_HEADER: [u8; 2] = [0x67, 0x66];

pub const STAT_HEADER_LENGTH: usize = 9;
pub const STAT_NUMBER: usize = 16;

pub const MAX_XP: u32 = 3_520_485_254;
pub const GOLD_INVENTORY_PER_LEVEL: u32 = 10_000;
pub const MAX_GOLD_STASH : u32 = 2_500_000;

/// Array used to find the index of each stat
pub const STAT_KEY: [Stat; STAT_NUMBER] = [
    Stat::Strength,
    Stat::Energy,
    Stat::Dexterity,
    Stat::Vitality,
    Stat::StatPointsLeft,
    Stat::SkillPointsLeft,
    Stat::LifeCurrent,
    Stat::LifeBase,
    Stat::ManaCurrent,
    Stat::ManaBase,
    Stat::StaminaCurrent,
    Stat::StaminaBase,
    Stat::Level,
    Stat::Experience,
    Stat::GoldInventory,
    Stat::GoldStash,
];

/// Length in bits of each stat
pub const STAT_BITLENGTH: [usize; STAT_NUMBER] =
    [10, 10, 10, 10, 10, 8, 21, 21, 21, 21, 21, 21, 7, 32, 25, 25];

pub const EXPERIENCE_TABLE: [u32; 99] = [
    0,
    500,
    1_500,
    3_750,
    7_875,
    14_175,
    22_680,
    32_886,
    44_396,
    57_715,
    72_144,
    90_180,
    112_725,
    140_906,
    176_132,
    220_165,
    275_207,
    344_008,
    430_010,
    537_513,
    671_891,
    839_864,
    1_049_830,
    1_312_287,
    1_640_359,
    2_050_449,
    2_563_061,
    3_203_826,
    3_902_260,
    4_663_553,
    5_493_363,
    6_397_855,
    7_383_752,
    8_458_379,
    9_629_723,
    10_906_488,
    12_298_162,
    13_815_086,
    15_468_534,
    17_270_791,
    19_235_252,
    21_376_515,
    23_710_491,
    26_254_525,
    29_027_522,
    32_050_088,
    35_344_686,
    38_935_798,
    42_850_109,
    47_116_709,
    51_767_302,
    56_836_449,
    62_361_819,
    68_384_473,
    74_949_165,
    82_104_680,
    89_904_191,
    98_405_658,
    107_672_256,
    117_772_849,
    128_782_495,
    140_783_010,
    153_863_570,
    168_121_381,
    183_662_396,
    200_602_101,
    219_066_380,
    239_192_444,
    261_129_853,
    285_041_630,
    311_105_466,
    339_515_048,
    370_481_492,
    404_234_916,
    441_026_148,
    481_128_591,
    524_840_254,
    572_485_967,
    624_419_793,
    681_027_665,
    742_730_244,
    809_986_056,
    883_294_891,
    963_201_521,
    1_050_299_747,
    1_145_236_814,
    1_248_718_217,
    1_361_512_946,
    1_484_459_201,
    1_618_470_619,
    1_764_543_065,
    1_923_762_030,
    2_097_310_703,
    2_286_478_756,
    2_492_671_933,
    2_717_422_497,
    2_962_400_612,
    3_229_426_756,
    3_520_485_254,
];
