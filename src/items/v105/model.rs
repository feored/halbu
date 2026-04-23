//! Typed v105 item model.
//!
//! See `docs/v105-item-format.md` §4–§11. All public types derive
//! `Serialize`/`Deserialize` for downstream consumers.

use serde::{Deserialize, Serialize};

/// Trailing pad bits captured at byte-alignment (round-trip insurance).
///
/// Populated by the item parser after each item byte-aligns. Width is
/// `0..=7` bits; `bits` packs them LSB-first in a single `u8`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitTail {
    /// LSB-first packed pad bits (only the low `bit_len` bits are valid).
    pub bits: u8,
    /// Number of pad bits read (0..=7).
    pub bit_len: u8,
}

/// A fully parsed v105 item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// Common 53-bit header (flags, item version, location).
    pub header: ItemHeader,
    /// Either an ear payload or the standard typed/extended item shape.
    pub kind: ItemKind,
    /// `Some(n)` if the §7.7 RotW quantity flag was set; otherwise `None`.
    pub rotw_quantity: Option<u8>,
    /// 0..=7 zero/non-zero pad bits captured at byte-alignment for round-trip fidelity.
    pub bit_tail: BitTail,
    /// Recursively parsed children for items with `nr_of_items_in_sockets > 0`.
    pub socketed_items: Vec<Item>,
}

/// Discriminator between an ear-trophy and a regular item.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Ear(EarData),
    Standard(StandardItem),
}

/// Standard (non-ear) item body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandardItem {
    /// Huffman-decoded 4-character ASCII type code (e.g. `b"hax "`, `b"r01 "`).
    pub type_code: [u8; 4],
    /// Number of items currently inserted in this item's sockets (parsed children).
    pub sockets_filled: u8,
    /// `Some(0..=2)` if this is a quest item (parsed via the `quest_difficulty` field).
    pub quest_difficulty: Option<u8>,
    /// Extended block; `None` when `simple_item == 1`.
    pub extended: Option<ItemExtended>,
}

/// Extended-item block (only present when `simple_item == 0`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemExtended {
    /// 32-bit instance id.
    pub id: u32,
    /// Item level (1..=99).
    pub level: u8,
    /// Quality discriminator.
    pub quality: ItemQuality,
    /// Quality-specific sub-block payload.
    pub quality_data: ItemQualityData,
    /// Optional 3-bit graphic variant id (when `multiple_pictures == 1`).
    pub picture_id: Option<u8>,
    /// Optional 11-bit class-specific auto-affix id (when `class_specific == 1`).
    pub class_specific_auto_affix: Option<u16>,
    /// Runeword payload, if `given_runeword == 1`.
    pub runeword: Option<RunewordData>,
    /// Personalized name (Anya quest), if `personalized == 1`.
    pub personalized_name: Option<String>,
    /// 5-bit tome data field for Tome of TP / Identify items.
    pub tome_data: Option<u8>,
    /// 1-bit realm-data / timestamp flag (always read for extended items).
    pub realm_data_flag: bool,
    /// 11-bit raw armor defense field (no `save_add` bias removed). `None` for non-armor.
    pub defense_raw: Option<u16>,
    /// 8-bit max durability. `None` for non-durability item types.
    pub max_durability: Option<u8>,
    /// 9-bit current durability. `None` if `max_durability == 0` or item lacks durability.
    pub current_durability: Option<u16>,
    /// New-in-v105 single bit read after the durability block. Preserve verbatim.
    pub v105_unknown_after_durability: bool,
    /// 9-bit stackable per-stack quantity (arrows, bolts, throwables). `None` for non-stackables.
    pub stack_quantity: Option<u16>,
    /// 4-bit total socket count, only present when the `socketed` flag is set.
    pub total_sockets: Option<u8>,
    /// 5-bit set-bonus mask (popcount = number of extra property lists). Only for set-quality.
    pub set_bonus_mask: Option<u8>,
    /// Main property list (terminated by `0x1FF`).
    pub properties: ItemPropertyList,
    /// Set-bonus property lists (one per popcount of `set_bonus_mask`).
    pub set_bonus_property_lists: Vec<ItemPropertyList>,
    /// Runeword property list (only if `given_runeword == 1`).
    pub runeword_property_list: Option<ItemPropertyList>,
}

/// Item quality enum (matches `quality` field at §5.1).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemQuality {
    LowQuality = 1,
    Normal = 2,
    Superior = 3,
    Magic = 4,
    Set = 5,
    Rare = 6,
    Unique = 7,
    Crafted = 8,
}

impl ItemQuality {
    /// Build from the 4-bit `quality` enum value.
    pub fn from_raw(value: u8) -> Option<Self> {
        match value {
            1 => Some(ItemQuality::LowQuality),
            2 => Some(ItemQuality::Normal),
            3 => Some(ItemQuality::Superior),
            4 => Some(ItemQuality::Magic),
            5 => Some(ItemQuality::Set),
            6 => Some(ItemQuality::Rare),
            7 => Some(ItemQuality::Unique),
            8 => Some(ItemQuality::Crafted),
            _ => None,
        }
    }

