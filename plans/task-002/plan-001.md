# Task 002 Implementation Plan: Embedded v105 Excel Lookup Tables

## Summary
Build a self-contained, internal Rust module that owns the v105 Excel reference tables required by the v105 item parser/encoder/move-API (tasks 003–007). Tables are embedded at compile time via `include_str!` (no runtime filesystem dependency), parsed lazily from tab-separated text into typed row structs, and exposed through small typed lookup functions keyed by the values that the bitstream actually carries (3-char item code, 9-bit stat ID, body-location index). This task delivers Feature 1 ("Embedded v105 Item Reference Data") in its data-table half — the v105 bitstream reference document is already produced by task 001.

## Scope

**In scope**
- A new module (suggested `src/excel/`) that owns:
  - Tab-separated text parser (no external crate; rejects/skips `Expansion` section-marker rows).
  - Embedded `include_str!` of the **minimum viable** Excel files for the move-items goal: `armor.txt`, `weapons.txt`, `misc.txt`, `itemtypes.txt`, `itemstatcost.txt`, `bodylocs.txt`.
  - Typed row structs holding only the columns consumed downstream (small, Serde-derived).
  - Lazy `OnceLock`-backed lookup tables (by primary key) built on first access.
  - `pub(crate)` lookup API consumed by tasks 003–007.
- Unit tests that verify well-known rows resolve correctly (e.g. `hax` → "Hand Axe", `cap` → "Cap", stat 31 `armorclass` → `save_bits=11, save_add=10`, body-loc 1 → `head`).

**Out of scope** (explicitly deferred)
- All item parsing/encoding logic — that is task 003/004.
- Affix display tables (`magicprefix.txt`, `magicsuffix.txt`, `rareprefix.txt`, `raresuffix.txt`) — only needed to *render* item names; the bitstream carries raw IDs that round-trip without lookup. Move-items does not need names. Defer until a downstream task explicitly requires it.
- Set/unique tables (`uniqueitems.txt`, `setitems.txt`) — same reasoning: raw IDs round-trip; only needed for display. Defer.
- Runeword table (`runes.txt`) — `given_runeword` is a single bit + 12-bit ID + 4-bit padding (preserved verbatim per `docs/v105-item-format.md` §5.3). No lookup needed for parse/encode/move. Defer.
- `gems.txt` — gems are themselves simple items written via §4 of the spec; their per-stat properties are only relevant if we *interpret* socket bonuses. Move-items does not need this. Defer.
- `properties.txt` — `itemstatcost.txt` alone drives bitstream decoding (per docs §7.1, Appendix A). Defer.
- Per-table indexes the parser doesn't need (e.g. CharsiMin..MalahMagicLvl vendor columns).
- Item parser, item model, encoder, move-API, save integration — tasks 003–007.
- Adding any new Cargo dependency.

## Technical Approach

