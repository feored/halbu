# Task 001 Implementation Plan: Internal v105 Item-Format Reference Document

## Summary
This task produces the authoritative internal reference for the Diablo II: Resurrected v105 `.d2s` item bitstream layout, synthesized from (a) the existing HTML reference docs under `docs/`, (b) the v105 Excel data tables under `assets/excel/v105/`, and (c) direct verification against the real binary fixtures under `assets/test/v105-real-characters/`. The deliverable is a single Markdown file at `docs/v105-item-format.md` that all subsequent tasks (002 embedded data, 003 parser, 004 encoder, 005 `.d2i` stash, 006 model integration, 007 move API) will consult as ground truth. It covers MASTERPLAN Feature 1 (specifically the third acceptance criterion: the documented internal reference under `docs/`). It is a research/analysis task only; no Rust source code is changed.

## Scope

- **In scope**:
  - Read and reconcile the existing HTML docs in `docs/` (treating them as references that may be misleading).
  - Read the relevant Excel tables in `assets/excel/v105/` to extract concrete bit-width and code-mapping data (especially `itemstatcost.txt` for property bit widths, `itemtypes.txt` for the type hierarchy, `bodylocs.txt` for equip slots, affix tables for the magic/rare/crafted bit fields, `uniqueitems.txt` / `setitems.txt` / `runes.txt` / `gems.txt` / `properties.txt` for typed lookups).
  - Verify each layout claim against at least one binary fixture in `assets/test/v105-real-characters/` using ad-hoc, throwaway tooling (small local Rust binary in a scratch directory or hex dumps via `xxd`/`hexdump`); throwaway code is **not** committed.
  - Produce `docs/v105-item-format.md` with the section structure listed below.
  - Document explicit deltas vs. the HTML references (where the existing docs are wrong, ambiguous, or version-specific to v99).
