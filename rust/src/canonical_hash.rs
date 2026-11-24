//! RFC 8785 canonical-JSON hashing primitive.
//!
//! Single source of truth for content hashing across Qontinui crates. The
//! production callers (audited 2026-11-22) are:
//!
//! - `image_slot::ImageSlotSpec::content_sha256` — the image-slot content
//!   address, recomputed in Python and TypeScript by the web app.
//! - `qontinui-runner/src-tauri/src/spec_api/hashing.rs` — spec-content hash
//!   (with provenance pruning).
//! - `qontinui-runner/src-tauri/src/spec_experimentation/versioning.rs` — the
//!   spec-version dedupe hash.
//! - `qontinui-runner/crates/spec-check/src/fetch.rs` — snapshot-body
//!   `content_sha256` for telemetry (arbitrary external JSON).
//! - `qontinui-runner/crates/spec-check/src/evaluator.rs` (`hash_spec`) — the
//!   IR spec content hash.
//! - `qontinui-coord/crates/coord/src/recurring_agents.rs` (`spec_digest`) —
//!   the recurring-agent spec digest devices echo back as `applied_digest`.
//! - `ledger::chain_hash` — through [`canonical_json`].
//!
//! Pipeline: any `Serialize` → number audit → `serde_json::Value` →
//! [`canonical_json`] (RFC 8785: UTF-16 key order, §3.2.2.2 string escapes,
//! ECMAScript numbers) → SHA-256 → `"sha256-<hex>"`. Key-order invariance is
//! provided by JCS; downstream callers don't need to pre-sort. Any conforming
//! JCS implementation in any language reproduces the digest of a value whose
//! numbers are `f64` or integers (an `f32` field is the one exception, below).
//!
//! RFC 8785 models every number as an IEEE-754 double, so [`canonical_hash`]
//! refuses NaN, the infinities and integers beyond ±[`MAX_SAFE_INTEGER`]
//! rather than hash a value another language would write differently.
//! A Rust `f32` is hashed as its widened `f64` (`0.1f32` hashes as
//! `0.10000000149011612`), because the value is converted to JSON first.
//! That differs from the shortest-`f32` text a JSON writer emits (`0.1`), so a
//! digest over an `f32`-bearing value is NOT reproducible by another language
//! reading that text. Hash `f64` fields, or a `Value` already holding the
//! intended double, when the digest crosses a language boundary.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// The largest integer RFC 8785 (I-JSON) carries exactly, 2^53 − 1. Larger
/// integers are rounded by every canonicalizer that reads numbers as IEEE 754
/// doubles, so a hash over them would differ between languages.
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Generic canonical content hash. Returns `"sha256-<hex>"`.
///
/// Refuses (rather than hashes) a value holding a number RFC 8785 cannot
/// carry exactly; see the module docs.
pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String, CanonicalHashError> {
    let mut audit = NumberAudit::default();
    value
        .serialize(&mut audit)
        .map_err(|e| CanonicalHashError::Serialize(e.to_string()))?;
    if let Some(problem) = audit.found {
        return Err(problem);
    }
    let value =
        serde_json::to_value(value).map_err(|e| CanonicalHashError::Serialize(e.to_string()))?;
    let canonical = canonical_json(&value);
    Ok(format!(
        "sha256-{}",
        hex::encode(Sha256::digest(canonical.as_bytes()))
    ))
}

/// Why a value has no canonical hash.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CanonicalHashError {
    /// NaN or an infinity at `path`. JSON has no spelling for either, and
    /// serializers write `null` in its place.
    #[error("{path}: NaN or infinity has no RFC 8785 form")]
    NonFiniteNumber {
        /// Where, in the value's own spelling (`contexts.dpr[1]`).
        path: String,
    },
    /// An integer beyond ±[`MAX_SAFE_INTEGER`] at `path`, which a
    /// canonicalizer reading numbers as doubles would round.
    #[error("{path}: {value} is beyond 2^53 - 1, which RFC 8785 does not carry exactly")]
    UnsafeInteger {
        /// Where, in the value's own spelling.
        path: String,
        /// The integer, in decimal.
        value: String,
    },
    /// The value could not be serialized to JSON.
    #[error("serialize failed: {0}")]
    Serialize(String),
}

