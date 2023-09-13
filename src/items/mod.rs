use huffman::Node;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cmp;
use std::collections::HashMap;
use std::sync::OnceLock;

use bit::BitIndex;

mod huffman;

use crate::bit_manipulation::ByteIO;
use crate::csv::{get_row, read_csv, Record};
use crate::{convert::u16_from, ParseError};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

const HEADER: [u8; 2] = [0x4A, 0x4D];
const MERCENARY_HEADER: [u8; 2] = [0x6A, 0x66];
const IRON_GOLEM_HEADER: [u8; 2] = [0x6B, 0x66];
const MOD_TRAILER: u16 = 0x1FF;

const TP_BOOK: &'static str = "tbk ";
const ID_BOOK: &'static str = "ibk ";

static ARMORS: OnceLock<Vec<Record>> = OnceLock::new();
static WEAPONS: OnceLock<Vec<Record>> = OnceLock::new();
static MISC: OnceLock<Vec<Record>> = OnceLock::new();
static ITEMSTATCOST: OnceLock<Vec<Record>> = OnceLock::new();
static ASSOCIATED_MODS: OnceLock<HashMap<usize, Vec<usize>>> = OnceLock::new();

// For min-max mods, max value must be read after min value in the same mod
fn linked_mods() -> &'static HashMap<usize, Vec<usize>> {
    ASSOCIATED_MODS.get_or_init(|| {
        HashMap::from([
            (52, vec![53]),     // magicmindam
            (17, vec![18]),     // item_maxdamage_percent...why max-min...
            (48, vec![49]),     // firemindam
            (50, vec![51]),     // lightmindam
            (54, vec![55, 56]), // coldmindam
            (57, vec![58, 59]), // poisonmindam
        ])
    })
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

