//! v105 single-item parser (`docs/v105-item-format.md` §4–§10).
//!
//! Recursive entry point: [`parse_item`]. Reads one complete item from the
//! current bit cursor (53-bit header → ear branch | type-code + extended +
//! property lists → v105 quantity trailer → byte-align → recursive socketed
//! children). Returns `Ok(Item)` and leaves the cursor positioned at the
//! start of the next byte-aligned item (or list end).

use crate::excel::{itemtypes, lookup_base, BaseRef};
use crate::utils::{read_bits, BytePosition};
use crate::ParseHardError;

use super::huffman::decode_type_code;
use super::model::{
    AltContainer, BitTail, EarData, EquippedSlot, Item, ItemExtended, ItemFlags, ItemHeader,
    ItemKind, ItemLocation, ItemPropertyList, ItemQuality, ItemQualityData, LocationId,
    RunewordData, StandardItem,
};
use super::properties::parse_property_list;

const MAX_RECURSION_DEPTH: u8 = 2;

/// Parse one item from the current bit cursor.
pub fn parse_item(bytes: &[u8], cursor: &mut BytePosition) -> Result<Item, ParseHardError> {
    parse_item_inner(bytes, cursor, 0)
}

fn parse_item_inner(
    bytes: &[u8],
    cursor: &mut BytePosition,
    depth: u8,
) -> Result<Item, ParseHardError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(ParseHardError {
            message: format!("Item parser recursion depth {depth} exceeds maximum."),
        });
    }

    let header = parse_header(bytes, cursor)?;
    let flags = header.flags;

    let kind = if flags.is_ear() {
        ItemKind::Ear(parse_ear(bytes, cursor)?)
    } else {
        ItemKind::Standard(parse_standard(bytes, cursor, &flags)?)
    };

    // §7.7 v105 quantity trailer — applies to all items, simple and extended,
    // including ears (per the PR placement outside the `if (!simple_item)` block).
    let has_qty = read_bits(bytes, cursor, 1)? != 0;
    let rotw_quantity = if has_qty { Some(read_bits(bytes, cursor, 8)? as u8) } else { None };

    // Byte-align; capture pad bits for round-trip insurance.
    let bit_tail = byte_align(bytes, cursor)?;

    // Recursively parse socketed children. Only standard, non-simple items
    // can have inserted children (§10).
    let mut socketed_items = Vec::new();
    let sockets_filled = match &kind {
        ItemKind::Standard(s) if !flags.simple_item() => s.sockets_filled,
        _ => 0,
    };
    for _ in 0..sockets_filled {
        socketed_items.push(parse_item_inner(bytes, cursor, depth + 1)?);
    }

    Ok(Item { header, kind, rotw_quantity, bit_tail, socketed_items })
}

fn parse_header(bytes: &[u8], cursor: &mut BytePosition) -> Result<ItemHeader, ParseHardError> {
    let raw_flags = read_bits(bytes, cursor, 32)?;
    let item_version = read_bits(bytes, cursor, 3)? as u8;
    let location_raw = read_bits(bytes, cursor, 3)? as u8;
    let equipped_raw = read_bits(bytes, cursor, 4)? as u8;
    let position_x = read_bits(bytes, cursor, 4)? as u8;
    let position_y = read_bits(bytes, cursor, 4)? as u8;
    let alt_raw = read_bits(bytes, cursor, 3)? as u8;

    Ok(ItemHeader {
        flags: ItemFlags { raw: raw_flags },
        item_version,
        location: ItemLocation {
            location_id: LocationId::from_raw(location_raw),
            equipped_id: EquippedSlot::from_raw(equipped_raw),
            position_x,
            position_y,
            alt_position_id: AltContainer::from_raw(alt_raw),
        },
    })
}

fn parse_ear(bytes: &[u8], cursor: &mut BytePosition) -> Result<EarData, ParseHardError> {
    let class_id = read_bits(bytes, cursor, 3)? as u8;
    let level = read_bits(bytes, cursor, 7)? as u8;
    let mut name_bytes = Vec::with_capacity(15);
    loop {
        let ch = read_bits(bytes, cursor, 7)? as u8;
        if ch == 0 {
            break;
        }
        name_bytes.push(ch);
        if name_bytes.len() > 15 {
            return Err(ParseHardError {
                message: "Ear name exceeded maximum length (15 chars)".to_string(),
            });
        }
    }
    let name = String::from_utf8_lossy(&name_bytes).into_owned();
    Ok(EarData { class_id, level, name })
}

