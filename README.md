# Halbu

A Rust library for reading and modifying Diablo II: Resurrected `.d2s` save files.

This library serves as the backend for **[Halbu Editor](https://github.com/feored/halbu-editor)**.

---

## Features

- Parse and modify `.d2s` save files
- Supports both D2R Legacy and RotW save formats (99/105 at the moment)
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

These sections are currently stored as raw bytes. The library tries to preserve them when writing the file back, but exact round-tripping is not guaranteed.


## Installation

```bash
cargo add halbu
```

## Quick start

```rust
use halbu::{Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("Hero.d2s")?;

    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    let mut save = parsed.save;

    save.character.name = "Halbu".to_string();
    save.skills.set_all(20);

    std::fs::write("Halbu.d2s", save.to_bytes()?)?;

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

## Compatibility checks

Before converting formats, check compatibility findings:

```rust
use halbu::{Save, Strictness};
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

    Ok(())
}
```

## Forced encode

`to_bytes_for(...)` enforces compatibility checks.  
`to_bytes_for_force(...)` bypasses compatibility checks and should only be used intentionally.

```rust
use halbu::{Save, Strictness};
use halbu::format::FormatId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("Hero.d2s")?;
    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    let save = parsed.save;

    let forced = save.to_bytes_for_force(FormatId::V105)?;
    std::fs::write("Hero-forced.d2s", forced)?;

    Ok(())
}
```

## Documentation

API documentation is available on docs.rs:

https://docs.rs/halbu
 
## Notes

Halbu models two related concepts:

- `GameEdition`: `D2RLegacy` or `RotW` (derived from save format/version)
- `ExpansionType`: `Classic`, `Expansion`, or `RotW` (canonical on `Save` as `save.expansion_type`)

Known layout versions currently mapped by this crate:

- `v99` -> `D2RLegacy`
- `v105` -> `RotW`

Use `save.expansion_type()` / `save.set_expansion_type(...)` to read/write character mode.
Use `save.game_edition()` to inspect the edition family.

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
