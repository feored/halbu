//! Body location reference table.
//!
//! Drives §9.2 (`equipped_id` decoding) of `docs/v105-item-format.md`.
//! Row index (1-based, with row 0 = `None`) is the `equipped_id` value
//! carried in the bitstream.
//!
//! Source: embedded `assets/excel/v105/bodylocs.txt`.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::parser::{col_idx, parse_tsv};

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/bodylocs.txt");

/// One row of `bodylocs.txt`.
#[derive(Debug, Clone)]
pub(crate) struct BodyLoc {
    /// Row index (`equipped_id` value carried in the bitstream).
    pub id: u8,
    /// Display name (e.g. `"Head"`).
    pub name: String,
    /// 4-letter slot code (e.g. `"head"`); empty for the `None` row at id 0.
    pub code: String,
}

static ROWS: OnceLock<Vec<BodyLoc>> = OnceLock::new();
static BY_CODE: OnceLock<HashMap<String, usize>> = OnceLock::new();

fn rows() -> &'static [BodyLoc] {
    ROWS.get_or_init(|| {
        let tsv = parse_tsv(TABLE_TEXT);
        let i_name = col_idx(&tsv.headers, "Body Location");
        let i_code = col_idx(&tsv.headers, "Code");
        tsv.rows
            .iter()
            .enumerate()
            .map(|(i, r)| BodyLoc {
                id: i as u8,
                name: r[i_name].to_string(),
                code: r[i_code].to_string(),
            })
            .collect()
    })
    .as_slice()
}

fn by_code_index() -> &'static HashMap<String, usize> {
    BY_CODE.get_or_init(|| {
        rows()
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.code.is_empty())
            .map(|(i, r)| (r.code.clone(), i))
            .collect()
    })
}

/// All body-location rows in declaration order.
pub(crate) fn all() -> &'static [BodyLoc] {
    rows()
}

/// Look up a body location by its `equipped_id` (row index).
pub(crate) fn by_id(id: u8) -> Option<&'static BodyLoc> {
    rows().get(id as usize)
}

/// Look up a body location by its 4-letter slot code (e.g. `"head"`).
pub(crate) fn by_code(code: &str) -> Option<&'static BodyLoc> {
    by_code_index().get(code).map(|&i| &rows()[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_zero_is_none_with_empty_code() {
        let r = by_id(0).expect("id 0 row");
        assert_eq!(r.name, "None");
        assert_eq!(r.code, "");
    }

    #[test]
    fn known_ids_resolve() {
        assert_eq!(by_id(1).unwrap().code, "head");
        assert_eq!(by_id(1).unwrap().name, "Head");
        assert_eq!(by_id(8).unwrap().code, "belt");
        assert_eq!(by_id(8).unwrap().name, "Belt");
    }

    #[test]
    fn by_code_finds_rarm() {
        let r = by_code("rarm").expect("rarm row");
        assert_eq!(r.id, 4);
        assert_eq!(r.name, "Right Arm");
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(by_code("zzzz").is_none());
    }

    #[test]
    fn all_rows_present() {
        // 11 declared rows: None + 10 slots.
        assert_eq!(all().len(), 11);
    }
}
