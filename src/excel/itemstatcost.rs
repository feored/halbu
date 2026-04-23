//! Item stat-cost reference table.
//!
//! Drives §7.1 / Appendix A of `docs/v105-item-format.md`: bit widths for
//! the property-list values and parameters, plus the per-stat encoding
//! dispatch (encode 0/2/3/4) used in §7.4–§7.6.
//!
//! Source: embedded `assets/excel/v105/itemstatcost.txt`.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::parser::{col_idx, parse_tsv, parse_u32, parse_u8};

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/itemstatcost.txt");

/// One row of `itemstatcost.txt` (only the columns the bitstream needs).
#[derive(Debug, Clone)]
pub(crate) struct StatCost {
    /// 9-bit stat ID carried in the bitstream (column `*ID`).
    pub id: u16,
    /// Stat name (e.g. `"strength"`); used for diagnostics and tests.
    pub name: String,
    /// Bit width of the value field (column `Save Bits`).
    pub save_bits: u8,
    /// Bias added to the stored value (column `Save Add`); `0` = no bias.
    pub save_add: u32,
    /// Bit width of the optional parameter field (column `Save Param Bits`); `0` if absent.
    pub save_param_bits: u8,
    /// Encoding dispatch (column `Encode`, default `0`): selects the §7.4–§7.6 layout.
    pub encode: u8,
}

static ROWS: OnceLock<Vec<StatCost>> = OnceLock::new();
static BY_ID: OnceLock<HashMap<u16, usize>> = OnceLock::new();

fn rows() -> &'static [StatCost] {
    ROWS.get_or_init(|| {
        let tsv = parse_tsv(TABLE_TEXT);
        let i_name = col_idx(&tsv.headers, "Stat");
        let i_id = col_idx(&tsv.headers, "*ID");
        let i_save_bits = col_idx(&tsv.headers, "Save Bits");
        let i_save_add = col_idx(&tsv.headers, "Save Add");
        let i_save_param_bits = col_idx(&tsv.headers, "Save Param Bits");
        let i_encode = col_idx(&tsv.headers, "Encode");

        tsv.rows
            .iter()
            .filter_map(|r| {
                let id_str = r[i_id].trim();
                if id_str.is_empty() {
                    return None;
                }
                let id: u16 = id_str.parse().ok()?;
                Some(StatCost {
                    id,
                    name: r[i_name].to_string(),
                    save_bits: parse_u8(r[i_save_bits]),
                    save_add: parse_u32(r[i_save_add]),
                    save_param_bits: parse_u8(r[i_save_param_bits]),
                    encode: parse_u8(r[i_encode]),
                })
            })
            .collect()
    })
    .as_slice()
}

fn by_id_index() -> &'static HashMap<u16, usize> {
    BY_ID.get_or_init(|| rows().iter().enumerate().map(|(i, r)| (r.id, i)).collect())
}

/// All stat-cost rows in declaration order.
pub(crate) fn all() -> &'static [StatCost] {
    rows()
}

/// Look up a stat-cost row by its 9-bit stat ID.
///
/// Note: `0x1FF` (511) is the property-list terminator and is not a row in
/// this table; callers should detect the terminator before calling this.
pub(crate) fn by_id(stat_id: u16) -> Option<&'static StatCost> {
    by_id_index().get(&stat_id).map(|&i| &rows()[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strength_row_matches_spec() {
        let s = by_id(0).expect("stat 0 strength");
        assert_eq!(s.name, "strength");
        assert_eq!(s.save_bits, 8);
        assert_eq!(s.save_add, 32);
        assert_eq!(s.save_param_bits, 0);
        assert_eq!(s.encode, 0);
    }

    #[test]
    fn armorclass_row_matches_spec() {
        let s = by_id(31).expect("stat 31 armorclass");
        assert_eq!(s.name, "armorclass");
        assert_eq!(s.save_bits, 11);
        assert_eq!(s.save_add, 10);
    }

    #[test]
    fn durability_widths() {
        let dur = by_id(72).expect("stat 72 durability");
        assert_eq!(dur.name, "durability");
        assert_eq!(dur.save_bits, 9);

        let max = by_id(73).expect("stat 73 maxdurability");
        assert_eq!(max.name, "maxdurability");
        assert_eq!(max.save_bits, 8);
    }

    #[test]
    fn item_charged_skill_carries_param() {
        // The interpreted layout from `docs/v105-item-format.md` §7.6 is a
        // composed value (skill / level / charges) packed into the value field;
        // `itemstatcost.txt` itself stores the raw widths the bitstream uses.
        // The presence of a non-zero `Save Param Bits` is the load-bearing
        // signal for the parser: the row carries an extra param field.
        let s = by_id(204).expect("stat 204 item_charged_skill");
        assert_eq!(s.name, "item_charged_skill");
        assert!(s.save_bits > 0);
        assert!(s.save_param_bits > 0);
    }

    #[test]
    fn sentinel_511_is_not_a_row() {
        assert!(by_id(511).is_none());
    }

    #[test]
    fn unknown_id_returns_none() {
        assert!(by_id(9999).is_none());
    }
}
