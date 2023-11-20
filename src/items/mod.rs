use huffman::{encode_char, Node};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use bit::BitIndex;

mod huffman;
mod tests;

use crate::bit_manipulation::ByteIO;
use crate::csv::{get_row, read_csv, Record};
use crate::{convert::u16_from, CustomError, D2SError, FileCutOffError, WrongHeaderError};
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

impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({0}) {1}: {2}", self.key, self.name, self.value)
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ItemMod {
    pub base: Mod,
    pub linked_mods: Vec<Mod>,
    pub param: Option<u32>,
}

impl fmt::Display for ItemMod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let param = match self.param {
            Some(i) => i.to_string(),
            None => String::default(),
        };
        let mut linked_mods = String::default();
        for m in &self.linked_mods {
            linked_mods.push_str(&format!("{0}\t", m));
        }
        write!(f, "{0} [{1} {2}]", self.base, linked_mods, param)
    }
}
impl ItemMod {
    // TODO: decompose param into useful parts
    pub fn to_bytes(mods: &Vec<ItemMod>) -> ByteIO {
        let mut mod_list = ByteIO::default();
        for m in mods {
            let mod_row: &HashMap<String, String> = &itemstatcost()[m.base.key as usize];
            let key_bits: usize = mod_row["Save Bits"].parse().unwrap();
            let save_add: i32 = match mod_row["Save Add"].as_str() {
                "" => 0i32,
                res => res.parse().unwrap(),
            };

            mod_list.write_bits(m.base.key, 9);
            if let Some(p) = m.param {
                let param_bits = match mod_row["Save Param Bits"].as_str() {
                    "" => 0,
                    any => any.parse().unwrap(),
                };
                if param_bits > 0 {
                    mod_list.write_bits(p, param_bits);
                }
            }
            mod_list.write_bits((m.base.value + save_add) as u32, key_bits);
            for linked_mod in &m.linked_mods {
                let linked_key_bits: usize =
                    itemstatcost()[linked_mod.key as usize]["Save Bits"].parse().unwrap();
                // Should linked mod also use save add?
                mod_list.write_bits(linked_mod.value as u32, linked_key_bits);
            }
        }
        mod_list.write_bits(MOD_TRAILER, 9);
        mod_list
    }
    pub fn parse(reader: &mut ByteIO) -> Result<Vec<ItemMod>, D2SError> {
        let mut mods = Vec::<ItemMod>::new();
        loop {
            let key_id = reader.read_bits(9)? as usize;
            if key_id as u16 == MOD_TRAILER {
                break;
            }
            let mut new_mod: ItemMod = ItemMod::default();
            if key_id > itemstatcost().len() {
                return Err(D2SError::Custom(CustomError {
                    message: format!("Key {0} does not exist in itemstatcost.", key_id),
                }));
            }
            let mod_row: &HashMap<String, String> = &itemstatcost()[key_id];
            let key_bits: usize = mod_row["Save Bits"].parse()?;
            let save_add: i32 = match mod_row["Save Add"].as_str() {
                "" => 0i32,
                res => res.parse()?,
            };
            new_mod.base.key = key_id as u16;
            let param_bits = mod_row["Save Param Bits"].clone();
            if param_bits != "" {
                new_mod.param = Some(reader.read_bits(param_bits.parse().unwrap()).unwrap());
            }
            let value = reader.read_bits(key_bits).unwrap();
            new_mod.base.value = value as i32 - save_add;
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

            mods.push(new_mod);
        }
        Ok(mods)
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
    Normal = 2, // TODO: add value in case of normal charms (12 bits)
    Superior(u8) = 3,
    Magic = 4,
    Set(u16) = 5,
    Rare(ItemName) = 6,
    Unique(u16) = 7,
    Crafted(ItemName) = 8,
}

impl From<&Quality> for u8 {
    fn from(value: &Quality) -> u8 {
        match value {
            Quality::Inferior(_) => 1,
            Quality::Normal => 2,
            Quality::Superior(_) => 3,
            Quality::Magic => 4,
            Quality::Set(_) => 5,
            Quality::Rare(_) => 6,
            Quality::Unique(_) => 7,
            Quality::Crafted(_) => 8,
        }
    }
}

impl TryFrom<u8> for Quality {
    type Error = D2SError;
    fn try_from(value: u8) -> Result<Self, D2SError> {
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
                return Err(D2SError::Custom(CustomError {
                    message: format!(
                        "Failed to convert quality value {0} to quality enum (values 1-8 valid).",
                        value
                    ),
                }))
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
    pub fn to_bytes(&self) -> ByteIO {
        let mut writer = ByteIO::new(&HEADER.to_vec(), true);
        writer.write_bits(self.exists as u16, 16);
        writer.write_bits(0u32, 32); // Unknown
        writer.write_bits(self.x, 32);
        println!("Writer so far: {0:?}", &writer.data);
        writer.write_bits(self.y, 32);
        writer.write_bits(HEADER[0], 8);
        writer.write_bits(HEADER[1], 8);
        writer.write_bits(self.item_count, 16);
        for i in 0..self.item_count {
            writer.concat(&self.items[i as usize].to_bytes());
        }
        writer
    }
    pub fn parse(reader: &mut ByteIO) -> Result<Corpse, D2SError> {
        reader.align_position();
        let reader_start = reader.clone();
        let mut corpse = Corpse::default();
        let mut header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(D2SError::WrongHeader(WrongHeaderError {
                section: String::from("Corpse"),
                reader: reader.clone(),
                expected: HEADER.to_vec(),
                actual: header.to_vec(),
            }));
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
            return Err(D2SError::WrongHeader(WrongHeaderError {
                section: String::from("Corpse"),
                reader: reader.clone(),
                expected: HEADER.to_vec(),
                actual: header.to_vec(),
            }));
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
        warn!(
            "Corpse data: {0:?} (is: {1:?}))",
            &reader.data[reader_start.position.current_byte..reader.position.current_byte + 1],
            corpse
        );
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
    fn parse(reader: &mut ByteIO, expansion: bool, hired: bool) -> Result<Self, D2SError> {
        reader.align_position();

        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != MERCENARY_HEADER {
            return Err(D2SError::WrongHeader(WrongHeaderError {
                section: String::from("Items Mercenary"),
                reader: reader.clone(),
                expected: MERCENARY_HEADER.to_vec(),
                actual: header.to_vec(),
            }));
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

    fn parse_expansion(reader: &mut ByteIO) -> Result<Self, D2SError> {
        let mut merc = MercenaryItems::default();

        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(D2SError::WrongHeader(WrongHeaderError {
                section: String::from("Items Mercenary Expansion"),
                reader: reader.clone(),
                expected: HEADER.to_vec(),
                actual: header.to_vec(),
            }));
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
    fn parse_classic(reader: &mut ByteIO) -> Result<Self, D2SError> {
        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != HEADER {
            return Err(D2SError::WrongHeader(WrongHeaderError {
                section: String::from("Items Mercenary Classic"),
                reader: reader.clone(),
                expected: MERCENARY_HEADER.to_vec(),
                actual: header.to_vec(),
            }));
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
    fn parse(reader: &mut ByteIO) -> Result<IronGolem, D2SError> {
        reader.align_position();

        let header = (reader.read_bits(16)? as u16).to_le_bytes();
        if header != IRON_GOLEM_HEADER {
            return Err(D2SError::WrongHeader(WrongHeaderError {
                section: String::from("Items Iron Golem"),
                reader: reader.clone(),
                expected: IRON_GOLEM_HEADER.to_vec(),
                actual: header.to_vec(),
            }));
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
    pub header: Header,
    pub data: Option<ExtendedItem>,
    pub socketed_items: Vec<Item>,
}

impl Item {
    pub fn parse(reader: &mut ByteIO) -> Result<Item, D2SError> {
        let mut item = Item::default();
        item.header = Header::parse(reader)?;
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

    pub fn to_bytes(&self) -> ByteIO {
        let mut writer = self.header.to_bytes();
        if let Some(extended_item) = &self.data {
            writer.concat_unaligned(&extended_item.to_bytes(&self.header));
        }
        if self.header.socketed && self.header.socketed_count > 0 {
            for item in &self.socketed_items {
                writer.data.extend_from_slice(&item.to_bytes().data); // align
            }
        }
        writer
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
    pub personalized_name: Option<String>,
    pub runeword_id: Option<u16>,
    pub realm_data: Option<Vec<u8>>,
    pub defense: Option<u16>,
    pub durability_max: u8,
    pub durability_current: Option<u16>,
    pub quantity: Option<u16>,
    pub total_sockets: Option<u8>,
    pub mods: Vec<Vec<ItemMod>>,
    pub set_item_mask: u8,
}

impl ExtendedItem {
    fn parse_rare_crafted(&mut self, reader: &mut ByteIO) -> Result<ItemName, D2SError> {
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

    fn to_bytes(&self, header: &Header) -> ByteIO {
        let mut extended = ByteIO::default();
        extended.write_bits(self.id, 32);
        extended.write_bits(self.level, 7);
        extended.write_bits(u8::from(&self.quality), 4);

        if let Some(gfx) = self.custom_graphics_id {
            extended.write_bit(true);
            extended.write_bits(gfx, 3);
        } else {
            extended.write_bit(false);
        }

        if let Some(auto_mod) = self.auto_mod {
            extended.write_bit(true);
            extended.write_bits(auto_mod, 11);
        } else {
            extended.write_bit(false);
        }

        match &self.quality {
            Quality::Inferior(inferior_state) => extended.write_bits(*inferior_state as u8, 3),
            Quality::Normal => (),
            Quality::Superior(res) => extended.write_bits(*res, 3),
            Quality::Magic => {
                extended.write_bits(self.prefixes[0], 11);
                extended.write_bits(self.suffixes[0], 11);
            }
            Quality::Set(set_id) => extended.write_bits(*set_id, 12),
            Quality::Rare(res) | Quality::Crafted(res) => {
                extended.write_bits(res.prefix, 8);
                extended.write_bits(res.suffix, 8);
                for i in 0..3usize {
                    if self.prefixes[i] != 0 {
                        extended.write_bit(true);
                        extended.write_bits(self.prefixes[i], 11);
                    } else {
                        extended.write_bit(false);
                    }

                    if self.suffixes[i] != 0 {
                        extended.write_bit(true);
                        extended.write_bits(self.suffixes[i], 11);
                    } else {
                        extended.write_bit(false);
                    }
                }
            }
            Quality::Unique(unique_id) => extended.write_bits(*unique_id, 12),
        }

        // TODO: Handle ear
        let mut item_lists = 0u8;
        if let Some(runeword_id) = self.runeword_id {
            extended.write_bits(runeword_id, 12);
            item_lists = 1 << 6;
            extended.write_bits(5u8, 4);
        }

        if let Some(name) = &self.personalized_name {
            for c in name.chars() {
                extended.write_bits(c as u8, 8);
            }
            extended.write_bits(0u8, 8);
        }

        if header.base == ID_BOOK || header.base == TP_BOOK {
            extended.write_bits(self.suffixes[0], 5);
        }

        if let Some(realm_data) = &self.realm_data {
            extended.write_bit(true);
            for byte in realm_data {
                extended.write_bits(*byte, 8);
            }
        } else {
            extended.write_bit(false);
        }

        let (item_type, _item_csv_row) = ItemType::get(&header.base);

        if item_type == ItemType::Armor {
            let def_row = get_row(&itemstatcost(), "Stat", "armorclass").unwrap();

            let def_real = match self.defense {
                Some(d) => d,
                None => 0,
            };

            let def_saved: u16 = def_real + def_row["Save Add"].parse::<u16>().unwrap();
            let def_bits = def_row["Save Bits"].parse().unwrap();

            extended.write_bits(def_saved, def_bits);
        }

        if item_type == ItemType::Armor || item_type == ItemType::Weapon {
            let max_durability_row = get_row(&itemstatcost(), "Stat", "maxdurability").unwrap();
            let max_durability_bits = max_durability_row["Save Bits"].parse().unwrap();
            let max_durability: u8 =
                self.durability_max + max_durability_row["Save Add"].parse::<u8>().unwrap();
            extended.write_bits(max_durability, max_durability_bits);

            if let Some(durability_value) = self.durability_current {
                let durability_row = get_row(&itemstatcost(), "Stat", "durability").unwrap();
                let durability: u16 =
                    durability_value + durability_row["Save Add"].parse::<u16>().unwrap();
                let durability_bits = durability_row["Save Bits"].parse().unwrap();
                extended.write_bits(durability, durability_bits);
            }
        }

        if let Some(qty) = &self.quantity {
            extended.write_bits(*qty, 9);
        }

        if let Some(sockets_num) = self.total_sockets {
            extended.write_bits(sockets_num, 4);
        }

        // Credit to https://github.com/dschu012/D2SLib/ for figuring out all the item_list masking bits
        if let Quality::Set(_) = self.quality {
            item_lists = item_lists | self.set_item_mask;
            extended.write_bits(self.set_item_mask, 5);
        }
        let mod_bytes = ItemMod::to_bytes(&self.mods[0]);
        extended.concat_unaligned(&mod_bytes);
        for i in 0..8usize {
            if item_lists.bit(i) {
                extended.concat_unaligned(&ItemMod::to_bytes(&self.mods[i + 1]));
            }
            let mut total_so_far = header.clone().to_bytes();
            total_so_far.concat_unaligned(&extended);
        }
        let mut actual_total_so_far = header.clone().to_bytes();
        actual_total_so_far.concat_unaligned(&extended);
        extended
    }

    pub fn parse(header: &Header, reader: &mut ByteIO) -> Result<Self, D2SError> {
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
                    Err(e) => return Err(D2SError::Custom(CustomError { message: e.to_string() })),
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

        let mut item_lists = 0u8;
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
            extended_item.personalized_name = Some(name);
        }

        if header.base == ID_BOOK || header.base == TP_BOOK {
            extended_item.suffixes[0] = reader.read_bits(5)? as u16;
        }

        let realm_data_present = reader.read_bit()?;
        if realm_data_present {
            let mut realm_data = Vec::<u8>::new();
            for _i in 0..16usize {
                realm_data.push(reader.read_bits(8)? as u8);
            }
            extended_item.realm_data = Some(realm_data);
        }

        let (item_type, item_row) = ItemType::get(&header.base);

        if item_type == ItemType::Armor {
            let mut defense = reader.read_bits(11)? as u16;
            // should defense be signed or unsigned? Signed in itemstatcost.txt is meaningless (durability is also 1 for signed and yet
            // can go up to 255 in game), and yet presence of Save Add would suggest -10 is possible
            defense -= match get_row(&itemstatcost(), "Stat", "armorclass") {
                Some(res) => res["Save Add"].parse().unwrap(),
                None => 0,
            };
            extended_item.defense = Some(cmp::max(defense, 0));
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
            extended_item.set_item_mask = reader.read_bits(5)? as u8;
            item_lists = item_lists | extended_item.set_item_mask;
        }
        extended_item.mods.push(ItemMod::parse(reader)?);
        for i in 0..8usize {
            if item_lists.bit(i) {
                extended_item.mods.push(ItemMod::parse(reader)?);
            }
        }

        Ok(extended_item)
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Header {
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
    pub picked_up_since_last_save: bool,
}

impl Header {
    pub fn parse(reader: &mut ByteIO) -> Result<Header, D2SError> {
        reader.align_position();
        let mut header: Header = Header::default();

        reader.read_bits(4)?; // unknown
        header.identified = reader.read_bit()?;
        reader.read_bits(3)?; // unknown
        header.broken = reader.read_bit()?;
        reader.read_bits(2)?; // unknown
        header.socketed = reader.read_bit()?;
        reader.read_bit()?; // unknown
        header.picked_up_since_last_save = reader.read_bit()?;
        reader.read_bits(2)?; // unknown
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
            Err(e) => return Err(D2SError::Custom(CustomError { message: e.to_string() })),
        };
        // TODO: Handle ground/dropped cases? https://github.com/ThePhrozenKeep/D2MOO/blob/4071d3f4c3cec4a7bb4319b8fe4ff157834fb217/source/D2Common/src/Items/Items.cpp#L5029
        header.slot = match Slot::try_from(reader.read_bits(4)? as u8) {
            Ok(res) => res,
            Err(e) => return Err(D2SError::Custom(CustomError { message: e.to_string() })),
        };

        header.column = reader.read_bits(4)? as u8;
        header.row = reader.read_bits(4)? as u8;

        let raw_storage = reader.read_bits(3)?;

        header.storage = match Storage::try_from(raw_storage as u8) {
            Ok(res) => res,
            Err(e) => return Err(D2SError::Custom(CustomError { message: e.to_string() })),
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

        Ok(header)
    }

    fn to_bytes(&self) -> ByteIO {
        let mut header = ByteIO::default();
        header.write_bits(0u8, 4); // unknown
        header.write_bit(self.identified);
        header.write_bits(0u8, 3); // unknown
        header.write_bit(self.broken);
        header.write_bits(0u8, 2); // unknown
        header.write_bit(self.socketed);
        header.write_bit(self.socketed); // unknown
        header.write_bit(self.picked_up_since_last_save);
        header.write_bits(0u8, 2); // unknown (bit 2)
        header.write_bit(self.ear);
        header.write_bit(self.starter_gear);
        header.write_bits(0u8, 3); // unknown
        header.write_bit(self.compact);
        header.write_bit(self.ethereal);
        header.write_bit(true); //unknown
        header.write_bit(self.personalized);
        header.write_bit(false); //unknown
        header.write_bit(self.runeword); //unknown
        header.write_bits(0u8, 8); //unknown
        header.write_bits(self.status as u8, 3);
        header.write_bits(self.slot as u8, 4);
        header.write_bits(self.column, 4);
        header.write_bits(self.row, 4);
        header.write_bits(self.storage as u8, 3);

        for c in self.base.chars() {
            for str_bit in encode_char(c).chars() {
                header.write_bit(if str_bit == '1' { true } else { false });
            }
        }
        header.write_bits(self.socketed_count, if self.compact { 1 } else { 3 });

        header
    }
}

impl Items {
    pub fn to_bytes(&self, expansion: bool, hired: bool) -> ByteIO {
        let mut writer = ByteIO::new(&HEADER, true);
        writer.write_bits(self.count, 16);
        for i in 0..self.count {
            writer.concat(&self.items[i as usize].to_bytes());
        }

        writer
    }

    pub fn parse(bytes: &[u8], expansion: bool, hired: bool) -> Items {
        let mut items = Items::default();
        let mut reader: ByteIO = ByteIO::new(bytes, false);
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
            let pos = reader.position.clone();
            warn!("{0:?}", pos);
            let item = Item::parse(&mut reader);
            warn!("{0:?}", item);
            warn!("{0:?}", reader.position);

            if pos.current_bit == 8 {
                warn!(
                    "{0:?}",
                    &reader.data[pos.current_byte + 1..reader.position.current_byte + 1]
                );
            }

            match item {
                Ok(res) => {
                    items.items.push(res);
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
