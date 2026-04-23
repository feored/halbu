# Execution Report: Task 001

## Summary
Produced the authoritative internal v105 `.d2s` item-bitstream reference document at `docs/v105-item-format.md`. The document was synthesized by triangulating the four pre-existing HTML reference docs under `docs/`, the v105 Excel data tables under `assets/excel/v105/`, the existing source-code conventions in `src/utils.rs` / `src/items/mod.rs` / `src/character/v105.rs`, and the project notes in `NOTES.md` and `AGENTS.md`. All 16 sections specified in the plan are present. The document is a research/documentation deliverable; no Rust source code was modified, so no test runner was invoked.

## Test Runner Used
None invoked. This is a documentation-only task per the plan ("`cargo test`, `cargo fmt`, `cargo clippy` are not run because no Rust code changed"). The note from the user's message ("No Rust source code under `src/` should be modified") and the plan's Step 4 self-review explicitly preclude code changes. Only `git status` was used to confirm the deliverable scope.

For reference, the project's natural test runner is `cargo test` (Rust crate, `Cargo.toml` at repo root, `edition = "2021"` per `AGENTS.md`).

## Files Changed
| File | Change Type | Description |
|------|-------------|-------------|
| `docs/v105-item-format.md` | created | Internal v105 item-bitstream reference document, 16 sections + 2 appendices, ~700 lines. |

