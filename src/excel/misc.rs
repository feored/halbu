//! Misc base-item reference table (potions, gems, runes, charms, scrolls, ...).
//!
//! Drives §6.4 (stackable detection / quantity field) and move-validation
//! (auto-belt placement, item-size) per `docs/v105-item-format.md`.
//!
//! Source: embedded `assets/excel/v105/misc.txt`.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::parser::{col_idx, opt_string, parse_bool_01, parse_tsv, parse_u8};

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/misc.txt");

/// One row of `misc.txt` (only the columns the parser/move-API need).
#[derive(Debug, Clone)]
pub(crate) struct MiscBase {
    /// 3-character primary key (e.g. `"elx"`).
    pub code: String,
    /// Display name (e.g. `"Elixir"`).
    pub name: String,
    /// Item-type code (e.g. `"elix"`, `"gem0"`, `"rune"`).
    pub typ: String,
    /// Optional secondary item-type code.
    pub type2: Option<String>,
    /// Inventory grid width (cells).
    pub inv_width: u8,
    /// Inventory grid height (cells).
    pub inv_height: u8,
    /// `true` if the base is stackable (§6.4 quantity field).
    pub stackable: bool,
    /// `true` if the base auto-places into the belt (potion hint for move-API).
    pub auto_belt: bool,
}

static ROWS: OnceLock<Vec<MiscBase>> = OnceLock::new();
static BY_CODE: OnceLock<HashMap<String, usize>> = OnceLock::new();

fn rows() -> &'static [MiscBase] {
    ROWS.get_or_init(|| {
        let tsv = parse_tsv(TABLE_TEXT);
        let i_name = col_idx(&tsv.headers, "name");
        let i_code = col_idx(&tsv.headers, "code");
        let i_type = col_idx(&tsv.headers, "type");
        let i_type2 = col_idx(&tsv.headers, "type2");
        let i_w = col_idx(&tsv.headers, "invwidth");
        let i_h = col_idx(&tsv.headers, "invheight");
        let i_stack = col_idx(&tsv.headers, "stackable");
        let i_belt = col_idx(&tsv.headers, "autobelt");

        tsv.rows
            .iter()
            .filter_map(|r| {
                let code = r[i_code].trim();
                if code.is_empty() {
                    return None;
                }
                Some(MiscBase {
                    code: code.to_string(),
                    name: r[i_name].to_string(),
                    typ: r[i_type].to_string(),
                    type2: opt_string(r[i_type2]),
                    inv_width: parse_u8(r[i_w]),
                    inv_height: parse_u8(r[i_h]),
                    stackable: parse_bool_01(r[i_stack]),
                    auto_belt: parse_bool_01(r[i_belt]),
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
                "misc.txt has duplicate code `{}`",
                row.code
            );
            idx.insert(row.code.clone(), i);
        }
        idx
    })
}

/// All misc base rows in declaration order.
pub(crate) fn all() -> &'static [MiscBase] {
    rows()
}

/// Look up a misc base by its 3-character code.
pub(crate) fn by_code(code: &str) -> Option<&'static MiscBase> {
    by_code_index().get(code).map(|&i| &rows()[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elixir_resolves() {
        let r = by_code("elx").expect("elx row");
        assert_eq!(r.name, "Elixir");
        assert_eq!(r.typ, "elix");
        assert_eq!(r.inv_width, 1);
        assert_eq!(r.inv_height, 1);
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(by_code("zzz").is_none());
    }
}
