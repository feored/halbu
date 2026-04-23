//! Call parse_item directly on items-tail bytes at the failing offset.

use std::fs;
use std::path::PathBuf;

use halbu::items::v105::parse_item;
use halbu::utils::BytePosition;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/test/v105-real-characters")
}

fn find_jm(bytes: &[u8]) -> usize {
    for off in 700..bytes.len().saturating_sub(2) {
        if bytes[off] == 0x4A && bytes[off + 1] == 0x4D {
            return off;
        }
    }
    panic!("no JM");
}

#[test]
fn parse_item_at_failing_offset() {
    std::env::set_var("HALBU_DIAG_HUFF", "1");
    let cases = [
        ("EliteStaves.d2s", 44usize),
        ("UberSlapper.d2s", 274),
        ("m_one.d2s", 472),
        ("Liu.d2s", 1370),
    ];
    for (name, off) in cases {
        let bytes = fs::read(fixtures_dir().join(name)).unwrap();
        let jm = find_jm(&bytes);
        let tail = &bytes[jm..];
        let mut cursor = BytePosition { current_byte: off, current_bit: 0 };
        eprintln!("\n=== {name} jm={jm} off={off} ===");
        match parse_item(tail, &mut cursor) {
            Ok(item) => eprintln!("  PARSED ok: cursor now byte={} bit={}, kind={:?}",
                cursor.current_byte, cursor.current_bit,
                match item.kind { halbu::items::v105::ItemKind::Standard(s) => String::from_utf8_lossy(&s.type_code).into_owned(), _ => "<ear>".into() }),
            Err(e) => eprintln!("  ERR: {}", e.message),
        }
    }
}