fn itemstatcost() -> &'static Vec<Record> {
    ITEMSTATCOST
        .get_or_init(|| read_csv(include_bytes!("../../assets/data/itemstatcost.txt")).unwrap())
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Mod {
    pub key: u16,
    pub value: i32,
    pub name: String,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ItemMod {
    pub base: Mod,
    pub linked_mods: Vec<Mod>,
    pub param: Option<i32>,
}

impl ItemMod {
    pub fn parse(reader: &mut ByteIO) -> Vec<ItemMod> {
        let mut mods = Vec::<ItemMod>::new();
        loop {
            let key_id = reader.read_bits(9).unwrap() as usize;
            if key_id as u16 == MOD_TRAILER {
                break;
            }
            let mut new_mod: ItemMod = ItemMod::default();
            let mod_row: &HashMap<String, String> = &itemstatcost()[key_id];
            let key_bits: usize = mod_row["Save Bits"].parse().unwrap();
            let save_add: i32 = match mod_row["Save Add"].as_str() {
                "" => 0i32,
                res => {
                    warn!("Save add value for {1}: {0}", res, key_id);
                    res.parse().unwrap()
                }
            };
            new_mod.base.key = key_id as u16;
            new_mod.base.value = cmp::max(0, reader.read_bits(key_bits).unwrap() as i32 - save_add);
            new_mod.base.name = mod_row["Stat"].clone();
            match linked_mods().get(&key_id) {
                Some(res) => {
                    for linked_key in res {
                        let bits: usize = itemstatcost()[*linked_key]["Save Bits"].parse().unwrap();
                        new_mod.linked_mods.push(Mod {
                            key: *linked_key as u16,
                            name: itemstatcost()[*linked_key]["Stat"].clone(),
                            value: reader.read_bits(bits).unwrap() as i32,
                        });
                    }
                }
                None => (),
            }
            if new_mod.linked_mods.len() == 0 {
                let param_bits = mod_row["Save Param Bits"].clone();
                if param_bits != "" {
                    new_mod.param =
                        Some(reader.read_bits(param_bits.parse().unwrap()).unwrap() as i32);
                }
            }
            mods.push(new_mod);
        }
        mods
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
enum ItemType {
    Armor,
    Weapon,
    Misc,
}

impl ItemType {
    fn get(base: &String) -> (ItemType, Record) {
        let mut row = get_row(&armors(), "code", base.trim());
        if !row.is_none() {
            return (ItemType::Armor, row.unwrap());
        } else {
            row = get_row(&weapons(), "code", base.trim());
            if !row.is_none() {
                return (ItemType::Weapon, row.unwrap());
            } else {
                row = get_row(&misc(), "code", base.trim());
                return (ItemType::Misc, row.unwrap());
            }
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Inferior {
    #[default]
    Crude = 0,
    Cracked = 1,
    Damaged = 2,
    LowQuality = 3,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ItemName {
    pub prefix: u8,
    pub suffix: u8,
}

#[repr(u8)]
#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub enum Quality {
    Inferior(Inferior) = 1,
    #[default]
    Normal = 2,
    Superior(u8) = 3,
    Magic = 4,
    Set(u16) = 5,
    Rare(ItemName) = 6,
    Unique(u16) = 7,
    Crafted(ItemName) = 8,
}

impl TryFrom<u8> for Quality {
    type Error = ParseError;
    fn try_from(value: u8) -> Result<Self, ParseError> {
        let result = match value {
            1 => Quality::Inferior(Inferior::default()),
            2 => Quality::Normal,
            3 => Quality::Superior(0),
            4 => Quality::Magic,
            5 => Quality::Set(0),
            6 => Quality::Rare(ItemName::default()),
            7 => Quality::Unique(0),
            8 => Quality::Crafted(ItemName::default()),
            _ => {
                return Err(ParseError {
                    message: format!(
                        "Failed to convert quality value {0} to enum to quality enum.",
                        value
                    ),
                })
            }
        };
        Ok(result)
    }
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
    Ground = 3,
    Cursor = 4,
    Dropping = 5,
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
    pub corpse: Corpse,
    pub mercenary: MercenaryItems,
    pub iron_golem: IronGolem,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Corpse {
    pub exists: bool,
    pub x: u32,
    pub y: u32,
    pub item_count: u16,
    pub items: Vec<Item>,
}

impl Corpse {
    fn parse(reader: &mut ByteIO) -> Result<Corpse, ParseError> {
        reader.align();
        let mut corpse = Corpse::default();
        if reader.len_left() < 4 {
            return Err(ParseError{message:format!("Corpse section is shorter than the required 4 bytes. Length: {0}. Returning defaults from this point on.", reader.data.len())});
        }
        let mut header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(ParseError{message:format!("Corpse section has invalid header, found {0:X?} instead of {1:X?}. Returning defaults from this point on.", header, HEADER)});
        }
        corpse.exists = reader.read_bits(16)?.bit(0);
        if !corpse.exists {
            return Ok(corpse);
        }
        reader.read_bits(32)?; // Unknown
        corpse.x = reader.read_bits(32)?;
        corpse.y = reader.read_bits(32)?;

        header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(ParseError{message:format!("Corpse section has invalid second header, found {0:X?} instead of {1:X?}. Returning defaults from this point on.", header, HEADER)});
        }

        corpse.item_count = reader.read_bits(16)? as u16;
        for i in 0..corpse.item_count {
            let item = Item::parse(reader);

            match item {
                Ok(res) => {
                    warn!("Parsed item #{0}", i);
                    corpse.items.push(res);
                }
                Err(res) => warn!("Skipping item, because of error: {0}", res.to_string()),
            }
        }
        Ok(corpse)
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct MercenaryItems {
    pub items_count: u16,
    pub items: Vec<Item>,
    pub id: u32, // TODO CHECK ACTUAL LENGTH
}

impl MercenaryItems {
    fn parse(reader: &mut ByteIO, expansion: bool, hired: bool) -> Result<Self, ParseError> {
        reader.align();
        if reader.len_left() < 2 {
            return Err(ParseError{message:format!("Mercenary items section is shorter than the required 4 bytes. Length: {0}. Returning defaults from this point on.", reader.data.len())});
        }
        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != MERCENARY_HEADER {
            return Err(ParseError{message:format!("Mercenary items section has invalid header, found {0:X?} instead of {1:X?}. Returning defaults from this point on.", header, MERCENARY_HEADER)});
        }
        if !hired {
            return Ok(MercenaryItems::default());
        }
        if expansion {
            return Ok(MercenaryItems::parse_expansion(reader)?);
        } else {
            return Ok(MercenaryItems::parse_classic(reader)?);
        }
    }
    fn parse_expansion(reader: &mut ByteIO) -> Result<Self, ParseError> {
        let mut merc = MercenaryItems::default();

        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(ParseError{message:format!("Mercenary items section has invalid header, found {0:X?} instead of {1:X?}. Returning defaults from this point on.", header, HEADER)});
        }

        merc.items_count = reader.read_bits(16)? as u16;

        for i in 0..merc.items_count {
            let item = Item::parse(reader);

            match item {
                Ok(res) => {
                    merc.items.push(res);
                    warn!("Parsed item #{0}", i)
                }
                Err(res) => warn!("Skipping item, because of error: {0}", res.to_string()),
            }
        }

        Ok(merc)
    }
    // TODO TEST ON CLASSIC
    fn parse_classic(reader: &mut ByteIO) -> Result<Self, ParseError> {
        if reader.len_left() < 10 {
            return Err(ParseError{message:format!("Mercenary items section is shorter than the required 10 bytes. Length: {0}. Returning defaults from this point on.", reader.data.len())});
        }
        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(ParseError{message:format!("Mercenary items section has invalid header, found {0:X?} instead of {1:X?}. Returning defaults from this point on.", header, HEADER)});
        }
        let mut merc = MercenaryItems::default();

        merc.id = reader.read_bits(32)?;

        Ok(merc)
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct IronGolem {
    pub item: Option<Item>,
}

impl IronGolem {
    fn parse(reader: &mut ByteIO) -> Result<IronGolem, ParseError> {
        reader.align();
        if reader.len_left() < 3 {
            return Err(ParseError{message:format!("Iron Golem section is shorter than the required 3 bytes. Length: {0}. Returning defaults from this point on.", reader.data.len())});
        }
        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != IRON_GOLEM_HEADER {
            return Err(ParseError{message:format!("Iron Golem section section has invalid header, found {0:X?} instead of {1:X?}. Returning defaults from this point on.", header, IRON_GOLEM_HEADER)});
        }
        let mut iron_golem = IronGolem::default();
        let exists = reader.read_bits(8)?.bit(0);
        if !exists {
            return Ok(iron_golem);
        }
        iron_golem.item = Some(Item::parse(reader)?);
        Ok(iron_golem)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut data = IRON_GOLEM_HEADER.to_vec();
        match &self.item {
            Some(x) => {
                data.push(1u8);
                // TODO ADD ITEM
            }
            None => data.push(0u8),
        }
        data
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Item {
    pub header: ItemHeader,
    pub data: Option<ExtendedItem>,
    pub socketed_items: Vec<Item>,
}

impl Item {
    pub fn parse(reader: &mut ByteIO) -> Result<Item, ParseError> {
        let mut item = Item::default();
        item.header = ItemHeader::parse(reader)?;
        if item.header.compact {
            return Ok(item);
        }
        item.data = Some(ExtendedItem::parse(&item.header, reader)?);
        if item.header.socketed && item.header.socketed_count > 0 {
            for _i in 0..item.header.socketed_count {
                item.socketed_items.push(Item::parse(reader)?);
            }
        }
        Ok(item)
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ExtendedItem {
    pub id: u32,
    pub level: u8,
    pub quality: Quality,
    pub custom_graphics_id: Option<u8>,
    pub auto_mod: Option<u16>,
    pub name_prefix: Option<u8>,
    pub name_suffix: Option<u8>,
    pub prefixes: [u16; 3],
    pub suffixes: [u16; 3],
    pub runeword_id: Option<u16>,
    pub defense: Option<u16>,
    pub durability_max: u8,
    pub durability_current: Option<u16>,
    pub quantity: Option<u16>,
    pub total_sockets: Option<u8>,
    pub mods: Vec<Vec<ItemMod>>,
}

impl ExtendedItem {
    fn parse_rare_crafted(&mut self, reader: &mut ByteIO) -> Result<ItemName, ParseError> {
        let mut name = ItemName::default();
        name.prefix = reader.read_bits(8)? as u8;
        name.suffix = reader.read_bits(8)? as u8;
        for i in 0..3usize {
            let prefix_present = reader.read_bit()?;
            if prefix_present {
                self.prefixes[i] = reader.read_bits(11)? as u16;
            }

            let suffix_present = reader.read_bit()?;
            if suffix_present {
                self.suffixes[i] = reader.read_bits(11)? as u16;
            }
        }
        Ok(name)
    }

    pub fn parse(header: &ItemHeader, reader: &mut ByteIO) -> Result<Self, ParseError> {
        let mut extended_item = ExtendedItem::default();
        extended_item.id = reader.read_bits(32)?;

        extended_item.level = reader.read_bits(7)? as u8;

        extended_item.quality = Quality::try_from(reader.read_bits(4)? as u8)?;

        let custom_graphics_present = reader.read_bit()?;

        if custom_graphics_present {
            extended_item.custom_graphics_id = Some(reader.read_bits(3)? as u8);
        }

        let auto_mod_present = reader.read_bit()?;
        if auto_mod_present {
            extended_item.auto_mod = Some(reader.read_bits(11)? as u16);
        }
        extended_item.quality = match extended_item.quality {
            Quality::Inferior(_) => {
                Quality::Inferior(match Inferior::try_from(reader.read_bits(3)? as u8) {
                    Ok(res) => res,
                    Err(e) => return Err(ParseError { message: e.to_string() }),
                })
            }
            Quality::Normal => Quality::Normal, // TODO: Handle charm case: https://github.com/ThePhrozenKeep/D2MOO/blob/4071d3f4c3cec4a7bb4319b8fe4ff157834fb217/source/D2Common/src/Items/Items.cpp#L5158
            Quality::Superior(_) => Quality::Superior(reader.read_bits(3)? as u8),
            Quality::Magic => {
                extended_item.prefixes[0] = reader.read_bits(11)? as u16;
                extended_item.suffixes[0] = reader.read_bits(11)? as u16;
                Quality::Magic
            }
            Quality::Set(_) => Quality::Set(reader.read_bits(12)? as u16),
            Quality::Rare(_) => Quality::Rare(extended_item.parse_rare_crafted(reader)?),
            Quality::Unique(_) => Quality::Unique(reader.read_bits(12)? as u16),
            Quality::Crafted(_) => Quality::Crafted(extended_item.parse_rare_crafted(reader)?),
        };

        // TODO: Handle ear

        let mut item_lists = 0u16;
        if header.runeword {
            extended_item.runeword_id = Some(reader.read_bits(12)? as u16);
            item_lists = 1 << (reader.read_bits(4)? + 1);
        }

        if header.personalized {
            // Test personalization with i.e japanese name
            let mut name: String = String::default();
            loop {
                let ch = char::from(reader.read_bits(8)? as u8);
                if ch == '\0' {
                    break;
                }
                name.push(ch);
            }
        }

        if header.base == ID_BOOK || header.base == TP_BOOK {
            extended_item.suffixes[0] = reader.read_bits(5)? as u16;
            warn!("Spell ID: {0}", extended_item.suffixes[0]);
        }

        let realm_data_present = reader.read_bit()?;
        if realm_data_present {
            warn!("Realm data found, skipping 128 bits.");
            reader.read_bits(128)?;
        }

        let (item_type, item_row) = ItemType::get(&header.base);
        warn!("item type: {0:?}", item_type);

        if item_type == ItemType::Armor {
            let mut defense = reader.read_bits(11)? as u16;
            warn!("base armor defense: {0}", defense);
            // should defense be signed or unsigned? Signed in itemstatcost.txt is meaningless (durability is also 1 for signed and yet
            // can go up to 255 in game), and yet presence of Save Add would suggest -10 is possible
            defense -= match get_row(&itemstatcost(), "Stat", "armorclass") {
                Some(res) => res["Save Add"].parse().unwrap(),
                None => 0,
            };
            extended_item.defense = Some(cmp::max(defense, 0));
            warn!("armor defense: {0}", defense);
        }

        if item_type == ItemType::Armor || item_type == ItemType::Weapon {
            let max_durability_base = match get_row(&itemstatcost(), "Stat", "maxdurability") {
                Some(res) => res["Save Add"].parse().unwrap(),
                None => 0u8,
            };
            extended_item.durability_max = reader.read_bits(8)? as u8;
            if extended_item.durability_max > 0 {
                extended_item.durability_current = Some(reader.read_bits(9)? as u16);
            }
            extended_item.durability_max += max_durability_base;
        }

        let stackable = item_row["stackable"] == "1";

        if stackable {
            extended_item.quantity = Some(reader.read_bits(9)? as u16);
        }
        if header.socketed {
            extended_item.total_sockets = Some(reader.read_bits(4)? as u8);
        }

        if let Quality::Set(_) = &extended_item.quality {
            item_lists = item_lists | reader.read_bits(5)? as u16;
        }
        extended_item.mods.push(ItemMod::parse(reader));
        warn!("Item lists to read: {0}", item_lists);
        for i in 0..15usize {
            if item_lists.bit(i) {
                extended_item.mods.push(ItemMod::parse(reader));
            }
        }

        warn!("{0:?}", extended_item);
        Ok(extended_item)
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
    fn parse(reader: &mut ByteIO) -> Result<ItemHeader, ParseError> {
        warn!("Starting new item!");
        reader.align();
        let starting_index = reader.position.clone();
        let mut header: ItemHeader = ItemHeader::default();
        reader.read_bits(4)?; // unknown

        header.identified = reader.read_bit()?;

        reader.read_bits(3)?; // unknown

        header.broken = reader.read_bit()?;

        reader.read_bits(2)?; // unknown

        header.socketed = reader.read_bit()?;

        reader.read_bits(4)?; // unknown (bit 2 is picked up since last save)

        header.ear = reader.read_bit()?;

        header.starter_gear = reader.read_bit()?;

        reader.read_bits(3)?; // unknown

        header.compact = reader.read_bit()?;

        header.ethereal = reader.read_bit()?;

        reader.read_bit()?; // unknown

        header.personalized = reader.read_bit()?;

        reader.read_bit()?;

        header.runeword = reader.read_bit()?;

        reader.read_bits(8)?; // unknown

        header.status = match Status::try_from(reader.read_bits(3)? as u8) {
            Ok(res) => res,
            Err(e) => return Err(ParseError { message: e.to_string() }),
        };

        // TODO: Handle ground/dropped cases? https://github.com/ThePhrozenKeep/D2MOO/blob/4071d3f4c3cec4a7bb4319b8fe4ff157834fb217/source/D2Common/src/Items/Items.cpp#L5029

        header.slot = match Slot::try_from(reader.read_bits(4)? as u8) {
            Ok(res) => res,
            Err(e) => return Err(ParseError { message: e.to_string() }),
        };

        header.column = reader.read_bits(4)? as u8;

        header.row = reader.read_bits(4)? as u8;

        let raw_storage = reader.read_bits(3)?;

        header.storage = match Storage::try_from(raw_storage as u8) {
            Ok(res) => res,
            Err(e) => return Err(ParseError { message: e.to_string() }),
        };

        let tree = Node::build_huffman_tree();
        let mut base_id: String = String::default();
        for _i in 0..4 {
            let mut base: String = String::default();
            loop {
                let base_raw = reader.read_bit()?;
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

        header.socketed_count = reader.read_bits(if header.compact { 1 } else { 3 })? as u8;
        warn!(
            "Header done, Total length: {0}",
            reader.position.total_bits() - starting_index.total_bits()
        );

        warn!("{0:?}", header);

        Ok(header)
    }
}

impl Items {
    pub fn parse(bytes: &[u8], expansion: bool, hired: bool) -> Items {
        let mut items = Items::default();
        let mut reader: ByteIO = ByteIO::new(bytes);
        if reader.data[0..2] != HEADER {
            warn!(
                "Found invalid header in items section: Expected {0:X?}, found {1:X?}. Returning empty items list.",
                HEADER,
                &reader.data[0..2]
            );
            return items;
        } else {
            debug!("Found correct header: {0:X?}", &reader.data[0..2]);
        }
        items.count = u16_from(&reader.data[2..4], "Items Count");
        debug!("Found item count: {0}", items.count);

        reader.position.current_byte = 4;

        for i in 0..items.count {
            let item = Item::parse(&mut reader);

            match item {
                Ok(res) => {
                    items.items.push(res);
                    warn!("Parsed item #{0}", i)
                }
                Err(res) => warn!("Skipping item, because of error: {0}", res.to_string()),
            }
        }

        items.corpse = match Corpse::parse(&mut reader) {
            Ok(res) => res,
            Err(e) => {
                warn!("Corpse parsing failed:  {0}", e.to_string());
                Corpse::default()
            }
        };

        items.mercenary = match MercenaryItems::parse(&mut reader, expansion, hired) {
            Ok(res) => res,
            Err(e) => {
                warn!("Mercenary items parsing failed: {0}", e.to_string());
                MercenaryItems::default()
            }
        };

        items.iron_golem = match IronGolem::parse(&mut reader) {
            Ok(res) => res,
            Err(e) => {
                warn!("Iron Golem parsing failed: {0}", e.to_string());
                IronGolem::default()
            }
        };

        //warn!("{0:?}", items);
        items
    }
}
