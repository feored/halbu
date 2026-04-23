# Review Report: Task 001 — Iteration 1

## Verdict
APPROVED

## Summary
The deliverable `docs/v105-item-format.md` (1051 lines) is a thorough, well-structured internal reference for the v105 item bitstream. All 16 sections promised in the plan are present and ordered as specified. The document uses an explicit five-tag provenance scheme (`[Excel]`, `[v99-doc]`, `[v99→v105 delta]`, `[code]`, `[fixture-unverified]`) that is more useful than the three-tag scheme called for in the plan and makes inheritance vs. confirmation visible at every claim. Code-derived claims (empty-trailer constants in §3.2, mercenary section framing in §11.1) were spot-verified byte-for-byte against `src/items/mod.rs`. The bitstream conventions in §2 correctly describe the LSB-first semantics of `crate::utils::read_bits`. The cross-reference table in §13 is concrete and actionable, and the Open Questions in §14 prioritise the highest-impact unknowns (Q2, Q5, Q6) that the parser task must verify before shipping.

## Test Results
N/A — documentation-only deliverable. `git status` shows a clean working tree (the doc was committed on branch `plans/001-item-handling`). No Rust source code was changed; running the test runner is not applicable per the plan's Definition of Done.

## Checklist Results
| Dimension | Status | Notes |
|-----------|--------|-------|
| TDD compliance | OK | N/A for a documentation deliverable; the plan explicitly waives test execution. |
| Test quality | OK | N/A. |
| All tests pass | OK | No tests run; pre-existing test suite untouched. |
| Correctness | OK | Spot-checked claims against source: §3.2 empty-trailer bytes match `src/items/mod.rs` exactly (V99 + V105 × Classic/Expansion/RotW × merc-hire); §2 LSB-first conventions match `src/utils.rs::read_bits`; §11.1 `jf` always-present claim matches the `V105_EMPTY_ITEMS_EXPANSION` constant. Excel-tagged file references (`itemstatcost.txt`, `bodylocs.txt`, `inventory.txt`) all exist under `assets/excel/v105/`. Fixture references in Appendix B all exist under `assets/test/v105-real-characters/`. |
| Completeness | OK | All 16 sections present (Overview, Bitstream Conventions, Item-List Framing, Simple Bits, Extended Header, Type-Specific Sub-Blocks, Property Lists, Special Item Types, Location/Equip, Inserted Items, Merc/Corpse/Golem, `.d2i` Variant, Cross-References, Open Questions, Appendix A, Appendix B). Feature 1 acceptance criterion 3 is satisfied. The doc explicitly resolves the `AGENTS.md` mercenary-hire-toggle limitation in §3.2 and §11.1, which is a significant deliverable beyond the bare minimum. |
| Code quality | OK | N/A — no code. Markdown is well-formatted, internally linked TOC works, tables are consistently structured. |
| Robustness | OK | Provenance tagging makes downstream task 003 able to defensively handle unverified claims (Q1–Q12). The doc never asserts a fixture-verified claim where one was not produced. |
| No regressions | OK | No Rust files modified; no risk of regression. |
| Deviations | NOTED | Three deviations from plan, all permitted by the plan's own language: (1) no scratch decoder written — plan permits "mark unverified rather than asserting"; (2) Appendix A is a column-index key plus 10 sample rows rather than a full 369-row dump — appropriate, since task 002 owns the embedded data and duplicating it here would invite drift; (3) Appendix B is a fixture-shape map rather than per-byte annotations — plan flags Appendix B as optional. The deviations are well-reasoned and called out transparently in the execution report. |
| Blockers resolved | OK | No blockers reported; none observed during review. |

## Changes Made
None. The document meets the plan's Definition of Done as written and no fixes were required.

| File | Change Description |
|------|--------------------|
| (none) | No project files were modified during this review. |

## Unresolved Issues
None that block downstream tasks. The 12 Open Questions in §14 are correctly framed as work for tasks 002–005, not as defects in this deliverable. Task 003 (parser) should treat Q2 (v105 unknown-bit gating), Q5 (`itemstatcost.txt` width parity with v99), and Q6 (new v105 stat IDs) as must-verify-before-shipping per the document's own guidance.

## Verdict
APPROVED
