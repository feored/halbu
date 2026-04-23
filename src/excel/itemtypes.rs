//! Item-type hierarchy reference table.
//!
//! Drives several decisions in `docs/v105-item-format.md`:
//! - §4.4 (quest-item detection → socket-count bit width)
//! - §6.4 (stackable detection via `Throwable` / `Quiver` flags)
//! - §9.2 (per-type body-loc eligibility for equipped slots)
//! - §10 (inserted-item parent-type classification)
//!
//! Plus class-restriction validation (used by the move-item API in task 007)
//! via the `Class` and `StaffMods` columns.
//!
//! Source: embedded `assets/excel/v105/itemtypes.txt`.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::parser::{col_idx, opt_string, parse_bool_01, parse_tsv};

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/itemtypes.txt");

/// One row of `itemtypes.txt` (only the columns the parser/move-API need).
#[derive(Debug, Clone)]
pub(crate) struct ItemType {
    /// 4-letter (or shorter) primary type code (e.g. `"helm"`, `"axe"`, `"shie"`).
    pub code: String,
    /// Display name (e.g. `"Shield"`).
    pub name: String,
    /// Parent type code in the equivalence hierarchy, if any.
    pub equiv1: Option<String>,
    /// Second parent type code, if any.
    pub equiv2: Option<String>,
    /// First eligible body-loc code (e.g. `"head"`, `"rarm"`), if any.
    pub body_loc1: Option<String>,
    /// Second eligible body-loc code, if any.
    pub body_loc2: Option<String>,
    /// `true` for throwable types (used in §6.4 stackable detection).
    pub throwable: bool,
    /// `true` for quiver types (used in §6.4 stackable detection).
    pub quiver: bool,
    /// `true` if items of this type carry a durability sub-block.
    pub repair: bool,
    /// Class associated with staff-mod skills (e.g. `"ama"`), if any.
    pub staff_mods_class: Option<String>,
    /// Hard class restriction for equipping (e.g. `"sor"`), if any.
    pub class: Option<String>,
}

static ROWS: OnceLock<Vec<ItemType>> = OnceLock::new();
static BY_CODE: OnceLock<HashMap<String, usize>> = OnceLock::new();

fn rows() -> &'static [ItemType] {
    ROWS.get_or_init(|| {
        let tsv = parse_tsv(TABLE_TEXT);
        let i_name = col_idx(&tsv.headers, "ItemType");
        let i_code = col_idx(&tsv.headers, "Code");
        let i_eq1 = col_idx(&tsv.headers, "Equiv1");
        let i_eq2 = col_idx(&tsv.headers, "Equiv2");
        let i_bl1 = col_idx(&tsv.headers, "BodyLoc1");
        let i_bl2 = col_idx(&tsv.headers, "BodyLoc2");
        let i_throwable = col_idx(&tsv.headers, "Throwable");
        let i_quiver = col_idx(&tsv.headers, "Quiver");
        let i_repair = col_idx(&tsv.headers, "Repair");
        let i_staff = col_idx(&tsv.headers, "StaffMods");
        let i_class = col_idx(&tsv.headers, "Class");

        tsv.rows
            .iter()
            .filter_map(|r| {
                let code = r[i_code].trim();
                if code.is_empty() {
                    return None;
                }
                Some(ItemType {
                    code: code.to_string(),
                    name: r[i_name].to_string(),
                    equiv1: opt_string(r[i_eq1]),
                    equiv2: opt_string(r[i_eq2]),
                    body_loc1: opt_string(r[i_bl1]),
                    body_loc2: opt_string(r[i_bl2]),
                    throwable: parse_bool_01(r[i_throwable]),
                    quiver: parse_bool_01(r[i_quiver]),
                    repair: parse_bool_01(r[i_repair]),
                    staff_mods_class: opt_string(r[i_staff]),
                    class: opt_string(r[i_class]),
                })
            })
            .collect()
    })
    .as_slice()
}

fn by_code_index() -> &'static HashMap<String, usize> {
    BY_CODE.get_or_init(|| rows().iter().enumerate().map(|(i, r)| (r.code.clone(), i)).collect())
}

/// All item-type rows in declaration order.
pub(crate) fn all() -> &'static [ItemType] {
    rows()
}

/// Look up an item type by its primary code.
pub(crate) fn by_code(code: &str) -> Option<&'static ItemType> {
    by_code_index().get(code).map(|&i| &rows()[i])
}

/// Returns `true` if `code` is `ancestor`, or any of its (transitive) parents
/// in the type-equivalence hierarchy via `Equiv1`/`Equiv2`.
///
/// Walks the parent chain depth-first with a small visited set to defeat any
/// malformed loops in the table data.
pub(crate) fn is_a(code: &str, ancestor: &str) -> bool {
    if code == ancestor {
        return true;
    }
    let mut visited: Vec<&str> = Vec::with_capacity(8);
    is_a_inner(code, ancestor, &mut visited)
}

fn is_a_inner(code: &str, ancestor: &str, visited: &mut Vec<&'static str>) -> bool {
    let row = match by_code(code) {
        Some(r) => r,
        None => return false,
    };
    if visited.iter().any(|v| *v == row.code.as_str()) {
        return false;
    }
    visited.push(row.code.as_str());
    for parent in [row.equiv1.as_deref(), row.equiv2.as_deref()].into_iter().flatten() {
        if parent == ancestor {
            return true;
        }
        if is_a_inner(parent, ancestor, visited) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_row_resolves() {
        let r = by_code("shie").expect("shie row");
        assert_eq!(r.name, "Shield");
        assert_eq!(r.body_loc1.as_deref(), Some("rarm"));
        assert_eq!(r.body_loc2.as_deref(), Some("larm"));
    }

    #[test]
    fn armor_row_resolves() {
        let r = by_code("tors").expect("tors row");
        assert_eq!(r.name, "Armor");
        assert_eq!(r.body_loc1.as_deref(), Some("tors"));
        assert_eq!(r.body_loc2.as_deref(), Some("tors"));
    }

    #[test]
    fn is_a_walks_equiv_chain() {
        // Shield has Equiv1=armo per file inspection.
        assert!(is_a("shie", "armo"));
        // Helm also belongs to armo.
        assert!(is_a("helm", "armo"));
        // A type is trivially `is_a` itself.
        assert!(is_a("axe", "axe"));
    }

    #[test]
    fn axe_is_in_weap_hierarchy() {
        // Axe → mele → weap is the file's chain.
        assert!(is_a("axe", "weap"));
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(by_code("zzzz").is_none());
        assert!(!is_a("zzzz", "armo"));
    }

    #[test]
    fn no_expansion_marker_row_present() {
        assert!(by_code("Expansion").is_none());
    }
}