/// The RFC 8785 (JSON Canonicalization Scheme) text of `value`: no
/// whitespace; object members sorted by the UTF-16 code units of their
/// unescaped names (§3.2.3); strings escaped per §3.2.2.2 (only `"`, `\\`
/// and U+0000–U+001F, the latter as `\b \t \n \f \r` or lower-case
/// `\u00xx`); numbers in ECMAScript `Number.prototype.toString` form
/// (§3.2.2.3, via `ryu-js`; `-0` is `0`).
///
/// A JSON number is an IEEE-754 double in RFC 8785, so an integer beyond
/// `±MAX_SAFE_INTEGER` is written as the double nearest to it; [`canonical_hash`]
/// refuses such integers (and NaN and the infinities) rather than hash a
/// rounded value, and the ledger's `validate`
/// keeps every envelope integer inside that range, where this is exact.
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(&mut out, value);
    out
}

fn write_canonical(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            // serde_json holds no non-finite number.
            let f = n.as_f64().unwrap_or(0.0);
            if f == 0.0 {
                out.push('0'); // also -0
            } else {
                out.push_str(ryu_js::Buffer::new().format_finite(f));
            }
        }
        Value::String(s) => write_canonical_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut members: Vec<(&String, &Value)> = map.iter().collect();
            members.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            out.push('{');
            for (i, (name, item)) in members.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical_string(out, name);
                out.push(':');
                write_canonical(out, item);
            }
            out.push('}');
        }
    }
}

fn write_canonical_string(out: &mut String, s: &str) {
    const BACKSLASH: char = '\\';
    out.push('"');
    for c in s.chars() {
        // §3.2.2.2: the two-character escapes, then `\u00xx` for the rest
        // of U+0000–U+001F; every other character is written as itself.
        let short = match c {
            '"' => Some('"'),
            BACKSLASH => Some(BACKSLASH),
            '\u{8}' => Some('b'),
            '\t' => Some('t'),
            '\n' => Some('n'),
            '\u{c}' => Some('f'),
            '\r' => Some('r'),
            _ => None,
        };
        if let Some(e) = short {
            out.push(BACKSLASH);
            out.push(e);
        } else if (c as u32) < 0x20 {
            out.push(BACKSLASH);
            out.push_str(&format!("u{:04x}", c as u32));
        } else {
            out.push(c);
        }
    }
    out.push('"');
}

type AuditError = serde::de::value::Error;

/// Walks a value through its `Serialize` impl and records the first number
/// RFC 8785 cannot carry exactly, with its path. It reads every number serde
/// emits, so a numeric field added later is audited without being listed here.
#[derive(Default)]
struct NumberAudit {
    path: Vec<String>,
    found: Option<CanonicalHashError>,
}

impl NumberAudit {
    fn here(&self) -> String {
        let mut out = String::new();
        for part in &self.path {
            if !out.is_empty() && !part.starts_with('[') {
                out.push('.');
            }
            out.push_str(part);
        }
        out
    }

    fn float(&mut self, value: f64) -> Result<(), AuditError> {
        if !value.is_finite() && self.found.is_none() {
            self.found = Some(CanonicalHashError::NonFiniteNumber { path: self.here() });
        }
        Ok(())
    }

    fn signed(&mut self, value: i128) -> Result<(), AuditError> {
        if value.unsigned_abs() > u128::from(MAX_SAFE_INTEGER) && self.found.is_none() {
            self.found = Some(CanonicalHashError::UnsafeInteger {
                path: self.here(),
                value: value.to_string(),
            });
        }
        Ok(())
    }

