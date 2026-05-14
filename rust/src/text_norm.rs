//! Text normalization pipeline (§5.11).
//!
//! Single source of truth for normalizing text-field values across Spec-Check
//! callers — the two-phase matcher (Plan 02) and the spec-distinctness
//! validator (Plan 04) both route through this module so identical UI copy
//! always compares equal regardless of whitespace, HTML-entity encoding, NFC
//! form, invisible bidi marks, or letter case.
//!
//! The pipeline is intentionally lossy in those five dimensions and
//! intentionally LOSSLESS otherwise — smart quotes, em dashes, and Unicode
//! ellipses are preserved verbatim because they carry semantic meaning in UI
//! copy.

use unicode_normalization::UnicodeNormalization;

/// The four IR criteria fields that flow through text normalization. Today
/// all four route through the same pipeline (§5.11 "uniform across `text`
/// / `textContains` / `textPattern` / `ariaLabel`"). The enum is the seam
/// for any future per-field divergence — keep the routing explicit at every
/// call site so a future split doesn't silently mis-normalize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextField {
    Text,
    TextContains,
    TextPattern,
    AriaLabel,
}

/// The full five-stage normalization pipeline. Public for use by the
/// matcher's per-element index and by the distinctness validator.
pub fn normalize_text(input: &str) -> String {
    // Stage 1: HTML entity decode (named + decimal + hex).
    let decoded: String = html_escape::decode_html_entities(input).into_owned();

    // Stage 2: Unicode NFC. NOT NFKC — NFKC folds ① → 1, too aggressive for UI copy.
    let nfc: String = decoded.chars().nfc().collect();

    // Stage 3: Strip zero-width + bidi format characters.
    let stripped: String = nfc.chars().filter(|c| !is_invisible_format(*c)).collect();

    // Stage 4: Whitespace fold — NBSPs → space, collapse runs, trim.
    let folded = fold_whitespace(&stripped);

    // Stage 5: Unicode case fold via to_lowercase (locale-independent per UAX #44).
    folded.to_lowercase()
}

/// Routes via `TextField`. Currently all four variants route to
/// `normalize_text`, but keep call sites explicit so a future per-field
/// split is mechanical.
pub fn normalize_for_field(input: &str, _field: TextField) -> String {
    normalize_text(input)
}

/// Codepoints that carry no visible glyph and should be stripped before
/// whitespace folding. Includes zero-width spaces, joiners, BOM, LRM/RLM,
/// and bidi isolate / embedding marks.
fn is_invisible_format(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' // ZWSP
        | '\u{200C}' // ZWNJ
        | '\u{200D}' // ZWJ
        | '\u{FEFF}' // BOM (ZWNBSP)
        | '\u{200E}' // LRM
        | '\u{200F}' // RLM
        | '\u{202A}' // LRE
        | '\u{202B}' // RLE
        | '\u{202C}' // PDF
        | '\u{202D}' // LRO
        | '\u{202E}' // RLO
        | '\u{2066}' // LRI
        | '\u{2067}' // RLI
        | '\u{2068}' // FSI
        | '\u{2069}' // PDI
    )
}

