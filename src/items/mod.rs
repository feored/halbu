use huffman::Node;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::sync::OnceLock;

mod huffman;

use crate::csv::{get_row, read_csv, Record};
use crate::{
    bit_manipulation::{read_bit, read_bits, BytePosition},
    convert::u16_from,
    Class, ParseError,
};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

const HEADER: [u8; 2] = [0x4A, 0x4D];
const MERCENARY_HEADER: [u8; 2] = [0x6A, 0x66];
const IRON_GOLEM_HEADER: [u8; 2] = [0x6B, 0x66];

const TP_BOOK: &'static str = "tbk ";
const ID_BOOK: &'static str = "ibk ";

static ARMORS: OnceLock<Vec<Record>> = OnceLock::new();
static WEAPONS: OnceLock<Vec<Record>> = OnceLock::new();
static MISC: OnceLock<Vec<Record>> = OnceLock::new();

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum InferiorState {
    #[default]
    Crude = 0,
    Cracked = 1,
    Damaged = 2,
    LowQuality = 3,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct RareData {
    pub name_prefix: u8,
    pub name_suffix: u8,
    pub mod_prefixes: [Option<u16>; 3],
    pub mod_suffixes: [Option<u16>; 3],
}

impl RareData {
    pub fn parse(bytes: &[u8], byte_index: &mut BytePosition) -> Self {
        let mut rare: RareData = RareData::default();
        rare.name_prefix = read_bits(&bytes, byte_index, 8).unwrap() as u8;
        rare.name_suffix = read_bits(&bytes, byte_index, 8).unwrap() as u8;
        for i in 0..3usize {
            let prefix_present = read_bit(&bytes, byte_index).unwrap();
            if prefix_present {
                rare.mod_prefixes[i] = Some(read_bits(&bytes, byte_index, 11).unwrap() as u16);
            }

            let suffix_present = read_bit(&bytes, byte_index).unwrap();
            if suffix_present {
                rare.mod_suffixes[i] = Some(read_bits(&bytes, byte_index, 11).unwrap() as u16);
            }
        }
        rare
    }
}

#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub enum QualityData {
    Inferior(InferiorState) = 1,
    #[default]
    Normal = 2,
    Superior(u8) = 3,
    Magic((u16, u16)) = 4,
    Set(u16) = 5,
    Rare(RareData) = 6,
    Unique(u16) = 7,
    Crafted(RareData) = 8,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Quality {
    Inferior = 1,
    #[default]
    Normal = 2,
    Superior = 3,
    Magic = 4,
    Set = 5,
    Rare = 6,
    Unique = 7,
    Crafted = 8,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Storage {
    #[default]
    None = 0,
    Inventory = 1,
    Cube = 4,
    Stash = 5,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Status {
    #[default]
    Stored = 0,
    Equipped = 1,
    Belt = 2,
    Cursor = 4,
    Socketed = 6,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Slot {
    #[default]
    None = 0,
    Helmet = 1,
    Amulet = 2,
    Armor = 3,
    WeaponRight = 4,
    WeaponLeft = 5,
    RingRight = 6,
    RingLeft = 7,
    Belt = 8,
    Boots = 9,
    Gloves = 10,
    SwitchRight = 11,
    SwitchLeft = 12,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Items {
    pub count: u16,
    pub items: Vec<Item>,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Corpse {
    pub exists: bool,
    pub corpses: Vec<CorpseItems>,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct CorpseItems {
    pub x: u32,
    pub y: u32,
    pub count: u16,
    pub items: Vec<Item>,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct MercenaryItems {
    pub count: u16,
    pub items: Vec<Item>,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct IronGolem {
    pub exists: bool,
    pub item: Item,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Item {
    pub header: ItemHeader,
    pub extended_data: Option<ExtendedItem>,
}

impl Item {
    pub fn parse(bytes: &[u8], byte_index: &mut BytePosition) -> Item {
        let mut item = Item::default();
        item.header = ItemHeader::parse(bytes, byte_index);
        if item.header.compact {
            return item;
        }
        item.extended_data = Some(ExtendedItem::parse(&item.header, bytes, byte_index));

        item
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ExtendedItem {
    pub id: u32,
    pub level: u8,
    pub quality: Quality,
    pub custom_graphics_id: Option<u8>,
    pub auto_mod: Option<u16>,
    pub quality_data: QualityData,
    pub runeword_id: Option<u16>,
}

fn armors() -> &'static Vec<Record> {
    ARMORS.get_or_init(|| read_csv(include_bytes!("../../assets/data/armor.txt")).unwrap())
}

fn weapons() -> &'static Vec<Record> {
    WEAPONS.get_or_init(|| read_csv(include_bytes!("../../assets/data/weapons.txt")).unwrap())
}

fn misc() -> &'static Vec<Record> {
    MISC.get_or_init(|| read_csv(include_bytes!("../../assets/data/misc.txt")).unwrap())
}

impl ExtendedItem {
    pub fn parse(header: &ItemHeader, bytes: &[u8], byte_index: &mut BytePosition) -> Self {
        let mut extended_item = ExtendedItem::default();
        extended_item.id = read_bits(&bytes, byte_index, 32).unwrap();

        extended_item.level = read_bits(&bytes, byte_index, 7).unwrap() as u8;

        extended_item.quality =
            Quality::try_from(read_bits(&bytes, byte_index, 4).unwrap() as u8).unwrap();

        let custom_graphics_present = read_bit(&bytes, byte_index).unwrap();

        if custom_graphics_present {
            extended_item.custom_graphics_id =
                Some(read_bits(&bytes, byte_index, 3).unwrap() as u8);
        }

        let auto_mod_present = read_bit(&bytes, byte_index).unwrap();
        if auto_mod_present {
            extended_item.auto_mod = Some(read_bits(&bytes, byte_index, 11).unwrap() as u16);
        }
        extended_item.quality_data = match extended_item.quality {
            Quality::Inferior => QualityData::Inferior(
                InferiorState::try_from(read_bits(&bytes, byte_index, 3).unwrap() as u8).unwrap(),
            ),
            Quality::Normal => QualityData::Normal,
            Quality::Superior => {
                QualityData::Superior(read_bits(&bytes, byte_index, 3).unwrap() as u8)
            }
            Quality::Magic => QualityData::Magic((
                read_bits(&bytes, byte_index, 11).unwrap() as u16,
                read_bits(&bytes, byte_index, 11).unwrap() as u16,
            )),
            Quality::Set => QualityData::Set(read_bits(&bytes, byte_index, 12).unwrap() as u16),
            Quality::Rare => QualityData::Rare(RareData::parse(&bytes, byte_index)),
            Quality::Unique => {
                QualityData::Unique(read_bits(&bytes, byte_index, 12).unwrap() as u16)
            }
            Quality::Crafted => QualityData::Crafted(RareData::parse(&bytes, byte_index)),
        };

        if header.runeword {
            extended_item.runeword_id = Some(read_bits(&bytes, byte_index, 12).unwrap() as u16);
            warn!("Runeword misc info: {0}", read_bits(&bytes, byte_index, 4).unwrap());
        }

        if header.personalized {
            // Test personalization with i.e japanese name
            let mut name: String = String::default();
            loop {
                let ch = char::from(read_bits(&bytes, byte_index, 8).unwrap() as u8);
                if ch == '\0' {
                    break;
                }
                name.push(ch);
            }
        }

        if header.base == ID_BOOK || header.base == TP_BOOK {
            warn!("Spell ID: {0}", read_bits(&bytes, byte_index, 5).unwrap());
        }

        let realm_data_present = read_bit(&bytes, byte_index).unwrap();
        if realm_data_present {
            warn!("Realm data found, skipping 128 bits.");
            read_bits(&bytes, byte_index, 128).unwrap();
        }

        let armor_row = get_row(&armors(), "code", &header.base[0..3]);
        let is_armor = !armor_row.is_none();

        let weapon_row = get_row(&weapons(), "code", &header.base[0..3]);
        let is_weapon = !weapon_row.is_none();
        warn!("is armor: {0}, is weapon: {1}", is_armor, is_weapon);

        warn!("{0:?}", extended_item);
        extended_item
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ItemHeader {
    pub identified: bool,
    pub broken: bool,
    pub socketed: bool,
    pub ear: bool,
    pub starter_gear: bool,
    pub compact: bool,
    pub ethereal: bool,
    pub personalized: bool,
    pub runeword: bool,
    pub status: Status,
    pub slot: Slot,
    pub column: u8,
    pub row: u8,
    pub storage: Storage,
    pub base: String,
    pub socketed_count: u8,
}

impl ItemHeader {
    pub fn parse(bytes: &[u8], byte_index: &mut BytePosition) -> ItemHeader {
        warn!("Starting new item!");
        let starting_index = byte_index.clone();
        let mut header: ItemHeader = ItemHeader::default();
        read_bits(&bytes, byte_index, 4).unwrap(); // unknown

        header.identified = read_bit(&bytes, byte_index).unwrap();

        read_bits(&bytes, byte_index, 3).unwrap(); // unknown

        header.broken = read_bit(&bytes, byte_index).unwrap();

        read_bits(&bytes, byte_index, 2).unwrap(); // unknown

        header.socketed = read_bit(&bytes, byte_index).unwrap();

        read_bits(&bytes, byte_index, 4).unwrap(); // unknown (bit 2 is picked up since last save)

        header.ear = read_bit(&bytes, byte_index).unwrap();

        header.starter_gear = read_bit(&bytes, byte_index).unwrap();

        read_bits(&bytes, byte_index, 3).unwrap(); // unknown

        header.compact = read_bit(&bytes, byte_index).unwrap();

        header.ethereal = read_bit(&bytes, byte_index).unwrap();

        read_bit(&bytes, byte_index).unwrap(); // unknown

        header.personalized = read_bit(&bytes, byte_index).unwrap();

        read_bit(&bytes, byte_index).unwrap();

        read_bit(&bytes, byte_index).unwrap();

        read_bits(&bytes, byte_index, 8).unwrap(); // unknown

        header.status = Status::try_from(read_bits(&bytes, byte_index, 3).unwrap() as u8).unwrap();

        header.slot = Slot::try_from(read_bits(&bytes, byte_index, 4).unwrap() as u8).unwrap();

        header.column = read_bits(&bytes, byte_index, 4).unwrap() as u8;

        header.row = read_bits(&bytes, byte_index, 3).unwrap() as u8;

        read_bit(&bytes, byte_index).unwrap(); // unknown

        let raw_storage = read_bits(&bytes, byte_index, 3).unwrap();

        header.storage = Storage::try_from(raw_storage as u8).unwrap();

        let tree = Node::build_huffman_tree();
        let mut base_id: String = String::default();
        for _i in 0..4 {
            let mut base: String = String::default();
            loop {
                let base_raw = read_bit(&bytes, byte_index).unwrap();
                base.push_str(if base_raw { "1" } else { "0" });
                match tree.decode(base.clone()) {
                    Some(c) => {
                        base_id.push(c);
                        break;
                    }
                    None => continue,
                }
            }
        }
        header.base = base_id;

        header.socketed_count =
            read_bits(&bytes, byte_index, if header.compact { 1 } else { 3 }).unwrap() as u8;
        warn!(
            "Header done, Total length: {0}",
            byte_index.total_bits() - starting_index.total_bits()
        );

        warn!("{0:?}", header);

        header
    }
}

impl Items {
    pub fn parse(bytes: &[u8]) -> Items {
        let items = Items::default();
        let mut byte_index = BytePosition::default();
        if bytes[0..2] != HEADER {
            warn!(
                "Found invalid header in items section: Expected {0:X?}, found {1:X?}.",
                HEADER,
                &bytes[0..2]
            );
        } else {
            debug!("Found correct header: {0:X?}", &bytes[0..2]);
        }

        debug!("Found item count: {0}", u16_from(&bytes[2..4], "Items Count"));

        byte_index.current_byte = 4;
        loop {
            let item = Item::parse(&bytes, &mut byte_index);
            if !item.header.compact {
                break;
            }
        }
        // loop {
        //     if bytes.len() - byte.index.current_byte < 2 {
        //         break;
        //     }
        //     bits =
        // }
        items
    }
}
