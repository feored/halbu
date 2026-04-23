use std::fs;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/test/v105-real-characters")
}

#[test]
fn dump_hex_around_failure() {
    // EliteStaves: failure at item 4, prev item ended at byte 44 (relative to items tail).
    // The items tail begins after the previous section. We need the absolute file offset.
    // Easier: just dump first 200 bytes of items tail of each.
    use halbu::{Save, Strictness};
    let names = [("EliteStaves.d2s", 4usize), ("UberSlapper.d2s", 12), ("m_one.d2s", 16), ("Liu.d2s", 40)];
    for (name, _idx) in names {
        let p = fixtures_dir().join(name);
        let bytes = fs::read(&p).unwrap();
        // Find items tail: it begins at "JM" magic. Skip header. Easier: scan "JM" pairs.
        // The first JM at offset >= 765 (typical) is the player items magic.
        let mut found = None;
        for off in 700..bytes.len().saturating_sub(2) {
            if bytes[off] == 0x4A && bytes[off+1] == 0x4D {
                found = Some(off);
                break;
            }
        }
        let off = found.expect("JM not found");
        eprintln!("\n=== {} === items tail starts at file offset {}", name, off);
        // dump first 1500 bytes of items tail
        let end = (off + 1500).min(bytes.len());
        for chunk_start in (off..end).step_by(16) {
            let chunk_end = (chunk_start+16).min(end);
            let rel = chunk_start - off;
            let hex: String = bytes[chunk_start..chunk_end].iter().map(|b| format!("{:02x} ", b)).collect();
            eprintln!("  {:04} (rel)  {}", rel, hex);
        }
        // Also try parsing
        match Save::parse(&bytes, Strictness::Strict) {
            Ok(_) => eprintln!("  parse OK"),
            Err(e) => eprintln!("  parse ERR: {}", e.message),
        }
    }
}
