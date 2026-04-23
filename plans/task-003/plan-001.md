# Task 003 Implementation Plan: v105 Item Bitstream Parser

## Summary
Implement a fully-typed v105 item bitstream parser covering every item shape present in the v105 fixtures, plus the framing of the four trailing item lists (player, corpse, mercenary, golem) and the optional RotW `lf` trailer. This task delivers the **read** half of MASTERPLAN Feature 2 and is the foundation for Feature 3 (encoding, task 004), Feature 5 (Save integration, task 006), and Feature 6 (move-item, task 007). It replaces the v105 raw-bytes path in `format::decode` with a structured model while leaving V99 untouched. The parser is data-driven by the embedded Excel tables loaded in task 002.

## Scope
**In scope**
- A new `src/items/v105/` submodule that owns the v105 item bitstream parser (bit-level decoder for every item shape; parser for the four item lists; resync-safe error model).
- Typed model: `Item`, `ItemHeader`, `ItemExtended`, `ItemQualityData`, `ItemPropertyList`, `ItemProperty`, `ItemList`, `ItemsTail` (or equivalent named differently — see §Type Signatures).
- Integration into `src/format/decode.rs`: when `decoded_layout == FormatId::V105`, parse the items tail with the new parser; preserve the old `items::Placeholder` path for V99.
- Coexistence with the current `items::Placeholder` API. The placeholder must remain available for V99 with no behavior change.
- Unit tests on synthetic byte sequences for each item shape; a self-contained integration check that walks at least 10 representative fixtures from `assets/test/v105-real-characters/` and asserts strict-clean parse + zero `ParseIssue`s.
- Resolution (or explicit deferment, with documented rationale) of `docs/v105-item-format.md` §14 Open Questions Q2, Q5, Q6 by inspecting fixtures during implementation.
- Extension of error/issue model with item-specific variants (new `IssueKind` variants only; `ParseHardError` stays a `String` carrier per existing convention).

**Out of scope**
- Encoding (task 004).
- `.d2i` shared-stash parsing (task 005).
- `Save` API integration beyond what `format::decode` needs to wire the parsed items in. The public-facing `Save` field redesign for v105 belongs to task 006.
- Move-item API (task 007).
- Comprehensive golden round-trip tests (task 008). This task only ships unit tests + a smoke test against fixtures.
- V99 item modeling (MASTERPLAN Out of Scope).
- Display-name rendering (no affix/set/unique/runeword tables consumed for parsing).

## Technical Approach

### Architecture
The parser is a single recursive descent over a `BytePosition`-backed bit cursor on the items tail. It is split into three layers:

1. **Tail framing layer** (`tail.rs`): parses the byte-aligned section magics (`JM`, `jf`, `kf`, `lf`), counts, and per-corpse / per-golem framing described in `docs/v105-item-format.md` §3 + §11. This layer is byte-oriented and uses `u16_from` / `u8_from` helpers from `crate::utils`.
2. **Item layer** (`item.rs`): given a bit-cursor positioned at the start of an item, parses the 53-bit common header (§4), branches on `is_ear` / `simple_item`, parses the extended block (§5), the type-specific sub-blocks (§6), the property lists (§7), the v105 quantity trailer (§7.7), and finally byte-aligns. Returns one `Item` and a final byte offset.
3. **Properties layer** (`properties.rs`): drives the per-stat read using the embedded `excel::itemstatcost` table. Implements the encode-mode dispatch (encode 0 standard, encode 2 chance-on-hit, encode 3 charges, encode 4 skill-tab) and the multi-stat groupings (`np > 1`).

Inserted (socketed) child items are parsed by recursive call to the item layer at the parent's byte-aligned end (§10).

### Reuse vs. new code
- **Reuse**: `crate::utils::{BytePosition, read_bits}`. These already implement LSB-first bit-level reads, bounds-checking, and structured `ParseHardError` returns — exactly the contract `docs/v105-item-format.md` §2 prescribes.
- **Reuse**: `crate::excel::itemstatcost::{by_id, all}`, `crate::excel::itemtypes`, `crate::excel::lookup_base`. Already loaded by task 002; no new excel work.
- **New helpers in `items/v105/bitcursor.rs`**: `read_bool` (1 bit → bool), `read_string_8bit_terminated` (personalized name, max 16), `read_string_7bit_terminated` (ear name), `read_huffman_typecode` (4 chars via embedded Huffman table). The Huffman table is a `const &[(char, u32 bits, u8 bit_count)]` lookup, embedded as a small `match` or PHF-style table. (No external crate.) Decoding uses bit-by-bit traversal of an embedded tree built once via `OnceLock`.
- **Reuse**: `ParseIssue` / `ParseHardError` for diagnostics; extend `IssueKind` with new non-exhaustive variants.