fn parse_standard(
    bytes: &[u8],
    cursor: &mut BytePosition,
    flags: &ItemFlags,
) -> Result<StandardItem, ParseHardError> {
    let type_code = decode_type_code(bytes, cursor)?;
    let code_str = std::str::from_utf8(&type_code).map_err(|_| ParseHardError {
        message: format!("Decoded item type code is not valid UTF-8: {:?}", type_code),
    })?;
    let trimmed = code_str.trim_end();

    let base = lookup_base(trimmed);
    let is_quest = is_quest_item(base);

    let (quest_difficulty, sockets_filled) = if is_quest {
        let diff = read_bits(bytes, cursor, 2)? as u8;
        let sock = read_bits(bytes, cursor, 1)? as u8;
        (Some(diff), sock)
    } else if flags.simple_item() {
        let sock = read_bits(bytes, cursor, 1)? as u8;
        (None, sock)
    } else {
        let sock = read_bits(bytes, cursor, 3)? as u8;
        (None, sock)
    };

    let extended = if flags.simple_item() {
        None
    } else {
        Some(parse_extended(bytes, cursor, flags, base, trimmed)?)
    };

    Ok(StandardItem { type_code, sockets_filled, quest_difficulty, extended })
}

fn is_quest_item(base: Option<BaseRef>) -> bool {
    let Some(base) = base else { return false };
    let (typ, type2) = match base {
        BaseRef::Armor(a) => (a.typ.as_str(), a.type2.as_deref()),
        BaseRef::Weapon(w) => (w.typ.as_str(), w.type2.as_deref()),
        BaseRef::Misc(m) => (m.typ.as_str(), m.type2.as_deref()),
    };
    if itemtypes::is_a(typ, "ques") {
        return true;
    }
    if let Some(t2) = type2 {
        if itemtypes::is_a(t2, "ques") {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Copy)]
struct BaseClassification {
    is_armor: bool,
    /// Reserved for task 007 (move API needs to know weapon-vs-armor for slot validation).
    #[allow(dead_code)]
    is_weapon: bool,
    has_durability: bool,
    is_stackable: bool,
}

fn classify(base: Option<BaseRef>) -> BaseClassification {
    match base {
        Some(BaseRef::Armor(a)) => BaseClassification {
            is_armor: true,
            is_weapon: false,
            has_durability: !a.no_durability,
            is_stackable: a.stackable,
        },
        Some(BaseRef::Weapon(w)) => BaseClassification {
            is_armor: false,
            is_weapon: true,
            has_durability: !w.no_durability,
            is_stackable: w.stackable,
        },
        Some(BaseRef::Misc(m)) => BaseClassification {
            is_armor: false,
            is_weapon: false,
            has_durability: false,
            is_stackable: m.stackable,
        },
        None => BaseClassification {
            is_armor: false,
            is_weapon: false,
            has_durability: false,
            is_stackable: false,
        },
    }
}

fn parse_extended(
    bytes: &[u8],
    cursor: &mut BytePosition,
    flags: &ItemFlags,
    base: Option<BaseRef>,
    code: &str,
) -> Result<ItemExtended, ParseHardError> {
    let id = read_bits(bytes, cursor, 32)?;
    let level = read_bits(bytes, cursor, 7)? as u8;
    let quality_raw = read_bits(bytes, cursor, 4)? as u8;
    let quality = ItemQuality::from_raw(quality_raw).ok_or_else(|| ParseHardError {
        message: format!("Unknown item quality value {quality_raw}."),
    })?;

    let multiple_pictures = read_bits(bytes, cursor, 1)? != 0;
    let picture_id =
        if multiple_pictures { Some(read_bits(bytes, cursor, 3)? as u8) } else { None };

    let class_specific = read_bits(bytes, cursor, 1)? != 0;
    let class_specific_auto_affix =
        if class_specific { Some(read_bits(bytes, cursor, 11)? as u16) } else { None };

    let quality_data = parse_quality_data(bytes, cursor, quality)?;

    let runeword = if flags.given_runeword() {
        Some(RunewordData {
            id: read_bits(bytes, cursor, 12)? as u16,
            padding: read_bits(bytes, cursor, 4)? as u8,
        })
    } else {
        None
    };

    let personalized_name =
        if flags.personalized() { Some(parse_personalized_name(bytes, cursor)?) } else { None };

    let tome_data = if code == "tbk" || code == "ibk" {
        Some(read_bits(bytes, cursor, 5)? as u8)
    } else {
        None
    };

    let realm_data_flag = read_bits(bytes, cursor, 1)? != 0;

    let class = classify(base);

    let defense_raw =
        if class.is_armor { Some(read_bits(bytes, cursor, 11)? as u16) } else { None };

    let (max_durability, current_durability) = if class.has_durability {
        let max = read_bits(bytes, cursor, 8)? as u8;
        let current = if max > 0 { Some(read_bits(bytes, cursor, 9)? as u16) } else { None };
        (Some(max), current)
    } else {
        (None, None)
    };

    // §6.3 — v105 unknown bit. PR places it after the durability block; doc
    // says "always read for any extended item, regardless of durability".
    // We read it always for extended items.
    let v105_unknown_after_durability = read_bits(bytes, cursor, 1)? != 0;

    let stack_quantity =
        if class.is_stackable { Some(read_bits(bytes, cursor, 9)? as u16) } else { None };

    let total_sockets =
        if flags.socketed() { Some(read_bits(bytes, cursor, 4)? as u8) } else { None };

    let set_bonus_mask =
        if quality == ItemQuality::Set { Some(read_bits(bytes, cursor, 5)? as u8) } else { None };

    let properties = parse_property_list(bytes, cursor)?;

    let mut set_bonus_property_lists: Vec<ItemPropertyList> = Vec::new();
    if let Some(mask) = set_bonus_mask {
        let extra_lists = (mask as u32).count_ones();
        for _ in 0..extra_lists {
            set_bonus_property_lists.push(parse_property_list(bytes, cursor)?);
        }
    }

    let runeword_property_list =
        if runeword.is_some() { Some(parse_property_list(bytes, cursor)?) } else { None };

    Ok(ItemExtended {
        id,
        level,
        quality,
        quality_data,
        picture_id,
        class_specific_auto_affix,
        runeword,
        personalized_name,
        tome_data,
        realm_data_flag,
        defense_raw,
        max_durability,
        current_durability,
        v105_unknown_after_durability,
        stack_quantity,
        total_sockets,
        set_bonus_mask,
        properties,
        set_bonus_property_lists,
        runeword_property_list,
    })
}

fn parse_quality_data(
    bytes: &[u8],
    cursor: &mut BytePosition,
    quality: ItemQuality,
) -> Result<ItemQualityData, ParseHardError> {
    Ok(match quality {
        ItemQuality::Normal => ItemQualityData::None,
        ItemQuality::LowQuality => {
            ItemQualityData::LowQuality { id: read_bits(bytes, cursor, 3)? as u8 }
        }
        ItemQuality::Superior => {
            ItemQualityData::Superior { file_index: read_bits(bytes, cursor, 3)? as u8 }
        }
        ItemQuality::Magic => {
            let prefix = read_bits(bytes, cursor, 11)? as u16;
            let suffix = read_bits(bytes, cursor, 11)? as u16;
            ItemQualityData::Magic { prefix, suffix }
        }
        ItemQuality::Set => ItemQualityData::Set { set_id: read_bits(bytes, cursor, 12)? as u16 },
        ItemQuality::Unique => {
            ItemQualityData::Unique { unique_id: read_bits(bytes, cursor, 12)? as u16 }
        }
        ItemQuality::Rare | ItemQuality::Crafted => {
            let name1 = read_bits(bytes, cursor, 8)? as u8;
            let name2 = read_bits(bytes, cursor, 8)? as u8;
            let mut affixes: [Option<u16>; 6] = [None; 6];
            for slot in affixes.iter_mut() {
                let present = read_bits(bytes, cursor, 1)? != 0;
                if present {
                    *slot = Some(read_bits(bytes, cursor, 11)? as u16);
                }
            }
            ItemQualityData::RareOrCrafted { name1, name2, affixes }
        }
    })
}

fn parse_personalized_name(
    bytes: &[u8],
    cursor: &mut BytePosition,
) -> Result<String, ParseHardError> {
    let mut name_bytes = Vec::with_capacity(15);
    loop {
        let ch = read_bits(bytes, cursor, 8)? as u8;
        if ch == 0 {
            break;
        }
        name_bytes.push(ch);
        if name_bytes.len() > 15 {
            return Err(ParseHardError {
                message: "Personalized name exceeded maximum length (15 chars)".to_string(),
            });
        }
    }
    Ok(String::from_utf8_lossy(&name_bytes).into_owned())
}

fn byte_align(bytes: &[u8], cursor: &mut BytePosition) -> Result<BitTail, ParseHardError> {
    if cursor.current_bit == 0 {
        return Ok(BitTail::default());
    }
    let pad_len = 8 - cursor.current_bit;
    let pad_bits = read_bits(bytes, cursor, pad_len)? as u8;
    Ok(BitTail { bits: pad_bits, bit_len: pad_len as u8 })
}
