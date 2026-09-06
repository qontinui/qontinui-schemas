#!/usr/bin/env bash
# release-pr-sanity — on a release-please PR, prove the PREVIOUS release exists.
#
# release-please decides "what is the latest release" by looking for a GitHub
# release / tag named `<component>-v<manifest version>` (manifest.ts: recent
# merged release PRs, then releases, then tags). A per-package
# `last-release-sha` is NOT honored once a manifest version is set
# (CONTRIBUTING.md, "Bootstrapping a new release-please component"). So when a
# release PR lands WITHOUT GitHub marking it merged — coord's SHA-rewriting
# rebase-land does exactly that — release-please never tags it, then on its
# next run finds no release for the manifest version, walks the ENTIRE history,
# and re-proposes every commit ever made, including any old `feat!`, as a bogus
# major bump. Every consumer pins `qontinui-types = "<2.0.0"`, so five checks
# go red with a version-resolution error that says nothing about the cause
# (qontinui-schemas#160, 2026-09-02 → 2026-09-07, 10+ coord `ci-not-green`
# cycles).
#
# This script fails ONE check with the cause and the recovery recipe instead.
# Three distinct exits, because a wrong verdict here is worse than none:
#   0 — every base manifest version has its tag
#   1 — at least one is MISSING (the recipe is printed)
#   2 — the check could not be evaluated (unreadable base, malformed or empty
#       manifest, a non-404 GitHub answer) — UNKNOWN, never reported as either
#
# Inputs (env): BASE_SHA — the PR base commit, whose manifest names the versions
# that must already be released. GH_TOKEN — for `gh api` (read-only). REPO —
# owner/name (defaults to GITHUB_REPOSITORY).
set -euo pipefail

REPO="${REPO:-${GITHUB_REPOSITORY:?REPO or GITHUB_REPOSITORY required}}"
BASE_SHA="${BASE_SHA:?BASE_SHA required}"

fail_unknown() {
  echo "::error::release-pr-sanity: cannot evaluate — $*"
  exit 2
}

base_manifest="$(git show "${BASE_SHA}:release-please-manifest.json")" \
  || fail_unknown "no release-please-manifest.json readable at base ${BASE_SHA}"
config="$(git show "${BASE_SHA}:release-please-config.json")" \
  || fail_unknown "no release-please-config.json readable at base ${BASE_SHA}"

# Tag shape, from the config's own keys (release-please defaults in the `//`).
# `has()` rather than `//`: jq's `//` treats an explicit `false` as absent.
include_component="$(jq -r 'if has("include-component-in-tag") then ."include-component-in-tag" else true end' <<<"$config")"
include_v="$(jq -r 'if has("include-v-in-tag") then ."include-v-in-tag" else true end' <<<"$config")"
separator="$(jq -r '."tag-separator" // "-"' <<<"$config")"

# Materialize the (path, version) rows first: errexit does not observe a
# process substitution, so `while ... < <(jq ...)` would turn a malformed
# manifest into zero iterations and a green verdict.
pairs="$(jq -r 'to_entries[] | "\(.key)\t\(.value)"' <<<"$base_manifest")" \
  || fail_unknown "base manifest at ${BASE_SHA} is not valid JSON"
[[ -n "$pairs" ]] || fail_unknown "base manifest at ${BASE_SHA} names no components"

missing=0
while IFS=$'\t' read -r path version; do
  component="$(jq -r --arg p "$path" '.packages[$p].component // $p' <<<"$config")"
  tag=""
  [[ "$include_component" == "true" ]] && tag="${component}${separator}"
  [[ "$include_v" == "true" ]] && tag="${tag}v"
  tag="${tag}${version}"
  if err="$(gh api "repos/${REPO}/git/ref/tags/${tag}" --silent 2>&1)"; then
    echo "ok       ${tag}  (path ${path})"
  elif [[ "$err" == *"(HTTP 404)"* ]]; then
    echo "MISSING  ${tag}  (path ${path}) — manifest on the base says ${version} but no such tag exists"
    missing=$((missing + 1))
  else
    fail_unknown "could not query tag ${tag} on ${REPO}: ${err}"
  fi
done <<<"$pairs"

if [[ "$missing" -gt 0 ]]; then
  echo "::error::release-pr-sanity: ${missing} manifest version(s) on the base have no release/tag."
  cat <<'MSG'

The previous release PR landed without GitHub marking it merged (a SHA-rewriting
land), so release-please never tagged it. The versions THIS PR proposes are
derived from the whole history and are NOT trustworthy — do not widen consumer
version bounds to make them fit.

TO FIX (recipe from qontinui-schemas#160):
  1. Find the landed release commit on main:
       git log --format='%h %an %s' -3 -- release-please-manifest.json
  2. For each MISSING tag above, create the release release-please would have:
       gh release create <tag> --target <that commit> --title '<component>: v<version>' \
         --notes "<the matching CHANGELOG.md section>"
  3. gh workflow run release-please.yml --ref main   # re-derives from the new release
  4. Close this PR if release-please leaves it unchanged (it only rewrites on a body change).
MSG
  exit 1
fi
echo "release-pr-sanity: every base manifest version is released."
