//! Content-addressed disk cache with LRU eviction.
//!
//! Phase 3 of the UI Bridge vision pipeline plan. Caller composes the cache
//! key from whatever inputs matter (window handle, mutation id, pipeline hash,
//! contract) via [`sha256_of`], looks up [`VisionCache::get`], and on miss
//! computes + stores via [`VisionCache::put`].
//!
//! Storage layout: `<root>/<sha256_hex>.<ext>`. Writes go through a `.tmp`
//! sibling then atomically rename, so concurrent agents on the same cache
//! directory never read half-written bytes (matches the atomic-write
//! convention adopted across the runner per cc-switch).
//!
//! Eviction is LRU on every insert that takes total bytes over `max_bytes`.
//! No time-based TTL — invalidation is mutation-keyed at the caller via the
//! cache key. An unchanged frame can coalesce indefinitely; any state-
//! changing action transparently busts the entry by bumping the mutation
//! component of the caller's key.

use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};

use crate::error::VisionError;

/// Content-addressed disk cache. Cheap to clone (shared via `Arc` in callers).
#[derive(Debug)]
pub struct VisionCache {
    root: PathBuf,
    max_bytes: u64,
    state: Mutex<CacheState>,
}

#[derive(Debug, Default)]
struct CacheState {
    entries: HashMap<String, CacheEntry>,
    /// Front = most recently used.
    lru: VecDeque<String>,
    total_bytes: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    path: PathBuf,
    bytes: u64,
}

/// A successful cache hit (from [`VisionCache::get`]) or freshly-inserted
/// entry (from [`VisionCache::put`]).
#[derive(Debug, Clone)]
pub struct CacheHit {
    pub sha256_hex: String,
    pub path: PathBuf,
    pub bytes: u64,
}

/// Snapshot of cache state. Returned by [`VisionCache::stats`].
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub entries: usize,
    pub total_bytes: u64,
    pub max_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl VisionCache {
    /// Open or create a cache at `root` with the given size cap.
    pub fn new(root: impl Into<PathBuf>, max_bytes: u64) -> Result<Self, VisionError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|e| {
            VisionError::CacheIo(format!("create_dir_all({}): {e}", root.display()))
        })?;
        Ok(Self {
            root,
            max_bytes,
            state: Mutex::new(CacheState::default()),
        })
    }

    /// Look up by key. Bumps the entry to MRU on hit.
    pub fn get(&self, key: &[u8; 32]) -> Option<CacheHit> {
        let hex = hex_encode(key);
        let mut state = self.state.lock().expect("cache mutex poisoned");
        if let Some(entry) = state.entries.get(&hex).cloned() {
            state.bump_lru(&hex);
            state.hits += 1;
            Some(CacheHit {
                sha256_hex: hex,
                path: entry.path,
                bytes: entry.bytes,
            })
        } else {
            state.misses += 1;
            None
        }
    }

    /// Insert `bytes` under `key`, naming the file with the given extension
    /// (no leading dot — e.g. `"jpg"`). Evicts older entries to stay within
    /// the size cap. Idempotent: re-inserting under the same key overwrites
    /// the previous entry and updates the byte accounting.
    pub fn put(&self, key: &[u8; 32], bytes: &[u8], ext: &str) -> Result<CacheHit, VisionError> {
        let hex = hex_encode(key);
        let final_path = self.root.join(format!("{hex}.{ext}"));
        let tmp_path = self.root.join(format!(".{hex}.{ext}.tmp"));

        // Atomic write: tempfile → rename. Best-effort sync before rename.
        {
            let mut f = File::create(&tmp_path).map_err(|e| {
                VisionError::CacheIo(format!("create({}): {e}", tmp_path.display()))
            })?;
            f.write_all(bytes)
                .map_err(|e| VisionError::CacheIo(format!("write({}): {e}", tmp_path.display())))?;
            let _ = f.sync_all();
        }
        fs::rename(&tmp_path, &final_path).map_err(|e| {
            // Clean up the stray tempfile on rename failure.
            let _ = fs::remove_file(&tmp_path);
            VisionError::CacheIo(format!(
                "rename({} -> {}): {e}",
                tmp_path.display(),
                final_path.display()
            ))
        })?;

        let mut state = self.state.lock().expect("cache mutex poisoned");
        let entry = CacheEntry {
            path: final_path.clone(),
            bytes: bytes.len() as u64,
        };
        if let Some(old) = state.entries.insert(hex.clone(), entry.clone()) {
            state.total_bytes = state.total_bytes.saturating_sub(old.bytes);
        }
        state.total_bytes += entry.bytes;
        state.bump_lru(&hex);
        state.evict_until(self.max_bytes);
        Ok(CacheHit {
            sha256_hex: hex,
            path: final_path,
            bytes: entry.bytes,
        })
    }

    /// Snapshot of current cache state.
    pub fn stats(&self) -> CacheStats {
        let state = self.state.lock().expect("cache mutex poisoned");
        CacheStats {
            entries: state.entries.len(),
            total_bytes: state.total_bytes,
            max_bytes: self.max_bytes,
            hits: state.hits,
            misses: state.misses,
            evictions: state.evictions,
        }
    }

    /// Cache root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl CacheState {
    fn bump_lru(&mut self, key: &str) {
        if let Some(pos) = self.lru.iter().position(|k| k == key) {
            self.lru.remove(pos);
        }
        self.lru.push_front(key.to_string());
    }

    fn evict_until(&mut self, max_bytes: u64) {
        while self.total_bytes > max_bytes {
            let Some(victim) = self.lru.pop_back() else {
                break;
            };
            if let Some(entry) = self.entries.remove(&victim) {
                self.total_bytes = self.total_bytes.saturating_sub(entry.bytes);
                self.evictions += 1;
                let _ = fs::remove_file(&entry.path);
            }
        }
    }
}

