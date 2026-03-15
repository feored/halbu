# Halbu

A Rust library for reading and modifying Diablo II: Resurrected `.d2s` save files.

This library serves as the backend for **[Halbu Editor](https://github.com/feored/halbu-editor)**.

---

## Features

- Parse and modify `.d2s` save files
- Supports both D2R Legacy and RotW save format versions (v99/v105)
- Edit:
  - character data
  - attributes
  - skills
  - quests
  - waypoints
  - mercenary information
- Strict or tolerant parsing modes

## Limitations

Some sections of the save format are not yet modeled:

- Items
- NPC section

These sections are stored as raw bytes. The library preserves them when writing, but exact round-tripping is not guaranteed.


## Installation

```bash
cargo add halbu
```

## Quick start

```rust
use halbu::{CompatibilityChecks, Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("Hero.d2s")?;

    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    let mut save = parsed.save;
    let target_format = save.format();

    save.character.name = "Halbu".to_string();
    save.skills.set_all(20);

    std::fs::write(
        "Halbu.d2s",
        save.encode_for(target_format, CompatibilityChecks::Enforce)?,
    )?;

    Ok(())
}
```

If you want tolerant parsing with diagnostics:

```rust
use halbu::{Save, Strictness};

let parsed = Save::parse(&bytes, Strictness::Lax)?;
if !parsed.issues.is_empty() {
    eprintln!("Parse issues: {:?}", parsed.issues);
}
```

More examples can be found in `examples/`.

Typed attribute access:

```rust
use halbu::attributes::AttributeId;

let strength = save.attributes.stat(AttributeId::Strength).value;
let experience = save.attributes.stat(AttributeId::Experience).value;
```

## Fast summary

Use the summary API for file lists and quick metadata reads without a full parse:

```rust
use halbu::{Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("Hero.d2s")?;
    let summary = Save::summarize(&bytes, Strictness::Lax)?;

    println!(
        "name={:?} class={:?} level={:?} expansion={:?}",
        summary.name, summary.class, summary.level, summary.expansion_type
    );

    Ok(())
}
```


## Compatibility and forced encode

Use compatibility checks before conversion.  
`encode_for(..., CompatibilityChecks::Enforce)` blocks on incompatible conversions.  
`encode_for(..., CompatibilityChecks::Ignore)` bypasses those checks and should only be used intentionally.

```rust
use halbu::{CompatibilityChecks, Save, Strictness};
use halbu::format::FormatId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("Hero.d2s")?;
    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    let save = parsed.save;

    let target = FormatId::V99;
    let issues = save.check_compatibility(target);
    if !issues.is_empty() {
        eprintln!("Compatibility issues: {issues:?}");
    }

    let forced = save.encode_for(target, CompatibilityChecks::Ignore)?;
    std::fs::write("Hero.d2s", forced)?;

    Ok(())
}
```

## Edition hint

If a file has an unknown/unsupported version, you can still ask Halbu for a best-effort
game edition hint (`D2RLegacy` vs `RotW`).
The hint compares v99/v105 layout coherence (character decode + attributes/skills/items headers)
and uses reserved markers as a tie-breaker.

```rust
use halbu::format::detect_edition_hint;
use halbu::GameEdition;

let hint = detect_edition_hint(&bytes);
if hint == Some(GameEdition::RotW) {
    // likely RotW edition (v105-family layout)
}
```


## Documentation

API documentation is available on docs.rs:

https://docs.rs/halbu
 
## Notes


Halbu models three related concepts:

- `FormatId`: concrete file format version/layout (`V99`, `V105`, or `Unknown(version)`)
- `GameEdition`: edition family (`D2RLegacy` or `RotW`), derived from known `FormatId` values
- `ExpansionType`: `Classic`, `Expansion`, or `RotW` (canonical on `Save` as `save.expansion_type()`)

Known layout versions mapped by this crate:

- `v99` -> `D2RLegacy`
- `v105` -> `RotW`

Use `save.expansion_type()` / `save.set_expansion_type(...)` to read/write expansion mode.
Use `save.game_edition()` to inspect the edition family.
Use `save.character.status()` to inspect status bits, and `save.character.set_hardcore(...)` / `set_ladder(...)` / `set_died(...)` for status mutations.

Current blocking rules (compatibility checks) include:
- Warlock requires RotW edition and RotW expansion type.
- RotW expansion type cannot be encoded to non-RotW editions.
- Druid/Assassin cannot be encoded as Classic.
- Unknown class ids cannot be safely converted to known target formats.

Level is stored in both the character section and the attributes section. Use `save.set_level(...)` to keep them in sync.

Additional notes about the format, quest flags, and general reverse-engineering work can be found in `NOTES.md`.

This repository also contains several example .d2s files used in tests to verify that parsing and round-trip encoding work correctly.


## References

These resources have helped me understand the .d2s format. Many thanks to their authors.

* http://user.xmission.com/~trevin/DiabloIIv1.09_File_Format.shtml
* https://github.com/dschu012/D2SLib
* https://github.com/d07RiV/d07riv.github.io/blob/master/d2r.html
* https://github.com/oaken-source/pyd2s
* https://github.com/WalterCouto/D2CE/blob/main/d2s_File_Format.md
* https://github.com/krisives/d2s-format
* https://github.com/nokka/d2s/
* https://github.com/ThePhrozenKeep/D2MOO
* https://d2mods.info/forum/kb/index?c=4
