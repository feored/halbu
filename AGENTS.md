# AGENTS.md

Guidance for AI agents working in this repository.

---

## Project Overview

**halbu** is a Rust library (`edition = "2021"`) for parsing, editing, and re-encoding Diablo II: Resurrected `.d2s` save files. It is the backend for the [halbu-editor](https://github.com/feored/halbu-editor) GUI.

Current version: `0.3.0` (see `CHANGELOG.md`)  
API docs: https://docs.rs/halbu

---

## Repository Layout

```
src/
  lib.rs             # Crate root; top-level types: Save, ParsedSave, Class, Strictness, ...
  utils.rs           # Shared bit-level read/write utilities (BytePosition, read_bits, write_bits)
  attributes/        # Bit-packed stat block (strength, mana, level, experience, ...)
  character/         # Per-format character section codec (v99.rs, v105.rs, mercenary/, ...)
  format/            # Top-level decode/encode orchestration; FormatId; compatibility rules
  items/             # Placeholder model (raw byte preservation; not fully modeled)
  npcs/              # Placeholder model (raw byte preservation; not fully modeled)
  quests/            # Quest section model and flag definitions
  skills/            # Skill points model; named D2R skill map
  validation/        # Backend-owned save validation rules
  waypoints/         # Waypoint section model
tests/
  save_format_golden.rs  # Integration/golden tests against real .d2s fixtures
assets/test/             # Binary .d2s fixtures (Joe.d2s, Warlock_v105.d2s, barb*.d2s, ...)
examples/
  edit_character.rs
  inspect_issues.rs
  waypoints.rs
```

---

## Key Concepts

### Format / Edition / Expansion — three separate things

| Type | What it represents | Values |
|---|---|---|
| `FormatId` | Concrete file layout version | `V99`, `V105`, `Unknown(u32)` |
| `GameEdition` | Game edition family | `D2RLegacy`, `RotW` |
| `ExpansionType` | In-game expansion mode stored in save | `Classic`, `Expansion`, `RotW` |

- `FormatId::V99` → `GameEdition::D2RLegacy`
- `FormatId::V105` → `GameEdition::RotW`
- `FormatId::Unknown(_)` → no edition implied

### Save Invariants

These fields are coupled and must **not** be set directly; use the provided setters:

- **Level**: stored in both `character` and `attributes` — always use `save.set_level(n)`.
- **ExpansionType**: stored differently per format — always use `save.set_expansion_type(t)`.
- **Format + version**: version byte must stay in sync with format — always use `save.set_format(f)`.

Violating these invariants produces saves that fail to parse cleanly in strict mode or behave incorrectly in-game.

### Parsing Modes

```rust
// Strict: fail on first hard error
Save::parse(&bytes, Strictness::Strict)?;

// Lax: continue and collect ParseIssue diagnostics
let parsed = Save::parse(&bytes, Strictness::Lax)?;
for issue in &parsed.issues { /* ... */ }
```

### Encoding and Compatibility

```rust
// Blocks on incompatible fields (e.g. Warlock -> V99)
save.encode_for(FormatId::V99, CompatibilityChecks::Enforce)?;

// Bypasses all compatibility checks — use with care
save.encode_for(FormatId::V99, CompatibilityChecks::Ignore)?;
```

Pre-check without encoding:

```rust
let issues = save.check_compatibility(FormatId::V99);
```

### Unmodeled Sections (Placeholders)

`items` and `npcs` are not fully modeled. They are stored as raw bytes and preserved through round-trips. Known limitation: changing `mercenary.id` between `0` and nonzero is a **blocking compatibility issue** because the mercenary item subsection in the raw tail is not rewritten.

---

## Module Responsibilities

- **`format/`**: Entry point for all encode/decode. `decode.rs` drives section parsing in order; `encode.rs` reassembles and recalculates checksum; `compatibility.rs` owns compatibility rules; `layout.rs` holds `FormatId` and byte layout constants; `summary.rs` handles the lightweight summary path.
- **`character/`**: Per-format character section parsing. `v99.rs` and `v105.rs` each handle format-specific byte layout differences. `mercenary/` handles the embedded 14-byte mercenary header.
- **`attributes/`**: Bit-packed stat block. Uses `BytePosition` + `read_bits`/`write_bits` from `utils.rs`.
- **`validation/`**: All validation rules live here. Returns `ValidationReport` with `ValidationIssue` items that are either blocking or warnings. Called independently from encoding via `save.validate()`.
- **`utils.rs`**: Low-level cursor (`BytePosition`) and bitstream I/O shared by `attributes` and `character` codecs. Not part of the public API.

---

## Testing

### Running Tests

```bash
cargo test                 # all unit + integration tests
cargo test --test save_format_golden   # integration tests only
cargo test -p halbu -- attributes      # filter by module
```

### Test Fixtures

Real `.d2s` save files live in `assets/test/`. They are embedded at compile time via `include_bytes!` in integration tests.

### Testing Guidelines

- **Golden / round-trip tests** are preferred for format changes. Add a fixture if covering a new save shape.
- Unit tests live in `#[cfg(test)] mod tests;` submodules in the relevant `mod.rs`.
- The helper `parse_strict_clean` in `tests/save_format_golden.rs` asserts both that parsing succeeds strictly *and* that no `ParseIssue`s are emitted — use this pattern for new golden tests.
- For semantic model comparisons, clear `raw_section`, `items`, and format bookkeeping before `assert_eq!` (see `assert_same_model` in the golden test file).

---

## Code Style

### Formatting

- `rustfmt.toml`: `edition = "2021"`, `use_small_heuristics = "Max"` — run `cargo fmt` before committing.

### Lints (`src/lib.rs`)

The crate root enables these warnings — do not regress them:
```rust
#![warn(
    anonymous_parameters,
    nonstandard_style,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences,
)]
```

Run `cargo clippy` and address warnings before submitting changes.

### Documentation

- All public types and functions must have `///` doc comments.
- Doc comments on `mod` declarations in `lib.rs` serve as the module overview — keep them current.
- Crate-level examples in `lib.rs` and `src/lib.rs` doc tests should compile and run (`cargo test --doc`).

### Conventions

- Prefer structured error types (`ParseHardError`, `EncodeError`) over `String`-based errors.
- Use `#[non_exhaustive]` on public enums that may grow (`CompatibilityCode`, `ValidationCode`, `IssueKind`, `FormatId`) — this is already applied; do not remove it.
- `pub(crate)` for internal cross-module helpers; avoid widening visibility unnecessarily.
- Serde derives (`Serialize`, `Deserialize`) are on all public model types — preserve this for downstream consumers.

---

## Versioning and Compatibility

- This is a library crate published to crates.io. Semver applies strictly.
- `#[non_exhaustive]` enums can add variants in minor releases.
- Removing `pub` items, changing function signatures, or adding required fields to public structs is a **breaking** (major) change.
- Update `CHANGELOG.md` with every change before bumping `Cargo.toml` version.

---

## Known Limitations (do not silently paper over)

- **Items section**: not modeled. Raw bytes preserved. Round-trip fidelity is best-effort.
- **NPC section**: not modeled. Placeholder only.
- **Mercenary hire-state toggle**: changing `mercenary.id` between `0` and nonzero is a blocking compatibility issue because the mercenary item subsection in the raw tail is not rewritten. Tracked in `NOTES.md`.

---

## Common Commands

```bash
cargo build            # compile
cargo test             # all tests
cargo fmt              # format
cargo clippy           # lint
cargo doc --open       # local API docs
cargo add halbu        # downstream install (reference only)
```

---

## Reference Material

- `NOTES.md` — reverse-engineering notes on the `.d2s` binary format (quests, attributes, mercenaries, waypoints, character sections). Read this before touching binary layout logic.
- `CHANGELOG.md` — version history and breaking change record.
- External references listed in `README.md` under "References".
