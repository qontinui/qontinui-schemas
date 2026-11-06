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
# major bump. On qontinui-schemas#160 (2026-09-02 → 2026-09-07, 10+ coord
# `ci-not-green` cycles) the consumers' then-present `<2.0.0` bounds turned
# five checks red with a version-resolution error that said nothing about the
# cause. Consumers are moving to bare path deps with no bound (CONTRIBUTING.md,
# "Consumer version bounds"). Once they get there, a bogus version is silently
# wrong rather than loudly red, and this check is what names it.
#
# This script fails ONE check with the cause and the recovery recipe instead.
#
# A manifest row with no `packages` entry in the config is NOT expected to have
# a tag. The cargo-workspace plugin writes such rows for path dependents it
# bumps (ci-exec: `publish = false`), and release-please only ever creates
# releases for configured paths, so that tag will never exist. Demanding it
# failed every release PR from 2026-10-22 with "do not land".
#
# It also checks the rule CONTRIBUTING.md gives for a qontinui-types MAJOR: the
# release PR must move a 0.x runner-client's MINOR ("On a qontinui-types MAJOR,
# give runner-client a real version too"). The plugin alone only patch-bumps it,
# so a `^0.2` user would silently pick up the new major. This reads the versions
# the PR proposes rather than the commits, so it catches every way the override
# can be lost. qontinui-schemas#269 lost its `Release-As: 0.3.0` to a
# `git cherry-pick -x` line and #262 then proposed 0.2.14. A runner-client at 1.0
# or later is not covered: a caret on it tracks its major, so the fix would be a
# different rule, and nothing has needed one yet.
#
# Three distinct exits, because a wrong verdict here is worse than none:
#   0 — every expected tag exists and the proposed versions obey the rule
#   1 — a tag is MISSING, or runner-client's minor did not move (recipe printed)
#   2 — the check could not be evaluated (a missing input, unreadable base or
#       head, malformed config or manifest, nothing to evaluate, a non-404 GitHub
#       answer) — UNKNOWN, never reported as either
#
# Inputs (env): BASE_SHA — the PR base commit, whose manifest names the versions
# that must already be released. HEAD_REF — the commit whose manifest holds the
# proposed versions (default HEAD, the PR checkout). GH_TOKEN — for `gh api`
# (read-only). REPO — owner/name (defaults to GITHUB_REPOSITORY).
set -euo pipefail

fail_unknown() {
  echo "::error::release-pr-sanity: cannot evaluate — $*"
  exit 2
}

# Not `${VAR:?}`: that exits 1, the same code as a real MISSING.
REPO="${REPO:-${GITHUB_REPOSITORY:-}}"
[[ -n "$REPO" ]] || fail_unknown "REPO or GITHUB_REPOSITORY required"
[[ -n "${BASE_SHA:-}" ]] || fail_unknown "BASE_SHA required"
HEAD_REF="${HEAD_REF:-HEAD}"

base_manifest="$(git show "${BASE_SHA}:release-please-manifest.json")" \
  || fail_unknown "no release-please-manifest.json readable at base ${BASE_SHA}"
config="$(git show "${BASE_SHA}:release-please-config.json")" \
  || fail_unknown "no release-please-config.json readable at base ${BASE_SHA}"

# `-s` so a second JSON document in the file is seen, rather than judged alone.
jq -es 'length == 1 and (.[0] | type == "object")' >/dev/null 2>&1 <<<"$config" \
  || fail_unknown "release-please-config.json at ${BASE_SHA} is not one JSON object"
jq -es 'length == 1 and (.[0] | type == "object" and all(.[]; type == "string"))' \
  >/dev/null 2>&1 <<<"$base_manifest" \
  || fail_unknown "base manifest at ${BASE_SHA} is not one object of version strings"

# Strict and anchored, so `(( ))` below can only ever see plain decimal integers
# (a leading zero would read as octal). A prerelease or build suffix is allowed.
version_re='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'

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
evaluated=0
while IFS=$'\t' read -r path version; do
  # jq, not grep: a path is a JSON key and may hold regex metacharacters.
  if [[ "$(jq -r --arg p "$path" '(.packages // {}) | has($p)' <<<"$config")" != "true" ]]; then
    echo "skip     ${path}@${version}  (no package in release-please-config.json, so release-please never tags it)"
    continue
  fi
  # Before the lookup: a malformed version would otherwise read as a MISSING tag,
  # and send someone off to create that release.
  [[ "$version" =~ $version_re ]] \
    || fail_unknown "base manifest row ${path} holds '${version}', not a major.minor.patch version"
  evaluated=$((evaluated + 1))
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