### Strictness contract
- `Strictness::Strict`: any failure (truncation mid-item, unknown stat ID, unknown item type code, sentinel never reached before EOF) returns `ParseHardError`.
- `Strictness::Lax`: the parser uses a "best-effort, then bail with diagnostic" policy. Bit-level resync inside an item is **not feasible** (no length prefix; recovery would require speculative parsing). The plan therefore documents that lax mode for items behaves like strict mode for *intra-item* failures but emits a `ParseIssue` and stops the items section at the failing item. Items already parsed are kept; remaining sections are dropped with an `IssueKind::TruncatedSection` issue. This behavior mirrors how `Attributes::parse` truncates on error.

### Data-driven correctness
Per `docs/v105-item-format.md` Appendix A and §7, **no stat bit width is hardcoded**. Every per-property read consults `excel::itemstatcost::by_id(stat_id)` for `save_bits`, `save_add`, `save_param_bits`, and `encode`. The multi-stat groupings (`np > 1`, e.g. fire/cold/poison damage triplets) are encoded as a small const table in `properties.rs` keyed by lead stat ID, listing the consecutive stat IDs to fold into one `ItemProperty::Grouped` value (the doc enumerates the known groupings; no further excel column is needed).

### Open Questions resolution strategy
- **Q2** (`v105_unknown_after_durability` always read for any extended item, or only durability-bearing?): implement per the doc's safest reading (always for `simple_item == 0`). During fixture verification, decode several rare-quality non-durability items (rings/amulets from `m_one.d2s`, `m_two.d2s`) and confirm bit alignment by attempting parse; if parse fails on those fixtures, switch to durability-gated. Document outcome in code comment.
- **Q5/Q6** (`itemstatcost.txt` widths and new stat IDs): rely on the v105 excel table loaded in task 002 — already the source of truth. Smoke test confirms.
- **Q1** (RotW `lf` trailer payload): preserve as opaque raw bytes on `ItemsTail`; do not attempt to parse internally. Defer to task 005.

## Prerequisites & Dependencies
- **Task 001** (v105 item-format reference doc): complete — `docs/v105-item-format.md` exists and is the canonical layout reference.
- **Task 002** (embedded excel data): complete — `src/excel/{itemstatcost,itemtypes,armor,weapons,misc,bodylocs}.rs` already provide the lookup APIs this task needs.
- No new external crates. The Huffman tree is hand-rolled. (`bit::BitIndex` already in dependency graph via `utils.rs`.)

## Implementation Steps

### Step 1 — Module skeleton
- Create `src/items/v105/mod.rs` with submodules: `item.rs`, `properties.rs`, `tail.rs`, `huffman.rs`, `model.rs`, `error.rs`, plus `tests.rs`.
- Add `pub mod v105;` to `src/items/mod.rs` (keep `Placeholder` and existing API untouched).
- All v105 types are `pub` from `items::v105` so task 006 can re-export them on `Save`. Internal helpers stay `pub(crate)`.
- **Output**: empty modules compile; existing tests still pass.

### Step 2 — Model types (`model.rs`)
Define the typed item model. All structs derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`. Public enums get `#[non_exhaustive]` per project convention.