**Module placement.** New module `src/excel/` (sibling to `src/items/`). Rationale: the data is shared by tasks 003 (parse), 004 (encode), 005 (`.d2i`), and 007 (move validation). Putting it under `src/items/data/` would imply ownership by the items module, but the same tables (e.g. `itemtypes.txt` for `bodylocs`) cross-cut later work. `src/excel/` matches the asset directory name and signals "ground-truth game data, not item logic". Module is declared `pub(crate) mod excel;` from `lib.rs` (not in the public API surface — downstream item types will re-export only what's needed via their own public interfaces in later tasks).

**Parsing strategy.** A tiny hand-written tab-separated parser, ~30 LOC, with these rules (verified against the actual files):
- Header row at line 1; remaining lines are data.
- Fields are separated by literal `\t`. Empty fields are valid (preserved as empty `&str`).
- A row whose first cell equals exactly `Expansion` is a Blizzard editor section delimiter and must be skipped. (Confirmed: `weapons.txt`, `armor.txt`, etc. carry these markers between Classic and Expansion sections.)
- Trailing `\r` (CRLF files) is stripped. Empty lines are skipped.
- Column lookup is **by header name** (not by index): a `HashMap<&str, usize>` is built once per file at first access. This insulates us from column-position drift between MPQ versions and matches the spirit of how Blizzard tools read these files.

**Per-table strategy.** For each embedded table:
1. A `RowXxx` struct with only the columns the rest of the codebase consumes. All fields are owned types (`String`, `u16`, `Option<u8>`, etc.) parsed once at first access.
2. A `pub(crate) fn all_xxx() -> &'static [RowXxx]` that lazily parses the embedded text on first call (`std::sync::OnceLock<Vec<RowXxx>>`).
3. A `pub(crate) fn lookup_xxx(key) -> Option<&'static RowXxx>` backed by a second `OnceLock<HashMap<Key, usize>>` mapping the natural key to a slice index.

**Lazy initialization.** `std::sync::OnceLock` (stable since 1.70). No `lazy_static` / `once_cell` dependency. Each table has its own `OnceLock`; first call into any lookup pays a one-time parse cost (microseconds; tables are <500 rows each).

**Numeric parsing.** Empty cells are common in these files. Helper: `fn parse_u32_or_default(s: &str) -> u32` that returns 0 for empty, parses otherwise. For `Save Bits` / `Save Add` / `Save Param Bits`, empty means "field not present" — represent as `u8` with 0 default (matching how the bitstream parser uses them: `if save_param_bits > 0`).

**No serde for parsing.** Internal row structs `derive(Debug, Clone)` only. They are not serialized over the wire. Item-model types in later tasks will *reference* these (by code/ID) and those types will derive `Serialize`/`Deserialize` per project convention. Per `AGENTS.md`: serde derives are required on **public model types**; these row structs are `pub(crate)` data tables, not public model types.

**Build-time vs. runtime parsing.** Considered a build script that converts excel → `phf` map at compile time. Rejected: adds `phf`/`phf_codegen` dev-dependency or a build script, both of which violate the "minimal-dependency by design" constraint in the task notes. Lazy runtime parsing of <2000 short rows total is well under 1 ms on first access; later accesses are `HashMap` lookups. Acceptable.

**Compiled-in size.** Total embedded text for the six MVP tables: ~600 KB raw text (weapons.txt is the largest at ~250 KB; armor.txt ~80 KB; misc.txt ~80 KB; itemtypes.txt ~30 KB; itemstatcost.txt ~150 KB; bodylocs.txt <1 KB). Acceptable for a save-editing library; equivalent to embedding a single small image asset.

## Prerequisites & Dependencies

- **Task 001 (complete):** `docs/v105-item-format.md` is the canonical source of truth for which columns map to which bitstream behaviour. Every column we extract is justified by a reference to a section of that document (cited inline in the per-table steps below).
- **No new Cargo dependencies.** Verified `Cargo.toml`: existing deps are `bit`, `serde`, `serde_with`, `unicode-script`, `unicode-segmentation`. We use `std::sync::OnceLock` and `std::collections::HashMap` from the standard library only.
- **Asset files already present** under `assets/excel/v105/`. No file additions needed.

## Implementation Steps

### Step 1 — Module scaffolding and tab-separated parser
Create:
- `src/excel/mod.rs` — module root with `pub(crate)` re-exports of each table's row type and lookup fn.
- `src/excel/parser.rs` — tiny TSV parser.

`parser.rs` exposes:
```rust
pub(super) struct Tsv<'a> {
    pub headers: Vec<&'a str>,
    pub rows: Vec<Vec<&'a str>>,
}

pub(super) fn parse_tsv(text: &'static str) -> Tsv<'static>;
pub(super) fn col_idx(headers: &[&str], name: &str) -> usize; // panics with clear msg if missing — table layout is a compile-time constant from our embedded asset, so absence is a bug, not a runtime error.
pub(super) fn parse_u32(s: &str) -> u32;        // empty/blank → 0
pub(super) fn parse_u8(s: &str) -> u8;          // empty/blank → 0
pub(super) fn parse_bool_01(s: &str) -> bool;   // "1" → true, anything else → false
```

Behaviour:
- Strips trailing `\r` per line.
- Skips blank lines.
- Skips rows whose first cell equals `"Expansion"`.
- Returns rows with fields by-reference into the embedded `&'static str` (zero-copy where possible; subsequent typed parsing copies to owned `String` fields).

Module declaration in `src/lib.rs`: add `pub(crate) mod excel;` (place near `mod items;`). Also add a brief `///` doc-comment on the module per project convention.

Files modified: `src/lib.rs`. Files created: `src/excel/mod.rs`, `src/excel/parser.rs`.

### Step 2 — `bodylocs.txt` (smallest table; serves as the template)
File: `src/excel/bodylocs.rs`. Embeds `assets/excel/v105/bodylocs.txt`.

Columns: `Body Location`, `Code`. The implicit row index (1-based, with row 0 = `None`) is the `equipped_id` value from `docs/v105-item-format.md` §9.2. Rows where `Code` is blank (the `None` row at index 0) are kept (as `code: ""`) so that `lookup_by_id(0)` returns the explicit "no slot" row.

```rust
#[derive(Debug, Clone)]
pub(crate) struct BodyLoc {
    pub id: u8,            // row index, 0..=10
    pub name: String,      // "Head"
    pub code: String,      // "head", or "" for id=0
}

pub(crate) fn all() -> &'static [BodyLoc];
pub(crate) fn by_id(id: u8) -> Option<&'static BodyLoc>;
pub(crate) fn by_code(code: &str) -> Option<&'static BodyLoc>;
```

Used by §9.2 `equipped_id` decoding. Note: spec §9.2 lists 12 slots (with weapon-swap II at 11/12), but `bodylocs.txt` only enumerates rows 0–10. Slots 11–12 are documented as reusing `rarm`/`larm` codes with an alt index; this is item-parser concern (task 003), not a data-table concern. We expose what's in the file; task 003 handles the swap mapping.

### Step 3 — `itemstatcost.txt` (most critical for parsing)
File: `src/excel/itemstatcost.rs`. Embeds `assets/excel/v105/itemstatcost.txt`.

Per `docs/v105-item-format.md` §7.1 / Appendix A, the columns we consume are:
- `Stat` — name (string, debugging/tests)
- `*ID` — the 9-bit bitstream stat ID (primary key, `u16` since IDs reach 369)
- `Save Bits` (`u8`) — value field width
- `Save Add` (`u32`) — bias (per row 1: strength has Save Add=32, fits in u32)
- `Save Param Bits` (`u8`) — optional param field width (0 if blank)
- `Encode` (`u8`) — 0/2/3/4 dispatch for §7.4–§7.6 (default 0 if blank)

```rust
#[derive(Debug, Clone)]
pub(crate) struct StatCost {
    pub id: u16,
    pub name: String,
    pub save_bits: u8,
    pub save_add: u32,
    pub save_param_bits: u8,
    pub encode: u8,
}

pub(crate) fn all() -> &'static [StatCost];
pub(crate) fn by_id(stat_id: u16) -> Option<&'static StatCost>;
```

**Sentinel handling:** `0x1FF` (511) is *not* a row in this table; it is the property-list terminator. `by_id(511)` returns `None`. The item parser checks for the terminator *before* calling this lookup — documented in a `///` comment on `by_id`.

**Determinism note:** the file has 369 rows (per task-001 spec §14 Q6); we do not verify the count in code (would be brittle if Blizzard ships a hotfix), but the unit tests in Step 8 anchor specific known stat IDs.

### Step 4 — `itemtypes.txt`
File: `src/excel/itemtypes.rs`. Embeds `assets/excel/v105/itemtypes.txt`.

Per spec §4.4 (quest-item detection drives socket-count bit width), §6.4 (stackable detection), §6.1/§6.2 (armor vs. weapon vs. misc dispatch is by base table, not itemtypes — but `itemtypes` carries the type→bodyloc mapping referenced in §9.2), and §10 (inserted-item parent classification).

Primary key: `Code` (the 4-char or shorter type code, e.g. `tors`, `shie`, `axe`).

Columns we consume:
- `ItemType` — display name (debug/tests).
- `Code` — primary key.
- `Equiv1`, `Equiv2` — parent type codes for the `itemtypes` hierarchy traversal (used to answer "is this item type in category X?" by walking up the tree). Both may be empty.
- `BodyLoc1`, `BodyLoc2` — the 3-letter body-loc codes this item type can equip into. Used by move-validation (task 007) to decide whether an item fits an equip slot. Both may be empty.
- `Throwable` (bool) — used in §6.4 stackable detection.
- `Quiver` (bool) — used in §6.4 stackable detection.
- `Repair` (bool) — used to decide presence of durability sub-block in §6.2 (cross-checked: `armor.txt`/`weapons.txt` `nodurability` is the per-item override, but `itemtypes.Repair` is a useful category hint).
- `StaffMods`, `Class` — class-restricted item detection (used by move-validation in task 007).

```rust
#[derive(Debug, Clone)]
pub(crate) struct ItemType {
    pub code: String,
    pub name: String,
    pub equiv1: Option<String>,
    pub equiv2: Option<String>,
    pub body_loc1: Option<String>,
    pub body_loc2: Option<String>,
    pub throwable: bool,
    pub quiver: bool,
    pub repair: bool,
    pub staff_mods_class: Option<String>, // class restriction for staff-mod (e.g. "ama", "sor")
    pub class: Option<String>,            // hard class restriction
}

pub(crate) fn all() -> &'static [ItemType];
pub(crate) fn by_code(code: &str) -> Option<&'static ItemType>;

/// Walk Equiv1/Equiv2 chain, returning true if `code` is `ancestor`
/// or any of its (transitive) parents in the type hierarchy.
pub(crate) fn is_a(code: &str, ancestor: &str) -> bool;
```

`is_a` performs a bounded DFS up the Equiv chain (max depth ~10 in practice; guard with a small visited set to defeat any malformed loops). This is the "in category X?" query used pervasively by the parser to decide field presence (e.g. `is_a(type, "stak")` for stackable, `is_a(type, "ques")` for quest items in §4.4).

### Step 5 — `armor.txt`, `weapons.txt`, `misc.txt` (base item tables)
Three sibling files: `src/excel/armor.rs`, `src/excel/weapons.rs`, `src/excel/misc.rs`. Each embeds its respective `.txt`.

Common shape (one struct per file, but named identically per their domain):

`armor.rs`:
```rust
#[derive(Debug, Clone)]
pub(crate) struct ArmorBase {
    pub code: String,        // 3-char primary key (e.g. "cap")
    pub name: String,        // "Cap"
    pub typ: String,         // itemtypes.Code (e.g. "helm")
    pub type2: Option<String>,
    pub norm_code: String,   // normal-tier base code
    pub uber_code: String,   // exceptional-tier base code
    pub ultra_code: String,  // elite-tier base code
    pub inv_width: u8,       // grid-width for size validation (move-API)
    pub inv_height: u8,
    pub gem_sockets: u8,     // max sockets allowed (used to validate moves into sockets)
    pub no_durability: bool, // §6.2: omit current_durability if max_durability == 0; this column predicts that
    pub block: u8,           // shield-block %; presence indicates shield (cross-check vs. itemtypes "shie")
    pub stackable: bool,
}
pub(crate) fn all() -> &'static [ArmorBase];
pub(crate) fn by_code(code: &str) -> Option<&'static ArmorBase>;
```

`weapons.rs`:
```rust
#[derive(Debug, Clone)]
pub(crate) struct WeaponBase {
    pub code: String,
    pub name: String,
    pub typ: String,           // e.g. "axe"
    pub type2: Option<String>,
    pub norm_code: String,
    pub uber_code: String,
    pub ultra_code: String,
    pub inv_width: u8,
    pub inv_height: u8,
    pub gem_sockets: u8,
    pub stackable: bool,       // §6.4 quantity field gate
    pub one_or_two_handed: bool,   // "1or2handed" col → for §6.2 / move-validation two-hand collision
    pub two_handed: bool,
    pub no_durability: bool,
}
pub(crate) fn all() -> &'static [WeaponBase];
pub(crate) fn by_code(code: &str) -> Option<&'static WeaponBase>;
```

`misc.rs`:
```rust
#[derive(Debug, Clone)]
pub(crate) struct MiscBase {
    pub code: String,
    pub name: String,
    pub typ: String,            // e.g. "elix", "gem0", "rune"
    pub type2: Option<String>,
    pub inv_width: u8,
    pub inv_height: u8,
    pub stackable: bool,
    pub auto_belt: bool,        // potion auto-belt placement (move-API hint)
}
pub(crate) fn all() -> &'static [MiscBase];
pub(crate) fn by_code(code: &str) -> Option<&'static MiscBase>;
```

**Cross-table base lookup helper (in `mod.rs`):**

The bitstream gives a 4-character type code (Huffman-decoded; the trailing space is part of the code). A consumer (parser) needs to ask "what is this code?" without knowing whether it's armor/weapon/misc. Provide:

```rust
#[derive(Debug, Clone, Copy)]
pub(crate) enum BaseRef<'a> {
    Armor(&'a ArmorBase),
    Weapon(&'a WeaponBase),
    Misc(&'a MiscBase),
}

pub(crate) fn lookup_base(code: &str) -> Option<BaseRef<'static>>;
```

`lookup_base` queries the three tables in order (armor → weapon → misc). The 3-character codes are unique across the three tables (verified by spot-checks; if a duplicate appears, lookup is deterministic in the order above and a debug-assert catches collisions during the lazy build).

**Important: code normalization.** The Huffman decoder produces 4-character codes (the type-code field is fixed-width 4 chars, padded with trailing spaces — see spec §4.3). The Excel `code` columns are the *un-padded* form (typically 3 chars, e.g. `cap`, `hax`). The lookup function must `trim_end()` the input before key comparison. Document this in the `lookup_base` doc comment so task 003 knows not to pre-trim.

### Step 6 — Lazy initialization plumbing
For each table, the pattern is:

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

const TABLE_TEXT: &str = include_str!("../../assets/excel/v105/<file>.txt");

static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
static INDEX: OnceLock<HashMap<Key, usize>> = OnceLock::new();

fn rows() -> &'static [Row] {
    ROWS.get_or_init(|| { /* parse_tsv(TABLE_TEXT) → Vec<Row> */ }).as_slice()
}

