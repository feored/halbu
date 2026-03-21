# Halbu

A Rust library for parsing, editing, and safely re-encoding Diablo II: Resurrected `.d2s` save files.

It serves as the backend for **[Halbu Editor](https://github.com/feored/halbu-editor)**.

---

## Features

- Parse and modify `.d2s` save files
- Supports the D2R Legacy and RotW layouts (`v99`, `v105`)
- Editable sections:
  - character data
  - attributes
  - skills
  - quests
  - waypoints
  - mercenary data
- Partial parsing via summary API
- Strict or tolerant parsing modes
- Validation for post-edit sanity checks
- Compatibility checks for format conversion


## Limitations

Some parts of the save format are not yet modeled:

- Items
- NPC section

These sections are preserved as raw bytes when possible, but may not round-trip identically after modifications.

Changing `mercenary.id` between `0` (no mercenary hired) and nonzero (mercenary hired) is currently reported as a blocking compatibility issue, because Halbu does not yet rewrite the mercenary item subsection inside the raw item tail. `CompatibilityChecks::Ignore` can still force encoding in this case.


## Installation

```bash
cargo add halbu
```


## Basic usage

Parse, modify, and write a save:

```rust
use halbu::{CompatibilityChecks, Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("Hero.d2s")?;

    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    let mut save = parsed.save;

    save.character.name = "Halbu".to_string();
    save.skills.set_all(20);

    let encoded = save.encode_for(save.format(), CompatibilityChecks::Enforce)?;
    std::fs::write("Halbu.d2s", encoded)?;

    Ok(())
}
```


## Parsing modes

Strict parsing fails on inconsistencies:

```rust
let parsed = Save::parse(&bytes, Strictness::Strict)?;
```

Lax parsing continues and reports issues:

```rust
let parsed = Save::parse(&bytes, Strictness::Lax)?;
if !parsed.issues.is_empty() {
    eprintln!("Parse issues: {:?}", parsed.issues);
}
```


## Validation

Validation is an optional step to check the save for inconsistencies that may prevent the game from loading the save (e.g. invalid character name or mercenary level).

```rust
let report = save.validate();
if !report.is_valid() {
    eprintln!("Validation issues: {:?}", report.issues);
}
```


## Compatibility and encoding

Compatibility checks apply when converting between formats during encoding.

```rust
use halbu::{CompatibilityChecks, Save, Strictness};
use halbu::format::FormatId;

let parsed = Save::parse(&bytes, Strictness::Strict)?;
let save = parsed.save;

let target = FormatId::V99;
let issues = save.check_compatibility(target);

if !issues.is_empty() {
    eprintln!("Compatibility issues: {issues:?}");
}

// Safe (blocks on incompatibility)
let encoded = save.encode_for(target, CompatibilityChecks::Enforce)?;

// Unsafe (bypasses checks)
let forced = save.encode_for(target, CompatibilityChecks::Ignore)?;
```


## Summary API

Read metadata without fully parsing the file:

```rust
let summary = Save::summarize(&bytes, Strictness::Lax)?;

println!(
    "name={:?} class={:?} level={:?} expansion={:?}",
    summary.name,
    summary.class,
    summary.level,
    summary.expansion_type
);
```

## Edition detection

For unknown versions, Halbu can try to guess which edition the save layout matches most closely:

```rust
use halbu::format::detect_edition_hint;
use halbu::GameEdition;

let hint = detect_edition_hint(&bytes);

if hint == Some(GameEdition::RotW) {
    // likely RotW (v105-style layout)
}
```


## Model overview

Halbu distinguishes between three related concepts:

- `FormatId` - concrete file format (`V99`, `V105`, or unknown)
- `GameEdition` - edition family (`D2RLegacy`, `RotW`)
- `ExpansionType` - in-game expansion mode (`Classic`, `Expansion`, `RotW`)

Typical usage:

- `save.format()` -> file format
- `save.game_edition()` -> edition family
- `save.expansion_type()` -> in-game expansion


## Compatibility rules (examples)

- Warlock requires RotW edition and expansion
- RotW expansion cannot be encoded to non-RotW formats
- Druid and Assassin cannot be encoded as Classic
- Unknown class IDs cannot be safely converted


## Notes

- Level is stored in multiple sections; use `save.set_level(...)` to keep it consistent
- When no mercenary is hired, Halbu normalizes the full mercenary header block to zero on encode
- Changing `mercenary.id` between `0` and nonzero is currently treated as a blocking compatibility issue because the mercenary item subsection is still preserved as raw bytes, though `CompatibilityChecks::Ignore` can still force encoding
- Additional reverse-engineering notes are available in `NOTES.md`


## Documentation

API docs: https://docs.rs/halbu  
Changelog: [CHANGELOG.md](CHANGELOG.md)

## References

These resources helped me understand the .d2s format. Many thanks to their authors.

* http://user.xmission.com/~trevin/DiabloIIv1.09_File_Format.shtml
* https://github.com/dschu012/D2SLib
* https://github.com/d07RiV/d07riv.github.io/blob/master/d2r.html
* https://github.com/oaken-source/pyd2s
* https://github.com/WalterCouto/D2CE/blob/main/d2s_File_Format.md
* https://github.com/krisives/d2s-format
* https://github.com/nokka/d2s/
* https://github.com/ThePhrozenKeep/D2MOO
* https://d2mods.info/forum/kb/index?c=4
