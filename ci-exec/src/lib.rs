//! The Qontinui CI executor.
//!
//! One executor, one manifest (plan
//! `2026-10-04-coord-managed-ci-for-every-tenant-with-an-actions-free-mode`,
//! D2/D3): a repo's `.qontinui/ci.toml` is the single source of what its CI
//! runs, and this crate is the single thing that runs it — on the runner app
//! lending a desktop, on the headless CI host agent, and in any shell through
//! the `qontinui-ci` binary, which needs no coord at all (D5).
//!
//! The executed commands come EXCLUSIVELY from the manifest at the checked-out
//! commit; a dispatch supplies a commit and a job name, never a command.
//!
//! | Module | What it owns |
//! |---|---|
//! | [`manifest`] | the `ci.toml` schema (v1 and v2 `[[jobs]]`) and its validation |
//! | [`executor`] | one job end-to-end: checkout → manifest → canonical gate → provisioning → steps → report → cleanup |
//! | [`checkout`] | the dispatch-scoped worktree and its cleanup |
//! | [`sibling`] | sibling-repo resolution and materialisation |
//! | [`tools`] | the closed tool registry and its version-keyed cache |
//! | [`services`] | the closed container-service registry and its lifecycle |
//! | [`junit`] | capturing a run's test report before cleanup removes it |
//! | [`host_sizing`] | build/test parallelism sized from the host |
//! | [`canonical`] | the `[canonical]` toolchain gate |
//! | [`dispatch`] | the dispatch payload and the [`dispatch::Admission`] seam |
//! | [`report`] | the [`report::Reporter`] seam and the capture-before-cleanup receipt |
//! | [`host`] | the traits a host implements |
//! | [`standalone`] | the host implementations for a plain shell (`qontinui-ci`) |

pub mod canonical;
pub mod checkout;
pub mod dispatch;
pub mod executor;
pub mod host;
pub mod host_sizing;
pub mod junit;
pub mod manifest;
pub mod report;
pub mod services;
pub mod sibling;
pub mod standalone;
pub mod tools;

/// The local directory NAME for a repo slug: `owner/name` → `name`; a bare
/// name passes through. Primary checkouts, dispatch worktrees and siblings are
/// all laid out by this name, because it is what a relative path dependency
/// (`../qontinui-schemas`) spells.
pub fn local_repo_name(repo: &str) -> &str {
    repo.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(repo)
}

#[cfg(test)]
mod tests {
    use super::local_repo_name;

    #[test]
    fn local_repo_name_is_the_slug_basename() {
        assert_eq!(
            local_repo_name("qontinui/qontinui-runner"),
            "qontinui-runner"
        );
        assert_eq!(local_repo_name("qontinui-runner"), "qontinui-runner");
        assert_eq!(local_repo_name("owner/"), "owner/");
    }
}
