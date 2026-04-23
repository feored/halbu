# MASTERPLAN: v105 Item Handling — Move Items Between Characters

## Vision

Enable users of the `halbu` library to move items between Diablo II: Resurrected v105 save files (character-to-character, character-to-stash, and between locations within a single save) without altering any item attribute other than its position, storage location, and owner context. Today, items are an opaque blob preserved as raw bytes; after this work, they become a first-class, fully-modeled part of the `Save` for the v105 format, unlocking the most-requested editor feature: relocating items safely. The V99 format remains on its existing raw-bytes path and is explicitly out of scope.

## Users & Stakeholders

- **halbu-editor maintainers**: The primary downstream consumer. They need a stable, Serde-friendly item model and a safe `move_item` API to wire into the GUI.
- **Library users (third-party tooling authors)**: Want programmatic access to a character's items and the ability to relocate them across saves without corrupting the file.
- **End-user players (indirect)**: Will use the editor to reorganize gear across characters and the shared stash. They expect their items to come out the other side bit-for-bit identical except for where they live.
- **halbu maintainers / reviewers**: Need the change to honor existing project invariants (Strict/Lax parsing, ParseIssue diagnostics, structured errors, `#[non_exhaustive]` enums, no runtime filesystem dependencies, golden round-trip testing against real fixtures).

## Features

### Feature 1: Embedded v105 Item Reference Data
**As a** library consumer, **I want to** have all item lookup data shipped inside the crate, **so that** the library works in any environment without external files.

Acceptance criteria:
- Excel tables required for item parsing (item codes → base type/size/category, item types hierarchy, per-stat bit layouts, affix tables, set/unique/runeword tables, runes, gems) are embedded at compile time via `include_str!`.
- No runtime filesystem reads are introduced.
- A documented internal reference describing the v105 item bitstream layout is captured under `docs/` (synthesized from existing docs + fixture analysis + excel data; supersedes misleading sections of the existing docs).

### Feature 2: Parse v105 Item Bitstream
**As a** library user, **I want to** load any v105 `.d2s` file and get a structured representation of every item it contains, **so that** I can read item data instead of opaque bytes.

Acceptance criteria:
- Every `.d2s` fixture in `assets/test/v105-real-characters/` parses successfully in `Strictness::Strict` with zero `ParseIssue`s (uses the `parse_strict_clean` pattern).
- All item shapes present in the fixtures are recognized: simple, extended, magic, rare, crafted, set, unique, runeword, ear, gold, gems, runes, charms, jewels, items with sockets, items with inserted items (socketed gems/runes/jewels), ethereal items, personalized items, and class-specific items.
- The property/affix list inside each item is decoded using bit widths derived from the embedded stat-cost table (not hardcoded guesses), and terminates correctly on the `0x1FF` (511) sentinel.
- The four item lists are all handled, each preceded by its `JM` magic + `u16` count: player items, corpse items, mercenary items, golem items.
- Each item carries enough location metadata to identify where it lives: storage container (inventory / belt / stash / cube / equipped), grid coordinates, and equipment slot where applicable.

### Feature 3: Encode v105 Items Round-Trip Cleanly
**As a** library user, **I want to** save a parsed v105 file and get back bytes that round-trip cleanly, **so that** I can trust that reading and writing is non-destructive.

Acceptance criteria:
- For every fixture in `assets/test/v105-real-characters/`, parsing then re-encoding produces output that re-parses to a semantically identical model.
- Byte-for-byte identity is achieved wherever the format admits a single canonical encoding; any unavoidable normalizations are documented.
- The mercenary-items raw-tail limitation called out in `AGENTS.md` is resolved for v105: toggling mercenary hire state is no longer a blocking compatibility issue on v105 saves.
- The encoder emits a valid checksum and preserves all four item-list sections (player, corpse, mercenary, golem) in the correct order with correct headers.

### Feature 4: Shared Stash (`.d2i`) Parsing
**As a** library user, **I want to** parse shared-stash files alongside character saves, **so that** I can move items between a character and the shared stash.

Acceptance criteria:
- All `.d2i` shared-stash fixtures present in `assets/test/v105-real-characters/` parse in Strict mode with zero `ParseIssue`s.
- Shared-stash files round-trip cleanly through encode.
- Shared-stash items are exposed through the same item model as character items.

### Feature 5: Items Integrated Into the Save Model (V105 Only)
**As a** library user, **I want to** access items as a typed field on `Save` for v105 saves, **so that** I can read and modify them through the public API.