fn index() -> &'static HashMap<Key, usize> {
    INDEX.get_or_init(|| {
        rows().iter().enumerate().map(|(i, r)| (r.key.clone(), i)).collect()
    })
}

pub(crate) fn by_key(k: Key) -> Option<&'static Row> {
    index().get(&k).map(|&i| &rows()[i])
}
```

For tables where the natural key is a `String` (item codes), the index is `HashMap<String, usize>` and the lookup is `by_code(code: &str)` using `index().get(code)` (works because `String: Borrow<str>`).

For `itemstatcost.rs`, the key is `u16`.
For `bodylocs.rs`, store both `Vec<Row>` and a `by_code` `HashMap<String, usize>`; `by_id` is direct slice indexing.

### Step 7 — `src/excel/mod.rs` public surface
```rust
//! Embedded v105 Excel reference tables.
//!
//! These tables are embedded at compile time (`include_str!`) from
//! `assets/excel/v105/`. They drive bit-level decisions in the v105 item
//! parser/encoder and target validation in the move-item API.
//!
//! Tables are parsed lazily on first access and cached for the process
//! lifetime. The module has no runtime filesystem dependency.
//!
//! Internal (`pub(crate)`); downstream public types in `src/items/` will
//! re-export only the names that need to leak into the public API.

