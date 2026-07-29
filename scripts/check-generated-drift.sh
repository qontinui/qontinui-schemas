#!/usr/bin/env bash
#
# check-generated-drift.sh — the anti-vacuity guard for the generated bindings.
#
# ONE implementation, called from BOTH sides of the cross-repo codegen
# contract:
#   - qontinui-schemas/.github/workflows/schema-drift.yml
#   - qontinui-runner/.github/workflows/qontinui-types-drift.yml
#
# It lives here, in qontinui-schemas, deliberately. The runner-side workflow
# already checks this repo out (at the coordinated ref) in order to regenerate
# into it, so `bash qontinui-schemas/scripts/check-generated-drift.sh` works
# verbatim there. That coupling is the point: the check then travels on the
# same branch as the bindings it guards, and the two workflows cannot silently
# diverge on the property they exist to enforce. They did diverge once already,
# within a single change, which is what motivated extracting this.
#
# ── WHY THIS EXISTS ─────────────────────────────────────────────────────────
#
# `git diff` compares the working tree against the INDEX, so it is blind to
# files git does not track. `ts/src/generated` had ZERO tracked files from
# 34709ada (2026-06-25) until the change that added this script: the whole
# directory was swept out of the index as collateral in a commit about a
# secret-scan caller, and a95d51da later took the barrel `index.ts` the same
# way. Nothing in .gitignore covers the path — it was never a decision.
#
# Both workflows therefore regenerated the bindings into an untracked
# directory and diffed them against nothing. The TypeScript arm of both was
# vacuously green for its entire life; only the Python arm carried signal.
# The same absence broke the npm package: the ts-v0.9.0 publish failed with
# `TS2307: Cannot find module './generated/SpecValidation'` and went unnoticed
# for three weeks, because the gate that should have caught it was the gate
# the same sweep had made vacuous.
#
# A gate that cannot go red is worse than no gate: it manufactures confidence.
#
# ── USAGE ───────────────────────────────────────────────────────────────────
#
#   check-generated-drift.sh --preflight   # before regenerating
#   check-generated-drift.sh --verify      # after regenerating
#
# Run from the root of a qontinui-schemas checkout.

set -euo pipefail

# The generated trees, defined ONCE. `generate_types.sh` derives its own
# TS_OUT_DIR/PY_OUT_DIR from its own location, so these two lists have to
# agree with it by convention. --preflight cross-checks that agreement rather
# than trusting it: if the script's output dirs ever move, every arm below
# would go green while codegen wrote somewhere unreviewed.
GENERATED_DIRS=(
    "ts/src/generated"
    "src/qontinui_schemas/generated"
)

usage() {
    echo "usage: $0 --preflight | --verify" >&2
    exit 2
}

[ "$#" -eq 1 ] || usage

# ── --preflight ─────────────────────────────────────────────────────────────
#
# Asserts that regeneration will actually regenerate.
#
# This closes the same vacuity hole one layer down. generate_types.sh does NOT
# fail when its generators are missing — it warns and skips:
#
#     if ! command -v node &> /dev/null; then
#         echo "WARNING: node not found. Skipping TypeScript generation."
#     else
#         rm -f "$TS_OUT_DIR"/*.d.ts ...
#
# The `rm -f` is inside the `else`, so on a skip the working tree is left
# pristine and every arm of --verify passes: files are tracked (arm 0), nothing
# is untracked (arm 1), nothing differs (arm 2). Green, with the TypeScript
# bindings never generated. `datamodel-codegen` has the identical shape, and it
# arrives via `pip install`, so any PATH change would make the Python arm
# vacuous behind a WARNING nobody reads.
#
# The arms verify the BASELINE. This verifies the REGENERATION.
preflight() {
    local rc=0

    for tool in node cargo datamodel-codegen; do
        if command -v "$tool" >/dev/null 2>&1; then
            echo "preflight: $tool -> $(command -v "$tool")"
        else
            echo "::error::$tool is not on PATH."
            echo "  generate_types.sh SKIPS generation when its generators are missing"
            echo "  and still exits 0, which would make this whole gate report a"
            echo "  green it did not verify. Failing instead."
            rc=1
        fi
    done

    for dir in "${GENERATED_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            echo "preflight: $dir present"
        else
            echo "::error::$dir does not exist in this checkout."
            echo "  generate_types.sh writes there by a path derived from its own"
            echo "  location. If that path has moved, codegen output lands somewhere"
            echo "  this gate does not inspect and every arm below passes vacuously."
            rc=1
        fi
    done

    return "$rc"
}