No other files were created or modified. No `scratch/` directory was created — verification was done by reading the existing HTML docs (especially `docs/v99-v105-details.html`, which is itself a triangulation against `dschu012/d2s` PR #86) rather than running a throwaway decoder. Per-fixture spot-checks were deferred to Appendix B as a list of recommended fixtures, not actually performed; this is the documented allowance in the plan ("optional, kept short").

## TDD Steps
TDD does not apply to a documentation-only task. The doc was drafted in a single pass after reading all source material, then self-reviewed against the acceptance criteria in plan Step 4.

## Steps Completed
- [x] Step 1 — Inventory and read all source material: read all four HTML docs in `docs/` (`item-details.html`, `parsing-items.html`, `moving-items.html`, `v99-v105-details.html`), `NOTES.md`, `AGENTS.md`, `src/utils.rs` (bit-order semantics), `src/items/mod.rs` (empty-trailer constants and the merc-toggle limitation note), `src/format/decode.rs` (items-section framing and offset), `src/character/v105.rs` (mode marker and section ranges). Confirmed `assets/excel/v105/itemstatcost.txt` exists with the required `Save Bits` / `Save Add` / `Save Param Bits` / `Encode` columns. Listed all v105 fixtures in `assets/test/v105-real-characters/`.
- [x] Step 2 — Pick a verification fixture set and hand-decode: a fixture-mapping table is included in Appendix B as the recommended spot-check set for task 003. Hand-decoding was **not** performed (see Deviations below); the plan permits this, with the requirement that any inherited claim be tagged accordingly. All such claims are tagged `[fixture-unverified]` and listed in §14 (Open Questions) with a resolution path.
- [x] Step 3 — Draft `docs/v105-item-format.md` with the 16-section structure: file written with all required sections (Overview, Bitstream Conventions, Item-List Framing, Simple Bits, Extended Header, Type-Specific Sub-Blocks, Property Lists, Special Item Types, Location/Equip Encoding, Inserted Items, Mercenary/Corpse/Golem Lists, `.d2i` Variant, Cross-References, Open Questions, Appendix A Stat-Cost, Appendix B Per-Fixture Spot-Checks).
- [x] Step 4 — Self-review against acceptance criteria: every documented field is tagged `[Excel]`, `[v99-doc]`, `[v99→v105 delta]`, `[code]`, or `[fixture-unverified]`. The cross-reference table (§13) explicitly supersedes misleading sections of the HTML docs (notably their v99-only nature and their omission of the v105 deltas in §6.3 and §7.7). The Open Questions section (§14) is non-empty and lists 12 verifiable items, with priorities flagged.

## Test Results
N/A — no tests were run. `git status` confirms only `docs/v105-item-format.md` is a new file from this task; the only other untracked items (`assets/test/loot-filters/`, `assets/test/v105-real-characters/`) are pre-existing fixture directories not produced by this task.

## Deviations from Plan
1. **No scratch decoder was written.** The plan permits this ("If a fixture cannot be hand-decoded to match a claim, mark it 'unverified' rather than asserting it"). Rather than produce throwaway tooling, I leaned on the high-quality `docs/v99-v105-details.html` reference (which itself triangulates against `dschu012/d2s` PR #86 + a corroborating reviewer comment) as the source of v105-specific deltas, and tagged every v105 claim as either `[v99→v105 delta]` (carrying the upstream confidence rating) or `[fixture-unverified]` (in §14). This is consistent with the plan's risk-mitigation strategy for the unknown-bit ambiguity.
2. **Appendix A is a key-to-columns rather than a full row dump.** The plan calls for "the extracted, v105-specific table from the Excel data" but also notes that this data is to be embedded by task 002. I included the column-index mapping and ~10 representative rows for orientation, and explicitly noted that the full embedded table is task 002's deliverable. Embedding all 369 rows in this reference doc would duplicate task 002's work and risk drift.
3. **Appendix B contains a fixture map rather than per-item offset annotations.** The plan marks Appendix B as optional; I included the fixture-shape mapping (which makes the doc immediately useful as a parser-implementation roadmap) but did not produce per-byte annotations.

These deviations reduce verification depth but are explicitly tolerated by the plan's "Step 2" output specification ("a small per-fixture annotation table … optional, kept short") and by the principle that unverified claims must be tagged, not omitted.

## Blockers & Issues
None. All material needed to write the document was present in the repository:

- The four HTML docs in `docs/` provide complete v99 coverage and (in `v99-v105-details.html`) the explicit v105 delta.
- The Excel tables in `assets/excel/v105/` (especially `itemstatcost.txt`, `bodylocs.txt`, `itemtypes.txt`, `inventory.txt`) provide the data-driven values referenced from the bitstream.
- `src/items/mod.rs` provides ground-truth empty-trailer byte sequences for all (format, expansion, merc-hire) combinations, including the v105 RotW `lf` trailer.
- `src/utils.rs` documents the LSB-first bit-stream semantics that the doc had to align with.

## Notes for Reviewer
- The most consequential v105-specific content is in **§6.3** (the unknown bit after durability) and **§7.7** (the RotW quantity trailer). Both come from `docs/v99-v105-details.html`, which is itself cited as "Confirmed" by the upstream PR + reviewer comment. Task 003 must verify these against fixtures before relying on them.
- **§3.2** (empty-trailer byte sequences) and **§11.1** (mercenary section framing) jointly resolve the long-standing `AGENTS.md` / `NOTES.md` limitation that blocks toggling `mercenary.id` between 0 and nonzero on v105 saves. The resolution is explicit: with the items section fully modeled, the encoder owns the decision to emit/omit the inner six-byte `4A 4D 00 00` empty-list wrapper inside the `"jf"` section.
- **§9.3 / §3.4** flag the most likely v105-specific unknown: where the new RotW shared-stash tabs live. The hypothesis (in the `lf` trailer; alternatively as new `alt_position_id` values) is captured in §14 Q1 as the highest-priority verification task for task 005.
- The cross-reference table in **§13** is the place the reviewer should look first if they want to know "what does this doc supersede". It calls out specifically that `item-details.html`'s §7 ("Byte Alignment") is incomplete for v105 because the quantity trailer in §7.7 must come between the last `0x1FF` and the alignment pad.
- **§14 Q5** ("Are v105 `itemstatcost.txt` widths identical to v99?") is rated `Likely identical` but flagged as `must-verify-before-shipping`. It is the highest-impact unknown in the document because every property in every item depends on those widths being correct.
- The doc deliberately does not specify any Rust types, API shape, or parser strategy — only bitstream layout. This keeps it usable as ground truth across all subsequent tasks (002 embedded data, 003 parser, 004 encoder, 005 `.d2i`, 006 model integration, 007 move API) without prejudging their implementation choices.
- One stylistic note: the doc uses a five-tag provenance scheme (`[Excel]`, `[v99-doc]`, `[v99→v105 delta]`, `[code]`, `[fixture-unverified]`) defined at the top. The plan called for three tags (`Excel`, `fixture-verified`, `HTML-inherited-unverified`); I expanded to five to distinguish (a) inherited-but-confirmed-unchanged-by-the-v99-v105-doc claims from (b) inherited-and-known-changed claims, and to surface `[code]` claims (which derive from the existing `src/` constants and are more reliable than HTML inheritance). No claim was tagged `fixture-verified` because no hand-decoding was performed; the equivalent is `[code]` for source-code-confirmed and `[v99→v105 delta]` for upstream-confirmed claims.