    fn unsigned(&mut self, value: u128) -> Result<(), AuditError> {
        if value > u128::from(MAX_SAFE_INTEGER) && self.found.is_none() {
            self.found = Some(CanonicalHashError::UnsafeInteger {
                path: self.here(),
                value: value.to_string(),
            });
        }
        Ok(())
    }

    fn nested<T: ?Sized + Serialize>(&mut self, part: String, value: &T) -> Result<(), AuditError> {
        self.path.push(part);
        let result = value.serialize(&mut *self);
        self.path.pop();
        result
    }
}

impl<'a> serde::Serializer for &'a mut NumberAudit {
    type Ok = ();
    type Error = AuditError;
    type SerializeSeq = AuditSeq<'a>;
    type SerializeTuple = AuditSeq<'a>;
    type SerializeTupleStruct = AuditSeq<'a>;
    type SerializeTupleVariant = AuditSeq<'a>;
    type SerializeMap = AuditSeq<'a>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _: bool) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), AuditError> {
        self.signed(v.into())
    }
    fn serialize_i16(self, v: i16) -> Result<(), AuditError> {
        self.signed(v.into())
    }
    fn serialize_i32(self, v: i32) -> Result<(), AuditError> {
        self.signed(v.into())
    }
    fn serialize_i64(self, v: i64) -> Result<(), AuditError> {
        self.signed(v.into())
    }
    fn serialize_i128(self, v: i128) -> Result<(), AuditError> {
        self.signed(v)
    }
    fn serialize_u8(self, v: u8) -> Result<(), AuditError> {
        self.unsigned(v.into())
    }
    fn serialize_u16(self, v: u16) -> Result<(), AuditError> {
        self.unsigned(v.into())
    }
    fn serialize_u32(self, v: u32) -> Result<(), AuditError> {
        self.unsigned(v.into())
    }
    fn serialize_u64(self, v: u64) -> Result<(), AuditError> {
        self.unsigned(v.into())
    }
    fn serialize_u128(self, v: u128) -> Result<(), AuditError> {
        self.unsigned(v)
    }
    fn serialize_f32(self, v: f32) -> Result<(), AuditError> {
        self.float(v.into())
    }
    fn serialize_f64(self, v: f64) -> Result<(), AuditError> {
        self.float(v)
    }
    fn serialize_char(self, _: char) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), AuditError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        self.nested(variant.to_owned(), value)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<AuditSeq<'a>, AuditError> {
        Ok(AuditSeq {
            audit: self,
            index: 0,
            pops_variant: false,
        })
    }
    fn serialize_tuple(self, _: usize) -> Result<AuditSeq<'a>, AuditError> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<AuditSeq<'a>, AuditError> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<AuditSeq<'a>, AuditError> {
        self.path.push(variant.to_owned());
        Ok(AuditSeq {
            audit: self,
            index: 0,
            pops_variant: true,
        })
    }
    fn serialize_map(self, _: Option<usize>) -> Result<AuditSeq<'a>, AuditError> {
        self.serialize_seq(None)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, AuditError> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self, AuditError> {
        self.path.push(variant.to_owned());
        Ok(self)
    }
}

impl serde::ser::SerializeStruct for &mut NumberAudit {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        self.nested(key.to_owned(), value)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}

impl serde::ser::SerializeStructVariant for &mut NumberAudit {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        self.nested(key.to_owned(), value)
    }
    fn end(self) -> Result<(), AuditError> {
        self.path.pop();
        Ok(())
    }
}

/// A sequence, tuple or map under audit: elements are addressed `[i]`, map
/// entries `[i]` in emission order (a key is audited like a value).
struct AuditSeq<'a> {
    audit: &'a mut NumberAudit,
    index: usize,
    /// A tuple variant pushed its variant name; `end` pops it.
    pops_variant: bool,
}

impl AuditSeq<'_> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), AuditError> {
        let part = format!("[{}]", self.index);
        self.index += 1;
        self.audit.nested(part, value)
    }
}