# ── --verify ────────────────────────────────────────────────────────────────
#
# Three arms, because `git diff` alone covers only one of the ways codegen
# output can move:
#
#   arm 0 (baseline)  — the checked-in copy exists at all. Without it arms 1
#                       and 2 are both meaningless.
#   arm 1 (untracked) — a NEW type appears, so the regen writes a file git does
#                       not track and `git diff` cannot see. Also catches a
#                       repeat of the index sweep, one file at a time.
#   arm 2 (diff)      — an EXISTING file changed, or was deleted because its
#                       type was removed.
#
# All three run before the function returns, so one invocation reports every
# problem rather than making you fix them one CI round-trip at a time.
verify() {
    local rc=0

    # arm 0 — baseline present?
    local dir n
    for dir in "${GENERATED_DIRS[@]}"; do
        n="$(git ls-files -- "$dir" | wc -l)"
        echo "baseline $dir: $n tracked file(s)"
        if [ "$n" -eq 0 ]; then
            echo "::error::$dir has ZERO tracked files, so the drift diff is vacuous."
            echo "  The generated bindings must be committed for this gate to have"
            echo "  a baseline to compare a fresh regeneration against."
            rc=1
        fi
    done

    # arm 1 — regenerated files git is not tracking.
    #
    # Deliberately NOT `--exclude-standard`. A .gitignore rule covering only
    # PART of a generated tree (say `*.d.ts`, leaving index.ts tracked) would
    # defeat arm 0 and arm 1 simultaneously: arm 0 sees a non-zero tracked
    # count and passes, and an ignore-respecting arm 1 skips exactly the files
    # the rule hides. Listing ignored files too closes that hole. `__pycache__`
    # is the only thing that legitimately appears under these paths without
    # being codegen output.
    #
    # Split across two statements on purpose. Written as one pipeline with a
    # trailing `|| true`, a FAILURE of `git ls-files` would be swallowed along
    # with grep's no-match exit, leaving `new_files` empty and this arm
    # silently reporting clean — arm 1 no-oping is precisely the bug class this
    # file exists to prevent. Here `git ls-files` runs unguarded (so `set -e`
    # aborts on a real failure) and only grep's exit is tolerated.
    local raw new_files
    raw="$(git ls-files --others -- "${GENERATED_DIRS[@]}")"
    # `|| true` is LOAD-BEARING, not redundant: `grep -v` exits 1 when it
    # prints nothing, which is the normal healthy case, and this script runs
    # under `set -e`. Removing it aborts the job on every clean run.
    new_files="$(printf '%s' "$raw" | grep -v '/__pycache__/' || true)"
    if [ -n "$new_files" ]; then
        echo "::error::Regeneration produced generated files that are not checked in:"
        printf '%s\n' "$new_files"
        rc=1
    fi

    # arm 2 — tracked files that changed or were deleted.
    #
    # `-I '^#   timestamp:'` ignores hunks whose only changed lines are the
    # `# timestamp: <ISO-8601>` header datamodel-codegen writes at the top of
    # every generated Python file. Those update on every run regardless of
    # schema content, so without the filter this could never go green even on
    # a repo with no real drift. Same trick schema-pg-sql-fresh.yml uses for
    # `-- Dumped by pg_dump version` header noise.
    if ! git diff --exit-code -I '^#   timestamp:' -- "${GENERATED_DIRS[@]}"; then
        echo "::error::Checked-in generated types differ from a fresh regeneration."
        rc=1
    fi

    if [ "$rc" -ne 0 ]; then
        echo ""
        echo "To fix: from a checkout with qontinui-runner as a sibling, run"
        echo "  bash ../qontinui-runner/src-tauri/scripts/generate_types.sh"
        echo "and commit the updated files into qontinui-schemas."
        echo "Or download this run's 'regenerated-bindings' artifact and commit"
        echo "its contents — those are the exact bytes this check expected."
    fi

    return "$rc"
}

case "$1" in
    --preflight) preflight ;;
    --verify)    verify ;;
    *)           usage ;;
esac
