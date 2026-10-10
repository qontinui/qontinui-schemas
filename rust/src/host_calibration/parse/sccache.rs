//! `sccache --show-stats --stats-format json`.
//!
//! The document is sccache's serialized `ServerInfo`:
//!
//! ```text
//! {"stats":{"compile_requests":…,"cache_hits":{"counts":{"Rust":…},"adv_counts":{…}},
//!   "cache_misses":{…},"cache_errors":{…},"not_cached":{"<reason>":n,…},…},
//!  "cache_location":"Local disk: \"…\"","cache_size":…,"max_cache_size":…,"version":"…"}
//! ```
//!
//! Parsed tolerantly: sccache has added fields across releases (`adv_counts`,
//! `version`, the `requests_*` counters), so every field but the hit and miss
//! counts is optional. `cache_location` is deliberately NOT carried: it embeds
//! a host path, and the fact set is host-agnostic (plan D12).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One sccache daemon's counters.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct SccacheStats {
    /// Cache hits, summed over languages.
    pub hits: u64,
    /// Cache misses, summed over languages.
    pub misses: u64,
    /// Hits per language (`Rust`, `C/C++`, …).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hits_by_language: BTreeMap<String, u64>,
    /// Cache errors, summed over languages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_errors: Option<u64>,
    /// `compile_requests`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compile_requests: Option<u64>,
    /// `requests_executed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requests_executed: Option<u64>,
    /// `requests_not_cacheable`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requests_not_cacheable: Option<u64>,
    /// `non_cacheable_compilations`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_cacheable_compilations: Option<u64>,
    /// `not_cached`: reason → count (`crate-type`, `incremental`, …).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub non_cacheable_reasons: BTreeMap<String, u64>,
    /// `cache_size`, bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_size_bytes: Option<u64>,
    /// `max_cache_size`, bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cache_size_bytes: Option<u64>,
    /// Entries of the count maps read (hits, misses, errors, `not_cached`)
    /// whose value was not a non-negative integer and so was LEFT OUT of the
    /// sums above. Non-zero means the hit/miss totals are lower bounds.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub non_integer_entries_skipped: u64,
    /// sccache's own version, when it reports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl SccacheStats {
    /// `hits / (hits + misses)`; `None` before any cacheable request (a rate
    /// over zero requests is UNKNOWN, not 0 %).
    pub fn hit_rate(&self) -> Option<f64> {
        let total = self.hits.checked_add(self.misses)?;
        (total > 0).then(|| self.hits as f64 / total as f64)
    }
}

/// Why a stats document did not parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SccacheParseError {
    /// Not JSON.
    #[error("sccache stats are not valid JSON: {0}")]
    InvalidJson(String),
    /// JSON without the `stats.cache_hits` / `stats.cache_misses` objects.
    #[error("sccache stats document has no stats.{0} object")]
    MissingField(&'static str),
}

