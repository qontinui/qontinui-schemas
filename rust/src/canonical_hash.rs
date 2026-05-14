//! RFC 8785 canonical-JSON hashing primitive.
//!
//! Single source of truth for content hashing across Qontinui crates. Two
//! known callers today:
//!
//! - `qontinui-runner/src-tauri/src/spec_api/hashing.rs` — spec-content hash
//!   (with provenance pruning).
//! - `qontinui-runner/crates/spec-check/src/fetch.rs` — snapshot-body
//!   `content_sha256` for telemetry.
//!
//! Pipeline: any `Serialize` → RFC 8785 JCS via `serde_jcs` → SHA-256 →
//! `"sha256-<hex>"`. Key-order invariance is provided by JCS; downstream
//! callers don't need to pre-sort.

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Generic canonical content hash. Returns `"sha256-<hex>"`.
pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String, CanonicalHashError> {
    let canonical =
        serde_jcs::to_string(value).map_err(|e| CanonicalHashError::Canonicalize(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(format!("sha256-{}", hex::encode(hasher.finalize())))
}

#[derive(Debug, Error)]
pub enum CanonicalHashError {
    #[error("canonicalize failed: {0}")]
    Canonicalize(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn deterministic_across_runs() {
        let v = json!({"a": [1, 2, 3], "b": "hi"});
        let h1 = canonical_hash(&v).unwrap();
        let h2 = canonical_hash(&v).unwrap();
        assert_eq!(h1, h2);
    }
}