```rust
pub struct Item {
    pub header: ItemHeader,
    pub kind: ItemKind,                        // Ear | Standard
    pub raw_unparsed_trailing_bits: BitTail,   // round-trip insurance (see §Type Signatures)
}

#[non_exhaustive]
pub enum ItemKind {
    Ear(EarData),
    Standard(StandardItem),
}

pub struct StandardItem {
    pub type_code: [u8; 4],          // Huffman-decoded 4-char ASCII (e.g. b"hax ")
    pub sockets_filled: u8,          // count of inserted children (0..=6)
    pub quest_difficulty: Option<u8>,// stat 356, only for quest items
    pub extended: Option<ItemExtended>,           // None when simple_item == 1
    pub rotw_quantity: Option<u8>,   // §7.7 trailer
    pub socketed_items: Vec<Item>,   // recursively parsed children
}

pub struct ItemExtended {
    pub id: u32,
    pub level: u8,
    pub quality: ItemQuality,
    pub picture_id: Option<u8>,
    pub class_specific_auto_affix: Option<u16>,
    pub quality_data: ItemQualityData,
    pub runeword: Option<RunewordData>,
    pub personalized_name: Option<String>,
    pub tome_data: Option<u8>,
    pub realm_data_flag: bool,
    pub defense_raw: Option<u16>,                // raw before save_add bias
    pub max_durability: Option<u16>,
    pub current_durability: Option<u16>,
    pub v105_unknown_after_durability: bool,     // §6.3
    pub stack_quantity: Option<u16>,             // §6.4
    pub total_sockets: Option<u8>,               // §6.5
    pub set_bonus_mask: Option<u8>,              // §6.6 popcount-source
    pub properties: ItemPropertyList,            // main props
    pub set_bonus_property_lists: Vec<ItemPropertyList>,
    pub runeword_property_list: Option<ItemPropertyList>,
}

#[non_exhaustive]
pub enum ItemQuality { LowQuality, Normal, Superior, Magic, Set, Rare, Unique, Crafted }

#[non_exhaustive]
pub enum ItemQualityData {
    None,
    LowQuality { id: u8 },
    Superior { file_index: u8 },
    Magic { prefix: u16, suffix: u16 },
    Set { set_id: u16 },
    Unique { unique_id: u16 },
    Rare { name1: u8, name2: u8, affixes: [Option<u16>; 6] },
    Crafted { name1: u8, name2: u8, affixes: [Option<u16>; 6] },
}

pub struct RunewordData { pub id: u16, pub padding: u8 }

pub struct EarData { pub class_id: u8, pub level: u8, pub name: String }

pub struct ItemHeader {           // bits 0..=52, including the 18 location bits
    pub flags: ItemFlags,         // bit-packed wrapper around the 32 reserved/flag bits
    pub item_version: u8,         // bits 32..=34
    pub location: ItemLocation,   // bits 35..=52, fully decoded
}

pub struct ItemFlags { /* identified, socketed, new, is_ear, starter, simple_item, ethereal,
                         personalized, given_runeword, plus reserved-bit preservation */ }

pub struct ItemLocation {
    pub location_id: LocationId,
    pub equipped_id: EquippedSlot,         // meaningful only when location == Equipped
    pub position_x: u8,                    // 0..16
    pub position_y: u8,                    // 0..16
    pub alt_position_id: AltContainer,     // meaningful only when location == Stored
}

#[non_exhaustive] pub enum LocationId { Stored, Equipped, Belt, Cursor, Socketed, Unknown(u8) }
#[non_exhaustive] pub enum EquippedSlot { None, Helm, Amulet, Body, RightArm, LeftArm,
                                          RightRing, LeftRing, Belt, Boots, Gloves,
                                          RightArmSwap, LeftArmSwap, Unknown(u8) }
#[non_exhaustive] pub enum AltContainer { NotInGrid, Inventory, Cube, Stash, Unknown(u8) }

pub struct ItemProperty {
    pub stat_id: u16,
    pub param: Option<u32>,
    pub values: Vec<i32>,            // single value for np=1; multiple for groupings
    pub encoding: PropertyEncoding,  // tagged for round-trip clarity
}

#[non_exhaustive]
pub enum PropertyEncoding { Standard, ChargeOrChanceOnHit, Charges, SkillTab, Grouped(u8) }

pub struct ItemPropertyList { pub properties: Vec<ItemProperty> }

pub struct BitTail {                 // see Step 7
    pub bits: Vec<u8>,
    pub bit_len: u16,
}
```

Top-level list types:

```rust
pub struct ItemList {
    pub kind: ItemListKind,
    pub items: Vec<Item>,
}

#[non_exhaustive]
pub enum ItemListKind { Player, MercEquipped, Golem }

pub struct CorpseEntry {
    pub unknown: u32,
    pub x: u32,
    pub y: u32,
    pub items: Vec<Item>,
}

pub struct ItemsTail {
    pub player: ItemList,
    pub corpses: Vec<CorpseEntry>,    // header always present; vec may be empty
    pub mercenary: Option<ItemList>,  // None when no merc hired (no inner JM list)
    pub mercenary_header_present: bool,// "jf" header presence (always true on expansion)
    pub golem: Option<Item>,          // None when has_golem == 0
    pub rotw_lf_trailer: Option<Vec<u8>>, // raw `lf` payload, RotW only
}
```

