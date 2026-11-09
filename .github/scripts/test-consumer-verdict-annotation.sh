#!/usr/bin/env bash
# Verdict table for consumer-verdict-annotation.sh, run in CI on every PR
# (job consumer-verdict-annotation-test in runner-consumer-check.yml). Asserts
# the exact line for both reasons with real values (qontinui-schemas#309's
# declared adaptation qontinui-runner#2794, as measured 2026-11-09), every
# refusal arm (exit 2, empty stdout), and that no output carries the
# not-a-verdict phrase. No network.
set -euo pipefail

script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/consumer-verdict-annotation.sh"
fails=0
cases=0

# expect_line <name> <expected stdout> <args...>
expect_line() {
  local name="$1" want="$2"
  shift 2
  cases=$((cases + 1))
  local got rc=0
  got="$(bash "$script" "$@" 2>/dev/null)" || rc=$?
  if [ "$rc" -ne 0 ] || [ "$got" != "$want" ]; then
    echo "FAIL $name: rc=$rc"
    echo "  want: $want"
    echo "  got:  $got"
    fails=$((fails + 1))
    return
  fi
  if [[ "$got" == *"(not a verdict on this PR)"* ]]; then
    echo "FAIL $name: output carries the not-a-verdict phrase"
    fails=$((fails + 1))
    return
  fi
  echo "ok   $name"
}

# expect_refusal <name> <args...>
expect_refusal() {
  local name="$1"
  shift
  cases=$((cases + 1))
  local got rc=0
  got="$(bash "$script" "$@" 2>/dev/null)" || rc=$?
  if [ "$rc" -ne 2 ] || [ -n "$got" ]; then
    echo "FAIL $name: want exit 2 and empty stdout, got rc=$rc stdout='$got'"
    fails=$((fails + 1))
    return
  fi
  echo "ok   $name"
}

repo="qontinui/qontinui-runner"
pending_sha="9791385c73a4e0d8f46305ceb8920b1f654e81f4"
red_sha="2c7e9073bbb7d20e65efcd4a2b980b74e53c924f"
pending_msg="qontinui-runner: breaking change awaiting declared adaptation qontinui/qontinui-runner#2794, required check(s) not yet green: Clippy (windows), test (ubuntu-22.04), test (windows-latest)"
red_msg="qontinui-runner: breaking change awaiting declared adaptation qontinui/qontinui-runner#2794, RED on required check(s): test (ubuntu-22.04), test (windows-latest)"

# --- the two emitted lines, exactly -----------------------------------------
expect_line "checks-pending exact line" \
  "::error title=adaptation-awaited qontinui/qontinui-runner#2794@9791385c73a4e0d8f46305ceb8920b1f654e81f4 checks-pending::${pending_msg}" \
  checks-pending "$repo" 2794 "$pending_sha" "$pending_msg"
expect_line "checks-red exact line" \
  "::error title=adaptation-awaited qontinui/qontinui-runner#2794@2c7e9073bbb7d20e65efcd4a2b980b74e53c924f checks-red::${red_msg}" \
  checks-red "$repo" 2794 "$red_sha" "$red_msg"
expect_line "multi-line message is flattened to one line" \
  "::error title=adaptation-awaited qontinui/qontinui-runner#2794@${red_sha} checks-red::a b" \
  checks-red "$repo" 2794 "$red_sha" $'a\nb'

# --- refusal arms ------------------------------------------------------------
expect_refusal "wrong arg count" checks-red "$repo" 2794 "$red_sha"
expect_refusal "reason: the probe's long form" declared-pr-checks-red "$repo" 2794 "$red_sha" m
expect_refusal "reason: empty" "" "$repo" 2794 "$red_sha" m
expect_refusal "repo: no owner" checks-red qontinui-runner 2794 "$red_sha" m
expect_refusal "repo: comma" checks-red "qontinui/qontinui,runner" 2794 "$red_sha" m
expect_refusal "repo: colon" checks-red "qontinui/qontinui:runner" 2794 "$red_sha" m
expect_refusal "repo: three segments" checks-red "a/b/c" 2794 "$red_sha" m
expect_refusal "repo: space" checks-red "qontinui/qontinui runner" 2794 "$red_sha" m
expect_refusal "repo: not-a-verdict phrase" checks-red "(not a verdict on this PR)" 2794 "$red_sha" m
expect_refusal "number: not digits" checks-red "$repo" "#2794" "$red_sha" m
expect_refusal "number: empty" checks-red "$repo" "" "$red_sha" m
expect_refusal "sha: short" checks-red "$repo" 2794 2c7e9073b m
expect_refusal "sha: uppercase" checks-red "$repo" 2794 "${red_sha^^}" m
expect_refusal "sha: 41 hex" checks-red "$repo" 2794 "${red_sha}0" m
expect_refusal "sha: empty" checks-red "$repo" 2794 "" m

echo ""
if [ "$fails" -ne 0 ]; then
  echo "$fails of $cases case(s) FAILED"
  exit 1
fi
echo "all $cases case(s) passed"