pub(crate) mod parser;
pub(crate) mod bodylocs;
pub(crate) mod itemstatcost;
pub(crate) mod itemtypes;
pub(crate) mod armor;
pub(crate) mod weapons;
pub(crate) mod misc;

pub(crate) use armor::ArmorBase;
pub(crate) use bodylocs::BodyLoc;
pub(crate) use itemstatcost::StatCost;
pub(crate) use itemtypes::ItemType;
pub(crate) use misc::MiscBase;
pub(crate) use weapons::WeaponBase;

#[derive(Debug, Clone, Copy)]
pub(crate) enum BaseRef<'a> {
    Armor(&'a ArmorBase),
    Weapon(&'a WeaponBase),
    Misc(&'a MiscBase),
}

pub(crate) fn lookup_base(code: &str) -> Option<BaseRef<'static>> { /* see Step 5 */ }
```

### Step 8 — Unit tests
Each table module gets a `#[cfg(test)] mod tests;` block (per `AGENTS.md` convention). Tests use the real embedded data; no mocks.

Anchor cases (chosen to fail loudly on any column-position drift):

`bodylocs`:
- `by_id(0)` → name `"None"`, code `""`.
- `by_id(1)` → name `"Head"`, code `"head"`.
- `by_id(8)` → name `"Belt"`, code `"belt"`.
- `by_code("rarm")` → id 4 (Right Arm).

`itemstatcost`:
- `by_id(0)` ("strength") → `save_bits=8, save_add=32, save_param_bits=0, encode=0` (verified: row 1 of the file shows `Save Bits=8, Save Add=32`).
- `by_id(31)` ("armorclass") → `save_bits=11, save_add=10` (per spec §6.1).
- `by_id(72)` ("durability") → `save_bits=9` (per spec §6.2).
- `by_id(73)` ("maxdurability") → `save_bits=8` (per spec §6.2).
- `by_id(204)` ("item_charged_skill") → `save_bits=16, save_param_bits=16, encode=3` (per spec §7.6).
- `by_id(511)` → `None` (sentinel).

`itemtypes`:
- `by_code("shie")` → name `"Shield"`, body_loc1=`Some("rarm")`, body_loc2=`Some("larm")`.
- `by_code("tors")` → name `"Armor"`, body_loc1=body_loc2=`Some("tors")`.
- `is_a("shie", "armo")` → `true` (shield → armor via Equiv1).
- `is_a("axe", "weap")` → `true` (transitive via Equiv chain — verify against actual file; if Equiv1 chain doesn't reach `weap`, adjust the assertion to use the actual ancestor present in the table).

`armor`:
- `by_code("cap")` → name `"Cap"`, inv_width=2, inv_height=2, typ=`"helm"` (verified from the `armor.txt` row).
- `by_code("xap")` (exceptional cap) returns `Some(_)`.

`weapons`:
- `by_code("hax")` → name `"Hand Axe"`, inv_width=1, inv_height=3, typ=`"axe"` (verified from the `weapons.txt` row).
- `by_code("axe")` → name `"Axe"`, two_handed=`false`, inv_width=1, inv_height=3.

`misc`:
- `by_code("elx")` → name `"Elixir"`, inv_width=1, inv_height=1, typ=`"elix"`.

Cross-table:
- `lookup_base("hax ")` (with trailing space, simulating Huffman output) → `BaseRef::Weapon(_)` matching Hand Axe.
- `lookup_base("cap ")` → `BaseRef::Armor(_)`.
- `lookup_base("zzz ")` → `None`.

Module-level test in `src/excel/mod.rs`:
- All six tables can be parsed without panicking. (Single test that calls `all()` on each.)

Tests live in `src/excel/<table>.rs` `#[cfg(test)] mod tests` blocks per project convention. No new files in `tests/` — this task is a data layer; integration tests come with the parser in task 003.

### Step 9 — Lints, formatting, and docs
- Run `cargo fmt`. Confirm the lints listed in `src/lib.rs` (`unreachable_pub`, `unused_qualifications`, etc.) are clean for the new module. `unreachable_pub` is the most likely warning source — keep all new items `pub(crate)` and ensure no spurious `pub`.
- Run `cargo clippy` and address any warnings.
- `///` doc comment on every `pub(crate)` item per project convention. Module-level `//!` doc on `src/excel/mod.rs` and each sub-module explaining what bitstream decision the table drives, with a back-reference to the relevant section of `docs/v105-item-format.md`.
- Run `cargo test --doc` to confirm any inline doc examples compile (none expected for `pub(crate)` items, but the project runs doctests in CI).

## Key Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| A column we read by name is renamed in a future MPQ extraction | Low | Medium | Lookup-by-header-name (not by index) + named anchor tests for known rows fail loudly with an actionable error pointing at the missing column. Parser's `col_idx` panics with a clear message naming the missing column and the table file. |
| Excel files have inconsistent line endings (CRLF vs LF) across platforms | Medium | Low | Parser strips trailing `\r` per line. |
| Embedded text inflates binary size meaningfully | Low | Low | Total ~600 KB. Acceptable; even a single PNG asset would be larger. Documented in module docs; future tasks can shrink by selecting columns at compile time via a build script if it ever matters. |
| Lazy-init contention if multiple threads race on first access | Low | Low | `OnceLock::get_or_init` is internally synchronized; the worst case is the init closure runs twice on contention but only the first stored value is used. Idempotent and safe. |
| `Expansion` section-marker rows leak into parsed output (e.g. as a row with `code = "Expansion"`) | Medium | Medium | Parser drops any row whose first cell equals `"Expansion"`. Verified by the unit tests not finding such a code in lookups. |
| Code-key collision across `armor.txt`/`weapons.txt`/`misc.txt` causing `lookup_base` ambiguity | Very low | Low | Debug-assert in the lazy build of the cross-table index that flags any duplicate `code`. Order is documented (armor → weapon → misc) so behaviour is deterministic even if a future drop introduces a collision. |
| Misinterpreting a blank `Save Add` as 0 when the absent value should be "no bias" | Low | Medium | Documented in `StatCost::save_add` doc comment that "0 = no bias" (which is also what blank means in practice — a blank `Save Add` and a literal `0` produce the same bitstream behaviour). Spec §7.1 equation `value = raw - save_add` returns `raw` unchanged for either. |
| Adding `pub(crate)` items that the items module won't use until task 003 trips `dead_code` warnings before then | Medium | Low | Tests in this task exercise every public lookup function; `dead_code` is suppressed for items reachable from tests in the same crate. If warnings persist, gate with `#[allow(dead_code)]` on a per-fn basis with a `// TODO(task-003): consumed by item parser` comment. |

## Assumptions

- The minimum viable table set for the move-items goal is the six listed in Scope. This is justified inline against `docs/v105-item-format.md` references in each step. Rationale: bits 35–52 (the only fields a move mutates per spec §4.1) only need `bodylocs` (slot validation), `itemtypes` (slot eligibility), and `armor`/`weapons`/`misc` (item size). The remaining tables drive *parsing* of fields the move never touches; only `itemstatcost` is needed to walk the property list past the location bits to find the byte-aligned end of each item. Affix/set/unique/runeword/gem tables become necessary only when we *display* names — explicit non-goal of move-items.
- Tab is the field separator (verified against `bodylocs.txt` and the headers of all six target files).
- The `Code` column value is unique within each of `armor.txt`, `weapons.txt`, `misc.txt`, and across them. Spot-checks during implementation confirm; debug-assert catches regressions.
- `*ID` column in `itemstatcost.txt` is unique and dense enough to use a `HashMap<u16, usize>` keyed lookup (no need for a sparse `Vec<Option<...>>` of size 512).
- `OnceLock` (Rust ≥ 1.70) is available. No MSRV is declared in `Cargo.toml`; if a stricter MSRV is set in a parallel task, fall back to a single-threaded `OnceCell`-style pattern using `std::sync::Once` + a `static mut` (still no external crate). To raise this proactively if MSRV is later locked below 1.70.
- The codebase does not currently have a `src/excel/` module; verified via the `src/` directory listing in `AGENTS.md`. No naming collision.

## Acceptance Criteria Mapping

Feature 1 acceptance criteria (the data-tables half — the doc half was task 001):

- [ ] **"Excel tables required for item parsing ... are embedded at compile time via `include_str!`"** → six `include_str!` invocations across `src/excel/{bodylocs,itemstatcost,itemtypes,armor,weapons,misc}.rs` (Steps 2–5). The remaining tables called out in Feature 1's enumeration (affixes, set/unique/runeword/runes/gems) are explicitly deferred per the task notes' "load only what's required for parse/encode/move" guidance; this is documented in the `src/excel/mod.rs` module docs and in the Out-of-Scope section above so the deferral is discoverable.
- [ ] **"No runtime filesystem reads are introduced"** → verified by `grep`ing the new module for `std::fs`, `File::`, `read_to_string` (must return zero hits). All data is embedded via `include_str!`. Step 9 gate.

## Definition of Done

1. `cargo build` succeeds with no new warnings.
2. `cargo fmt --check` clean.
3. `cargo clippy --all-targets` clean for the new module (project-wide warnings unchanged).
4. `cargo test` passes; the new tests under `src/excel/` all pass against embedded real data.
5. `cargo test --doc` still passes (no regressions to existing doctests).
6. `grep -rE 'std::fs|File::open|read_to_string' src/excel` returns zero matches (sanity gate for the no-filesystem-reads criterion).
7. Manual review: every `pub(crate)` item in `src/excel/` has a `///` doc comment, and `src/excel/mod.rs` plus each sub-module carries a `//!` doc comment that cross-references the relevant section of `docs/v105-item-format.md`.
8. No new entries in `Cargo.toml [dependencies]`.
9. `CHANGELOG.md` is **not** updated by this task — Feature 8 (changelog) is task 009. (Documenting this explicitly to avoid premature changelog churn.)