- **Output**: model compiles, derives serde.

### Step 3 — Huffman decoder (`huffman.rs`)
- Embed the v105 Huffman table as a `const &[(char, u8 bits, u8 nbits)]` (the same tree as v99 — `docs/v105-item-format.md` §4.3 confirms unchanged). Source the table values from `docs/item-details.html` §10.
- Build a tree (`OnceLock<HuffmanNode>`) on first use.
- Public function: `decode_one_char(cursor: &mut BytePosition, bytes: &[u8]) -> Result<u8, ParseHardError>`.
- Public function: `decode_type_code(cursor, bytes) -> Result<[u8; 4], ParseHardError>` (calls `decode_one_char` four times).
- **Output**: unit tests on known short codes (e.g. `'r' '0' '1' ' '` for runes).

### Step 4 — Property list parser (`properties.rs`)
- `pub(crate) fn parse_property_list(bytes, cursor, strictness, issues) -> Result<ItemPropertyList, ParseHardError>`.
- Algorithm:
  1. Loop:
     - Read 9 bits → `stat_id`.
     - If `stat_id == 0x1FF`, return.
     - Look up `excel::itemstatcost::by_id(stat_id)`. If `None` → in Strict, return `ParseHardError::UnknownStatId`; in Lax, push `ParseIssue { kind: IssueKind::UnknownStatId, ... }` and bail (cannot continue: width unknown).
     - If `save_param_bits > 0`, read `save_param_bits` bits → `param` (Option).
     - Dispatch on `(encode, np_for(stat_id))`:
       - `encode == 2` (chance-on-hit): read 16-bit param (override `save_param_bits` if it disagrees — flag in `ParseIssue`), read 7-bit value. Tag `PropertyEncoding::ChargeOrChanceOnHit`.
       - `encode == 3` (charges, stat 204): read 16-bit param + 16-bit value. Tag `PropertyEncoding::Charges`.
       - `encode == 4` or `descfunc=14` skill-tab pattern (lead stats 188..=194): pack `param` semantics into `PropertyEncoding::SkillTab`. Width remains as `save_param_bits` from excel.
       - Multi-stat grouping (`np_for(stat_id) > 1`): read `np` consecutive values, each `save_bits` of stat `stat_id+i`, biased by stat `stat_id+i`.save_add. Tag `PropertyEncoding::Grouped(np)`.
       - Else: standard `read_bits(save_bits)` → biased value.
- Const table for groupings (lead stat → np), populated from `docs/v105-item-format.md` §7.2 (stats 17, 48, 50, 52, 54, 57). Pure const; verified during Step 8.
- **Output**: unit tests using synthetic byte sequences encoding strength=10, then armor class, then a damage grouping, then sentinel.

### Step 5 — Single-item parser (`item.rs`)
- `pub fn parse_item(bytes, cursor, strictness, issues) -> Result<Item, ParseHardError>`.
- Parses, in order:
  1. 53-bit common header → `ItemHeader`. Decode location enums.
  2. Branch on `is_ear`:
     - `is_ear == 1`: parse `EarData` (§8.2: 3+7 bits + 7-bit-char null-term), then v105 quantity trailer (§7.7), then byte-align. Return `ItemKind::Ear`.
     - Else: continue.
  3. `decode_type_code` → 4-byte ASCII.
  4. Quest-item check (`itemtypes` lookup): if quest type, read 2-bit `quest_difficulty` then 1-bit socket count. Else simple_item ? 1-bit : 3-bit socket count.
  5. If `simple_item == 0`: parse extended block (§5.1–§5.6), then type-specific sub-blocks (§6.1–§6.6), then property lists (§7), then v105 quantity trailer (§7.7).
  6. If `simple_item == 1`: skip directly to v105 quantity trailer (§7.7 — applies to all items).
  7. Byte-align cursor (advance to next byte boundary; record padding bits in `BitTail` only if non-zero per Step 7).
  8. Recursively parse `sockets_filled` child items by calling `parse_item` again. Append into `socketed_items`.