    /// Raw 4-bit value as encoded.
    pub fn as_raw(self) -> u8 {
        self as u8
    }
}

/// Quality-specific sub-block payload (§5.2).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemQualityData {
    /// `quality == Normal` carries no sub-block.
    None,
    /// 3-bit low-quality sub-id (Crude/Cracked/Damaged/Low-Quality).
    LowQuality { id: u8 },
    /// 3-bit superior file index.
    Superior { file_index: u8 },
    /// Magic prefix + suffix (11 + 11 bits).
    Magic { prefix: u16, suffix: u16 },
    /// 12-bit set id (index into `setitems.txt`).
    Set { set_id: u16 },
    /// 12-bit unique id.
    Unique { unique_id: u16 },
    /// Rare/Crafted name pair plus 6 optional 11-bit affix ids.
    RareOrCrafted { name1: u8, name2: u8, affixes: [Option<u16>; 6] },
}

/// Runeword block payload (§5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunewordData {
    /// 12-bit runeword id (index into runewords table).
    pub id: u16,
    /// 4-bit padding (observed always 5 in v99; preserved verbatim).
    pub padding: u8,
}

/// Ear PvP-trophy payload (§8.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarData {
    /// 3-bit class index of the slain player.
    pub class_id: u8,
    /// 7-bit level at death.
    pub level: u8,
    /// Null-terminated 7-bit-character name.
    pub name: String,
}

/// Common 53-bit header at the start of every item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemHeader {
    /// 32-bit raw flags word (LSB-first); see [`ItemFlags`] for accessors.
    pub flags: ItemFlags,
    /// 3-bit `item_version` field (observed 5 / `0b101` on D2R items).
    pub item_version: u8,
    /// Decoded 18-bit location block.
    pub location: ItemLocation,
}

/// Bit-packed wrapper around the 32-bit flags word.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemFlags {
    /// Raw 32-bit flags word, preserved verbatim. Bit accessors below pull
    /// individual semantic flags out of this value at the documented offsets.
    pub raw: u32,
}

impl ItemFlags {
    /// `true` if bit 4 (identified) is set.
    pub fn identified(self) -> bool {
        self.bit(4)
    }
    /// `true` if bit 11 (socketed) is set.
    pub fn socketed(self) -> bool {
        self.bit(11)
    }
    /// `true` if bit 13 (new) is set.
    #[allow(clippy::new_ret_no_self, reason = "bitfield accessor named after the spec field name")]
    pub fn new(self) -> bool {
        self.bit(13)
    }
    /// `true` if bit 16 (`is_ear`) is set.
    pub fn is_ear(self) -> bool {
        self.bit(16)
    }
    /// `true` if bit 17 (`starter_item`) is set.
    pub fn starter_item(self) -> bool {
        self.bit(17)
    }
    /// `true` if bit 21 (`simple_item`) is set.
    pub fn simple_item(self) -> bool {
        self.bit(21)
    }
    /// `true` if bit 22 (ethereal) is set.
    pub fn ethereal(self) -> bool {
        self.bit(22)
    }
    /// `true` if bit 24 (personalized) is set.
    pub fn personalized(self) -> bool {
        self.bit(24)
    }
    /// `true` if bit 26 (`given_runeword`) is set.
    pub fn given_runeword(self) -> bool {
        self.bit(26)
    }

    fn bit(self, idx: u32) -> bool {
        (self.raw >> idx) & 1 != 0
    }
}

/// Decoded 18-bit location block (§9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemLocation {
    /// Container kind / disposition.
    pub location_id: LocationId,
    /// Equip slot (only meaningful when `location_id == Equipped`).
    pub equipped_id: EquippedSlot,
    /// Grid column (also belt slot index when `location_id == Belt`). 0..=15.
    pub position_x: u8,
    /// Grid row (unused for belt). 0..=15.
    pub position_y: u8,
    /// Container when `location_id == Stored`.
    pub alt_position_id: AltContainer,
}

/// `location_id` enum (§9.1).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationId {
    Stored,
    Equipped,
    Belt,
    Cursor,
    Socketed,
    Unknown(u8),
}

impl LocationId {
    /// Decode raw 3-bit value.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => LocationId::Stored,
            1 => LocationId::Equipped,
            2 => LocationId::Belt,
            4 => LocationId::Cursor,
            6 => LocationId::Socketed,
            other => LocationId::Unknown(other),
        }
    }
    /// Raw 3-bit value.
    pub fn as_raw(self) -> u8 {
        match self {
            LocationId::Stored => 0,
            LocationId::Equipped => 1,
            LocationId::Belt => 2,
            LocationId::Cursor => 4,
            LocationId::Socketed => 6,
            LocationId::Unknown(other) => other,
        }
    }
}

/// `equipped_id` enum (§9.2).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquippedSlot {
    None,
    Helm,
    Amulet,
    Body,
    RightArm,
    LeftArm,
    RightRing,
    LeftRing,
    Belt,
    Boots,
    Gloves,
    RightArmSwap,
    LeftArmSwap,
    Unknown(u8),
}

