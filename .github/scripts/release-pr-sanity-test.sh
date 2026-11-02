#!/usr/bin/env bash
# Verdict table for release-pr-sanity.sh, run in CI on every PR to main (the
# script itself only runs on release PRs). Each case builds a throwaway repo holding a
# base and a head manifest, stubs `gh` with the tags that exist, and asserts the
# exit code and a line of the output. Needs git and jq; no network.
set -euo pipefail

script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/release-pr-sanity.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# The shipped config's shape: ci-exec is deliberately not a package.
shipped_config='{"include-component-in-tag":true,"include-v-in-tag":true,"tag-separator":"-",
  "packages":{"rust":{"component":"rust"},
              "rust-runner-client":{"component":"rust-runner-client"},
              "ts":{"component":"ts"}}}'
tagged='rust-v4.8.0 rust-runner-client-v0.2.13 ts-v2.4.1'
ordinary='{"rust":"4.8.0","rust-runner-client":"0.2.13","ts":"2.4.1","ci-exec":"0.1.1"}'

# Stub: 404 unless the requested tag is in $GH_STUB_TAGS; "boom" fails otherwise.
mkdir -p "$work/bin"
cat >"$work/bin/gh" <<'STUB'
#!/usr/bin/env bash
[[ "${GH_STUB_MODE:-}" == "boom" ]] && { echo "HTTP 502: bad gateway" >&2; exit 1; }
tag="${2##*/tags/}"
for t in $GH_STUB_TAGS; do [[ "$t" == "$tag" ]] && exit 0; done
echo "gh: Not Found (HTTP 404)" >&2
exit 1
STUB
chmod +x "$work/bin/gh"

failures=0
n=0

# run_case <name> <want-exit> <want-substring> <base-manifest> <head-manifest> [tags] [gh-mode]
# Env for one case: CASE_CONFIG replaces the config (DELETED removes it);
# CASE_HEAD_REF=base points HEAD_REF at the base commit, and otherwise HEAD_REF is
# left unset, which is how the workflow runs it. A base manifest of DELETED leaves
# the file out of the base commit; a head manifest of DELETED removes it at head.
run_case() {
  local name="$1" want_rc="$2" want_text="$3" base="$4" head="$5"
  local tags="${6-$tagged}" mode="${7-}"
  local config="${CASE_CONFIG:-$shipped_config}"
  local repo="$work/repo-$((n += 1))" out rc=0 base_sha
  local -a head_ref_env=()
  git init -q "$repo"
  git -C "$repo" config user.name test
  git -C "$repo" config user.email test@example.invalid
  [[ "$config" == "DELETED" ]] || printf '%s\n' "$config" >"$repo/release-please-config.json"
  [[ "$base" == "DELETED" ]] || printf '%s\n' "$base" >"$repo/release-please-manifest.json"
  git -C "$repo" add -A
  git -C "$repo" commit -qm base --allow-empty
  base_sha="$(git -C "$repo" rev-parse HEAD)"
  if [[ "$head" == "DELETED" ]]; then
    rm -f "$repo/release-please-manifest.json"
  else
    printf '%s\n' "$head" >"$repo/release-please-manifest.json"
  fi
  if [[ "${CASE_HEAD_REF:-}" == "base" ]]; then
    head_ref_env=("HEAD_REF=$base_sha")
  fi
  git -C "$repo" add -A
  git -C "$repo" commit -qm head --allow-empty
  out="$(cd "$repo" && env -u HEAD_REF "${head_ref_env[@]}" PATH="$work/bin:$PATH" REPO=o/r \
    BASE_SHA="$base_sha" GH_STUB_TAGS="$tags" GH_STUB_MODE="$mode" bash "$script" 2>&1)" || rc=$?
  if [[ "$rc" == "$want_rc" && "$out" == *"$want_text"* ]]; then
    echo "ok    $name"
  else
    echo "FAIL  $name: wanted rc=$want_rc with '$want_text', got rc=$rc:"
    sed 's/^/        /' <<<"$out"
    failures=$((failures + 1))
  fi
}

run_case "ordinary release passes" 0 "is consistent" \
  "$ordinary" '{"rust":"4.8.1","rust-runner-client":"0.2.14","ts":"2.5.0","ci-exec":"0.1.2"}'
run_case "an unconfigured manifest row needs no tag" 0 "skip     ci-exec@0.1.1" \
  "$ordinary" "$ordinary"
run_case "a configured package with no tag fails" 1 "MISSING  ts-v2.4.1" \
  "$ordinary" "$ordinary" 'rust-v4.8.0 rust-runner-client-v0.2.13'