- Helper functions for each numbered sub-step inside the file to keep `parse_item` short.
- Each step on failure: in Strict, propagate `ParseHardError`; in Lax, push issue and propagate (no resync within an item — documented in module doc).
- **Output**: unit tests for each item shape — see Step 8.

### Step 6 — Tail framing parser (`tail.rs`)
- `pub fn parse_items_tail(bytes, expansion_type, mercenary_hired, strictness, issues) -> Result<ItemsTail, ParseHardError>`.
- Algorithm:
  1. Expect `4A 4D` magic, read `u16` LE count, parse `count` items at the bit cursor (each item byte-aligns at end). → `player`.
  2. If expansion (Expansion or RotW): expect `4A 4D` corpse magic + `u16` corpse count. For each corpse: read 12 bytes (`u32 unknown`, `u32 x`, `u32 y`), then `4A 4D` + `u16` items count, parse items. → `corpses`.
  3. If expansion: expect `6A 66` ("jf"). `mercenary_header_present = true`. If `mercenary_hired`: expect inner `4A 4D` + `u16` + items → `Some(ItemList::MercEquipped)`. Else `mercenary = None`.
  4. If expansion: expect `6B 66` ("kf") + `u8` `has_golem`. If non-zero, parse exactly one `Item` (no JM/count) → `Some(item)`.
  5. If RotW (`expansion_type == ExpansionType::RotW`): consume the remainder as the `lf` trailer raw bytes → `Some(rest.to_vec())`. Validate that it begins with `01 00 6C 66` for sanity; mismatched magic emits a non-fatal `ParseIssue` and the bytes are still preserved.
- Each `expect_magic` failure: `ParseHardError` in Strict; `ParseIssue { kind: IssueKind::InvalidSignature, section: Some("items.<list>") }` and bail in Lax.
- **Output**: unit tests using the `V105_EMPTY_ITEMS_*` byte constants from `src/items/mod.rs` — every empty-trailer constant must round-trip parse without issues, producing zero items in every list.

### Step 7 — Round-trip insurance: `BitTail` (placeholder for unparsed bit remainders)
**Purpose**: per task 003 deliverables, every `Item` carries a "raw-bits backup field for any unparsed remainder". For v105, the only known unparsed remainder is the byte-alignment pad (0–7 zero bits). We capture it explicitly to avoid silently losing non-zero pad bits on encode.
- `BitTail { bits: Vec<u8>, bit_len: u16 }` records the pad bits read.
- Populated in Step 5 step 7. Empty (`bit_len == 0`) for byte-aligned items.
- Encoder (task 004) writes `bit_len` bits from `bits` LSB-first.
- If pad bits are ever non-zero on a fixture, emit a `ParseIssue` of severity `Warning`. (Unexpected per the doc but preserves fidelity.)

### Step 8 — Error / issue model extensions
- Extend `IssueKind` in `src/lib.rs` with non-exhaustive variants (additive only; semver-safe):
  - `UnknownStatId`
  - `UnknownItemTypeCode`
  - `ItemListMagicMismatch`
  - `ItemPropertyOverflow` (param wider than 32 bits, etc.)
- Do **not** modify `ParseHardError` shape; it remains a `String` carrier. New errors set descriptive `message` strings.
- Add a `pub(crate)` constructor `ParseHardError::new(msg)` if not already present (it is currently struct-literal).
- **Output**: enum compiles; all current matches still exhaustive due to `#[non_exhaustive]`.

### Step 9 — Wire into `format::decode`
- In `src/format/decode.rs`, after the existing skills-section decode (line 421), branch on `decoded_layout`:
  - `FormatId::V99`: keep existing `items::parse(...)` call producing `items::Placeholder` exactly as today.
  - `FormatId::V105`: call `items::v105::tail::parse_items_tail(&bytes[items_offset..], parsed_save.expansion_type(), parsed_save.character.mercenary.is_hired(), strictness, &mut issues)`. Store the result in a *new* field on `Save`.
- The `Save` struct currently has `pub items: items::Placeholder`. To avoid blocking task 006's larger refactor, this task adds a sibling field:
  ```rust
  #[serde(default)]
  pub items_v105: Option<items::v105::ItemsTail>,
  ```
  - Populated only when `decoded_layout == V105`. V99 sets it to `None`.
  - The `items::Placeholder` field also continues to receive the raw-bytes copy for V105 in this task, so existing encode (task 004 boundary) does not break. Once task 004 lands, the placeholder for V105 will be vestigial; task 006 removes it.