/// SHA-256 of arbitrary bytes. Useful for caller-side key composition:
/// `sha256_of(format!("win={win}/mut={mut_id}/pipe={pipe_hex}").as_bytes())`.
pub fn sha256_of(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn hex_encode(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn key(s: &str) -> [u8; 32] {
        sha256_of(s.as_bytes())
    }

    #[test]
    fn put_get_round_trip() {
        let dir = TempDir::new().unwrap();
        let cache = VisionCache::new(dir.path(), 1024 * 1024).unwrap();
        let k = key("alpha");
        let hit = cache.put(&k, b"hello", "txt").unwrap();
        assert_eq!(hit.bytes, 5);

        let got = cache.get(&k).expect("should hit");
        assert_eq!(got.sha256_hex, hit.sha256_hex);
        assert_eq!(std::fs::read(&got.path).unwrap(), b"hello");

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.total_bytes, 5);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn miss_increments_counter() {
        let dir = TempDir::new().unwrap();
        let cache = VisionCache::new(dir.path(), 1024).unwrap();
        let k = key("never-inserted");
        assert!(cache.get(&k).is_none());
        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn lru_evicts_oldest_when_over_cap() {
        let dir = TempDir::new().unwrap();
        // Cap of 10 bytes — fits exactly 2 of the 5-byte entries below.
        let cache = VisionCache::new(dir.path(), 10).unwrap();
        let k1 = key("one");
        let k2 = key("two");
        let k3 = key("three");

        cache.put(&k1, b"AAAAA", "txt").unwrap(); // total=5
        cache.put(&k2, b"BBBBB", "txt").unwrap(); // total=10
                                                  // Touch k1 so it's MRU.
        let _ = cache.get(&k1);
        cache.put(&k3, b"CCCCC", "txt").unwrap(); // total=15 → evict k2

        assert!(cache.get(&k1).is_some(), "k1 should survive (MRU)");
        assert!(cache.get(&k2).is_none(), "k2 evicted (LRU)");
        assert!(cache.get(&k3).is_some(), "k3 survives (freshly inserted)");

        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.total_bytes, 10);
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn reinsert_same_key_updates_bytes_accounting() {
        let dir = TempDir::new().unwrap();
        let cache = VisionCache::new(dir.path(), 1024).unwrap();
        let k = key("same");
        cache.put(&k, b"AAAA", "txt").unwrap(); // 4 bytes
        cache.put(&k, b"BBBBBBBB", "txt").unwrap(); // 8 bytes

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.total_bytes, 8);

        let hit = cache.get(&k).unwrap();
        assert_eq!(std::fs::read(&hit.path).unwrap(), b"BBBBBBBB");
    }

    #[test]
    fn atomic_write_leaves_no_temp_files_on_success() {
        let dir = TempDir::new().unwrap();
        let cache = VisionCache::new(dir.path(), 1024).unwrap();
        let k = key("atomic");
        cache.put(&k, b"payload", "bin").unwrap();
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].starts_with('.'), "no .tmp leftover");
    }

    #[test]
    fn sha256_of_is_deterministic() {
        assert_eq!(sha256_of(b"hello"), sha256_of(b"hello"));
        assert_ne!(sha256_of(b"hello"), sha256_of(b"world"));
    }
}