# Every row skipped means the config names none of the manifest's components,
# which says the config is wrong. It does not say every release exists.
[[ "$evaluated" -gt 0 ]] || fail_unknown "no manifest row at ${BASE_SHA} has a package in the config"

if [[ "$missing" -gt 0 ]]; then
  echo "::error::release-pr-sanity: ${missing} manifest version(s) on the base have no release/tag."
  cat <<'MSG'

The previous release PR landed without GitHub marking it merged (a SHA-rewriting
land), so release-please never tagged it. The versions THIS PR proposes are
derived from the whole history and are NOT trustworthy — do not land them.

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
echo "release-pr-sanity: every configured package's base manifest version is released."

# --- A qontinui-types major must move runner-client's minor ------------------
# Runs only after the tags check passed: if that failed, the proposed versions
# are derived from the whole history and comparing them says nothing.
head_manifest="$(git show "${HEAD_REF}:release-please-manifest.json")" \
  || fail_unknown "no release-please-manifest.json readable at ${HEAD_REF}"
jq -es 'length == 1 and (.[0] | type == "object")' >/dev/null 2>&1 <<<"$head_manifest" \
  || fail_unknown "the manifest at ${HEAD_REF} is not one JSON object"

# Prints the row's version, or fails on a malformed manifest or an absent row
# (`-e` exits non-zero on null). An absent row is UNKNOWN rather than "no bump".
version_of() {
  jq -er --arg p "$2" '.[$p] | select(type == "string")' <<<"$1"
}
types_base="$(version_of "$base_manifest" rust)" \
  || fail_unknown "the manifest at ${BASE_SHA} has no version string for rust"
types_head="$(version_of "$head_manifest" rust)" \
  || fail_unknown "the manifest at ${HEAD_REF} has no version string for rust"
client_base="$(version_of "$base_manifest" rust-runner-client)" \
  || fail_unknown "the manifest at ${BASE_SHA} has no version string for rust-runner-client"
client_head="$(version_of "$head_manifest" rust-runner-client)" \
  || fail_unknown "the manifest at ${HEAD_REF} has no version string for rust-runner-client"

for v in "$types_base" "$types_head" "$client_base" "$client_head"; do
  [[ "$v" =~ $version_re ]] || fail_unknown "'${v}' is not a major.minor.patch version"
done
types_base_major="${types_base%%.*}"
types_head_major="${types_head%%.*}"
client_base_rest="${client_base#*.}"
client_head_rest="${client_head#*.}"
client_base_minor="${client_base_rest%%.*}"
client_head_minor="${client_head_rest%%.*}"

if ((types_head_major > types_base_major)) \
  && [[ "${client_base%%.*}" == "0" && "${client_head%%.*}" == "0" ]] \
  && ((client_head_minor <= client_base_minor)); then
  echo "::error::release-pr-sanity: qontinui-types goes ${types_base} -> ${types_head} but qontinui-runner-client only goes ${client_base} -> ${client_head}."
  next_minor="$((client_base_minor + 1))"
  cat <<MSG

A crates.io user on ^${client_base%%.*}.${client_base_minor} would silently pick up qontinui-types ${types_head_major}.x
(CONTRIBUTING.md, "On a qontinui-types MAJOR, give runner-client a real version too").
Do not land these versions.

TO FIX:
  1. Land a commit touching ONLY rust-runner-client/ whose message ends with the
     footer  Release-As: ${client_base%%.*}.${next_minor}.0
     A footer counts only if nothing but footers follows it to the end of the
     message, and blank lines do not protect it. So no later prose paragraph, and
     no "(cherry picked from commit ...)" line from \`git cherry-pick -x\`.
  2. release-please rewrites this PR on that push. This check re-runs on the rewrite.
MSG
  exit 1
fi
echo "release-pr-sanity: runner-client ${client_base} -> ${client_head} is consistent with qontinui-types ${types_base} -> ${types_head}."