run_case "a major with the patch bump plugin default fails (#262 as it stood)" 1 "only goes 0.2.13 -> 0.2.14" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.2.14","ts":"2.5.0","ci-exec":"0.1.2"}'
run_case "a major with no runner-client move fails" 1 "Release-As: 0.3.0" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.2.13","ts":"2.4.1","ci-exec":"0.1.1"}'
run_case "a major with the override passes" 0 "0.2.13 -> 0.3.0 is consistent" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.3.0","ts":"2.5.0","ci-exec":"0.1.2"}'
run_case "two-digit minors compare as numbers, not text" 1 "Release-As: 0.11.0" \
  '{"rust":"9.0.0","rust-runner-client":"0.10.0"}' '{"rust":"10.0.0","rust-runner-client":"0.9.0"}' \
  'rust-v9.0.0 rust-runner-client-v0.10.0'
run_case "a minor qontinui-types bump does not demand one" 0 "is consistent" \
  "$ordinary" '{"rust":"4.9.0","rust-runner-client":"0.2.14","ts":"2.4.1","ci-exec":"0.1.1"}'
run_case "runner-client crossing to 1.0 is not this rule's case" 0 "is consistent" \
  '{"rust":"4.8.0","rust-runner-client":"0.2.13"}' '{"rust":"5.0.0","rust-runner-client":"1.0.0"}' \
  'rust-v4.8.0 rust-runner-client-v0.2.13'
run_case "runner-client already past 0.x is not this rule's case" 0 "is consistent" \
  '{"rust":"4.8.0","rust-runner-client":"1.2.0"}' '{"rust":"5.0.0","rust-runner-client":"1.2.1"}' \
  'rust-v4.8.0 rust-runner-client-v1.2.0'
run_case "a qontinui-types major going DOWN is not this rule's case" 0 "is consistent" \
  '{"rust":"5.0.0","rust-runner-client":"0.3.0"}' '{"rust":"4.9.0","rust-runner-client":"0.3.1"}' \
  'rust-v5.0.0 rust-runner-client-v0.3.0'
run_case "a prerelease suffix is a version" 0 "is consistent" \
  "$ordinary" '{"rust":"4.9.0-rc.1","rust-runner-client":"0.2.14"}'
run_case "a build suffix is a version" 0 "is consistent" \
  "$ordinary" '{"rust":"4.9.0+b.1","rust-runner-client":"0.2.14"}'
run_case "the message names the caret without a prerelease suffix" 1 "user on ^0.2 would" \
  '{"rust":"4.8.0","rust-runner-client":"0.2.13-rc.1"}' '{"rust":"5.0.0","rust-runner-client":"0.2.14"}' \
  'rust-v4.8.0 rust-runner-client-v0.2.13-rc.1'
CASE_HEAD_REF=base run_case "HEAD_REF chooses the manifest that is read" 0 "0.2.13 -> 0.2.13" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.2.14"}'
run_case "a malformed head manifest is UNKNOWN" 2 "cannot evaluate" \
  "$ordinary" '{not json'
run_case "a head manifest with no rust row is UNKNOWN" 2 "no version string for rust" \
  "$ordinary" '{"rust-runner-client":"0.2.13"}'
run_case "a non-numeric version is UNKNOWN" 2 "not a major.minor.patch" \
  "$ordinary" '{"rust":"five","rust-runner-client":"0.2.14"}'
run_case "a leading zero is UNKNOWN" 2 "not a major.minor.patch" \
  "$ordinary" '{"rust":"05.0.0","rust-runner-client":"0.2.14"}'
run_case "trailing garbage on a version is UNKNOWN" 2 "not a major.minor.patch" \
  "$ordinary" '{"rust":"5.0.0garbage","rust-runner-client":"0.2.14"}'
run_case "a leading zero in a minor is UNKNOWN" 2 "not a major.minor.patch" \
  "$ordinary" '{"rust":"4.8.1","rust-runner-client":"0.08.0"}'
run_case "a leading zero in a patch is UNKNOWN" 2 "not a major.minor.patch" \
  "$ordinary" '{"rust":"4.8.01","rust-runner-client":"0.2.14"}'
run_case "garbage in the proposed runner-client version is UNKNOWN" 2 "not a major.minor.patch" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.3.0x"}'
run_case "a malformed base version is UNKNOWN, not a missing release" 2 "holds 'five'" \
  '{"rust":"five","rust-runner-client":"0.2.13"}' "$ordinary"
