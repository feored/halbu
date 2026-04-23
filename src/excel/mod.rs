//! Embedded v105 Excel reference tables.
//!
//! These tables are embedded at compile time (`include_str!`) from
//! `assets/excel/v105/`. They drive bit-level decisions in the v105 item
//! parser/encoder and target validation in the move-item API. See
//! `docs/v105-item-format.md` for the bitstream layout these tables resolve.
//!
//! Tables are parsed lazily on first access (via `OnceLock`) and cached for
//! the process lifetime. The module has no runtime filesystem dependency.
//!
//! Internal (`pub(crate)`); downstream public types in `src/items/` will
//! re-export only the names that need to leak into the public API.
//!
//! # Minimum-viable scope
//!
//! Only the six tables required for parse/encode/move are embedded:
//! `armor`, `weapons`, `misc`, `itemtypes`, `itemstatcost`, `bodylocs`.
//! Affix/set/unique/runeword/gem tables are deferred — they are needed only
//! to *render* item display names, not to decode/encode/move items.

pub(crate) mod armor;
pub(crate) mod bodylocs;
pub(crate) mod itemstatcost;
pub(crate) mod itemtypes;
pub(crate) mod misc;
pub(crate) mod parser;
pub(crate) mod weapons;

pub(crate) use armor::ArmorBase;
pub(crate) use misc::MiscBase;
pub(crate) use weapons::WeaponBase;

// `BodyLoc`, `StatCost`, `ItemType` are accessed by sub-module path
// (`excel::bodylocs::by_id(..)` etc.) by downstream tasks; no re-export needed.

/// Cross-table reference to a base item resolved from a 3-character code.
///
/// Returned by [`lookup_base`]. The Huffman-decoded type code in the bitstream
/// is fixed-width 4 chars (trailing space padded); [`lookup_base`] trims input
/// before lookup, so callers may pass either form.
#[derive(Debug, Clone, Copy)]
pub(crate) enum BaseRef {
    /// Armor base (`assets/excel/v105/armor.txt`).
    Armor(&'static ArmorBase),
    /// Weapon base (`assets/excel/v105/weapons.txt`).
    Weapon(&'static WeaponBase),
    /// Misc base (`assets/excel/v105/misc.txt`).
    Misc(&'static MiscBase),
}

/// Resolve a 3- or 4-character item code (trailing spaces tolerated) against
/// the three base tables in order: armor → weapon → misc.
///
/// Returns `None` if the code is not present in any of the three tables.
/// Callers do **not** need to pre-trim the Huffman-decoded 4-char code.
pub(crate) fn lookup_base(code: &str) -> Option<BaseRef> {
    let key = code.trim_end();
    if let Some(a) = armor::by_code(key) {
        return Some(BaseRef::Armor(a));
    }
    if let Some(w) = weapons::by_code(key) {
        return Some(BaseRef::Weapon(w));
    }
    if let Some(m) = misc::by_code(key) {
        return Some(BaseRef::Misc(m));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_tables_parse_without_panicking() {
        assert!(!armor::all().is_empty());
        assert!(!weapons::all().is_empty());
        assert!(!misc::all().is_empty());
        assert!(!itemtypes::all().is_empty());
        assert!(!itemstatcost::all().is_empty());
        assert!(!bodylocs::all().is_empty());
    }

    #[test]
    fn lookup_base_handles_padded_weapon_code() {
        match lookup_base("hax ") {
            Some(BaseRef::Weapon(w)) => assert_eq!(w.name, "Hand Axe"),
            other => panic!("expected Weapon(Hand Axe), got {other:?}"),
        }
    }

    #[test]
    fn lookup_base_handles_padded_armor_code() {
        match lookup_base("cap ") {
            Some(BaseRef::Armor(a)) => assert_eq!(a.name, "Cap"),
            other => panic!("expected Armor(Cap), got {other:?}"),
        }
    }

    #[test]
    fn lookup_base_returns_none_for_unknown_code() {
        assert!(lookup_base("zzz ").is_none());
    }
}
