//! Synthetic-byte unit tests for the v105 item parser.
//!
//! Empty-trailer round-trip checks live here; component-level tests for
//! Huffman, properties, and individual fields live in their respective
//! sibling modules.

use crate::ExpansionType;

use super::tail::parse_items_tail;

const V105_EMPTY_CLASSIC: [u8; 4] = [0x4A, 0x4D, 0x00, 0x00];
const V105_EMPTY_EXPANSION: [u8; 13] =
    [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];
const V105_EMPTY_EXPANSION_MERC: [u8; 17] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00,
];
const V105_EMPTY_ROTW: [u8; 19] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00, 0x01, 0x00, 0x6C,
    0x66, 0x00, 0x00,
];
const V105_EMPTY_ROTW_MERC: [u8; 23] = [
    0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x4A, 0x4D, 0x00, 0x00, 0x6B, 0x66,
    0x00, 0x01, 0x00, 0x6C, 0x66, 0x00, 0x00,
];

#[test]
fn empty_classic_parses_clean() {
    let tail = parse_items_tail(&V105_EMPTY_CLASSIC, ExpansionType::Classic, false).unwrap();
    assert!(tail.player.items.is_empty());
    assert!(tail.corpses.is_empty());
    assert!(tail.mercenary.is_none());
    assert!(!tail.mercenary_header_present);
    assert!(tail.golem.is_none());
    assert!(tail.rotw_lf_trailer.is_none());
}

#[test]
fn empty_expansion_no_merc_parses_clean() {
    let tail = parse_items_tail(&V105_EMPTY_EXPANSION, ExpansionType::Expansion, false).unwrap();
    assert!(tail.player.items.is_empty());
    assert!(tail.corpses.is_empty());
    assert!(tail.mercenary.is_none());
    assert!(tail.mercenary_header_present);
    assert!(tail.golem.is_none());
    assert!(tail.rotw_lf_trailer.is_none());
}

#[test]
fn empty_expansion_with_merc_parses_clean() {
    let tail =
        parse_items_tail(&V105_EMPTY_EXPANSION_MERC, ExpansionType::Expansion, true).unwrap();
    assert!(tail.player.items.is_empty());
    let merc = tail.mercenary.expect("merc list present");
    assert!(merc.items.is_empty());
    assert!(tail.golem.is_none());
}

#[test]
fn empty_rotw_no_merc_parses_clean() {
    let tail = parse_items_tail(&V105_EMPTY_ROTW, ExpansionType::RotW, false).unwrap();
    assert!(tail.mercenary.is_none());
    let lf = tail.rotw_lf_trailer.expect("lf trailer present");
    assert_eq!(lf, vec![0x01, 0x00, 0x6C, 0x66, 0x00, 0x00]);
}

#[test]
fn empty_rotw_with_merc_parses_clean() {
    let tail = parse_items_tail(&V105_EMPTY_ROTW_MERC, ExpansionType::RotW, true).unwrap();
    let merc = tail.mercenary.expect("merc list present");
    assert!(merc.items.is_empty());
    let lf = tail.rotw_lf_trailer.expect("lf trailer present");
    assert_eq!(lf, vec![0x01, 0x00, 0x6C, 0x66, 0x00, 0x00]);
}
