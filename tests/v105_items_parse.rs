//! Smoke test: every real v105 `.d2s` fixture in
//! `assets/test/v105-real-characters/` must parse strict-clean and yield a
//! populated `items_v105` tail. `.d2i` shared-stash files are out of scope
//! for task 003 (task 005).

use std::fs;
use std::path::PathBuf;

use halbu::{Save, Strictness};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/test/v105-real-characters")
}

#[test]
fn every_v105_real_character_parses_strict_clean() {
    let dir = fixtures_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("failed reading {}: {e}", dir.display()));

    let mut d2s_files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("d2s"))
        .collect();
    d2s_files.sort();

    assert!(!d2s_files.is_empty(), "no .d2s fixtures found in {}", dir.display());

    let mut failures: Vec<String> = Vec::new();
    let mut succeeded: usize = 0;

    for path in &d2s_files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("{name}: read error: {e}"));
                continue;
            }
        };
        match Save::parse(&bytes, Strictness::Strict) {
            Ok(parsed) => {
                if !parsed.issues.is_empty() {
                    failures.push(format!("{name}: parse issues: {:?}", parsed.issues));
                    continue;
                }
                if parsed.save.items_v105.is_none() {
                    failures.push(format!(
                        "{name}: parsed but items_v105 is None (format detected as {:?})",
                        parsed.save.format()
                    ));
                    continue;
                }
                succeeded += 1;
            }
            Err(e) => {
                failures.push(format!("{name}: hard error: {}", e.message));
            }
        }
    }

    if !failures.is_empty() {
        let total = d2s_files.len();
        panic!(
            "v105 fixture smoke: {}/{} succeeded, {} failed:\n  {}",
            succeeded,
            total,
            failures.len(),
            failures.join("\n  ")
        );
    }
}