run_case "an empty base version is UNKNOWN, not a missing release" 2 "holds ''" \
  '{"rust":"","rust-runner-client":"0.2.13"}' "$ordinary"
run_case "a malformed base runner-client version is UNKNOWN" 2 "holds '0.2.13x'" \
  '{"rust":"4.8.0","rust-runner-client":"0.2.13x"}' "$ordinary"
run_case "a head manifest of two JSON documents is UNKNOWN" 2 "is not one JSON object" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.2.14"} {"rust":"4.8.0","rust-runner-client":"0.2.13"}'
run_case "a stray second document after a valid head is UNKNOWN" 2 "is not one JSON object" \
  "$ordinary" '{"rust":"5.0.0","rust-runner-client":"0.3.0"} {"x":"y"}'
run_case "a stray first document before a valid base is UNKNOWN" 2 "not one object of version strings" \
  '["x"] {"rust":"4.8.0","rust-runner-client":"0.2.13"}' "$ordinary"
run_case "two objects in the base manifest is UNKNOWN" 2 "not one object of version strings" \
  '{"rust":"4.8.0","rust-runner-client":"0.2.13"} {"x":"y"}' "$ordinary"
CASE_CONFIG='{"packages":{}} {"packages":{}}' run_case "two objects in the config is UNKNOWN" 2 "is not one JSON object" \
  "$ordinary" "$ordinary"
run_case "a numeric head row is UNKNOWN and named as such" 2 "no version string for rust" \
  "$ordinary" '{"rust":5,"rust-runner-client":"0.2.14"}'
run_case "an unreadable base manifest is UNKNOWN" 2 "no release-please-manifest.json readable at" \
  DELETED "$ordinary"
CASE_CONFIG=DELETED run_case "an unreadable config is UNKNOWN" 2 "no release-please-config.json readable at" \
  "$ordinary" "$ordinary"
run_case "a base with no rust row is UNKNOWN" 2 "no version string for rust" \
  '{"rust-runner-client":"0.2.13"}' '{"rust":"5.0.0","rust-runner-client":"0.2.14"}' 'rust-runner-client-v0.2.13'
run_case "a head with no runner-client row is UNKNOWN" 2 "no version string for rust-runner-client" \
  "$ordinary" '{"rust":"5.0.0"}'
run_case "an unreadable head manifest is UNKNOWN" 2 "no release-please-manifest.json readable" \
  "$ordinary" DELETED
run_case "a base with no runner-client row is UNKNOWN" 2 "no version string for rust-runner-client" \
  '{"rust":"4.8.0"}' '{"rust":"5.0.0","rust-runner-client":"0.3.0"}' 'rust-v4.8.0'
run_case "a null base row is UNKNOWN, not a missing release" 2 "not one object of version strings" \
  '{"rust":null,"rust-runner-client":"0.2.13"}' "$ordinary"
CASE_CONFIG='[]' run_case "a non-object config is UNKNOWN" 2 "is not one JSON object" \
  "$ordinary" "$ordinary"
run_case "a manifest of only unconfigured rows is UNKNOWN" 2 "has a package in the config" \
  '{"ci-exec":"0.1.1"}' '{"ci-exec":"0.1.1"}'
run_case "a non-404 GitHub answer is UNKNOWN" 2 "could not query tag" \
  "$ordinary" "$ordinary" "$tagged" boom

# Missing inputs: a run with no repo at all still reads UNKNOWN, not MISSING.
n=$((n + 1))
rc=0
out="$(env -u REPO -u GITHUB_REPOSITORY -u BASE_SHA bash "$script" 2>&1)" || rc=$?
if [[ "$rc" == 2 && "$out" == *"REPO or GITHUB_REPOSITORY required"* ]]; then
  echo "ok    no repo named is UNKNOWN"
else
  echo "FAIL  no repo named: wanted rc=2, got rc=$rc: $out"
  failures=$((failures + 1))
fi
n=$((n + 1))
rc=0
out="$(env -u BASE_SHA REPO=o/r bash "$script" 2>&1)" || rc=$?
if [[ "$rc" == 2 && "$out" == *"BASE_SHA required"* ]]; then
  echo "ok    no base sha is UNKNOWN"
else
  echo "FAIL  no base sha: wanted rc=2, got rc=$rc: $out"
  failures=$((failures + 1))
fi

echo
if [[ "$failures" -gt 0 ]]; then
  echo "release-pr-sanity-test: $failures of $n case(s) failed"
  exit 1
fi
[[ "$n" -gt 0 ]] || { echo "release-pr-sanity-test: ran no cases"; exit 1; }
echo "release-pr-sanity-test: all $n cases passed"