- This dual storage is explicit, called out in a doc comment, and tracked as "transitional shape" so task 006 can clean it up.
- **Output**: parser is invoked in the real decode path; existing V99 tests untouched.

### Step 10 — Fixture smoke test
- New integration test file: `tests/v105_items_parse.rs`.
- Iterate at least 10 fixtures from `assets/test/v105-real-characters/` covering the shapes in `docs/v105-item-format.md` Appendix B:
  - `NormalShortBows.d2s` (simple-only)
  - `m_one.d2s`, `m_skillers.d2s` (magic/rare without durability — Q2 verification)
  - `Liu.d2s`, `Shiva.d2s`, `Xen.d2s` (set/unique)
  - `UberSlapper.d2s` (runeword + sockets — Q7 verification)
  - `BarbHelms.d2s`, `Circlets.d2s`, `Claws.d2s` (class-specific)
  - `Throws.d2s`, `NormalJavs.d2s` (stackables)
  - `Locker.d2s` (max-shape)
- For each: `Save::parse(&bytes, Strictness::Strict)` must succeed and produce `parsed.issues.is_empty()`.
- Also assert: `parsed.save.items_v105.is_some()`, `items_v105.player.items` has at least one item, every item in every list has a known type code (i.e. `excel::lookup_base(type_code).is_some()` or it is a quest/jewel type from `itemtypes`).
- This is the **gate** for task 003 done.
- **Output**: integration test passes.

### Step 11 — Open Questions resolution
After Step 10 passes, run a one-time inspection (notes in commit message, no production code change) for:
- **Q2**: confirm bit alignment on rare rings/amulets in `m_one.d2s`. Document outcome inline in `item.rs` next to the `v105_unknown_after_durability` read.
- **Q7**: confirm 4-bit padding value on Delirium-or-other runeword in `UberSlapper.d2s`.
- **Q8**: confirm `realm_data_flag` is always 0 (warn-only).
If Step 10 passes for all 10+ fixtures with strict-clean, Q2/Q5/Q6 are implicitly resolved.

### Step 12 — Code style + lint pass
- `cargo fmt`, `cargo clippy --all-targets`. Address warnings.
- All public types have `///` doc comments per `AGENTS.md`.
- New module doc comment on `items::v105` explains the v105-only scope and references `docs/v105-item-format.md`.

## Key Risks & Mitigations
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Q2 wrong (the §6.3 unknown bit is durability-gated, not always read) | Medium | High — every non-durability extended item misaligns | Smoke test in Step 10 includes rare rings/amulets; if it fails, swap to durability-gated and retest. The doc explicitly flags this ambiguity. |
| Q5 wrong (a v105 stat width differs from excel) | Low | High — silent property misread on one stat | Excel is the source of truth (task 002); strict parse + zero issues across 10+ fixtures provides empirical coverage. |
| Huffman tree differs in v105 | Low | Critical — every type code wrong | First fixture parse will fail loudly (unknown type code from `lookup_base`). Mitigation: assert `lookup_base(type_code).is_some()` for every parsed item in Step 10, fail the test on first miss. |
| Multi-stat groupings (§7.2) incomplete | Medium | Medium — affected items misalign | Const table is small and well-documented; Step 10 fixtures include damage-bearing items. Failure surfaces as strict-parse failure on a single fixture. |
| Property list with `encode == 2/3` not in any tested fixture | Medium | Low — code path untested | Add explicit unit tests with synthetic bytes for encode 2 / encode 3 stats (IDs 198, 204) regardless of fixture coverage. |
| `lf` trailer payload ≠ `01 00 6C 66 00 00` exactly in some fixture | Medium | Medium — opaque preservation suffices but the warn is noisy | Preserve raw bytes regardless; treat magic mismatch as warn-only `ParseIssue`. |
| Recursive socket parse hits arbitrary depth on bad input | Low | Medium — stack blowup | Bound recursion depth to 1 (§10 says nested sockets are not expected); deeper nesting returns `ParseHardError`. |
| Lax-mode failure semantics differ from existing sections | Medium | Low — surprising behavior | Document in module doc comment that intra-item failures bail the items section in Lax mode (no resync). Matches `attributes` semantics. |
| `Save` dual-storage of items (Placeholder + items_v105) confuses callers | Low | Low — transitional only | Doc comment on both fields; cleaned up in task 006. |

