# Contributing

Thanks for your interest in contributing to qontinui. This document explains how to submit changes and what you're agreeing to when you do.

## License: AGPL-3.0-or-later

This project is licensed under the **GNU Affero General Public License v3 or later** (`AGPL-3.0-or-later`). The full text is in [`LICENSE`](LICENSE).

What this means for you as a contributor:

- Anyone can use, modify, and redistribute the code under the same AGPL terms.
- If you (or anyone else) runs a modified version of qontinui as a network service, the AGPL requires you to publish your modifications under AGPL too — this is the "network copyleft" provision and the reason this project chose AGPL.
- For typical self-hosting, internal use, forking, or contributing back, AGPL is no different from GPL in practice.

If your employer has policies about contributing to AGPL projects, please confirm with them before you submit a PR.

## Contributor License Agreement (CLA)

All non-trivial contributions require signing the qontinui CLA before they can be merged. The CLA grants Joshua Spinak (the project author) the right to relicense your contribution under any future license — this is the standard open-core pattern (Apache, Google, MongoDB, GitLab all use a similar CLA) and exists so the project can adapt its license posture in the future without re-litigating contributor consent.

The CLA does **not** transfer copyright. You retain copyright in your contributions; you grant a relicensing right.

**How to sign:** the project uses [cla-assistant.io](https://cla-assistant.io/) — when you open a pull request, the CLA bot will comment with a one-click link. Sign once and the bot remembers you across all qontinui repositories. The CLA text is also kept in [`CLA.md`](CLA.md) for reference.

Trivial changes (typo fixes, single-line documentation tweaks) do not require a CLA, at the maintainer's discretion.

## Code style

This repository follows the conventions documented in `CLAUDE.md` (project root). Highlights:

- Edit existing files in preference to creating new ones; keep changes scoped to the task.
- No comments unless the *why* is non-obvious. Don't narrate what well-named code already says.
- No backwards-compatibility shims unless explicitly requested.
- For Python: Black + ruff. For TypeScript/JavaScript: project's Prettier + ESLint config. For Rust: `cargo fmt` + `cargo clippy`.
- Run the test suite before opening a PR. CI will reject unformatted or lint-failing code.

## Submitting a change

1. Fork the repository and create a feature branch.
2. Make your change. Add tests where appropriate.
3. Run the local checks (lint, format, tests) — the per-repo `README` documents the exact commands.
4. Open a pull request against `main`. Describe the *why* in the PR body, not the *what* — the diff already shows the *what*.
5. Sign the CLA if the bot prompts you.
6. A maintainer will review. Expect feedback; we keep the bar high because every change becomes part of the shipped product.

## Releasing

This repo publishes three artifacts to two registries:

| Artifact | Path | Registry | Tag prefix |
|---|---|---|---|
| `qontinui-types` (Rust crate) | `rust/` | crates.io | `rust-v*` |
| `qontinui-runner-client` (Rust crate) | `rust-runner-client/` | crates.io | `rust-runner-client-v*` |
| `@qontinui/shared-types` (npm package) | `ts/` | npm | `ts-v*` |

Versioning and changelog generation are managed by [release-please](https://github.com/googleapis/release-please) in monorepo mode (`release-please-config.json` + `release-please-manifest.json`). Conventional-commits is a hard rule (enforced by `commitlint.yml` on every PR) — `feat:` → minor, `fix:` → patch, `feat!:` / `BREAKING CHANGE:` → major.

### The standard release flow

1. **Land conventional commits on `main`.** Each PR's commits are linted; merge whatever subset of `feat:` / `fix:` / etc. you want in the next release.
2. **release-please opens a release PR** (`chore: release ...`) on every push to `main`. The PR contains version bumps in the Cargo.toml's / package.json + auto-generated CHANGELOG entries derived from the commits since the last release.
3. **Review the release PR carefully** — read the version bumps, read the CHANGELOG entries. Edit them if needed (release-please respects manual edits).
4. **Merge the release PR.** This:
   - Creates per-component tags: e.g. `rust-v0.1.2`, `rust-runner-client-v0.1.0`, `ts-v0.2.4`.
   - Tag pushes fire `publish-rust.yml` (Rust crates) or `publish.yml` (TS package).

The publish workflows: re-run full `rust-ci.yml` on the tagged SHA (F2 hard gate), check whether the version is already on the registry (skip-if-published guard), run `cargo publish --dry-run` (F4 packaging pre-flight), then the real `cargo publish`. Any failure leaves the registry unchanged.

**No auto-merge on release PRs.** The release PR is the human gate; auto-merging it defeats the purpose. `cargo publish` is irreversible — `cargo yank` only hides the version, it does not free the version number for reuse.

### Manual override (emergency / first-publish ceremony)

For situations where release-please isn't appropriate — emergency hotfix, the very-first-publish ceremony for a new crate, or a publish retry — bypass the release PR and tag directly:

```bash
# Bump the version in Cargo.toml / package.json and the manifest
# (release-please needs the manifest in sync going forward).
git commit -am "chore: prepare rust-v0.1.2 manual publish"
git push origin main

git tag rust-v0.1.2
git push origin rust-v0.1.2
```

Or use the `workflow_dispatch` escape hatch (skips the tag, fires the workflow directly):

```bash
gh workflow run publish-rust.yml -f crate=both
gh workflow run publish-rust.yml -f crate=qontinui-types
gh workflow run publish-rust.yml -f crate=qontinui-runner-client
```

Both paths still go through the F2 + F4 gates.

### Pre-release / RC channel

The first publish of any new crate to crates.io should ship as `-rc.1` first, validate the registry round-trip (consumer pulls from crates.io, integration works), then promote to stable. This is decision 5 in `qontinui-dev-notes/rust-release-engineering/SESSION_PROMPT.md`.

```bash
# Phase 7: bump rust/Cargo.toml to 0.1.2-rc.1 + update manifest
git tag rust-v0.1.2-rc.1 && git push origin rust-v0.1.2-rc.1

# Validate by adding the RC to a throwaway test repo:
#   qontinui-types = "=0.1.2-rc.1"
# and `cargo build`-ing.

# Phase 10 promotion: bump to 0.1.2 and re-tag.
git tag rust-v0.1.2 && git push origin rust-v0.1.2
```

**Cross-crate RC dependency wrinkle.** When `qontinui-types` is at an RC version on crates.io but `qontinui-runner-client` needs to consume it (e.g. during the phase 8 first-publish of runner-client), Cargo's caret range `^0.1` does NOT match pre-release versions like `0.1.2-rc.1` — pre-releases require an explicit pre-release version specifier. Pin runner-client's `qontinui-types` dep to `=0.1.2-rc.1` for the duration of the RC, then change it back to `^0.1.2` after the types crate promotes to stable.

### When a publish fails

`cargo publish` is irreversible. The recovery options:

- **Bug found post-publish, version still healthy enough to ship:** open a follow-up PR, let release-please bump to the next patch (`0.1.2` → `0.1.3`), re-publish.
- **Bug makes the version actively dangerous:** `cargo yank --version 0.1.2 qontinui-types` makes the version invisible to new dependency resolutions but does NOT free the version number for re-publish. Then bump to `0.1.3` and re-publish.
- **Workflow failed before the real publish (e.g. dry-run caught an issue):** fix the issue, push a new tag (re-tagging is fine since the workflow's skip-if-published guard makes re-runs safe).

The `cargo publish --dry-run` step (F4) catches the most common failure modes (missing license file, malformed manifest, unbuildable extracted package) before the real publish ever runs. Treat a dry-run failure as a normal CI failure.

### Provisioning credentials

- **`CRATES_IO_TOKEN`** (repo secret): a publish-only scoped token from [crates.io/me](https://crates.io/me). Restrict scopes to publish + yank only — never use a full-access token. Rotate periodically.
- **npm trusted publishing** (OIDC): `publish.yml` uses GitHub Actions' OIDC integration with npm, no static token required. See npm's "Trusted Publishers" documentation if the OIDC trust needs to be re-established.

## Reporting bugs / requesting features

Open a GitHub issue. For security vulnerabilities, please email the maintainers directly rather than filing a public issue.

## Code of conduct

Be kind. Be specific. No harassment. Discussions stay on the technical merits.
