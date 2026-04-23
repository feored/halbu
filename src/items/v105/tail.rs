//! v105 items-tail framing parser (`docs/v105-item-format.md` §3, §11).

use crate::utils::{u16_from, u8_from, BytePosition};
use crate::{ExpansionType, ParseHardError};

use super::item::parse_item;
use super::model::{CorpseEntry, Item, ItemList, ItemListKind, ItemsTail};

const MAGIC_JM: [u8; 2] = [0x4A, 0x4D];
const MAGIC_JF: [u8; 2] = [0x6A, 0x66];
const MAGIC_KF: [u8; 2] = [0x6B, 0x66];
const MAGIC_LF: [u8; 2] = [0x6C, 0x66];

/// Parse the entire items-tail payload starting at byte 0 of `bytes`.
pub fn parse_items_tail(
    bytes: &[u8],
    expansion_type: ExpansionType,
    mercenary_hired: bool,
) -> Result<ItemsTail, ParseHardError> {
    let mut byte_off: usize = 0;

    // Player items.
    let player_items = parse_jm_list(bytes, &mut byte_off, "player")?;
    let player = ItemList { kind: ItemListKind::Player, items: player_items };

    // Classic saves end here.
    if matches!(expansion_type, ExpansionType::Classic) {
        return Ok(ItemsTail {
            player,
            corpses: Vec::new(),
            mercenary: None,
            mercenary_header_present: false,
            golem: None,
            rotw_lf_trailer: None,
        });
    }

    // Corpse-items list (§11.2).
    expect_magic(bytes, &mut byte_off, MAGIC_JM, "corpse")?;
    let corpse_count = read_u16(bytes, &mut byte_off, "corpse_count")? as usize;
    let mut corpses: Vec<CorpseEntry> = Vec::with_capacity(corpse_count);
    for i in 0..corpse_count {
        let unknown = read_u32(bytes, &mut byte_off, "corpse.unknown")?;
        let x = read_u32(bytes, &mut byte_off, "corpse.x")?;
        let y = read_u32(bytes, &mut byte_off, "corpse.y")?;
        expect_magic(bytes, &mut byte_off, MAGIC_JM, &format!("corpse[{i}].items"))?;
        let items = parse_items_inner(bytes, &mut byte_off, &format!("corpse[{i}].items"))?;
        corpses.push(CorpseEntry { unknown, x, y, items });
    }

    // Mercenary section: "jf" magic always present in expansion. Inner JM list
    // only present if the mercenary is hired.
    expect_magic(bytes, &mut byte_off, MAGIC_JF, "mercenary")?;
    let mercenary_header_present = true;
    let mercenary = if mercenary_hired {
        let items = parse_jm_list(bytes, &mut byte_off, "mercenary")?;
        Some(ItemList { kind: ItemListKind::MercEquipped, items })
    } else {
        None
    };

    // Iron Golem section.
    expect_magic(bytes, &mut byte_off, MAGIC_KF, "golem")?;
    let has_golem = read_u8(bytes, &mut byte_off, "has_golem")?;
    let golem = if has_golem != 0 {
        let mut cursor = BytePosition { current_byte: byte_off, current_bit: 0 };
        let item = parse_item(bytes, &mut cursor)?;
        byte_off = cursor.next_byte_offset();
        Some(item)
    } else {
        None
    };

    // RotW `lf` trailer.
    let rotw_lf_trailer = if matches!(expansion_type, ExpansionType::RotW) {
        let rest = bytes.get(byte_off..).unwrap_or(&[]).to_vec();
        // Best-effort sanity check: `01 00 6C 66 ...`
        if rest.len() >= 4 && (rest[0] != 0x01 || rest[1] != 0x00 || rest[2..4] != MAGIC_LF) {
            // Preserve regardless; a stricter signal can be added later.
        }
        Some(rest)
    } else {
        None
    };

    Ok(ItemsTail { player, corpses, mercenary, mercenary_header_present, golem, rotw_lf_trailer })
}

fn parse_jm_list(
    bytes: &[u8],
    byte_off: &mut usize,
    section: &str,
) -> Result<Vec<Item>, ParseHardError> {
    expect_magic(bytes, byte_off, MAGIC_JM, section)?;
    parse_items_inner(bytes, byte_off, section)
}

fn parse_items_inner(
    bytes: &[u8],
    byte_off: &mut usize,
    section: &str,
) -> Result<Vec<Item>, ParseHardError> {
    let count = read_u16(bytes, byte_off, &format!("{section}.count"))? as usize;
    let mut out: Vec<Item> = Vec::with_capacity(count);
    let mut cursor = BytePosition { current_byte: *byte_off, current_bit: 0 };
    for i in 0..count {
        let item = parse_item(bytes, &mut cursor).map_err(|e| ParseHardError {
            message: format!("{section}: failed parsing item {i}: {}", e.message),
        })?;
        out.push(item);
    }
    *byte_off = cursor.next_byte_offset();
    Ok(out)
}

fn expect_magic(
    bytes: &[u8],
    byte_off: &mut usize,
    expected: [u8; 2],
    section: &str,
) -> Result<(), ParseHardError> {
    let slice = bytes.get(*byte_off..*byte_off + 2).ok_or_else(|| ParseHardError {
        message: format!(
            "Truncated items tail: expected `{section}` magic at offset {} but only {} bytes remain.",
            *byte_off,
            bytes.len().saturating_sub(*byte_off)
        ),
    })?;
    if slice != expected {
        return Err(ParseHardError {
            message: format!(
                "Invalid `{section}` magic at offset {}: expected {:02X?}, found {:02X?}.",
                *byte_off, expected, slice
            ),
        });
    }
    *byte_off += 2;
    Ok(())
}

fn read_u8(bytes: &[u8], byte_off: &mut usize, name: &str) -> Result<u8, ParseHardError> {
    let slice = bytes.get(*byte_off..*byte_off + 1).ok_or_else(|| ParseHardError {
        message: format!("Truncated items tail: cannot read u8 `{name}` at offset {}.", *byte_off),
    })?;
    let v = u8_from(slice, "items_tail_u8")?;
    *byte_off += 1;
    let _ = name;
    Ok(v)
}

fn read_u16(bytes: &[u8], byte_off: &mut usize, name: &str) -> Result<u16, ParseHardError> {
    let slice = bytes.get(*byte_off..*byte_off + 2).ok_or_else(|| ParseHardError {
        message: format!("Truncated items tail: cannot read u16 `{name}` at offset {}.", *byte_off),
    })?;
    let v = u16_from(slice, "items_tail_u16")?;
    *byte_off += 2;
    let _ = name;
    Ok(v)
}

fn read_u32(bytes: &[u8], byte_off: &mut usize, name: &str) -> Result<u32, ParseHardError> {
    let slice = bytes.get(*byte_off..*byte_off + 4).ok_or_else(|| ParseHardError {
        message: format!("Truncated items tail: cannot read u32 `{name}` at offset {}.", *byte_off),
    })?;
    let v = crate::utils::u32_from(slice, "items_tail_u32")?;
    *byte_off += 4;
    let _ = name;
    Ok(v)
}