impl serde::ser::SerializeSeq for AuditSeq<'_> {
    type Ok = ();
    type Error = AuditError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), AuditError> {
        self.element(value)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}

impl serde::ser::SerializeTuple for AuditSeq<'_> {
    type Ok = ();
    type Error = AuditError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), AuditError> {
        self.element(value)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleStruct for AuditSeq<'_> {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), AuditError> {
        self.element(value)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleVariant for AuditSeq<'_> {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), AuditError> {
        self.element(value)
    }
    fn end(self) -> Result<(), AuditError> {
        if self.pops_variant {
            self.audit.path.pop();
        }
        Ok(())
    }
}

impl serde::ser::SerializeMap for AuditSeq<'_> {
    type Ok = ();
    type Error = AuditError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), AuditError> {
        // The key and the value of one entry share the entry's index.
        let part = format!("[{}]", self.index);
        self.audit.nested(part, key)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), AuditError> {
        self.element(value)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Known answers generated by the independent `rfc8785` Python package
    // (Trail of Bits) from the same inputs — never by `json.dumps` — for the
    // cases where JCS implementations are known to differ: prefix keys, a
    // control-character key, a supplementary-plane vs BMP key, ECMAScript
    // number forms (1e-7, 1e+21, 1e20, -0, 0.5, ...), and RFC 8785's own
    // §3.2.3 sorting example and §3.2.4 example, whose outputs match the
    // RFC's published text.
    /// Generated by rfc8785 0.1.4 (independent JCS); see the test's doc comment.
    const JCS_VECTORS: &[(&str, &str, &str)] = &[
        (
            "prefix keys",
            r#"{"a b": 1, "a": 2}"#,
            "{\"a\":2,\"a b\":1}",
        ),
        (
            "control-char key",
            r#"{"A": 1, "\n": 2}"#,
            "{\"\\n\":2,\"A\":1}",
        ),
        (
            "supplementary vs BMP key",
            r#"{"\ud83d\ude00": 1, "\uff01": 2}"#,
            "{\"\u{1F600}\":1,\"\u{FF01}\":2}",
        ),
        (
            "numbers",
            r#"[1e-07, 1e+21, 1e+20, 1e-06, -0.0, 0.5, 4.5, 0.002, 333333333.3333333, 1e+30, 5e-324, 1.7976931348623157e+308, 9007199254740991, -9007199254740991, 1e-27, 100, 0.30000000000000004]"#,
            "[1e-7,1e+21,100000000000000000000,0.000001,0,0.5,4.5,0.002,333333333.3333333,1e+30,5e-324,1.7976931348623157e+308,9007199254740991,-9007199254740991,1e-27,100,0.30000000000000004]",
        ),
        (
            "rfc8785 3.2.3 sorting example",
            r#"{"\u20ac": "Euro Sign", "\r": "Carriage Return", "\ufb33": "Hebrew Letter Dalet With Dagesh", "1": "One", "\ud83d\ude00": "Emoji: Grinning Face", "\u0080": "Control", "\u00f6": "Latin Small Letter O With Diaeresis"}"#,
            "{\"\\r\":\"Carriage Return\",\"1\":\"One\",\"\u{80}\":\"Control\",\"\u{F6}\":\"Latin Small Letter O With Diaeresis\",\"\u{20AC}\":\"Euro Sign\",\"\u{1F600}\":\"Emoji: Grinning Face\",\"\u{FB33}\":\"Hebrew Letter Dalet With Dagesh\"}",
        ),
        (
            "rfc8785 3.2.4 example",
            r#"{"numbers": [333333333.3333333, 1e+30, 4.5, 0.002, 1e-27], "string": "\u20ac$\u000f\nA'B\"\\\\\"/", "literals": [null, true, false]}"#,
            "{\"literals\":[null,true,false],\"numbers\":[333333333.3333333,1e+30,4.5,0.002,1e-27],\"string\":\"\u{20AC}$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\"}",
        ),
        (
            "string escapes",
            r#"{"s": "\u0000\b\t\n\u000b\f\r\u001f\u007f\u2028\"\\/"}"#,
            "{\"s\":\"\\u0000\\b\\t\\n\\u000b\\f\\r\\u001f\u{7F}\u{2028}\\\"\\\\/\"}",
        ),
    ];
    #[test]
    fn canonical_json_matches_an_independent_jcs() {
        for (name, input, expected) in JCS_VECTORS {
            let value: Value = serde_json::from_str(input).unwrap();
            assert_eq!(canonical_json(&value), *expected, "{name}");
        }
    }
    // One case per divergence row of the plan's table; expected texts come
    // from the independent `rfc8785` package, never from this code.
    const DIVERGENCE_CASES: &[(&str, &str, &str)] = &[
        (
            "prefix + space",
            r##"{"a b": 1, "a": 2}"##,
            "{\"a\":2,\"a b\":1}",
        ),
        (
            "prefix + bang",
            r##"{"a!": 1, "a": 2}"##,
            "{\"a\":2,\"a!\":1}",
        ),
        (
            "quote vs hash",
            r##"{"\"": 1, "#": 2}"##,
            "{\"\\\"\":1,\"#\":2}",
        ),
        (
            "control vs A",
            r##"{"A": 1, "\n": 2}"##,
            "{\"\\n\":2,\"A\":1}",
        ),
        (
            "control vs space",
            r##"{" ": 1, "\n": 2}"##,
            "{\"\\n\":2,\" \":1}",
        ),
        (
            "U+0001 vs U+000A",
            r##"{"\n": 1, "\u0001": 2}"##,
            "{\"\\u0001\":2,\"\\n\":1}",
        ),
        (
            "quote vs control",
            r##"{"\n": 1, "\"": 2}"##,
            "{\"\\n\":1,\"\\\"\":2}",
        ),
        (
            "backslash vs control",
            r##"{"\n": 1, "\\": 2}"##,
            "{\"\\n\":1,\"\\\\\":2}",
        ),
        (
            "astral vs BMP",
            r##"{"\uff01": 1, "\ud83d\ude00": 2}"##,
            "{\"\u{1F600}\":2,\"\u{FF01}\":1}",
        ),
    ];

    #[test]
    fn every_divergence_case_is_rfc_8785_order() {
        for (name, input, expected) in DIVERGENCE_CASES {
            let v: Value = serde_json::from_str(input).unwrap();
            assert_eq!(canonical_json(&v), *expected, "{name}");
        }
    }

    #[test]
    fn hash_is_stable_across_key_order() {
        let a = json!({"alpha": 1, "beta": 2, "gamma": 3});
        let b = json!({"gamma": 3, "alpha": 1, "beta": 2});
        assert_eq!(canonical_hash(&a).unwrap(), canonical_hash(&b).unwrap());
    }

    #[test]
    fn output_has_sha256_prefix_and_64_hex_chars() {
        let h = canonical_hash(&json!({"x": 1})).unwrap();
        assert!(h.starts_with("sha256-"));
        let hex_part = &h["sha256-".len()..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn prefix_keys_hash_in_rfc_8785_order() {
        // sha256 of `{"a":2,"a b":1}`, the RFC 8785 form (computed by hashlib
        // over rfc8785.dumps, not by this crate).
        assert_eq!(
            canonical_hash(&json!({"a b": 1, "a": 2})).unwrap(),
            "sha256-1b4441f4bbb54a6a66f7655c342a7e05f27ce3f9547d25570bf8b4ed70be0600"
        );
    }

    #[test]
    fn nested_object_hash_matches_an_independent_jcs() {
        let v: Value = serde_json::from_str(
            r##"{"prefix + space": {"a b": 1, "a": 2}, "prefix + bang": {"a!": 1, "a": 2}, "quote vs hash": {"\"": 1, "#": 2}, "control vs A": {"A": 1, "\n": 2}, "control vs space": {" ": 1, "\n": 2}, "U+0001 vs U+000A": {"\n": 1, "\u0001": 2}, "quote vs control": {"\n": 1, "\"": 2}, "backslash vs control": {"\n": 1, "\\": 2}, "astral vs BMP": {"\uff01": 1, "\ud83d\ude00": 2}, "z": [1, 2.5, "x"]}"##,
        )
        .unwrap();
        assert_eq!(
            canonical_hash(&v).unwrap(),
            "sha256-5f5f6ceb2bde5e53a7cda1b78dcfa6ef92d3519df2dd8d62346b5dc5b62e04fc"
        );
    }

    #[test]
    fn refuses_numbers_rfc_8785_cannot_carry() {
        assert_eq!(
            canonical_hash(&vec![1.0, f64::NAN]),
            Err(CanonicalHashError::NonFiniteNumber { path: "[1]".into() })
        );
        assert_eq!(
            canonical_hash(&f64::INFINITY),
            Err(CanonicalHashError::NonFiniteNumber {
                path: String::new()
            })
        );
        for (n, text) in [
            (9_007_199_254_740_992_i64, "9007199254740992"),
            (-9_007_199_254_740_992_i64, "-9007199254740992"),
        ] {
            assert_eq!(
                canonical_hash(&vec![n]),
                Err(CanonicalHashError::UnsafeInteger {
                    path: "[0]".into(),
                    value: text.into()
                })
            );
        }
        assert!(canonical_hash(&vec![9_007_199_254_740_991_i64]).is_ok());
        assert!(canonical_hash(&vec![-9_007_199_254_740_991_i64]).is_ok());
    }

    #[test]
    fn refusal_paths_follow_the_serialized_shape() {
        use std::collections::BTreeMap;
        let mut m = BTreeMap::new();
        m.insert("k", vec![f64::NAN]);
        assert_eq!(
            canonical_hash(&m),
            Err(CanonicalHashError::NonFiniteNumber {
                path: "[0][0]".into()
            })
        );
        #[derive(Serialize)]
        struct Inner {
            n: f64,
        }
        #[derive(Serialize)]
        enum E {
            S { inner: Inner },
            T(u8, f64),
        }
        assert_eq!(
            canonical_hash(&E::S {
                inner: Inner { n: f64::INFINITY }
            }),
            Err(CanonicalHashError::NonFiniteNumber {
                path: "S.inner.n".into()
            })
        );
        assert_eq!(
            canonical_hash(&E::T(1, f64::NAN)),
            Err(CanonicalHashError::NonFiniteNumber {
                path: "T[1]".into()
            })
        );
        // A real Value map (the external-JSON path) reports the entry index.
        let v: Value = serde_json::from_str(r#"{"a": 1, "b": 9007199254740993}"#).unwrap();
        assert_eq!(
            canonical_hash(&v),
            Err(CanonicalHashError::UnsafeInteger {
                path: "[1]".into(),
                value: "9007199254740993".into()
            })
        );
    }

    #[test]
    fn struct_and_equivalent_json_hash_the_same() {
        #[derive(Serialize)]
        struct S {
            zeta: u8,
            #[serde(rename = "a b")]
            a_b: &'static str,
            a: Vec<f64>,
        }
        let s = S {
            zeta: 1,
            a_b: "x",
            a: vec![0.5, 2.0],
        };
        let j = json!({"a": [0.5, 2.0], "a b": "x", "zeta": 1});
        assert_eq!(canonical_hash(&s).unwrap(), canonical_hash(&j).unwrap());
    }

    #[test]
    fn f32_is_hashed_as_its_widened_double() {
        // Documented departure (module docs): not the shortest f32 form.
        #[derive(Serialize)]
        struct F {
            x: f32,
        }
        assert_eq!(
            canonical_hash(&F { x: 0.1 }).unwrap(),
            canonical_hash(&json!({"x": 0.10000000149011612_f64})).unwrap()
        );
    }
}
