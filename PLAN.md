# Item Parsing Implementation Plan

## Background

`halbu` currently stores the entire items section as raw bytes (`items::Placeholder`).
This plan tracks the work to replace that with a fully parsed, editable item model.

### Key format constraints

Items are stored as a **packed LSB-first bitstream** with no length prefixes and no
end-of-item sentinels. Reaching item N requires walking items 0 through N−1.
Walking a single complex item requires three external data tables:

1. **Huffman tree** — decodes the variable-length type code (8–36 bits)
2. **Item type classification** — determines which optional fields are present (defense, durability, quantity)
3. **ItemStatCost bit-width table** — provides `sB`, `sP`, `np` per stat ID to skip property lists

The `"JM" + uint16` header at the start of each item list gives the **total item count for free**
without walking the stream, but nothing more.

### Format target order

Implementation targets **v105 (0x69 — Reign of the Warlock) first**, since it is a strict superset
of v99: all v99 fields are present in v105 plus two small additions. A correct v105 parser handles
v99 by simply skipping the version-gated extra bits.

### Static data strategy

All lookup tables (Huffman tree, ItemStatCost widths, type classification) are embedded as
**compile-time Rust constants**. This matches the existing library approach (empty-trailer byte
arrays, etc.) and avoids any runtime file I/O dependency.

Source for constant data: extracted v105 game files at `assets/excel/v105/`.
`dschu012/d2s` only has v96/v99 constant snapshots — no v105 support exists there.
The ground-truth source is the actual game data extracted from the D2R CASC archive.

---

## Epic 1 — Static Data Tables

> Prerequisites for everything. No items can be parsed without these tables.

- [ ] **1.1 Huffman tree**
  - Embed the full D2R Huffman decode tree as a compile-time constant in `src/items/huffman.rs`
  - Implement `decode_type_code(bits, offset) -> ([u8; 4], bits_consumed)` — reads 4 Huffman chars
  - Implement `encode_type_code(type_code: [u8; 4]) -> (u32, bits_count)` — for the write path
  - Unit tests: decode known type codes (`"hp1 "`, `"rin "`, `"hax "`, `"r01 "`, `"war "`) and
    verify round-trip encode → decode

- [ ] **1.2 ItemStatCost bit-width table**
  - Extract `sB`, `sP`, `np` for all stat IDs from `assets/excel/v105/itemstatcost.txt`
  - Embed as `static STAT_WIDTHS: [(u8, u8, u8); N]` (index = stat ID) in `src/items/stat_data.rs`
  - Unit tests: spot-check known IDs (0=strength: sB=8 sP=0 np=1; 54=coldmindam: np=3;
    204=item_charged_skill: sB=16 sP=16 e=3)

- [ ] **1.3 Item type classification tables**
  - Embed sets for: armor type codes, weapon type codes, stackable type codes, quest item type codes
  - Source from `assets/excel/v105/armor.txt`, `weapons.txt`, `misc.txt`, `itemtypes.txt`
  - Expose `fn classify(type_code: &[u8; 4]) -> ItemCategory` returning `Armor | Weapon | Other`
    and `fn is_stackable(type_code: &[u8; 4]) -> bool`, `fn is_quest(type_code: &[u8; 4]) -> bool`
  - Unit tests: `"cap "` → Armor, `"hax "` → Weapon, `"hp1 "` → Other + Stackable,
    `"war "` (Warlock-specific item if any) → correct category

---

## Epic 2 — Structural Walk (Parse Item Boundaries)

> Allows locating where each item starts and ends in the bitstream.
> All stories in this epic read fields but do **not** need to interpret values semantically.

- [ ] **2.1 Parse simple bits of a single item**
  - Read: flags (32b), item version (3b), location block (18b), Huffman type code (var), socket count (1 or 3b)
  - Handle ear path: ear_class (3b) + ear_level (7b) + null-terminated 7-bit name
  - Handle quest item path: quest_difficulty (2b) + socket count (1b)
  - Return a `SimpleItemHeader` struct with all parsed flag bits, location fields, and type code
  - Unit test: parse the worked example from `docs/item-details.html` —
    a Minor Healing Potion `hp1` at inventory (0,0), verify 73 bits consumed before alignment

- [ ] **2.2 Walk a complete item to its byte-aligned end offset**
  - For simple items: align after simple bits → done
  - For extended items: skip extended header (id 32b + level 7b + quality 4b + conditional fields),
    quality-specific data, runeword block, personalized name, tome data, timestamp,
    defense (armor), durability (armor/weapon), **v105: 1 unknown bit after durability**,
    quantity (stackable), socket count (socketed flag), plist_flag (set quality),
    then walk 1 + popcount(plist_flag) + (1 if runeword) property lists using `sB`/`sP`/`np`,
    **v105: 1-bit has_quantity + conditional 8-bit quantity trailer (ALL items)**,
    then byte-align
  - Recurse for socketed children
  - Return: `(byte_end_offset, simple_header)` — the byte offset after the complete item
  - Unit test: walk all items from the golden `.d2s` fixtures, verify the summed item data
    ends exactly at the known section boundary

- [ ] **2.3 Walk an item list and return all item headers**
  - Read `"JM"` marker + uint16 count; walk each item using story 2.2
  - Return `Vec<SimpleItemHeader>` — one entry per item with type, location, flags, and byte range
  - This is v105/v99 transparent (version passed in, gating the extra bits)
  - Test: parse Joe.d2s (v99) and Warlock_v105.d2s (v105) — print item counts and spot-check
    a known item type; verify no bytes are left over at the section boundary

---

## Epic 3 — Section Boundaries & Container Grouping

> Parses the full items block of a save, covering all sub-sections.