/// Whitespace fold: NBSP (U+00A0), narrow NBSP (U+202F), and ASCII
/// whitespace (`\t\r\n\v\f ` ) collapse to a single ASCII space; runs are
/// merged; leading and trailing whitespace are trimmed.
fn fold_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_space_run = false;
    for c in input.chars() {
        let is_ws = matches!(
            c,
            '\u{00A0}' | '\u{202F}' | ' ' | '\t' | '\r' | '\n' | '\u{000B}' | '\u{000C}'
        );
        if is_ws {
            if !in_space_run && !out.is_empty() {
                out.push(' ');
            }
            in_space_run = true;
        } else {
            out.push(c);
            in_space_run = false;
        }
    }
    // Trim trailing space (we never added a leading one because `out.is_empty()` gated it).
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_named_html_entity() {
        assert_eq!(normalize_text("Save &amp; Quit"), "save & quit");
    }

    #[test]
    fn decodes_numeric_html_entity() {
        assert_eq!(normalize_text("&#34;hi&#34;"), "\"hi\"");
    }

    #[test]
    fn applies_nfc() {
        // e + combining acute → single codepoint é, then lowercased = é.
        assert_eq!(normalize_text("e\u{0301}"), "é");
    }

    #[test]
    fn does_not_apply_nfkc() {
        // ① (U+2460) is preserved — NFKC would fold to "1".
        assert_eq!(normalize_text("①"), "①");
    }

    #[test]
    fn strips_zero_width_space() {
        assert_eq!(normalize_text("a\u{200B}b"), "ab");
    }

    #[test]
    fn strips_zero_width_joiner() {
        assert_eq!(normalize_text("a\u{200D}b"), "ab");
    }

    #[test]
    fn strips_bom() {
        assert_eq!(normalize_text("\u{FEFF}hi"), "hi");
    }

    #[test]
    fn strips_lrm_rlm() {
        assert_eq!(normalize_text("a\u{200E}b\u{200F}c"), "abc");
    }

    #[test]
    fn strips_bidi_isolate_marks() {
        assert_eq!(normalize_text("a\u{2066}b\u{2069}c"), "abc");
    }

    #[test]
    fn folds_nbsp_to_space() {
        assert_eq!(normalize_text("a\u{00A0}b"), "a b");
    }

    #[test]
    fn folds_narrow_nbsp_to_space() {
        assert_eq!(normalize_text("a\u{202F}b"), "a b");
    }

    #[test]
    fn collapses_runs_of_whitespace() {
        assert_eq!(normalize_text("a  \t  b"), "a b");
    }

    #[test]
    fn trims_leading_trailing_whitespace() {
        assert_eq!(normalize_text("  hi  "), "hi");
    }

    #[test]
    fn case_folds_basic_ascii() {
        assert_eq!(normalize_text("Hello"), "hello");
    }

    #[test]
    fn case_folds_german_sharp_s() {
        // NOTE — discrepancy with plan 01-foundation.md Step 3 table.
        //
        // The plan asserts "Straße" → "strasse" with the comment "Handles
        // ß → ss". This is factually incorrect for `str::to_lowercase`.
        //
        // Rust's `str::to_lowercase` performs locale-independent Unicode
        // *lowercasing* (UAX #44), NOT *case folding*. Under lowercasing,
        // U+00DF (ß) is already lowercase and stays as ß — there is no
        // Unicode-defined lowercase mapping that expands it to "ss". The
        // "ß → SS" expansion only happens in the OTHER direction via
        // `str::to_uppercase`: "Straße".to_uppercase() == "STRASSE".
        //
        // Case *folding* (e.g. Unicode's CaseFolding.txt status=F) WOULD
        // fold ß → ss, but Rust's std does not expose case folding — only
        // lowercasing/uppercasing. Adding a case-folding crate to support
        // this single edge case is not worth the dependency.
        //
        // For our use case (uniform pipeline routed through identical UI
        // copy on both producer and consumer sides), the lowercasing
        // behavior is correct: producer and consumer both see "ß" and
        // both will compare equal. The plan's stated expectation was
        // simply mistaken about what `str::to_lowercase` does.
        //
        // Verified empirically with rustc (2026-05-13):
        //   "Straße".to_lowercase() == "straße"  // bytes: 73 74 72 61 C3 9F 65
        assert_eq!(normalize_text("Straße"), "straße");
    }

    #[test]
    fn case_folds_turkish_capital_i_with_dot() {
        // İ (U+0130, Latin Capital Letter I with Dot Above).
        // str::to_lowercase: U+0130 → "i" + COMBINING DOT ABOVE (U+0307).
        assert_eq!(normalize_text("İ"), "i\u{0307}");
    }

    #[test]
    fn preserves_smart_quotes() {
        // U+201C, U+201D = curly quotes. After lowercasing, they remain unchanged.
        assert_eq!(normalize_text("\u{201C}hi\u{201D}"), "\u{201C}hi\u{201D}");
    }

    #[test]
    fn preserves_em_dash() {
        // Em dash (U+2014) survives the pipeline. Note: it's surrounded by
        // single spaces — those are preserved as a single space (already a
        // single ASCII space, no folding needed).
        assert_eq!(normalize_text("a — b"), "a — b");
    }

    #[test]
    fn preserves_ellipsis_char() {
        // U+2026 (HORIZONTAL ELLIPSIS) survives unchanged.
        assert_eq!(normalize_text("a…"), "a…");
    }

    #[test]
    fn full_pipeline_composes() {
        // Input: leading whitespace, HTML entity, NBSP, all-caps, zero-width
        // space, trailing whitespace.
        // Pipeline:
        //   1. HTML decode:   "  & \u{00A0}HELLO\u{200B}  "
        //   2. NFC:           same (no precomposable chars)
        //   3. Strip ZWSP:    "  & \u{00A0}HELLO  "
        //   4. Fold ws:       "& HELLO"
        //   5. Lowercase:     "& hello"
        assert_eq!(normalize_text("  &amp; \u{00A0}HELLO\u{200B}  "), "& hello");
    }
}
