//! The non-pressure fact groups of plan D3 — compile cache, build volume,
//! coarse disk use — and the per-computer roll-up [`CalibrationFacts`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::leak::LeakRecord;
use super::measured::{Measured, MeasuredManifest, MeasuredState};
use super::parse::sccache::SccacheStats;
use super::pressure::{CgroupPressureFacts, Platform};

/// Compile-cache trust class (plan D4; the CI classes are the runner-fleet
/// plan's D12 classes, not re-decided here).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CompileCacheClass {
    /// Agent sessions' per-user cache (PR-class-equivalent).
    Agent,
    /// CI pull-request / candidate jobs.
    Pr,
    /// CI main-push jobs.
    Trusted,
    /// CI jobs for public repos.
    Public,
}

/// What a compile-cache probe found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompileCacheReading {
    /// No sccache binary on this host — a measured fact, not a failed read.
    NotInstalled,
    /// The daemon answered with these counters.
    Stats(SccacheStats),
}

/// One cache class's reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompileCacheClassFacts {
    /// The class.
    pub class: CompileCacheClass,
    /// The reading; `unavailable` when the probe failed or timed out.
    pub reading: Measured<CompileCacheReading>,
    /// Whether builds of this class are routed through the cache
    /// (`RUSTC_WRAPPER` set by the wrapper/profile).
    pub wrapper_configured: Measured<bool>,
}

/// The `compile_cache` fact.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct CompileCacheFacts {
    /// One entry per class probed on this host.
    pub classes: Vec<CompileCacheClassFacts>,
}

/// What kind of fast build volume this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildVolumeKind {
    /// Linux: the dedicated calibration tmpfs.
    Tmpfs,
    /// Windows: a Dev Drive (ReFS).
    DevDrive,
}

/// The `build_volume` fact (plan D5 lever 2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BuildVolumeState {
    /// Which kind of volume this platform would use.
    pub kind: BuildVolumeKind,
    /// Whether the volume is mounted / present.
    pub mount_present: Measured<bool>,
    /// Whether the root-owned calibration marker identifies it as
    /// calibration-provisioned (routing requires it).
    pub calibration_marker_present: Measured<bool>,
    /// Filesystem size, bytes.
    pub size_bytes: Measured<u64>,
    /// Bytes used.
    pub used_bytes: Measured<u64>,
    /// Inodes free (tmpfs has its own inode table; ENOSPC on inodes is the
    /// failure mode the floor guards).
    pub inodes_free: Measured<u64>,
    /// Pages of the volume that are in swap, bytes. `not_supported` on a Dev
    /// Drive (not memory-backed).
    pub swap_backed_bytes: Measured<u64>,
}

impl BuildVolumeState {
    /// No volume present: every size axis is `not_supported` for a volume
    /// that does not exist, which is distinct from a failed read.
    pub fn absent(kind: BuildVolumeKind) -> Self {
        Self {
            kind,
            mount_present: Measured::Measured(false),
            calibration_marker_present: Measured::Measured(false),
            size_bytes: Measured::NotSupported,
            used_bytes: Measured::NotSupported,
            inodes_free: Measured::NotSupported,
            swap_backed_bytes: Measured::NotSupported,
        }
    }

    fn manifest_into(&self, out: &mut MeasuredManifest) {
        let mut put = |k: &str, s: MeasuredState| {
            out.insert(format!("build_volume.{k}"), s);
        };
        put("mount_present", self.mount_present.state());
        put(
            "calibration_marker_present",
            self.calibration_marker_present.state(),
        );
        put("size_bytes", self.size_bytes.state());
        put("used_bytes", self.used_bytes.state());
        put("inodes_free", self.inodes_free.state());
        put("swap_backed_bytes", self.swap_backed_bytes.state());
    }
}

/// A coarse disk consumer class (plan D3: `du` bounded to declared roots).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DiskConsumer {
    /// Cargo target dirs.
    TargetDirs,
    /// Compile caches (sccache dirs).
    CompileCaches,
    /// Docker images, containers and volumes.
    Docker,
    /// Agent worktrees, excluding their target dirs.
    Worktrees,
}

/// One consumer's footprint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiskUseEntry {
    /// The consumer class.
    pub consumer: DiskConsumer,
    /// Bytes used.
    pub bytes: Measured<u64>,
    /// True when the walk hit its time budget, so `bytes` is a LOWER bound.
    #[serde(default)]
    pub truncated: bool,
}

