//! Decode the type code at the failing-item position for each fixture, with
//! verbose per-character bit traces, to determine whether the bug is upstream
//! field widths or genuinely a malformed input.

use std::fs;
use std::path::PathBuf;

use halbu::utils::{read_bits, BytePosition};

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

fn try_decode_at(bytes: &[u8], jm: usize, item_off: usize, label: &str) {
    // item starts at jm + item_off (cursor offset relative to JM)
    // BUT: the cursor is reset relative to bytes[byte_off..]? No — the parser
    // passes the WHOLE file `bytes` slice with `current_byte = byte_off` etc.
    // Actually: `parse_items_inner` builds `cursor = BytePosition { current_byte: *byte_off, ... }`
    // and passes the FULL `bytes` slice. So absolute offsets work directly.
    let mut cursor = BytePosition { current_byte: jm + item_off, current_bit: 0 };

    // Read 32-bit flags
    let flags = read_bits(bytes, &mut cursor, 32).unwrap();
    eprintln!("{label}: flags=0x{:08x}", flags);
    eprintln!("  identified={} socketed={} is_ear={} simple={} personalized={} runeword={}",
        (flags >> 4) & 1, (flags >> 11) & 1, (flags >> 16) & 1,
        (flags >> 21) & 1, (flags >> 24) & 1, (flags >> 26) & 1);

    // Read header (3+3+4+4+4+3 = 21 bits)
    let item_version = read_bits(bytes, &mut cursor, 3).unwrap();
    let location_id = read_bits(bytes, &mut cursor, 3).unwrap();
    let equipped = read_bits(bytes, &mut cursor, 4).unwrap();
    let pos_x = read_bits(bytes, &mut cursor, 4).unwrap();
    let pos_y = read_bits(bytes, &mut cursor, 4).unwrap();
    let alt = read_bits(bytes, &mut cursor, 3).unwrap();
    eprintln!("  item_version={item_version} location_id={location_id} equipped={equipped} pos_x={pos_x} pos_y={pos_y} alt={alt}");
    eprintln!("  cursor now at byte {} bit {}", cursor.current_byte, cursor.current_bit);

    // If is_ear=0, attempt huffman type code by reading bits one at a time.
    eprintln!("  next 32 bits as binary stream (LSB-first reads):");
    let saved_byte = cursor.current_byte;
    let saved_bit = cursor.current_bit;
    let mut bits = Vec::new();
    for _ in 0..32 {
        bits.push(read_bits(bytes, &mut cursor, 1).unwrap() as u8);
    }
    let bitstr: String = bits.iter().map(|b| if *b == 0 { '0' } else { '1' }).collect();
    eprintln!("    {bitstr}");

    // Try decode
    cursor = BytePosition { current_byte: saved_byte, current_bit: saved_bit };
    match halbu::items::v105::__diag_decode_type_code(bytes, &mut cursor) {
        Ok(code) => {
            let s = String::from_utf8_lossy(&code[..]);
            eprintln!("  type_code DECODED: {:?}", s);
        }
        Err(e) => eprintln!("  type_code ERR: {}", e.message),
    }
}

#[test]
fn decode_failing_items() {
    let cases = [
        ("EliteStaves.d2s", 44usize),
        ("UberSlapper.d2s", 274),
        ("m_one.d2s", 472),
        ("Liu.d2s", 1370),
    ];
    for (name, off) in cases {
        let bytes = fs::read(fixtures_dir().join(name)).unwrap();
        let jm = find_jm(&bytes);
        eprintln!("\n=== {name} jm={jm} item_off={off} ===");
        // Pass items-tail-relative slice (what the parser sees)
        let tail_bytes = &bytes[jm..];
        try_decode_at(tail_bytes, 0, off, name);
    }
}
