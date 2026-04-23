//! Armor base-item reference table.
//!
//! Drives §6.1 (armor sub-block presence) and move-validation (item-size,
//! socket capacity) per `docs/v105-item-format.md`.
//!
//! Source: embedded `assets/excel/v105/armor.txt`.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::parser::{col_idx, opt_string, parse_bool_01, parse_tsv, parse_u8};

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/armor.txt");

/// One row of `armor.txt` (only the columns the parser/move-API need).
#[derive(Debug, Clone)]
pub(crate) struct ArmorBase {
    /// 3-character primary key (e.g. `"cap"`).
    pub code: String,
    /// Display name (e.g. `"Cap"`).
    pub name: String,
    /// Item-type code (e.g. `"helm"`); cross-references `itemtypes.txt`.
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
    /// `true` if the base has no durability sub-block (§6.2).
    ///
    /// Derived from the `durability` column being `0`. The `nodurability`
    /// column in `armor.txt` is also consulted, but `durability == 0` is the
    /// authoritative signal: if a base has no durability stat, no sub-block
    /// is read regardless of the `nodurability` flag's value.
    pub no_durability: bool,
    /// Shield-block percentage; non-zero indicates shield.
    pub block: u8,
    /// `true` if the base is stackable (§6.4).
    pub stackable: bool,
}

static ROWS: OnceLock<Vec<ArmorBase>> = OnceLock::new();
static BY_CODE: OnceLock<HashMap<String, usize>> = OnceLock::new();

fn rows() -> &'static [ArmorBase] {
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
        let i_dur = col_idx(&tsv.headers, "durability");
        let i_nodur = col_idx(&tsv.headers, "nodurability");
        let i_block = col_idx(&tsv.headers, "block");
        let i_stack = col_idx(&tsv.headers, "stackable");

        tsv.rows
            .iter()
            .filter_map(|r| {
                let code = r[i_code].trim();
                if code.is_empty() {
                    return None;
                }
                let dur = parse_u8(r[i_dur]);
                let nodur_flag = parse_bool_01(r[i_nodur]);
                Some(ArmorBase {
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
                    no_durability: dur == 0 || nodur_flag,
                    block: parse_u8(r[i_block]),
                    stackable: parse_bool_01(r[i_stack]),
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
                "armor.txt has duplicate code `{}`",
                row.code
            );
            idx.insert(row.code.clone(), i);
        }
        idx
    })
}

/// All armor base rows in declaration order.
pub(crate) fn all() -> &'static [ArmorBase] {
    rows()
}

/// Look up an armor base by its 3-character code.
pub(crate) fn by_code(code: &str) -> Option<&'static ArmorBase> {
    by_code_index().get(code).map(|&i| &rows()[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_resolves() {
        let r = by_code("cap").expect("cap row");
        assert_eq!(r.name, "Cap");
        assert_eq!(r.typ, "helm");
        assert_eq!(r.inv_width, 2);
        assert_eq!(r.inv_height, 2);
        assert_eq!(r.norm_code, "cap");
        assert_eq!(r.uber_code, "xap");
        assert_eq!(r.ultra_code, "uap");
    }

    #[test]
    fn exceptional_cap_present() {
        assert!(by_code("xap").is_some());
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(by_code("zzz").is_none());
    }
}
