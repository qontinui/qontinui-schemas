#!/usr/bin/env bash
# Print the ONE typed annotation a red `consumer-check (<consumer>, ...)` job
# emits when leg 2 resolved a declared, open, main-based adaptation whose own
# required checks are pending or red:
#
#   ::error title=adaptation-awaited <owner>/<repo>#<n>@<sha> <checks-pending|checks-red>::<message>
#
# The TITLE is the contract: coord reads check-run annotation titles (titles
# only) to tell "red because the declared adaptation's CI is not green yet"
# from a real break. The grammar is fixed, one line, no commas or colons
# (GitHub escapes both inside a workflow-command property), and it must never
# carry "(not a verdict on this PR)" — this red IS a verdict on the pair.
#
# Usage: consumer-verdict-annotation.sh <reason> <owner/repo> <n> <sha> <message>
#   reason   checks-pending | checks-red
#   owner/repo  over [A-Za-z0-9_.-]
#   n        the adaptation PR number (digits)
#   sha      the adaptation head the probe read (40 lowercase hex)
#   message  free text (newlines are flattened so the output stays one line)
#
# Refuses with exit 2 and NOTHING on stdout on any malformed input, so a
# caller's fallback (the untitled line) is what lands and coord reads the red
# as unexplained — never a wrong title.
set -euo pipefail

refuse() {
  echo "consumer-verdict-annotation: refused: $*" >&2
  exit 2
}

[ "$#" -eq 5 ] || refuse "expected 5 arguments, got $#"

reason="$1"
repo="$2"
num="$3"
sha="$4"
message="$5"

case "$reason" in
  checks-pending|checks-red) ;;
  *) refuse "reason '$reason' is not checks-pending or checks-red" ;;
esac

[[ "$repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] \
  || refuse "repo '$repo' is not <owner>/<repo> over [A-Za-z0-9_.-]"
[[ "$num" =~ ^[0-9]+$ ]] || refuse "PR number '$num' is not digits"
[[ "$sha" =~ ^[0-9a-f]{40}$ ]] || refuse "sha '$sha' is not 40 lowercase hex"

title="adaptation-awaited ${repo}#${num}@${sha} ${reason}"

case "$title" in
  *,*) refuse "title would contain a comma: $title" ;;
  *:*) refuse "title would contain a colon: $title" ;;
  *"(not a verdict on this PR)"*) refuse "title would carry the not-a-verdict phrase" ;;
esac

message="${message//$'\r'/ }"
message="${message//$'\n'/ }"

printf '::error title=%s::%s\n' "$title" "$message"
