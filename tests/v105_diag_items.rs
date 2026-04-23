//! Diagnostic test: print item-level details for failing fixtures.

use std::fs;
use std::path::PathBuf;

use halbu::{Save, Strictness};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/test/v105-real-characters")
}

#[test]
fn diag_failing_fixtures_items() {
    std::env::set_var("HALBU_DIAG_ITEMS", "1");
    let names = ["EliteStaves.d2s", "Liu.d2s", "UberSlapper.d2s", "m_one.d2s"];
    for name in names {
        let p = fixtures_dir().join(name);
        let bytes = fs::read(&p).unwrap();
        eprintln!("\n=== {} ===", name);
        match Save::parse(&bytes, Strictness::Strict) {
            Ok(_) => eprintln!("OK"),
            Err(e) => eprintln!("ERR: {}", e.message),
        }
    }
}
