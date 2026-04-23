//! v105 (Reign of the Warlock) item bitstream parser.
//!
//! This module decodes the v105 item tail of a `.d2s` save into a typed,
//! mutable, Serde-friendly model. See `docs/v105-item-format.md` for the
//! authoritative bitstream layout this code implements.
//!
//! # Scope
//!
//! - Read-only parsing for v105 (`FormatId::V105`). v99 is **out of scope**
//!   and continues to use the existing raw-bytes placeholder in the parent
//!   module.
//! - The encoder is task 004; the move-item API is task 007. The data model
//!   here is designed to be sufficient for both: every item carries enough
//!   raw-byte / raw-bit context to round-trip without re-deriving any field.
//!
//! # Lax-mode contract
//!
//! Bit-level resync inside an item is not feasible (no length prefix; nested
//! recursion through socketed children). In `Strictness::Lax`, intra-item
//! failures (truncated bitstream, unknown stat ID, unknown item-type code)
//! emit a `ParseIssue` and bail out of the items section at the failing
//! item. Items already parsed are kept; remaining lists are dropped with an
//! `IssueKind::TruncatedSection` issue. This mirrors `Attributes::parse`.

pub mod huffman;
pub mod item;
pub mod model;
pub mod properties;
pub mod tail;

#[cfg(test)]
mod tests;

pub use item::parse_item;
pub use model::{
    AltContainer, BitTail, CorpseEntry, EarData, EquippedSlot, Item, ItemExtended, ItemFlags,
    ItemHeader, ItemKind, ItemList, ItemListKind, ItemLocation, ItemProperty, ItemPropertyList,
    ItemQuality, ItemQualityData, ItemsTail, LocationId, PropertyEncoding, RunewordData,
    StandardItem,
};
pub use tail::parse_items_tail;
