//! v105 item property list parser (`docs/v105-item-format.md` §7).
//!
//! Each property list is a stream of `(stat_id [+param] +value(s))` records
//! terminated by the 9-bit sentinel `0x1FF`. Bit widths come from the
//! embedded `excel::itemstatcost` table; nothing is hardcoded.

use crate::excel::itemstatcost;
use crate::utils::{read_bits, BytePosition};
use crate::ParseHardError;

use super::model::{ItemProperty, ItemPropertyList, PropertyEncoding};

/// 9-bit sentinel value terminating each property list.
pub(crate) const PROPERTY_LIST_TERMINATOR: u16 = 0x1FF;

/// Multi-stat run lengths for `np > 1` lead stats (§7.2). Lead stat → `np`.
fn np_for(stat_id: u16) -> u8 {
    match stat_id {
        17 => 2, // item_maxdamage_percent (min/max %ED)
        48 => 2, // firemindam (min, max)
        50 => 2, // lightmindam (min, max)
        52 => 2, // magicmindam (min, max)
        54 => 3, // coldmindam (min, max, length)
        57 => 3, // poisonmindam (min, max, length)
        _ => 1,
    }
}

/// Parse one property list at the current bit cursor; returns when sentinel is read.
pub(crate) fn parse_property_list(
    bytes: &[u8],
    cursor: &mut BytePosition,
) -> Result<ItemPropertyList, ParseHardError> {
    let mut list = ItemPropertyList::default();
    loop {
        let stat_id = read_bits(bytes, cursor, 9)? as u16;
        if std::env::var("HALBU_DIAG_STATS").is_ok() {
            eprintln!("DIAG stat_id={stat_id}");
        }
        if stat_id == PROPERTY_LIST_TERMINATOR {
            return Ok(list);
        }
        let prop = parse_one_property(bytes, cursor, stat_id)?;
        list.properties.push(prop);
    }
}

