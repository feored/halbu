//! D2R Huffman codec for 4-character item type codes.
//!
//! The tree is the same in v99 and v105 (per `docs/v99-v105-details.html`
//! §3.2). Bits are read **LSB-first** to match `crate::utils::read_bits`,
//! and tree traversal is `0 = left`, `1 = right`. See
//! `docs/item-details.html` §10 for the full table sourced here.

use std::sync::OnceLock;

use crate::utils::{read_bits, BytePosition};
use crate::ParseHardError;

/// Per-character (char, value, bit-count) entries.
///
/// `value` is the integer obtained when the encoder writes `nbits` bits
/// LSB-first; equivalently, it is the integer the decoder accumulates
/// LSB-first while consuming `nbits` bits from the stream. The bit traversed
/// first during decode is `value & 1`.
const TABLE: &[(u8, u32, u8)] = &[
    (b' ', 1, 2),
    (b'b', 10, 4),
    (b's', 4, 4),
    (b'7', 30, 5),
    (b'9', 14, 5),
    (b'a', 15, 5),
    (b'c', 2, 5),
    (b'g', 11, 5),
    (b'h', 24, 5),
    (b'l', 23, 5),
    (b'm', 22, 5),
    (b'n', 44, 6),
    (b'o', 127, 7),
    (b'p', 19, 5),
    (b'r', 7, 5),
    (b't', 6, 5),
    (b'u', 16, 5),
    (b'w', 0, 5),
    (b'x', 28, 5),
    (b'2', 12, 6),
    (b'8', 8, 6),
    (b'd', 35, 6),
    (b'e', 3, 6),
    (b'f', 50, 6),
    (b'k', 18, 6),
    (b'1', 31, 7),
    (b'i', 63, 7),
    (b'v', 59, 7),
    (b'3', 91, 7),
    (b'y', 40, 7),
    (b'4', 95, 8),
    (b'0', 223, 8),
    (b'5', 104, 8),
    (b'q', 155, 8),
    (b'z', 27, 8),
    (b'j', 232, 9),
];

#[derive(Debug)]
struct Node {
    /// `Some` if this is a leaf; the decoded character.
    leaf: Option<u8>,
    /// Index of the `0` (left) child in the arena (or `usize::MAX` if absent).
    zero: usize,
    /// Index of the `1` (right) child in the arena (or `usize::MAX` if absent).
    one: usize,
}

const NIL: usize = usize::MAX;

#[derive(Debug)]
struct Tree {
    nodes: Vec<Node>,
}

static TREE: OnceLock<Tree> = OnceLock::new();

fn tree() -> &'static Tree {
    TREE.get_or_init(|| {
        let mut nodes: Vec<Node> = Vec::with_capacity(128);
        nodes.push(Node { leaf: None, zero: NIL, one: NIL }); // root

        for &(ch, pattern, nbits) in TABLE {
            let mut current = 0usize;
            for i in 0..nbits {
                let bit = ((pattern >> i) & 1) != 0;
                let child = if bit { nodes[current].one } else { nodes[current].zero };
                let next_index = if child == NIL {
                    nodes.push(Node { leaf: None, zero: NIL, one: NIL });
                    let idx = nodes.len() - 1;
                    if bit {
                        nodes[current].one = idx;
                    } else {
                        nodes[current].zero = idx;
                    }
                    idx
                } else {
                    child
                };
                current = next_index;
            }
            assert!(nodes[current].leaf.is_none(), "duplicate Huffman code for {ch}");
            nodes[current].leaf = Some(ch);
        }
        Tree { nodes }
    })
}

/// Decode one character from the Huffman bitstream at the current cursor.
pub(crate) fn decode_one_char(
    bytes: &[u8],
    cursor: &mut BytePosition,
) -> Result<u8, ParseHardError> {
    let tree = tree();
    let mut current = 0usize;
    loop {
        if let Some(ch) = tree.nodes[current].leaf {
            return Ok(ch);
        }
        let bit = read_bits(bytes, cursor, 1)?;
        let next = if bit != 0 { tree.nodes[current].one } else { tree.nodes[current].zero };
        if next == NIL {
            return Err(ParseHardError {
                message: "Huffman decode hit invalid path (no child for bit).".to_string(),
            });
        }
        current = next;
    }
}

/// Decode the standard 4-character item type code.
pub(crate) fn decode_type_code(
    bytes: &[u8],
    cursor: &mut BytePosition,
) -> Result<[u8; 4], ParseHardError> {
    let mut out = [0u8; 4];
    for slot in out.iter_mut() {
        *slot = decode_one_char(bytes, cursor)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::write_bits;

    fn encode_chars(chars: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut pos = BytePosition::default();
        for &ch in chars {
            let (_, pat, nbits) = TABLE.iter().find(|(c, _, _)| *c == ch).copied().unwrap();
            write_bits(&mut out, &mut pos, pat, nbits as usize).unwrap();
        }
        out
    }

    #[test]
    fn round_trip_known_codes() {
        for code in [b"hax ", b"r01 ", b"cap ", b"hp1 ", b"jav "] {
            let bytes = encode_chars(code);
            let mut cursor = BytePosition::default();
            let decoded = decode_type_code(&bytes, &mut cursor).expect("decode");
            assert_eq!(&decoded, code, "round-trip failed for {:?}", code);
        }
    }

    #[test]
    fn each_char_decodes_unique() {
        for &(ch, _, _) in TABLE {
            let bytes = encode_chars(&[ch]);
            let mut cursor = BytePosition::default();
            let decoded = decode_one_char(&bytes, &mut cursor).expect("decode");
            assert_eq!(decoded, ch);
        }
    }
}
