//! Weapon base-item reference table.
//!
//! Drives §6.2 (weapon sub-block presence) and move-validation
//! (item-size, two-handed collision, socket capacity) per
//! `docs/v105-item-format.md`.
//!
//! Source: embedded `assets/excel/v105/weapons.txt`.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::parser::{col_idx, opt_string, parse_bool_01, parse_tsv, parse_u8};

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/weapons.txt");

/// One row of `weapons.txt` (only the columns the parser/move-API need).
#[derive(Debug, Clone)]
pub(crate) struct WeaponBase {
    /// 3-character primary key (e.g. `"hax"`).
    pub code: String,
    /// Display name (e.g. `"Hand Axe"`).
    pub name: String,
    /// Item-type code (e.g. `"axe"`).
    pub typ: String,
    /// Optional secondary item-type code.
    pub type2: Option<String>,
    /// Normal-tier base code.
    pub norm_code: String,
    /// Exceptional-tier base code.
    pub uber_code: String,
    /// Elite-tier base code.
    pub ultra_code: String,
    /// Inventory grid width (cells).
    pub inv_width: u8,
    /// Inventory grid height (cells).
    pub inv_height: u8,
    /// Maximum number of sockets allowed on this base.
    pub gem_sockets: u8,
    /// `true` if the base is stackable (§6.4 quantity field).
    pub stackable: bool,
    /// `true` if the weapon can be wielded one- or two-handed
    /// (`1or2handed` column; flexible-grip weapons such as swords).
    pub one_or_two_handed: bool,
    /// `true` if the weapon is strictly two-handed (`2handed` column).
    pub two_handed: bool,
    /// `true` if the base has no durability sub-block.
    pub no_durability: bool,
}

static ROWS: OnceLock<Vec<WeaponBase>> = OnceLock::new();
static BY_CODE: OnceLock<HashMap<String, usize>> = OnceLock::new();

fn rows() -> &'static [WeaponBase] {
    ROWS.get_or_init(|| {
        let tsv = parse_tsv(TABLE_TEXT);
        let i_name = col_idx(&tsv.headers, "name");
        let i_code = col_idx(&tsv.headers, "code");
        let i_type = col_idx(&tsv.headers, "type");
        let i_type2 = col_idx(&tsv.headers, "type2");
        let i_norm = col_idx(&tsv.headers, "normcode");
        let i_uber = col_idx(&tsv.headers, "ubercode");
        let i_ultra = col_idx(&tsv.headers, "ultracode");
        let i_w = col_idx(&tsv.headers, "invwidth");
        let i_h = col_idx(&tsv.headers, "invheight");
        let i_sockets = col_idx(&tsv.headers, "gemsockets");
        let i_stack = col_idx(&tsv.headers, "stackable");
        let i_1or2 = col_idx(&tsv.headers, "1or2handed");
        let i_2h = col_idx(&tsv.headers, "2handed");
        let i_nodur = col_idx(&tsv.headers, "nodurability");

        tsv.rows
            .iter()
            .filter_map(|r| {
                let code = r[i_code].trim();
                if code.is_empty() {
                    return None;
                }
                Some(WeaponBase {
                    code: code.to_string(),
                    name: r[i_name].to_string(),
                    typ: r[i_type].to_string(),
                    type2: opt_string(r[i_type2]),
                    norm_code: r[i_norm].to_string(),
                    uber_code: r[i_uber].to_string(),
                    ultra_code: r[i_ultra].to_string(),
                    inv_width: parse_u8(r[i_w]),
                    inv_height: parse_u8(r[i_h]),
                    gem_sockets: parse_u8(r[i_sockets]),
                    stackable: parse_bool_01(r[i_stack]),
                    one_or_two_handed: parse_bool_01(r[i_1or2]),
                    two_handed: parse_bool_01(r[i_2h]),
                    no_durability: parse_bool_01(r[i_nodur]),
                })
            })
            .collect()
    })
    .as_slice()
}

fn by_code_index() -> &'static HashMap<String, usize> {
    BY_CODE.get_or_init(|| {
        let r = rows();
        let mut idx = HashMap::with_capacity(r.len());
        for (i, row) in r.iter().enumerate() {
            debug_assert!(
                !idx.contains_key(&row.code),
                "weapons.txt has duplicate code `{}`",
                row.code
            );
            idx.insert(row.code.clone(), i);
        }
        idx
    })
}

/// All weapon base rows in declaration order.
pub(crate) fn all() -> &'static [WeaponBase] {
    rows()
}

/// Look up a weapon base by its 3-character code.
pub(crate) fn by_code(code: &str) -> Option<&'static WeaponBase> {
    by_code_index().get(code).map(|&i| &rows()[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_axe_resolves() {
        let r = by_code("hax").expect("hax row");
        assert_eq!(r.name, "Hand Axe");
        assert_eq!(r.typ, "axe");
        assert_eq!(r.inv_width, 1);
        assert_eq!(r.inv_height, 3);
    }

    #[test]
    fn axe_is_one_handed() {
        let r = by_code("axe").expect("axe row");
        assert_eq!(r.name, "Axe");
        // From `weapons.txt`: invwidth=2, invheight=3, not strictly two-handed.
        assert_eq!(r.inv_width, 2);
        assert_eq!(r.inv_height, 3);
        assert!(!r.two_handed);
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(by_code("zzz").is_none());
    }
}
