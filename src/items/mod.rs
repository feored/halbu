use crate::{convert::u16_from, D2SError, WrongHeaderError};
use item::*;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use crate::bit_manipulation::ByteIO;
use bit::BitIndex;
use log::{debug, warn};

mod huffman;
mod tests;

pub mod item;

const HEADER: [u8; 2] = [0x4A, 0x4D];
const MERCENARY_HEADER: [u8; 2] = [0x6A, 0x66];
const IRON_GOLEM_HEADER: [u8; 2] = [0x6B, 0x66];

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Items {
    pub count: u16,
    pub items: Vec<Item>,
    pub corpse: Corpse,
    pub mercenary: MercenaryItems,
    pub iron_golem: IronGolem,
}

impl Display for Items {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str_items = String::new();
        for i in 0..self.count {
            str_items.push_str(&format!("Item #{0}\n{1}\n", i, self.items[i as usize]));
        }
        write!(f, "{0} items\nItems:\n{1}", self.count, str_items)
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Corpse {
    pub exists: bool,
    pub unknown_0: u32,
    pub x: u32,
    pub y: u32,
    pub item_count: u16,
    pub items: Vec<Item>,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct MercenaryItems {
    pub items_count: u16,
    pub items: Vec<Item>,
    pub id: u32, // TODO CHECK ACTUAL LENGTH
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct IronGolem {
    pub item: Option<Item>,
}

impl Corpse {
    pub fn to_bytes(&self) -> ByteIO {
        let mut writer = ByteIO::default();
        writer.write_bytes(&HEADER);
        writer.write_bits(self.exists as u16, 16);
        if !self.exists {
            return writer;
        }
        writer.write_bits(self.unknown_0, 32); // Unknown
        writer.write_bits(self.x, 32);
        writer.write_bits(self.y, 32);
        writer.write_bits(HEADER[0], 8);
        writer.write_bits(HEADER[1], 8);
        writer.write_bits(self.item_count, 16);
        for i in 0..self.item_count {
            writer.concat_aligned(&self.items[i as usize].to_bytes());
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
        corpse.unknown_0 = reader.read_bits(32)?; // Unknown
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
                    corpse.items.push(res);
                }
                Err(res) => warn!("Skipping item, because of error: {0}", res.to_string()),
            }
        }
        // warn!(
        //     "Corpse data: {0:?} (is: {1:?}))",
        //     &reader.data[reader_start.position.current_byte..reader.position.current_byte + 1],
        //     corpse
        // );
        Ok(corpse)
    }
}

impl MercenaryItems {
    fn to_bytes(&self, expansion: bool, hired: bool) -> ByteIO {
        let mut writer = ByteIO::default();
        writer.write_bytes(&MERCENARY_HEADER);
        if !hired {
            return writer;
        }

        if !expansion {
            writer.write_bytes(&HEADER);
            return writer;
        }

        writer.write_bits(self.items_count, 16);
        for i in 0..self.items_count {
            writer.concat_aligned(&self.items[i as usize].to_bytes());
        }

        writer
    }

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
                    //warn!("Parsed item #{0}", i)
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

    fn to_bytes(&self) -> ByteIO {
        let mut writer = ByteIO::default();
        writer.write_bytes(&IRON_GOLEM_HEADER);
        if let Some(item) = &self.item {
            writer.write_byte(1u8);
            writer.concat_aligned(&item.to_bytes());
        } else {
            writer.write_byte(0u8);
        }

        writer
    }
}

impl Items {
    pub fn to_bytes(&self, expansion: bool, hired: bool) -> ByteIO {
        let mut writer = ByteIO::new(&HEADER, true);
        writer.write_bits(self.count, 16);
        for i in 0..self.count {
            writer.concat_aligned(&self.items[i as usize].to_bytes());
        }
        writer.concat_aligned(&self.corpse.to_bytes());
        writer.concat_aligned(&self.mercenary.to_bytes(expansion, hired));
        writer.concat_aligned(&self.iron_golem.to_bytes());

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
        }
        items.count = u16_from(&reader.data[2..4], "Items Count");

        reader.position.current_byte = 4;

        for _i in 0..items.count {
            let pos = reader.position.clone();
            let item = Item::parse(&mut reader);

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

        if !expansion {
            return items;
        }

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