/// Parse `sccache --show-stats --stats-format json` output.
pub fn parse_sccache_stats_json(text: &str) -> Result<SccacheStats, SccacheParseError> {
    let doc: Value = serde_json::from_str(text.trim())
        .map_err(|e| SccacheParseError::InvalidJson(e.to_string()))?;
    let stats = doc
        .get("stats")
        .ok_or(SccacheParseError::MissingField("stats"))?;
    let mut skipped = 0u64;
    let hits_by_language = language_counts(stats.get("cache_hits"), &mut skipped)
        .ok_or(SccacheParseError::MissingField("cache_hits"))?;
    let misses = language_counts(stats.get("cache_misses"), &mut skipped)
        .ok_or(SccacheParseError::MissingField("cache_misses"))?;
    let cache_errors =
        language_counts(stats.get("cache_errors"), &mut skipped).map(|m| m.values().sum());
    let non_cacheable_reasons =
        count_map(stats.get("not_cached"), &mut skipped).unwrap_or_default();
    let u = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64);
    Ok(SccacheStats {
        hits: hits_by_language.values().sum(),
        misses: misses.values().sum(),
        hits_by_language,
        cache_errors,
        compile_requests: u(stats, "compile_requests"),
        requests_executed: u(stats, "requests_executed"),
        requests_not_cacheable: u(stats, "requests_not_cacheable"),
        non_cacheable_compilations: u(stats, "non_cacheable_compilations"),
        non_cacheable_reasons,
        cache_size_bytes: u(&doc, "cache_size"),
        max_cache_size_bytes: u(&doc, "max_cache_size"),
        non_integer_entries_skipped: skipped,
        version: doc
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// A `PerLanguageCount` (`{"counts":{…}}`) → its `counts` map.
fn language_counts(v: Option<&Value>, skipped: &mut u64) -> Option<BTreeMap<String, u64>> {
    count_map(v?.get("counts"), skipped)
}

/// A `{"key": n}` object → map. Entries whose value is not a non-negative
/// integer are left out AND counted into `skipped`, never dropped silently.
fn count_map(v: Option<&Value>, skipped: &mut u64) -> Option<BTreeMap<String, u64>> {
    let obj = v?.as_object()?;
    let mut out = BTreeMap::new();
    for (k, n) in obj {
        match n.as_u64() {
            Some(n) => {
                out.insert(k.clone(), n);
            }
            None => *skipped += 1,
        }
    }
    Some(out)
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATS: &str = r#"{"stats":{"compile_requests":120,"requests_unsupported_compiler":0,
      "requests_not_compile":10,"requests_not_cacheable":7,"requests_executed":103,
      "cache_errors":{"counts":{},"adv_counts":{}},
      "cache_hits":{"counts":{"Rust":60,"C/C++":4},"adv_counts":{"rust":60}},
      "cache_misses":{"counts":{"Rust":36},"adv_counts":{"rust":36}},
      "cache_timeouts":0,"cache_read_errors":0,"non_cacheable_compilations":0,
      "forced_recaches":0,"cache_write_errors":0,"cache_writes":36,
      "cache_write_duration":{"secs":1,"nanos":0},"cache_read_hit_duration":{"secs":0,"nanos":5},
      "compiler_write_duration":{"secs":0,"nanos":0},"compilations":36,"compile_fails":0,
      "not_cached":{"crate-type":5,"incremental":2},"dist_compiles":{},"dist_errors":0},
      "cache_location":"Local disk: \"/somewhere/sccache\"","cache_size":524288000,
      "max_cache_size":10737418240,"use_preprocessor_cache_mode":false,"version":"0.10.0"}"#;

    #[test]
    fn parses_a_current_document() {
        let s = parse_sccache_stats_json(STATS).unwrap();
        assert_eq!(s.hits, 64);
        assert_eq!(s.misses, 36);
        assert_eq!(s.hits_by_language.get("Rust"), Some(&60));
        assert_eq!(s.cache_errors, Some(0));
        assert_eq!(s.compile_requests, Some(120));
        assert_eq!(s.requests_not_cacheable, Some(7));
        assert_eq!(s.non_cacheable_reasons.get("crate-type"), Some(&5));
        assert_eq!(s.cache_size_bytes, Some(524_288_000));
        assert_eq!(s.max_cache_size_bytes, Some(10_737_418_240));
        assert_eq!(s.version.as_deref(), Some("0.10.0"));
        assert_eq!(s.hit_rate(), Some(0.64));
        // The host path is not carried.
        assert!(!serde_json::to_string(&s).unwrap().contains("somewhere"));
    }

    #[test]
    fn parses_an_older_document_without_optional_fields() {
        let s = parse_sccache_stats_json(
            r#"{"stats":{"cache_hits":{"counts":{}},"cache_misses":{"counts":{}}},"cache_size":null}"#,
        )
        .unwrap();
        assert_eq!((s.hits, s.misses), (0, 0));
        assert_eq!(s.hit_rate(), None);
        assert_eq!(s.cache_size_bytes, None);
        assert_eq!(s.version, None);
    }

    #[test]
    fn non_integer_count_entries_are_counted_not_silently_dropped() {
        let s = parse_sccache_stats_json(
            r#"{"stats":{"cache_hits":{"counts":{"Rust":3,"C":-1}},"cache_misses":{"counts":{"Rust":"x"}},"not_cached":{"a":1.5}}}"#,
        )
        .unwrap();
        assert_eq!((s.hits, s.misses), (3, 0));
        assert_eq!(s.non_integer_entries_skipped, 3);
        assert_eq!(
            parse_sccache_stats_json(STATS)
                .unwrap()
                .non_integer_entries_skipped,
            0
        );
    }

    #[test]
    fn rejects_non_stats() {
        assert!(matches!(
            parse_sccache_stats_json("Compile requests 1"),
            Err(SccacheParseError::InvalidJson(_))
        ));
        assert_eq!(
            parse_sccache_stats_json(r#"{"x":1}"#),
            Err(SccacheParseError::MissingField("stats"))
        );
        assert_eq!(
            parse_sccache_stats_json(r#"{"stats":{"cache_hits":{"counts":{}}}}"#),
            Err(SccacheParseError::MissingField("cache_misses"))
        );
    }
}
