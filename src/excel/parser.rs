//! Tab-separated value parser for embedded v105 Excel reference tables.
//!
//! Hand-rolled (no external crate). Used only by sibling table modules
//! in `crate::excel`. See `docs/v105-item-format.md` for context on which
//! bitstream decisions these tables drive.

/// Lightweight result of parsing a tab-separated text blob.
///
/// Field references borrow from the input `&'static str`; subsequent typed
/// parsing copies into owned fields on each table's row struct.
pub(super) struct Tsv<'a> {
    /// Header cells, in source order.
    pub headers: Vec<&'a str>,
    /// Data rows. Each row is the same length as `headers`.
    pub rows: Vec<Vec<&'a str>>,
}

/// Parse a tab-separated table.
///
/// Behaviour:
/// - Header row is line 1.
/// - Trailing `\r` (CRLF files) is stripped.
/// - Empty lines are skipped.
/// - Rows whose first cell equals `"Expansion"` are skipped (Blizzard editor section markers).
/// - Rows shorter than the header are padded with empty cells; rows longer are truncated.
pub(super) fn parse_tsv(text: &'static str) -> Tsv<'static> {
    let mut lines = text.lines();
    let header_line = lines.next().unwrap_or("");
    let headers: Vec<&str> = split_line(header_line);
    let n = headers.len();

    let mut rows: Vec<Vec<&str>> = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        let mut cells: Vec<&str> = split_line(line);
        if !cells.is_empty() && cells[0] == "Expansion" {
            continue;
        }
        if cells.len() < n {
            cells.resize(n, "");
        } else if cells.len() > n {
            cells.truncate(n);
        }
        rows.push(cells);
    }

    Tsv { headers, rows }
}

fn split_line(line: &str) -> Vec<&str> {
    let line = line.strip_suffix('\r').unwrap_or(line);
    line.split('\t').collect()
}

/// Resolve the column index for a header name.
///
/// Panics with a clear message if the column is absent. The embedded tables
/// are compile-time constants, so a missing column is a programming bug
/// (e.g. column rename in a future MPQ extraction), not a runtime error.
pub(super) fn col_idx(headers: &[&str], name: &str) -> usize {
    headers.iter().position(|h| *h == name).unwrap_or_else(|| {
        panic!("excel: required column `{name}` missing from table (have: {headers:?})")
    })
}

/// Parse a `u32` cell. Empty/blank cells return `0`.
pub(super) fn parse_u32(s: &str) -> u32 {
    let s = s.trim();
    if s.is_empty() {
        0
    } else {
        s.parse::<u32>().unwrap_or(0)
    }
}

/// Parse a `u8` cell. Empty/blank cells return `0`.
pub(super) fn parse_u8(s: &str) -> u8 {
    let s = s.trim();
    if s.is_empty() {
        0
    } else {
        s.parse::<u8>().unwrap_or(0)
    }
}

/// Parse a boolean-ish cell. `"1"` → `true`; anything else → `false`.
pub(super) fn parse_bool_01(s: &str) -> bool {
    s.trim() == "1"
}

/// Convert an empty cell to `None`, otherwise `Some(owned_string)`.
pub(super) fn opt_string(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "A\tB\tC\nfoo\t1\tx\nExpansion\nbar\t\t\nbaz\t42";

    #[test]
    fn parses_headers_and_rows() {
        let t = parse_tsv(SAMPLE);
        assert_eq!(t.headers, vec!["A", "B", "C"]);
        // 3 data rows: foo, bar (with empty cells), baz (padded)
        assert_eq!(t.rows.len(), 3);
        assert_eq!(t.rows[0], vec!["foo", "1", "x"]);
        assert_eq!(t.rows[1], vec!["bar", "", ""]);
        assert_eq!(t.rows[2], vec!["baz", "42", ""]);
    }

    #[test]
    fn skips_expansion_marker_rows() {
        let t = parse_tsv(SAMPLE);
        assert!(t.rows.iter().all(|r| r[0] != "Expansion"));
    }

    #[test]
    fn col_idx_locates_columns() {
        let t = parse_tsv(SAMPLE);
        assert_eq!(col_idx(&t.headers, "A"), 0);
        assert_eq!(col_idx(&t.headers, "C"), 2);
    }

    #[test]
    #[should_panic(expected = "required column")]
    fn col_idx_panics_on_missing() {
        let t = parse_tsv(SAMPLE);
        let _ = col_idx(&t.headers, "Z");
    }

    #[test]
    fn numeric_helpers_handle_blanks() {
        assert_eq!(parse_u32(""), 0);
        assert_eq!(parse_u32("  "), 0);
        assert_eq!(parse_u32("12"), 12);
        assert_eq!(parse_u8(""), 0);
        assert_eq!(parse_u8("9"), 9);
    }

    #[test]
    fn bool_01_maps_one_to_true() {
        assert!(parse_bool_01("1"));
        assert!(!parse_bool_01("0"));
        assert!(!parse_bool_01(""));
        assert!(!parse_bool_01("yes"));
    }

    #[test]
    fn opt_string_distinguishes_empty() {
        assert_eq!(opt_string(""), None);
        assert_eq!(opt_string("   "), None);
        assert_eq!(opt_string("hi"), Some("hi".to_string()));
    }

    #[test]
    fn handles_crlf_line_endings() {
        let crlf = "A\tB\r\nfoo\t1\r\nbar\t2\r\n";
        let t = parse_tsv(crlf);
        assert_eq!(t.headers, vec!["A", "B"]);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0], vec!["foo", "1"]);
        assert_eq!(t.rows[1], vec!["bar", "2"]);
    }
}
