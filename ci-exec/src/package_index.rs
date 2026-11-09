//! Where a Python registry entry's distribution comes from, and, in test
//! builds only, the registry overlay that the provisioning e2e enters through.
//!
//! These live outside `tools.rs` because that file is over 3,500 lines and
//! owed a split; new code goes in new modules (plan
//! `2026-11-06-ci-node-provisioning-children-inherit-only-an-allowlisted-environment`,
//! 3.2).

/// Where `pip install` fetches an [`crate::tools::Installer::PythonPackage`]
/// entry's distribution from.
///
/// An explicit per-entry field, so the source of a distribution is a property
/// of the closed registry rather than of whatever pip configuration the host
/// carries. It is belt and braces with `PIP_CONFIG_FILE`, which every
/// provisioning child gets pointed at the null device ([`crate::child_env`]),
/// so no config file can add an `extra-index-url` or `find-links` beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PackageIndex {
    /// PyPI's simple index. The only production value, pinned by
    /// `tests::every_curated_python_entry_installs_from_pypi`.
    PyPi,
    /// A local directory of wheels and NO index: the overlay's probe entry,
    /// which must install without touching the network.
    #[cfg(test)]
    LocalWheels(&'static str),
}

impl PackageIndex {
    /// The pip arguments that select this index.
    pub(crate) fn pip_args(self) -> Vec<&'static str> {
        match self {
            Self::PyPi => vec!["--index-url", "https://pypi.org/simple"],
            #[cfg(test)]
            Self::LocalWheels(dir) => vec!["--no-index", "--find-links", dir],
        }
    }
}

/// Tool-registry entries that only a TEST build can resolve.
///
/// The registry is closed (`tools.rs`, "The closed registry"): a manifest names
/// a curated entry and nothing else. The provisioning e2e
/// (`child_env::tests::provisioning_children_see_only_allowlisted_names`) needs
/// to provision a probe distribution through the real `tools::provision`, as
/// the executor does, so `tools::lookup` consults this overlay in a test build.
/// `KNOWN_TOOLS` itself stays closed: `known_tool_names()` never lists an
/// overlay entry, a non-test build has no overlay, and [`overlay::register`]
/// refuses a curated name, so an entry can add a tool but never replace one.
#[cfg(test)]
pub(crate) mod overlay {
    use crate::tools::ToolSpec;
    use std::sync::{Mutex, PoisonError};

    static ENTRIES: Mutex<Vec<&'static ToolSpec>> = Mutex::new(Vec::new());

    /// Make `spec` resolvable by `tools::lookup` for the rest of this test
    /// process. Leaked on purpose: the registry hands out `&'static` entries.
    pub(crate) fn register(spec: ToolSpec) -> &'static ToolSpec {
        assert!(
            !crate::tools::known_tool_names().contains(&spec.name),
            "the overlay may add a tool, never replace the curated {:?}",
            spec.name
        );
        let mut entries = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
        assert!(
            !entries.iter().any(|t| t.name == spec.name),
            "{:?} is already registered",
            spec.name
        );
        let spec: &'static ToolSpec = Box::leak(Box::new(spec));
        entries.push(spec);
        spec
    }

    /// The registered entry called `name`, if any.
    pub(crate) fn lookup(name: &str) -> Option<&'static ToolSpec> {
        ENTRIES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .copied()
            .find(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{known_tool_names, lookup, Installer, ToolSpec};

    /// Every curated Python entry installs from PyPI's simple index, spelled
    /// out here rather than read from the type. The test is not vacuous: it
    /// names the three entries it checked.
    #[test]
    fn every_curated_python_entry_installs_from_pypi() {
        let mut checked = Vec::new();
        for name in known_tool_names() {
            let spec = lookup(name).expect("a listed name resolves");
            if let Installer::PythonPackage { index, .. } = &spec.install {
                assert_eq!(*index, PackageIndex::PyPi, "{name}");
                assert_eq!(
                    index.pip_args(),
                    vec!["--index-url", "https://pypi.org/simple"],
                    "{name}"
                );
                checked.push(name);
            }
        }
        assert_eq!(checked, vec!["poetry", "twine", "datamodel-code-generator"]);
    }

    #[test]
    fn a_local_wheel_dir_selects_no_index_at_all() {
        assert_eq!(
            PackageIndex::LocalWheels("/wheels").pip_args(),
            vec!["--no-index", "--find-links", "/wheels"]
        );
    }

    /// An overlay entry resolves through `tools::lookup` but never joins the
    /// closed list, and a curated name cannot be shadowed.
    #[test]
    fn the_overlay_adds_without_opening_the_registry() {
        let spec = overlay::register(ToolSpec {
            name: "qontinui-overlay-unit-probe",
            verify_args: &["--version"],
            install: Installer::PythonPackage {
                distribution: "qontinui-overlay-unit-probe",
                module: "qontinui_overlay_unit_probe",
                command: "qontinui-overlay-unit-probe",
                index: PackageIndex::LocalWheels("/wheels"),
            },
        });
        let found = lookup("qontinui-overlay-unit-probe").expect("registered");
        assert!(std::ptr::eq(found, spec));
        assert!(!known_tool_names().contains(&"qontinui-overlay-unit-probe"));

        let refused = std::panic::catch_unwind(|| {
            overlay::register(ToolSpec {
                name: "poetry",
                verify_args: &["--version"],
                install: Installer::PythonPackage {
                    distribution: "poetry",
                    module: "poetry",
                    command: "poetry",
                    index: PackageIndex::LocalWheels("/wheels"),
                },
            })
        });
        assert!(refused.is_err(), "a curated name must not be shadowed");
        let Installer::PythonPackage { index, .. } = &lookup("poetry").unwrap().install else {
            panic!("poetry must stay a PythonPackage");
        };
        assert_eq!(*index, PackageIndex::PyPi);
    }
}
