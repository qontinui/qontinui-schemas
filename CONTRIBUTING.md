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

## CI & Merge Readiness

A PR is ready to merge when every required workflow is green on the PR's HEAD commit. This repo's CI is small — three real merge gates plus three release-time workflows. Don't conflate them.

### Merge gates

These must be green on your PR before merge:

- `commitlint.yml` — runs on every PR (`commitlint`). Hard-enforces conventional-commits via `@commitlint/config-conventional`. PRs with non-conforming commit messages must rewrite history before merge. (The *why* — release-please derives version bumps from these — lives in [`## Releasing`](#releasing).)
- `rust-ci.yml` — `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build --workspace --all-targets`, `cargo test --workspace` on `ubuntu-latest` (`rust-ci`). Path-filtered to `Cargo.toml`, `Cargo.lock`, `rust/**`, `rust-runner-client/**`, and the workflow file itself — required *when it runs*.
- `schema-drift.yml` — regenerates the TS + Python bindings via the qontinui-runner codegen script and fails on drift (`check-drift`). Path-filtered to `rust/src/**`, `qontinui-runner/src-tauri/src/schema_export.rs`, and `qontinui-runner/src-tauri/scripts/generate_types.sh`. The latter two are belt-and-braces — those paths live in the runner repo, not here, so in practice this gate fires whenever a PR touches `rust/src/**`. Required *when it runs*. If it goes red, regenerate locally via the qontinui-runner sibling checkout (or rely on the artifact uploaded by the failing run, once that's added) and commit the result.

"Required when it runs" is the rulesets default — checks that didn't trigger on a PR don't show as `pending` and don't block merge.

### Not merge gates

These three are release-time, not PR-time. See [`## Releasing`](#releasing) for full mechanics; the one-liners below exist only so the merge-gate set above isn't conflated with them:

- `release-please.yml` — runs on push to `main` to maintain the release PR.
- `publish.yml` — `ts-v*` tag-triggered npm publish.
- `publish-rust.yml` — `rust-v*` / `rust-runner-client-v*` tag-triggered crates.io publish.

If any of the three goes red on a tag push, that's a release-time problem — file an issue but don't treat it as merge-blocking on unrelated PRs.

### Test locally first

For changes that touch Rust:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --all-targets
cargo test --workspace
```

For changes that touch `rust/src/**`, the schema-drift check will fire in CI. To pre-flight it locally you need both `qontinui-schemas` and `qontinui-runner` checked out as siblings, plus the same `datamodel-code-generator` version CI uses:

```bash
pip install 'datamodel-code-generator==0.57.0'
bash qontinui-runner/src-tauri/scripts/generate_types.sh
git -C qontinui-schemas diff --exit-code -I '^#   timestamp:' \
    ts/src/generated src/qontinui_schemas/generated
```

The version pin matters — unpinned upstream releases tweak Pydantic output and surface as spurious drift. If you don't have qontinui-runner cloned, the CI run is your check.

### Active workstream awareness

CI is a shared surface. Before opening a PR that touches `.github/workflows/` or anything CI-adjacent, check what's already in flight:

```bash
gh pr list --repo qontinui/qontinui-schemas --state open
```

If there's a related open PR, coordinate (or rebase onto it) rather than opening a parallel attempt.

### Branch protection

The merge-gate set above is mechanically enforced by the `main-merge-gates` Repository Ruleset on `qontinui-schemas` `main` (ruleset id `16104588`, [admin UI](https://github.com/qontinui/qontinui-schemas/rules/16104588)). The rule blocks force-push, branch deletion, and any merge to `main` whose PR doesn't have these check contexts green:

- `commitlint` — required on every PR.
- `rust-ci` — required *when run*, i.e. only on PRs touching `Cargo.toml`, `Cargo.lock`, `rust/**`, `rust-runner-client/**`, or the workflow file itself.
- `check-drift` — required *when run*, i.e. only on PRs touching `rust/src/**`. (`schema-drift.yml` also lists two `qontinui-runner/...` paths in its `paths:` filter as belt-and-braces, but those paths live in the runner repo and can't appear in a `qontinui-schemas` PR.)

Required-when-run is the rulesets default: checks that didn't trigger on a PR don't show as `pending` and don't block merge. PRs also have to go through a pull request — direct push to `main` is blocked.

#### Admin bypass

The ruleset has `OrganizationAdmin` as a `bypass_mode: always` actor. The org owner (currently jspinak) can override any rule — required checks, force-push block, deletion block — without going through the gate. This exists for two reasons:

1. **Solo-maintainer rescue.** With one admin, getting locked out by a misconfigured rule has no recovery path short of GitHub Support.
2. **Human-policy escape valve.** Some merge decisions are judgment calls the ruleset can't natively express (e.g. a release-please flow, a surprise GitHub Actions context rename, a documented hosted-runner pathology). An admin override is the mechanical answer for those cases.

If you find yourself overriding routinely, the rule is wrong, not the override. Fix the rule.

#### How to override (admin runbook)

When you legitimately need to merge a red PR — documented infrastructure block, in-flight project-side fix, etc. — and the escape-valve criteria above are satisfied:

1. Confirm the failure matches a tracked plan or open issue and link it in the PR description.
2. Click `Merge` on the PR. GitHub surfaces a "Bypass branch protections" prompt for org admins. Select "Bypass and merge."
3. Note in the merge commit message which rule was bypassed and why.

If a rule fires unexpectedly — e.g. a `.github/workflows/*.yml` job was renamed and the check context the ruleset pins no longer matches — update the ruleset, don't override repeatedly. Renaming a workflow job is a silent ruleset break: the ruleset references check contexts by name (`commitlint`, `rust-ci`, `check-drift`), and those names follow each workflow file's `jobs.<id>`. Sync the ruleset whenever those rename.

#### release-please PRs always need admin-bypass

`release-please.yml` opens its release PR (`chore: release main` on the `release-please--branches--main` branch) using the default `GITHUB_TOKEN`. GitHub's loop-prevention rule suppresses downstream workflows on bot-opened PRs using `GITHUB_TOKEN`, so **`commitlint` never fires on the release-please PR** — `gh api .../commits/<head>/check-runs` returns `total: 0`. With the required-checks rule live, the release PR can never go green naturally; merging it requires the admin "Bypass and merge" path every time. This is expected; it's exactly the kind of human-judgment escape valve the bypass exists for. If release frequency makes the bypass routine and the friction is real, switch `release-please.yml` to a PAT (workflow scope) or GitHub App `token:` so the PR is opened under a non-`GITHUB_TOKEN` identity and `commitlint` fires naturally.

### Quick checklist before clicking merge

- [ ] Local `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build --workspace --all-targets`, `cargo test --workspace` pass on whatever you're authoring on
- [ ] Commit messages conform to conventional-commits (`commitlint` will reject non-conforming ones)
- [ ] `rust-ci` green if it ran (or didn't run because no Rust paths matched)
- [ ] `check-drift` green if it ran (or didn't run because no schema paths matched)
- [ ] No open PR is doing the same work

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

For situations where release-please isn't appropriate — emergency hotfix, the very-first-publish ceremony for a new crate, or a publish retry — bypass the release PR and tag directly. **Three files must move in lock-step:**

1. `<crate>/Cargo.toml` (or `package.json`) — version bump.
2. `release-please-manifest.json` — the component entry must match the new version.
3. `release-please-config.json` — set the component's `last-release-sha` to the prep PR's squash-merge SHA so release-please anchors at this release going forward (see "Why `last-release-sha` matters" below).

If you skip step 3, release-please's next "chore: release main" PR will re-propose every conventional commit since the previous `last-release-sha`, including the change you just shipped — see PRs #34 → #35 → #36 → #37 (2026-05-10) for the full thrash and recovery.

Recommended flow — prep PR + tag from main, mirroring phases 10a/b/c of `qontinui-dev-notes/rust-release-engineering`:

```bash
git checkout -b chore/types-0.1.3 origin/main

# Bump the version in Cargo.toml + manifest, refresh Cargo.lock.
# (last-release-sha can't be set yet — you don't know the squash-merge
# SHA until after the PR merges — handle in step 4.)
cargo update -p qontinui-types
git commit -am "chore: release qontinui-types 0.1.3"
git push -u origin chore/types-0.1.3
gh pr create --title "chore: release qontinui-types 0.1.3"
# review, merge — note the squash-merge SHA (e.g. 7cf1daf)

# After merge, tag from main:
git checkout main && git pull
git tag rust-v0.1.3
git push origin rust-v0.1.3
# Tag fires publish-rust.yml → verify-ci → F4 dry-run → cargo publish

# Step 4: bump last-release-sha in a tiny follow-up PR (or fold into a
# bigger anchor-fix PR if you have one) so the next release-please run
# starts scanning from this release, not from before:
git checkout -b chore/release-please-anchor-types-0.1.3 origin/main
# edit release-please-config.json: rust.last-release-sha = "<merge-sha>"
git commit -am "chore(release-please): anchor rust at <merge-sha>"
git push -u origin chore/release-please-anchor-types-0.1.3
gh pr create
```

Or use the `workflow_dispatch` escape hatch (skips the tag, fires the workflow directly):

```bash
gh workflow run publish-rust.yml -f crate=both
gh workflow run publish-rust.yml -f crate=qontinui-types
gh workflow run publish-rust.yml -f crate=qontinui-runner-client
```

Both paths still go through the F2 + F4 gates.

#### Why `last-release-sha` matters

release-please decides which version to propose by scanning conventional commits between `last-release-sha` (or a default origin-of-history if absent) and `HEAD` for each component's path. It does NOT consult git tags or compare against the manifest version directly — the manifest tells release-please "we're currently at vN" but `last-release-sha` tells release-please "we already know about every commit up to this point."

So when a manual-override release is shipped without updating `last-release-sha`, the post-release run sees:

- manifest: `0.1.3` ✓
- last-release-sha: still pointing at the previous release (or absent)
- conventional-commits in window: includes the same `feat:` that drove `0.1.3`

…and proposes `0.1.3 → 0.2.0` again. The ergonomics of skipping the follow-up PR: every manual override generates one stale "chore: release main" PR that you close manually. The ergonomics of doing the follow-up PR: one extra ~3-line PR, but the release-please flow stays clean.

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

### Bootstrapping a new release-please component

Adding a new package to `release-please-config.json` and `release-please-manifest.json` does **not** by itself give release-please enough state to behave correctly. release-please's monorepo strategy needs an *anchor tag* per component (`<component>-v<manifest-version>`) — without one, it falls back to walking all of git history for that component's path and applies conventional-commit rules from the very beginning, which routinely produces wrong bumps (e.g. attributing the commit that *introduced* the package as a `feat:` and proposing a minor bump on its first appearance).

The `bootstrap-sha` and `last-release-sha` per-package config keys are documented but are **not** honored on this code path once the manifest version is set; they only apply in narrower scenarios. In practice, the only reliable bootstrap is to do the new component's first release manually (the same direct-tag ceremony described above under "Manual override"):

1. Bump the new component's `Cargo.toml` / `package.json` to the intended first-publish version, in lockstep with `release-please-manifest.json`.
2. Open + merge a `chore:` PR with those bumps.
3. Tag the merge SHA as `<component>-v<version>` and push the tag — this fires the publish workflow and creates the first GitHub release.
4. From then on, release-please finds the anchor tag, walks only post-tag commits affecting the component's path, and proposes correct bumps.

Until step 3 lands, expect release-please to open release PRs proposing wrong bumps on the new component. Force-close them; they'll be regenerated correctly once the anchor tag exists.

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