Acceptance criteria:
- The `items` placeholder is replaced by a structured model on the `Save` (or a v105-specific items container) when the format is V105.
- V99 saves continue to use the existing raw-bytes path with no behavior change; V99 golden tests still pass unchanged.
- Item models derive `Serialize` and `Deserialize` per project convention.
- New public enums that may grow are marked `#[non_exhaustive]`; internal helpers are `pub(crate)`.
- Errors use structured types consistent with `ParseHardError` / `EncodeError` style.

### Feature 6: Move-Item Public API
**As a** halbu-editor developer, **I want to** call a single function to move an item to a new location (within a save or to another save), **so that** I can implement drag-and-drop without writing format code.

Acceptance criteria:
- A public API exists to move an item between any two locations: inventory, belt, stash, cube, equipped slots, mercenary inventory, and the shared stash — both within one `Save` and across two `Save` instances.
- All item attributes other than position/storage container/owner context are preserved exactly across the move.
- The operation validates the target: bounds-check grid coordinates, container-fits-item-size, equip-slot-accepts-item-type, class-restricted items rejected for wrong class, two-handed weapon collision rules, etc. Failures return a structured error and leave both saves unchanged.
- Moving items between two saves updates both source and destination atomically (in-memory): item disappears from source, appears at destination, both saves still encode cleanly.

### Feature 7: Comprehensive Round-Trip Golden Tests
**As a** maintainer, **I want to** have automated coverage over every real fixture, **so that** regressions in item handling are caught immediately.

Acceptance criteria:
- A golden test asserts strict-clean parse + round-trip semantic equality for every file in `assets/test/v105-real-characters/`.
- Tests follow the existing `tests/save_format_golden.rs` pattern and helper conventions.
- An end-to-end integration test demonstrates the headline story: load two v105 saves, move an item from one to the other, encode both, re-parse both, and assert the item moved with all attributes preserved except location/owner.

### Feature 8: Documentation & Changelog
**As a** library consumer, **I want to** see what changed and how to use the new API, **so that** I can adopt the new version confidently.

Acceptance criteria:
- `CHANGELOG.md` is updated with the new feature, behavior changes, and the resolved mercenary limitation.
- Public types and functions have `///` doc comments per project convention.
- The crate-level docs and module docs in `lib.rs` reflect the new item model for v105.
- The v105-only scope and the unchanged V99 behavior are explicitly documented.

## Tasks

| # | Task | Features Covered | Priority |
|---|------|------------------|----------|
| 1 | Analyze existing docs, excel files, and v105 fixtures; produce an internal v105 item-format reference document | Feature 1 | High |
| 2 | Build embedded excel data loading for the lookup tables required by item parsing (item codes, item types, stat costs, affixes, set/unique/runeword/rune/gem tables) | Feature 1 | High |
| 3 | Implement the v105 item bitstream parser covering every item shape present in the fixtures, plus the four item-list sections (player, corpse, mercenary, golem) and the `JM` + count framing | Feature 2 | High |
| 4 | Implement the v105 item encoder with round-trip fidelity and a correct checksum, replacing the raw-bytes tail for v105 | Feature 3 | High |
| 5 | Add `.d2i` shared-stash parsing and encoding using the same item model | Feature 4 | High |
| 6 | Integrate the item model into `Save` for v105 only; keep V99 on the existing raw-bytes path unchanged | Feature 5 | High |
| 7 | Implement the public move-item API with full target validation and cross-save support | Feature 6 | High |
| 8 | Add golden round-trip tests for every fixture in `assets/test/v105-real-characters/`, plus the end-to-end "move items between two saves" integration test | Feature 7 | High |
| 9 | Update `CHANGELOG.md`, doc comments, module overviews, and note the resolved mercenary limitation | Feature 8 | Medium |

## Out of Scope

- V99 item modeling. V99 saves continue to preserve items as raw bytes via the existing placeholder path. No backwards-compatibility shims for V99 item structures will be introduced.
- Creating, deleting, crafting, rerolling, identifying, repairing, upgrading, socketing/unsocketing, or otherwise mutating any item attribute other than its location.
- Modeling the `npcs` section. It remains a raw-bytes placeholder.
- Editing item properties, affixes, sockets contents, ethereal/personalized flags, durability, quantity, or any other on-item field.
- A new GUI or CLI; this work is library-only. The halbu-editor will consume the new API separately.
- Validating game-rule legality beyond what is required to keep moves safe (e.g. no enforcement of level requirements on equipped items, no stat-requirement checks against the character's attributes).
- Supporting save formats other than v105 for the new item model.
- Runtime loading of excel data from the filesystem; all required tables must be embedded at compile time.

