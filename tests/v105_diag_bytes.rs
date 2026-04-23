//! Dump the relative-to-items-tail bytes around each failing item to compare
//! shapes across fixtures. Items-tail is offset 4 from "JM" (after JM + 2-byte
//! count). Item start_byte values from diag are *cursor* offsets from the
//! "JM" magic (i.e. file_off = jm_off + start_byte).

use std::fs;
use std::path::PathBuf;

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

fn dump_range(label: &str, bytes: &[u8], start: usize, len: usize) {
    let end = (start + len).min(bytes.len());
    eprintln!("{label} bytes[{start}..{end}]:");
    for chunk in (start..end).step_by(16) {
        let chunk_end = (chunk + 16).min(end);
        let hex: String = bytes[chunk..chunk_end].iter().map(|b| format!("{:02x} ", b)).collect();
        let bin0 = format!("{:08b}", bytes[chunk]);
        eprintln!("  +{:03}  {hex}  [byte0_bin={bin0}]", chunk - start);
    }
}

#[test]
fn dump_failing_item_bytes() {
    // (fixture, cursor_offset_of_failing_item)
    let cases = [
        ("EliteStaves.d2s", 44usize),
        ("UberSlapper.d2s", 274),
        ("m_one.d2s", 472),
        ("Liu.d2s", 1370),
    ];
    for (name, cur_off) in cases {
        let bytes = fs::read(fixtures_dir().join(name)).unwrap();
        let jm = find_jm(&bytes);
        let file_off = jm + cur_off;
        eprintln!("\n=== {name} (jm={jm}, item_file_off={file_off}) ===");
        dump_range("  failing", &bytes, file_off, 64);
    }
}