- [ ] **3.1 Parse all save-level item sections**
  - Parse in order: character item list → corpse section (outer JM count + per-corpse 12-byte
    header + item list) → merc section (`"jf"` header + item list, only if merc present) →
    golem section (`"kf"` + has_golem byte + single item if non-zero) →
    v105 RotW only: `"lf"` section (2 bytes + 2 bytes)
  - Return a structured `ParsedItemSections` with a `Vec<SimpleItemHeader>` per section
  - Test: verify merc item count, golem flag, corpse count for known test fixtures

- [ ] **3.2 Group character items by container**
  - Partition character item list by `alt_position_id` and `location_id`:
    inventory (alt=1), equipped body slots (location=1), belt (location=2),
    Horadric Cube (alt=4), stash (alt=5)
  - Expose counts per container: `fn item_counts(&self) -> ItemCounts`
  - Test: verify the pala_sword.d2s and similar fixtures show expected item distribution
    (sword equipped, shield equipped, etc.)

---

## Epic 4 — Extended Item Data

> Fully parses all fields of non-simple items. Each story adds more semantic detail.

- [ ] **4.1 Parse extended item header (identity)**
  - Fields: id (32b), level (7b), quality (4b), multiple_pictures + picture_id, class_specific + auto_affix_id
  - Extend the item struct with these fields
  - Test: parse items from golden saves, verify quality values and ilvls on known items

- [ ] **4.2 Parse quality-specific data**
  - Low (3b low_quality_id), Normal (nothing), Superior (3b file_index),
    Magic (11b prefix + 11b suffix), Set (12b set_id), Unique (12b unique_id),
    Rare/Crafted (8b+8b name IDs + 6 × (1b flag + conditional 11b affix))
  - Test: check a known magic item's prefix/suffix IDs, a known unique item's unique_id

- [ ] **4.3 Parse remaining extended fields**
  - Runeword ID (12b) + padding (4b), personalized name (null-terminated 8-bit chars),
    tome data (5b), timestamp (1b), defense rating (11b − 10), max/current durability,
    quantity (9b stackable), total_nr_of_sockets (4b), plist_flag (5b)
  - **v105**: store the 1-bit unknown field after durability as `v105_unknown_after_durability: bool`
  - **v105**: store the quantity trailer as `v105_stash_quantity: Option<u8>`
  - Test: check defense values on armor, socket counts on socketed items, personalized names

- [ ] **4.4 Parse magic property lists**
  - Read 9-bit stat ID loop until sentinel `0x1FF`; for each entry read `sP` param bits + `sB` value bits,
    subtract `sA` for actual value
  - Handle multi-property stats (`np > 1`): one stat_id triggers `np` consecutive sub-entry reads
  - Handle encoding 2 (chance-to-cast): split param into skill_level (6b) + skill_id (10b)
  - Handle encoding 3 (charges): split param same as e=2; split value into current (8b) + max (8b)
  - Handle dF=14 skill-tab bonus: split param into tab (3b) + class (13b)
  - Read set bonus lists (popcount of plist_flag) and runeword list if present
  - Store all lists in the item struct
  - Test: verify known items have expected properties (e.g. a unique with well-known stats,
    a runeword item with its bonus list, a set item with plist entries)

---

## Epic 5 — v105 Parity (already threaded into Epics 2–4)

> These items are included in-line above but listed here for visibility.

- [ ] **5.1 v105 extra bit after durability** *(threaded into 2.2 and 4.3)*
  - Read/write 1 unknown bit after durability block for extended armor/weapon items when `format == V105`
  - Store value for round-trip fidelity (`v105_unknown_after_durability`)

- [ ] **5.2 v105 quantity trailer** *(threaded into 2.2 and 4.3)*
  - Read/write `has_quantity (1b)` + conditional `quantity (8b)` before `Align()` for **ALL** items
    (outside the `simple_item == 0` block) when `format == V105`

- [ ] **5.3 Confirm Huffman tree and ItemStatCost are unchanged in v105**
  - Diff `magical_properties` between v99 constant data (`dschu012/d2s`) and v105 `itemstatcost.txt`
  - If new stat IDs exist (likely for Warlock skills), add them to the table
  - Document findings in `docs/v99-v105-details.html` open questions section

---

## Epic 6 — Write Path & Integration

> Mirrors the read path; gates the `Placeholder` replacement.

- [ ] **6.1 Encode a single item back to binary**
  - Mirror the read path in field order and bit widths
  - Write simple bits (flags, version, location, Huffman type, socket count)
  - Write extended data (all conditional fields in correct order)
  - Write property lists with correct sentinel
  - Write v105 extra fields at correct positions
  - Write byte-alignment padding
  - Recurse for socketed children
  - Test: parse a raw item blob → encode → compare bytes exactly

- [ ] **6.2 Encode full item sections**
  - Write all sections in order: character items, corpse, merc, golem, lf (v105 RotW)
  - Test: full save round-trip with parsed items — parse bytes → encode → parse again,
    assert semantic equality (same as existing `assert_same_model` pattern)

- [ ] **6.3 Replace `Placeholder` with fully parsed model**
  - Change `Save.items` from `items::Placeholder` to the new parsed model type
  - Update `format/decode.rs` and `format/encode.rs` to use the new parse/generate functions
  - Keep `raw_section` escape hatch or hard-error on unknown section bytes
  - All existing golden tests must continue to pass
  - Add new golden tests that assert on item counts, types, and properties

---

## Non-goals (out of scope for this plan)

- `.d2x` shared stash file parsing (different file format, not `.d2s`)
- Item modification / editing API (reading is the prerequisite; editing comes after)
- Full localization / display name resolution (stat IDs and type codes are sufficient for editing)
- Mercenary item subsection rewrite when hire-state toggles (existing known limitation, tracked in `NOTES.md`)