- **Out of scope**:
  - Any change to `src/`, `tests/`, `Cargo.toml`, or other crate-level files (per MASTERPLAN: parser is task 003, encoder 004, etc.).
  - Embedding the Excel data into the crate (task 002).
  - V99 item-format documentation (MASTERPLAN "Out of Scope": V99 stays raw-bytes).
  - Modeling `npcs` (out of scope per MASTERPLAN).
  - Documenting the move-item algorithm/validation rules (task 007's plan will own that).

## Technical Approach

The reference is produced by **triangulation** rather than trusting any single source:

1. **HTML docs** give a structural skeleton and historical context but predate D2R v105 and contain known errors (e.g. quest section sizes, attribute layouts that already differ from what `src/attributes/` and `src/character/v105.rs` actually do). Use them only as a starting outline.
2. **Excel tables** are the authoritative source for everything that is data-driven in the bitstream — most importantly, `itemstatcost.txt` columns `Save Bits`, `Save Add`, `Save Param Bits`, `Encode`, and the per-stat grouping used to fold related stats into a single property record (e.g. min/max damage, +skill tab triplets, charges quadruplets). Extract these into the doc as tables so task 003 has a single source to translate from.
3. **Binary fixtures** are the final arbiter. The `v105-real-characters/` fixtures were chosen to span every item shape (note the naming: `Normal*`/`Excep*`/`Elite*` for tier coverage, class-specific weapons like `BarbHelms`, `NecroHeads`, `DruidPelts`, `Circlets`, `Claws`, `Throws`, `Belts`, `Locker.d2s` for full storage, `m_one`/`m_two`/`m_skillers` for charm/jewel coverage, `UberSlapper` and named characters for runeword/unique coverage, plus two `.d2i` shared stashes). Confirm each documented bit field by hand-decoding at least one item from a chosen fixture and recording the offsets.

The reference document is **format-only**: no Rust code, no API design, no parser strategy. It is structured so that task 003 can implement a parser by walking the document top-to-bottom.

Existing patterns from the codebase that the doc must respect:
- The codec uses an LSB-first bitstream with byte-level `BytePosition` cursor + `read_bits`/`write_bits` helpers in `src/utils.rs`. Bit-order conventions (which bit of a byte is read first, how multi-byte fields are assembled) must match what those helpers do — describe the bitstream in those terms, not in abstract MSB-first terms that the existing HTML docs sometimes use.
- Section framing uses ASCII magic markers (`JM` for items) followed by little-endian counts, mirroring how `quests`/`waypoints`/`attributes` are framed in their respective modules.

## Prerequisites & Dependencies

- **Prior tasks**: none. This is task 001 and the first task in the plan.
- **External setup**: a hex viewer (`xxd`, `hexdump -C`) and optionally a scratch Rust binary placed *outside* the crate (e.g. `~/scratch/d2s-dump/`) for ad-hoc decoding. Nothing committed.
- **Reading prerequisites** before writing:
  - `AGENTS.md` (project conventions, especially the items-section "not modeled" caveat).
  - `NOTES.md` (existing reverse-engineering notes — confirm none contradict, supersede where stale).
  - `src/utils.rs` (for the exact bit/byte read semantics the doc must align with).
  - `src/format/decode.rs` and `src/format/layout.rs` (to see where the items section starts in the byte stream and what surrounds it).
  - `src/character/v105.rs` and `src/character/mercenary/` (for the mercenary header layout that precedes the merc item list).

## Implementation Steps

### Step 1 — Inventory and read all source material
- Read all four HTML docs in `docs/` end-to-end. Note known-bad sections explicitly so the new doc can supersede them.
- Read the relevant Excel files in `assets/excel/v105/`:
  - `itemtypes.txt`, `itemstatcost.txt` (if present — confirm exact filename; otherwise the bit widths come from `properties.txt` joined with the stat list), `armor.txt`, `weapons.txt`, `misc.txt`, `bodylocs.txt`, `magicprefix.txt`, `magicsuffix.txt`, `rareprefix.txt`, `raresuffix.txt`, `uniqueitems.txt`, `sets.txt`, `setitems.txt`, `runes.txt`, `gems.txt`, `properties.txt`, `automagic.txt`, `qualityitems.txt`, `lowqualityitems.txt`.
- Confirm presence/absence of `itemstatcost.txt` under `assets/excel/v105/` with `glob` — if missing, note this in the doc and identify which file actually carries `Save Bits` / `Save Add` / `Save Param Bits` for v105 (the Excel set may have it under a different name in the D2R drop).
- Read `src/utils.rs`, `src/format/decode.rs`, `src/format/layout.rs`, `src/character/v105.rs`, `src/character/mercenary/mod.rs`, `src/items/` (the placeholder), and `tests/save_format_golden.rs` to understand exact framing context.
- **Output**: working notes (kept in scratch, not committed); list of "claims to verify against fixtures".

### Step 2 — Pick a verification fixture set and hand-decode
Choose at least one fixture per item-shape category to verify the doc against. Suggested mapping (one is enough per category but more strengthens confidence):
- Simple items only: `NormalShortBows.d2s` or any `Normal*` fixture with low item complexity.
- Magic / rare / crafted: `m_one.d2s`, `m_two.d2s`, `m_skillers.d2s`.
- Set / unique: pick from named-character fixtures (`Liu`, `Shiva`, `Xen`, `UberSlapper`).
- Runeword + sockets + inserted items: `UberSlapper.d2s` or any fixture whose extracted items include sockets.
- Class-specific: `BarbHelms.d2s`, `NecroHeads.d2s`, `DruidPelts.d2s`, `Circlets.d2s`, `Claws.d2s`, `NormalAmazonWep.d2s`.
- Throwables / quantity items: `Throws.d2s`, `NormalJavs.d2s`.
- Belts / boots / gloves storage variants: `Belts.d2s`, `Boots.d2s`, `Gloves.d2s`.
- Maximum storage shape (full inventory + cube + stash + equipped + belt): `Locker.d2s`.
- Mercenary items: any fixture that has a hired mercenary (identify by inspecting the mercenary section header — at least one of the named-character fixtures).
- Corpse/golem item lists: typically empty, but verify the framing bytes are present and zero-counted in any fixture.
- Shared stash (`.d2i`): both `SharedStashSoftCoreV2.d2i` and `ModernSharedStashSoftCoreV2.d2i` — note any header differences between the two.

For each chosen fixture, run a scratch decoder or hex dump to confirm: total `JM`-list count, the `JM` header bytes for each item, the simple-header bit positions, the extended-header transitions, the property list terminator at `0x1FF`, and the section boundaries between player / corpse / mercenary / golem item lists.

- **Output**: a small per-fixture annotation table (lives inside the final doc as an appendix, optional) listing offset → field name → value for one example item per category.

### Step 3 — Draft `docs/v105-item-format.md` with the section structure below
Create the file at `docs/v105-item-format.md`. Required top-level sections:

1. **Overview & Scope** — v105 only; references but supersedes HTML docs; explains the "JM"-framed list-of-lists structure with player / corpse / mercenary / golem item lists. Calls out which HTML doc sections are wrong or v99-only.
2. **Bitstream Conventions** — exact LSB/MSB ordering, byte-stride, field alignment rules, signed-vs-unsigned encoding, and how multi-bit fields cross byte boundaries. Aligned with `src/utils.rs` semantics.
3. **Item-List Framing** — the four item lists, in order. For each: the `JM` magic, the `u16` little-endian count, the position within the `.d2s` byte stream (relative to the end of the character / quest / waypoint / npc / stat / skill / mercenary-header sections), and the empty-list encoding.
4. **Item Header — Simple Items** — every bit field of the fixed-width simple-item portion: `JM` magic, identified-flag, socketed-flag, new-flag, ear-flag, simple-flag, ethereal-flag, personalized-flag, runeword-flag, all reserved bits with their observed values, location/equipped/grid coordinates/storage-container fields, item-code (3 ASCII characters + trailing space → 4 bytes), and gold/ear specializations. Each field documented with: bit offset from the start of the item, bit width, encoding, observed-in-fixture example.
5. **Extended Item Header** — for `simple == 0` items: number-of-sockets-filled, unique item ID, item level, quality (enum table), variable-graphic flag + value, autoaffix flag + value, and all the quality-conditional sub-blocks (low-quality index, magic-prefix/suffix IDs, set ID, rare prefix/suffix IDs + 6-affix variable-length tail, unique ID, runeword ID + parameter, personalized name as 7-bit-per-char string, tome-of-id flags, realm-data flag and reserved bits).
6. **Type-Specific Sub-Blocks** — armor (defense rating, max/current durability, indestructible flag); weapons (max/current durability, indestructible flag); stackable (quantity); body-armor (defense + durability); class-specific autoaffix sub-fields; the "set list mask" 5-bit flag block that gates which set bonus property lists follow.
7. **Property List Encoding (Stat-Cost Driven)** — the document's largest section. Defines:
   - The 9-bit stat-id field, terminator value `0x1FF`.
   - How `Save Bits`, `Save Add`, `Save Param Bits` from the stat-cost table map to bit widths and the bias subtracted/added for sign.
   - The grouping rules where one stat-id consumes additional adjacent stats as sub-fields (min damage / max damage pairs, all-resist triplets, +N to skill-tab triplets `(class, tab, level)`, charges quadruplets `(skill, level, current, max)`, +N to single-skill `(class, skill, level)` etc.). Provide a complete v105 group table extracted from the Excel data, not a guess.
   - The five property-list slots that an item can have, in encoding order: base props, set-bonus props (gated per set-mask bit, up to 5 lists), runeword props. Document the exact framing between successive lists.
8. **Special Item-Type Encodings** — gold (small + large gold variants), ears (class + level + 7-bit name), books/scrolls (specid), gems/runes/jewels embedded as inserted items with their own full item record, indestructible/ethereal interactions with durability fields.
9. **Location / Equip-Slot Encoding** — the location enum (stored vs. equipped), the equip-slot enum keyed to `bodylocs.txt`, the storage-container enum (inventory, cube, stash, belt, equipped, mercenary inventory, shared-stash sub-tab), grid-coordinate widths and the "belt slot index encoded in column" trick. Document the row/column bit widths and which container imposes which bounds — sourced from `belts.txt`, fixture observations, and the existing HTML doc cross-checked.
10. **Inserted Items (Sockets)** — how socketed gems/runes/jewels are appended after the parent item (count taken from the parent's filled-sockets field), the framing recursion, and the constraint that inserted items are themselves simple-form (verify against fixtures).
11. **Mercenary, Corpse, and Golem Item Lists** — for each of the three trailing lists, document the full framing, the relationship to the mercenary header / corpse-count fields earlier in the save, and the empty-list bytes. Resolve the "mercenary hire toggle is a blocking compatibility issue" note from `AGENTS.md` by documenting exactly which bytes need rewriting and where the mercenary item list lives relative to the mercenary header.
12. **Shared Stash (`.d2i`) Variant** — the differences from `.d2s`: header magic and version, the absence of character/quest/waypoint sections, the page/tab structure if any, and how the `JM` item list is framed. Document any differences between the two `.d2i` fixtures (legacy vs. "Modern...V2").
13. **Cross-References to Existing HTML Docs** — explicit table: claim from each HTML doc → status (correct / outdated / wrong) → corrected statement in this doc.
14. **Open Questions / Unverified Claims** — anything that could not be verified from the fixtures available; flagged so task 003 knows where to be defensive.
15. **Appendix A — Stat-Cost Bit Layout Table** — the extracted, v105-specific table from the Excel data.
16. **Appendix B — Per-Fixture Spot-Checks** — the offset→field annotations from Step 2 (optional, kept short).

Each documented field must specify: name, bit width, encoding (unsigned / signed-with-bias / enum / ASCII / 7-bit-string), source (HTML doc / Excel column / fixture-verified), and a one-line note when behavior is conditional.

- **Output**: `docs/v105-item-format.md` written to disk.

### Step 4 — Self-review against acceptance criteria
- Walk the doc top-to-bottom and confirm every Feature-1 / Feature-2 / Feature-4 acceptance criterion that touches "format" has a corresponding section answering "where in the bitstream is this represented".
- Confirm the doc enables a reader to write a parser without consulting the HTML files again.
- Confirm every claim is either tagged "Excel-derived" or "fixture-verified" or explicitly flagged as inherited-from-HTML-unverified in the Open Questions section.

## Key Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| The HTML docs are subtly wrong about D2R v105-specific changes (e.g. a new flag bit, an extended quality enum value) and the new doc inherits the error | Medium | High — task 003 builds a broken parser | Every layout claim must be fixture-verified against `v105-real-characters/`. If a fixture cannot be hand-decoded to match a claim, mark it "unverified" rather than asserting it. |
| `itemstatcost.txt` is absent or named differently in `assets/excel/v105/` | Medium | High — without bit widths, property decoding cannot be specified | Confirm filename in Step 1; if absent, identify which file in the v105 Excel drop carries the same columns and document the source of truth there. The MASTERPLAN already lists this as conditional ("if present"). |
| Some item shapes from the acceptance criteria (e.g. crafted, ethereal, personalized) are not present in any fixture | Low | Medium | Cross-check with the HTML doc and Excel data; document as "format-derived but not fixture-confirmed" in Open Questions, and recommend task 008 add a fixture for any gap. |
| The `.d2i` shared-stash format differs between the two fixtures in ways that imply a pre-V2 vs. V2 split that can't both be supported by one model | Low | Medium | Document both layouts side-by-side and let task 005 decide whether to support both or only the modern V2 form. |
| Mercenary-item-list location relative to the mercenary header is mis-described in HTML docs and the resolution path for the `AGENTS.md` limitation is wrong | Medium | High — blocks Feature 3 acceptance criterion 3 | Verify in a fixture with a hired mercenary (any of the named-character fixtures); document the actual byte layout including the magic header that precedes the merc item list. |
| Bit-order convention in HTML docs is opposite to what `src/utils.rs` does | Low | Medium | Section 2 (Bitstream Conventions) is written by reading `src/utils.rs` first, then translating any HTML-doc bit-numbering to match. |

## Assumptions

- "Internal reference" means a single Markdown file under `docs/`; the file name `docs/v105-item-format.md` (suggested in the task notes) is acceptable.
- Throwaway verification tooling (scratch Rust binary, hex dumps) is not committed; only the Markdown deliverable is committed.
- Excel files in `assets/excel/v105/` are the v105 D2R drop and are authoritative for v105 (per MASTERPLAN Feature 1, they are intended to be embedded by task 002).
- The HTML docs in `docs/` may be removed or marked superseded later (task 009) but are kept in place during task 001; the new doc supersedes them by reference, not by deletion.
- v99 item layout is intentionally undocumented here (out of scope per MASTERPLAN).

## Acceptance Criteria Mapping

This task only directly satisfies one MASTERPLAN acceptance criterion:

- [ ] Feature 1, criterion 3: "A documented internal reference describing the v105 item bitstream layout is captured under `docs/` (synthesized from existing docs + fixture analysis + excel data; supersedes misleading sections of the existing docs)." → satisfied by Step 3 producing `docs/v105-item-format.md` and Step 4 self-review.

The doc additionally **enables** but does not itself satisfy:
- Feature 1 criteria 1 & 2 (embedded data, no runtime FS) — task 002 implementation.
- Feature 2 (parser) — task 003 will reference this doc.
- Feature 3 (encoder) — task 004 will reference this doc, especially Section 11 for the mercenary tail resolution.
- Feature 4 (`.d2i` stash) — task 005 will reference Section 12.

## Definition of Done

- `docs/v105-item-format.md` exists, contains all 16 sections listed in Step 3 (or explicitly marks any section "N/A" with reason), and is committed.
- Every documented field is tagged with its provenance (`Excel`, `fixture-verified`, `HTML-inherited-unverified`).
- Section 13 contains an explicit cross-reference table to the four existing HTML docs flagging which of their statements are now superseded.
- Section 14 (Open Questions) is non-empty if and only if there are genuinely unresolved claims; an empty section indicates the document is complete and self-sufficient.
- A reader unfamiliar with the codebase can use the doc alone (plus `src/utils.rs` for bit-order semantics) to begin implementing the parser in task 003.
- No changes outside `docs/` are made by this task. `cargo test`, `cargo fmt`, `cargo clippy` are not run because no Rust code changed; this is verified by `git status` showing only `docs/v105-item-format.md` (and possibly nothing else) staged.