impl EquippedSlot {
    /// Decode raw 4-bit value.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => EquippedSlot::None,
            1 => EquippedSlot::Helm,
            2 => EquippedSlot::Amulet,
            3 => EquippedSlot::Body,
            4 => EquippedSlot::RightArm,
            5 => EquippedSlot::LeftArm,
            6 => EquippedSlot::RightRing,
            7 => EquippedSlot::LeftRing,
            8 => EquippedSlot::Belt,
            9 => EquippedSlot::Boots,
            10 => EquippedSlot::Gloves,
            11 => EquippedSlot::RightArmSwap,
            12 => EquippedSlot::LeftArmSwap,
            other => EquippedSlot::Unknown(other),
        }
    }

    /// Raw 4-bit value.
    pub fn as_raw(self) -> u8 {
        match self {
            EquippedSlot::None => 0,
            EquippedSlot::Helm => 1,
            EquippedSlot::Amulet => 2,
            EquippedSlot::Body => 3,
            EquippedSlot::RightArm => 4,
            EquippedSlot::LeftArm => 5,
            EquippedSlot::RightRing => 6,
            EquippedSlot::LeftRing => 7,
            EquippedSlot::Belt => 8,
            EquippedSlot::Boots => 9,
            EquippedSlot::Gloves => 10,
            EquippedSlot::RightArmSwap => 11,
            EquippedSlot::LeftArmSwap => 12,
            EquippedSlot::Unknown(other) => other,
        }
    }
}

/// `alt_position_id` enum (§9.3).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AltContainer {
    NotInGrid,
    Inventory,
    Cube,
    Stash,
    Unknown(u8),
}

impl AltContainer {
    /// Decode raw 3-bit value.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => AltContainer::NotInGrid,
            1 => AltContainer::Inventory,
            4 => AltContainer::Cube,
            5 => AltContainer::Stash,
            other => AltContainer::Unknown(other),
        }
    }
    /// Raw 3-bit value.
    pub fn as_raw(self) -> u8 {
        match self {
            AltContainer::NotInGrid => 0,
            AltContainer::Inventory => 1,
            AltContainer::Cube => 4,
            AltContainer::Stash => 5,
            AltContainer::Unknown(other) => other,
        }
    }
}

/// One decoded property record from a property list (§7.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemProperty {
    /// 9-bit stat id.
    pub stat_id: u16,
    /// Optional parameter field (raw, no decomposition into skill/level).
    pub param: Option<u32>,
    /// One value for `np == 1`; multiple consecutive values for `np > 1`.
    /// Stored as raw (pre-`save_add`-bias removal) `i64` to losslessly preserve
    /// any encoding without coercion bugs.
    pub values: Vec<i64>,
    /// Encoding tag for clarity / round-trip.
    pub encoding: PropertyEncoding,
}

/// Property encoding dispatch tag (§7.4–§7.6).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyEncoding {
    /// Single-value standard read with optional param.
    Standard,
    /// `Encode == 2`: 16-bit param + 7-bit value (chance-on-hit family).
    ChanceOnHit,
    /// `Encode == 3`: 16-bit param + 16-bit value (charges).
    Charges,
    /// `Encode == 4`: skill-tab packed param.
    SkillTab,
    /// Multi-stat grouping (`np > 1`); inner `u8` is the run length (`np`).
    Grouped(u8),
}

/// A single property list, terminated by `0x1FF`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemPropertyList {
    /// Properties, in the order parsed from the bitstream.
    pub properties: Vec<ItemProperty>,
}

/// One of the four trailing item lists (player / merc / golem). The corpse
/// list uses [`CorpseEntry`] because of its per-corpse 12-byte sub-header.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemList {
    /// List discriminator.
    pub kind: ItemListKind,
    /// Items in declaration order.
    pub items: Vec<Item>,
}

/// Discriminator for [`ItemList`].
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemListKind {
    #[default]
    Player,
    MercEquipped,
    Golem,
}

/// One per-corpse entry inside the corpse-items section (§3.1 / §11.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpseEntry {
    /// Unknown 32-bit field at the start of each per-corpse block.
    pub unknown: u32,
    /// World x-coordinate of the corpse.
    pub x: u32,
    /// World y-coordinate of the corpse.
    pub y: u32,
    /// Items lying on this corpse.
    pub items: Vec<Item>,
}

/// Fully parsed v105 items tail.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemsTail {
    /// Player items (inventory + belt + cube + stash + equipped, all in one list).
    pub player: ItemList,
    /// Corpse-items section (always present; vec may be empty).
    pub corpses: Vec<CorpseEntry>,
    /// Mercenary-items list (`None` when no merc is hired; Some([]) when hired but empty).
    pub mercenary: Option<ItemList>,
    /// Whether the `"jf"` mercenary-section magic was present (always true on expansion).
    pub mercenary_header_present: bool,
    /// Iron-golem item (`None` when `has_golem == 0`).
    pub golem: Option<Item>,
    /// Raw `lf` trailer payload (RotW only). Preserved opaque for round-trip.
    pub rotw_lf_trailer: Option<Vec<u8>>,
}