## Assumptions
- Task 002's excel API (`excel::itemstatcost::by_id`, `excel::lookup_base`, etc.) is stable and complete for this task's needs. Confirmed via direct read of `src/excel/mod.rs` and `src/excel/itemstatcost.rs`.
- The v105 Huffman tree is identical to v99 (per `docs/v105-item-format.md` §4.3 / `v99-v105-details.html`). Hardcoded tree values come from `docs/item-details.html` §10.
- `expansion_type` is correctly populated on `Save` by the time `parse_items_tail` is called. Confirmed: `format::decode` sets it before reaching the items section.
- `mercenary.is_hired()` accurately reflects the `jf` inner-list presence. Confirmed by the existing `V105_EMPTY_ITEMS_EXPANSION_MERC` constant in `src/items/mod.rs`.
- The existing `ParseHardError { message: String }` shape is acceptable for new error variants; no need to add structured codes in this task. Codes can be inferred from `IssueKind` in the Lax path.
- Adding a sibling `items_v105: Option<...>` field on `Save` in this task (Step 9) is acceptable as a transitional shape that task 006 will reorganize. If reviewer feedback rejects this, fall back: store in a `static thread_local` set during decode for the smoke test only — but that is much worse, so prefer the field.

## Acceptance Criteria Mapping
Feature 2 acceptance criteria from MASTERPLAN:

- [ ] Every `.d2s` fixture in `assets/test/v105-real-characters/` parses successfully in Strict with zero issues — **Step 10** (smoke covers ≥10; full coverage is task 008's golden test, but this task's smoke must not regress that goal).
- [ ] All item shapes recognized — **Steps 5, 6, 8** unit tests + Step 10 fixtures cover simple/extended/magic/rare/crafted/set/unique/runeword/ear/gold/gems/runes/charms/jewels/sockets/inserted/ethereal/personalized/class-specific.
- [ ] Property/affix list decoded using bit widths from embedded stat-cost table; terminates on `0x1FF` — **Step 4**.
- [ ] Four item lists handled with `JM` + `u16` count framing (player, corpse, mercenary, golem) — **Step 6**.
- [ ] Each item carries location metadata (storage, grid coords, equip slot) — **Step 2** (`ItemLocation` field on `ItemHeader`).

Feature 1 acceptance criteria touched (already satisfied by task 002, but consumed here):

- [ ] Excel tables embedded — consumed via `crate::excel::*`; no runtime FS reads added.

## Definition of Done
1. `cargo build` clean; `cargo fmt` + `cargo clippy --all-targets` produce no new warnings.
2. `cargo test` — every existing test still passes (V99 path untouched).
3. `cargo test --test v105_items_parse` (new) — passes for ≥10 representative fixtures, all in Strict mode, all with `parsed.issues.is_empty()`.
4. Unit tests in `src/items/v105/tests.rs` cover, with synthetic bytes:
   - one simple item (e.g. a healing potion)
   - one extended-magic item with one property
   - one extended-rare item with all 6 affix slots filled
   - one extended-set item with `set_bonus_mask` popcount > 0
   - one extended-unique item
   - one runeword item (`given_runeword == 1`)
   - one ear item
   - one personalized item
   - one item with sockets containing 2 inserted simple children
   - one ethereal item (durability path)
   - one item with the `encode == 2` chance-on-hit property (stat 198)
   - one item with the `encode == 3` charges property (stat 204)
   - one item with a multi-stat grouping property (stat 48 fire damage min/max)
   - the v105 quantity trailer present (`Some(n)`) and absent (`None`) cases
5. The `parse_items_tail` parser successfully round-trips the five `V105_EMPTY_ITEMS_*` byte constants from `src/items/mod.rs` to empty `ItemsTail` and back to byte-identical (re-encode is task 004; for now, assert structural emptiness only).
6. New `IssueKind` variants are visible in `pub` API and marked via the existing `#[non_exhaustive]` attribute.
7. New module documentation (`src/items/v105/mod.rs`) cross-references `docs/v105-item-format.md` and explicitly states the v105-only scope plus the lax-mode no-resync behavior.
8. Open Questions Q2, Q7, Q8 outcomes documented as code comments at the relevant read sites.
9. No changes to `CHANGELOG.md` in this task — that is task 009's job. (Defer to avoid duplicate entries.)
