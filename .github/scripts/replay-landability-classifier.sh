#!/usr/bin/env bash
#
# Replay of the LANDABILITY CLASSIFIER from
# .github/workflows/runner-consumer-check.yml (the "Leg 2: resolve the declared
# adaptation PR" step) against a real consumer repo + SHA, using the same live
# GitHub API reads.
#
# WHY THIS EXISTS. The consumer gate's shell has no other test surface: it only
# runs inside a 38-minute cross-repo compile on a pull_request event, so a
# change to the classifier is otherwise unfalsifiable until it has already
# green-lit (or wrongly reddened) a real PR. This script makes the classifier's
# verdict reproducible in seconds, against SHAs whose correct answer is known.
#
# The two discriminating cases from the 2026-08-25 incident:
#
#   # must print: declared-pr-checks-red -> Clippy (windows), Frontend unit
#   #             tests (vitest), test (ubuntu-22.04), test (windows-latest)
#   # and must NOT name "Clippy diff-scoped (advisory)", which was equally red
#   # but is not a required context.
#   ./replay-landability-classifier.sh qontinui/qontinui-runner \
#       0a76247ded0b6e0f25549512e320d62f284d895a
#
#   # must print: ok (green on every required context)
#   ./replay-landability-classifier.sh qontinui/qontinui-runner \
#       391aaeca53d6ff1091a0c131ac859e5a38962aca
#
#   # must print: no-required-contexts — qontinui-supervisor's main is
#   # unprotected, so landability is not CI-gated there.
#   ./replay-landability-classifier.sh qontinui/qontinui-supervisor <any-sha>
#
# KEEP IN SYNC with the workflow step. If you change the classifier there,
# change it here and re-run all three cases.
set -euo pipefail
CONSUMER_REPO="$1"; sha="$2"
rules_json="$(gh api "repos/${CONSUMER_REPO}/rules/branches/main")"
required="$(printf '%s' "$rules_json" \
  | jq -r '[.[] | select(.type == "required_status_checks")
                | .parameters.required_status_checks[].context]
           | unique | .[]' | tr -d '\r')"
if [ -z "$required" ]; then echo "VERDICT: no-required-contexts (::warning:: + fall through to compile-only)"; exit 0; fi
echo "required: $(printf '%s' "$required" | tr '\n' ',' | sed 's/,$//')"
checks_json="$(gh api --paginate "repos/${CONSUMER_REPO}/commits/${sha}/check-runs?per_page=100")"
statuses_json="$(gh api "repos/${CONSUMER_REPO}/commits/${sha}/status?per_page=100")"
check_table="$(printf '%s' "$checks_json" \
  | jq -rs '[.[].check_runs[]] | group_by(.name) | map(max_by(.id)) | .[]
           | [ .name, (if .status != "completed" then "pending"
                       elif (.conclusion // "") | IN("success","neutral","skipped") then "success"
                       else "red" end) ] | @tsv')"
status_table="$(printf '%s' "$statuses_json" \
  | jq -r '[.statuses[]?] | .[]
           | [ .context, (if .state == "success" then "success"
                          elif .state == "pending" then "pending"
                          else "red" end) ] | @tsv')"
all_checks="$(printf '%s\n%s\n' "$check_table" "$status_table" | tr -d '\r')"
red_list=""; pending_list=""
while IFS= read -r ctx; do
  [ -n "$ctx" ] || continue
  st="$(printf '%s\n' "$all_checks" | awk -F'\t' -v c="$ctx" '$1 == c { print $2; exit }')"
  case "${st:-pending}" in
    success) ;;
    red)     red_list="${red_list}${red_list:+, }${ctx}" ;;
    *)       pending_list="${pending_list}${pending_list:+, }${ctx}" ;;
  esac
done <<REQUIRED
$required
REQUIRED
if [ -n "$red_list" ];     then echo "VERDICT: declared-pr-checks-red -> $red_list"; exit 0; fi
if [ -n "$pending_list" ]; then echo "VERDICT: declared-pr-checks-pending -> $pending_list"; exit 0; fi
echo "VERDICT: ok (green on every required context)"