fn parse_one_property(
    bytes: &[u8],
    cursor: &mut BytePosition,
    stat_id: u16,
) -> Result<ItemProperty, ParseHardError> {
    let row = itemstatcost::by_id(stat_id).ok_or_else(|| {
        if std::env::var("HALBU_DIAG_STATS").is_ok() {
            eprintln!("DIAG UNKNOWN stat_id={stat_id}");
        }
        ParseHardError {
            message: format!("Unknown item stat id {stat_id} encountered in property list."),
        }
    })?;

    match row.encode {
        2 => {
            // Chance-on-hit: 16-bit param (overrides save_param_bits if disagree),
            // 7-bit value.
            let param = read_bits(bytes, cursor, 16)?;
            let value_bits = if row.save_bits == 0 { 7 } else { row.save_bits as usize };
            let raw = read_bits(bytes, cursor, value_bits)? as i64;
            let value = raw - row.save_add as i64;
            Ok(ItemProperty {
                stat_id,
                param: Some(param),
                values: vec![value],
                encoding: PropertyEncoding::ChanceOnHit,
            })
        }
        3 => {
            // Charges: 16-bit param + 16-bit value.
            let param = read_bits(bytes, cursor, 16)?;
            let value_bits = if row.save_bits == 0 { 16 } else { row.save_bits as usize };
            let raw = read_bits(bytes, cursor, value_bits)? as i64;
            let value = raw - row.save_add as i64;
            Ok(ItemProperty {
                stat_id,
                param: Some(param),
                values: vec![value],
                encoding: PropertyEncoding::Charges,
            })
        }
        4 => {
            // Skill-tab packed param. Field widths are exactly as in excel.
            let param = if row.save_param_bits > 0 {
                Some(read_bits(bytes, cursor, row.save_param_bits as usize)?)
            } else {
                None
            };
            let raw = read_bits(bytes, cursor, row.save_bits as usize)? as i64;
            let value = raw - row.save_add as i64;
            Ok(ItemProperty {
                stat_id,
                param,
                values: vec![value],
                encoding: PropertyEncoding::SkillTab,
            })
        }
        _ => {
            // Standard or grouped read.
            let param = if row.save_param_bits > 0 {
                Some(read_bits(bytes, cursor, row.save_param_bits as usize)?)
            } else {
                None
            };
            let np = np_for(stat_id);
            if np <= 1 {
                let raw = read_bits(bytes, cursor, row.save_bits as usize)? as i64;
                let value = raw - row.save_add as i64;
                Ok(ItemProperty {
                    stat_id,
                    param,
                    values: vec![value],
                    encoding: PropertyEncoding::Standard,
                })
            } else {
                let mut values = Vec::with_capacity(np as usize);
                for i in 0..np {
                    let sub_id = stat_id + u16::from(i);
                    let sub = itemstatcost::by_id(sub_id).ok_or_else(|| ParseHardError {
                        message: format!(
                            "Unknown grouped sub-stat id {sub_id} for lead {stat_id}."
                        ),
                    })?;
                    let raw = read_bits(bytes, cursor, sub.save_bits as usize)? as i64;
                    values.push(raw - sub.save_add as i64);
                }
                Ok(ItemProperty { stat_id, param, values, encoding: PropertyEncoding::Grouped(np) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::write_bits;

    fn write_prop_terminator(bytes: &mut Vec<u8>, pos: &mut BytePosition) {
        write_bits(bytes, pos, PROPERTY_LIST_TERMINATOR as u32, 9).unwrap();
    }

    #[test]
    fn parses_strength_then_terminator() {
        // strength: id=0, sB=8, sA=32. Encoded value 10 → raw = 10 + 32 = 42.
        let mut bytes = Vec::new();
        let mut pos = BytePosition::default();
        write_bits(&mut bytes, &mut pos, 0u32, 9).unwrap();
        write_bits(&mut bytes, &mut pos, 42u32, 8).unwrap();
        write_prop_terminator(&mut bytes, &mut pos);

        let mut cursor = BytePosition::default();
        let list = parse_property_list(&bytes, &mut cursor).unwrap();
        assert_eq!(list.properties.len(), 1);
        let p = &list.properties[0];
        assert_eq!(p.stat_id, 0);
        assert_eq!(p.values, vec![10]);
        assert!(matches!(p.encoding, PropertyEncoding::Standard));
    }

    #[test]
    fn parses_fire_damage_grouping() {
        // firemindam id=48 sB=8 sA=0; firemaxdam id=49 sB=9 sA=0 (np=2).
        // Sub-stats in a grouped property each use their own save_bits width.
        let mut bytes = Vec::new();
        let mut pos = BytePosition::default();
        write_bits(&mut bytes, &mut pos, 48u32, 9).unwrap();
        write_bits(&mut bytes, &mut pos, 5u32, 8).unwrap(); // min (stat 48, 8 bits)
        write_bits(&mut bytes, &mut pos, 12u32, 9).unwrap(); // max (stat 49, 9 bits)
        write_prop_terminator(&mut bytes, &mut pos);

        let mut cursor = BytePosition::default();
        let list = parse_property_list(&bytes, &mut cursor).unwrap();
        assert_eq!(list.properties.len(), 1);
        let p = &list.properties[0];
        assert_eq!(p.stat_id, 48);
        assert_eq!(p.values, vec![5, 12]);
        assert!(matches!(p.encoding, PropertyEncoding::Grouped(2)));
    }

    #[test]
    fn parses_chance_on_hit_encoding_2() {
        // item_skillonhit id=198 sB=7 sA=0 sP=16 encode=2.
        let mut bytes = Vec::new();
        let mut pos = BytePosition::default();
        write_bits(&mut bytes, &mut pos, 198u32, 9).unwrap();
        write_bits(&mut bytes, &mut pos, 0xABCDu32, 16).unwrap();
        write_bits(&mut bytes, &mut pos, 50u32, 7).unwrap();
        write_prop_terminator(&mut bytes, &mut pos);

        let mut cursor = BytePosition::default();
        let list = parse_property_list(&bytes, &mut cursor).unwrap();
        let p = &list.properties[0];
        assert_eq!(p.stat_id, 198);
        assert_eq!(p.param, Some(0xABCD));
        assert_eq!(p.values, vec![50]);
        assert!(matches!(p.encoding, PropertyEncoding::ChanceOnHit));
    }

    #[test]
    fn parses_charges_encoding_3() {
        // item_charged_skill id=204 sB=16 sA=0 sP=16 encode=3.
        let mut bytes = Vec::new();
        let mut pos = BytePosition::default();
        write_bits(&mut bytes, &mut pos, 204u32, 9).unwrap();
        write_bits(&mut bytes, &mut pos, 0x1234u32, 16).unwrap();
        write_bits(&mut bytes, &mut pos, 0xBEEFu32, 16).unwrap();
        write_prop_terminator(&mut bytes, &mut pos);

        let mut cursor = BytePosition::default();
        let list = parse_property_list(&bytes, &mut cursor).unwrap();
        let p = &list.properties[0];
        assert_eq!(p.stat_id, 204);
        assert_eq!(p.param, Some(0x1234));
        assert_eq!(p.values, vec![0xBEEF_i64]);
        assert!(matches!(p.encoding, PropertyEncoding::Charges));
    }

    #[test]
    fn empty_list_just_sentinel() {
        let mut bytes = Vec::new();
        let mut pos = BytePosition::default();
        write_prop_terminator(&mut bytes, &mut pos);
        let mut cursor = BytePosition::default();
        let list = parse_property_list(&bytes, &mut cursor).unwrap();
        assert!(list.properties.is_empty());
    }
}