/// The coarse disk-use-by-consumer fact.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct DiskUseFacts {
    /// Filesystem free bytes for the volume holding the declared roots.
    pub free_bytes: Measured<u64>,
    /// Per-consumer footprints.
    pub consumers: Vec<DiskUseEntry>,
}

/// Every calibration fact for one computer at one sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CalibrationFacts {
    /// Where it was measured.
    pub platform: Platform,
    /// Workload-group pressure.
    pub cgroup_pressure: Measured<CgroupPressureFacts>,
    /// Compile-cache effectiveness.
    pub compile_cache: Measured<CompileCacheFacts>,
    /// Fast build volume state.
    pub build_volume: Measured<BuildVolumeState>,
    /// Detected leaks. `unavailable` when the scan did not run — an empty
    /// measured list means "scanned, none found".
    pub leaks: Measured<Vec<LeakRecord>>,
    /// Coarse disk use.
    pub disk_use: Measured<DiskUseFacts>,
}

impl CalibrationFacts {
    /// The flattened measured manifest, the computers plan's shape: one key
    /// per axis (`cgroup_pressure.ci.cpu_pressure`, `build_volume.used_bytes`,
    /// …) plus one per fact group.
    pub fn measured_manifest(&self) -> MeasuredManifest {
        let mut m = MeasuredManifest::new();
        m.insert("cgroup_pressure".into(), self.cgroup_pressure.state());
        m.insert("compile_cache".into(), self.compile_cache.state());
        m.insert("build_volume".into(), self.build_volume.state());
        m.insert("leaks".into(), self.leaks.state());
        m.insert("disk_use".into(), self.disk_use.state());
        if let Some(p) = self.cgroup_pressure.value() {
            p.manifest_into(&mut m);
        }
        if let Some(c) = self.compile_cache.value() {
            for e in &c.classes {
                let class = serde_json::to_value(e.class)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default();
                m.insert(format!("compile_cache.{class}.reading"), e.reading.state());
                m.insert(
                    format!("compile_cache.{class}.wrapper_configured"),
                    e.wrapper_configured.state(),
                );
            }
        }
        if let Some(v) = self.build_volume.value() {
            v.manifest_into(&mut m);
        }
        if let Some(d) = self.disk_use.value() {
            m.insert("disk_use.free_bytes".into(), d.free_bytes.state());
            for e in &d.consumers {
                let c = serde_json::to_value(e.consumer)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default();
                m.insert(format!("disk_use.{c}"), e.bytes.state());
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_installed_is_measured_not_unknown() {
        let e = CompileCacheClassFacts {
            class: CompileCacheClass::Agent,
            reading: Measured::Measured(CompileCacheReading::NotInstalled),
            wrapper_configured: Measured::Measured(false),
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({
                "class": "agent",
                "reading": {"state": "measured", "value": {"kind": "not_installed"}},
                "wrapper_configured": {"state": "measured", "value": false}
            })
        );
    }

    #[test]
    fn manifest_flattens_every_axis() {
        let f = CalibrationFacts {
            platform: Platform::Windows,
            cgroup_pressure: Measured::Measured(CgroupPressureFacts::without_cgroups(
                Platform::Windows,
            )),
            compile_cache: Measured::Measured(CompileCacheFacts {
                classes: vec![CompileCacheClassFacts {
                    class: CompileCacheClass::Agent,
                    reading: Measured::Unavailable,
                    wrapper_configured: Measured::Measured(true),
                }],
            }),
            build_volume: Measured::Measured(BuildVolumeState::absent(BuildVolumeKind::DevDrive)),
            leaks: Measured::Measured(vec![]),
            disk_use: Measured::Measured(DiskUseFacts {
                free_bytes: Measured::Measured(1),
                consumers: vec![DiskUseEntry {
                    consumer: DiskConsumer::TargetDirs,
                    bytes: Measured::Measured(5),
                    truncated: true,
                }],
            }),
        };
        let m = f.measured_manifest();
        assert_eq!(m["compile_cache.agent.reading"], MeasuredState::Unavailable);
        assert_eq!(m["build_volume.size_bytes"], MeasuredState::NotSupported);
        assert_eq!(m["build_volume.mount_present"], MeasuredState::Measured);
        assert_eq!(m["disk_use.target_dirs"], MeasuredState::Measured);
        assert_eq!(
            m["cgroup_pressure.runner.cpu_pressure"],
            MeasuredState::NotSupported
        );
        assert_eq!(m["leaks"], MeasuredState::Measured);
        // Round-trips.
        let back: CalibrationFacts =
            serde_json::from_value(serde_json::to_value(&f).unwrap()).unwrap();
        assert_eq!(back, f);
    }
}
