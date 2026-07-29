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
# A plain `git diff` compares the working tree against the INDEX, so it is
# blind both to files git does not track and to anything already staged.
# `ts/src/generated` had ZERO tracked files from 34709ada (2026-06-25) until
# the change that added this script: the whole directory was swept out of the
# index as collateral in a commit about a secret-scan caller, and a95d51da
# later took the barrel `index.ts` the same way. Nothing in .gitignore covers
# the path — it was never a decision.
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
# Run from the root of a qontinui-schemas checkout. Both entry points assert
# that: the relative paths below and arm 1's path-prefix filter are only
# meaningful there, and `git ls-files` silently matches nothing when a
# repo-relative pathspec is resolved from a subdirectory.

set -euo pipefail

# The generated trees, defined ONCE — repo-root-relative, which
# assert_repo_root() below makes safe to assume.
TS_GENERATED_DIR="ts/src/generated"
PY_GENERATED_DIR="src/qontinui_schemas/generated"
GENERATED_DIRS=(
    "$TS_GENERATED_DIR"
    "$PY_GENERATED_DIR"
)

# Where generate_types.sh lives, relative to this repo's root.
#
# It lives in qontinui-runner, NOT here. Both workflows that run this gate lay
# the two repos out as siblings under $GITHUB_WORKSPACE and invoke the gate
# with the qontinui-schemas checkout as cwd, so the sibling-relative default
# resolves in CI. Override it for a local layout that is not sibling-shaped.
GENERATE_TYPES_SH="${GENERATE_TYPES_SH:-../qontinui-runner/src-tauri/scripts/generate_types.sh}"

# The exact assignments in generate_types.sh that this gate knows how to
# evaluate. See cross_check_output_dirs() for why these are matched verbatim.
EXPECT_SCRIPT_DIR='SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"'
EXPECT_PROJECT_ROOT='PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"'

usage() {
    echo "usage: $0 --preflight | --verify" >&2
    exit 2
}

[ "$#" -eq 1 ] || usage

# Every pathspec and path-prefix in this file is repo-root-relative. Running
# from a subdirectory does not error — `git ls-files -- ts/src/generated` from
# `ts/` simply matches nothing — so it would turn arm 1 into a no-op and make
# arm 1's `__pycache__` prefix filter meaningless. Assert instead of assuming.
#
# `--show-prefix` (empty at the root) rather than comparing `--show-toplevel`
# to `pwd`: on Windows/MSYS git prints `D:/...` while `pwd` prints `/d/...`,
# so a string compare would fail spuriously for every local run.
assert_repo_root() {
    local prefix rc=0
    # Callers use `assert_repo_root || rc=1`, which disables `set -e` inside
    # this function — so capture the failure explicitly. Left implicit, a
    # git-less environment would leave `prefix` empty and be waved through.
    prefix="$(git rev-parse --show-prefix)" || rc=$?
    if [ "$rc" -ne 0 ]; then
        echo "::error::not inside a git work tree; this gate has nothing to compare against."
        return 1
    fi
    if [ -n "$prefix" ]; then
        echo "::error::run this from the root of the qontinui-schemas checkout."
        echo "  cwd is '$prefix' inside the repo; every path this gate uses is"
        echo "  repo-root-relative and would silently match nothing from here."
        return 1
    fi
}

# Collapse `.`, `..` and duplicate slashes in an absolute path, textually.
#
# Textual rather than `realpath`/`cd && pwd -P` because the whole point is to
# resolve a path that may NOT exist — a generator whose output dir has moved
# writes to a directory this checkout has never seen. Both operands are
# anchored on a `pwd -P` result, so the only symlink that could survive the
# `..` collapse would be one inside the generator's own relative suffix.
lexical_normalize() {
    local path="$1" out="" part
    local -a parts
    IFS='/' read -r -a parts <<< "$path"
    for part in "${parts[@]}"; do
        case "$part" in
            ''|.) ;;
            ..)   out="${out%/*}" ;;
            *)    out="$out/$part" ;;
        esac
    done
    printf '%s' "${out:-/}"
}

# ── output-directory cross-check ────────────────────────────────────────────
#
# This gate inspects GENERATED_DIRS. generate_types.sh derives its own
# TS_OUT_DIR/PY_OUT_DIR from its own location. Nothing but convention links
# the two, and asserting merely that the committed dirs EXIST does not test
# the link at all: if the generator's paths move, the committed dirs still
# exist, preflight still passes, codegen writes somewhere unreviewed, and all
# three verify arms then pass vacuously — files still tracked (arm 0), nothing
# new untracked (arm 1), nothing changed (arm 2). That is precisely the
# failure class this file exists to eliminate.
#
# So resolve what the generator will ACTUALLY write and require it to equal
# what we inspect.
#
# We do not run the generator (it costs a full cargo release build) and we do
# not source it (everything from its `cargo build` line down has side effects).
# We parse the four assignments that produce the paths — and we assert their
# SHAPE before evaluating them:
#
#   * SCRIPT_DIR and PROJECT_ROOT must match, character for character, the
#     forms this function knows how to reproduce. That exact match is what
#     licenses computing the generator's project root as `dirname(script)/..`.
#   * TS_OUT_DIR / PY_OUT_DIR must be a literal suffix under `$PROJECT_ROOT`,
#     with no further expansion in it.
#   * The set of `*_OUT_DIR` variables must be exactly {TS_OUT_DIR,
#     PY_OUT_DIR}, so a generator that grows a THIRD output tree fails here
#     instead of writing it somewhere no arm inspects.
#
# Any deviation is a hard failure with an explicit "teach this check the new
# shape" message. A cross-check that can degrade to "assume it's fine" would
# reintroduce the vacuity it is supposed to close, so this one cannot: it
# either proves agreement or reds.
cross_check_output_dirs() {
    if [ ! -f "$GENERATE_TYPES_SH" ]; then
        echo "::error::generate_types.sh not found at '$GENERATE_TYPES_SH'."
        echo "  This gate inspects ${GENERATED_DIRS[*]}, but the generator derives"
        echo "  its own output dirs from its own location. Without the script the"
        echo "  agreement cannot be proven, and an unproven agreement is exactly"
        echo "  the vacuity this gate exists to prevent."
        echo "  Both drift workflows check qontinui-runner out as a sibling of this"
        echo "  repo. For any other layout, set GENERATE_TYPES_SH to the path of"
        echo "  qontinui-runner/src-tauri/scripts/generate_types.sh."
        return 1
    fi

    # Read it once. The rc is captured rather than left to `set -e`, which the
    # caller's `|| rc=1` has already disabled inside this function: an
    # unreadable generator must not be silently reduced to empty input.
    local src rc_read=0
    src="$(cat "$GENERATE_TYPES_SH")" || rc_read=$?
    if [ "$rc_read" -ne 0 ]; then
        echo "::error::could not read '$GENERATE_TYPES_SH' (cat exited $rc_read)."
        echo "  The output-dir agreement cannot be proven, so this fails rather"
        echo "  than assuming it holds."
        return 1
    fi

    local out_re='^(TS|PY)_OUT_DIR="\$PROJECT_ROOT/([A-Za-z0-9._/-]+)"$'
    local any_out_re='^([A-Za-z_][A-Za-z0-9_]*_OUT_DIR)='

    local line t
    local script_dir_n=0 project_root_n=0
    local script_dir_line="" project_root_line=""
    local ts_suffix="" py_suffix=""
    local -a out_dir_names=()
    local -a bad_out_dir_lines=()

    while IFS= read -r line; do
        # Strip leading whitespace so an indented assignment is still SEEN
        # (and then judged on its shape) rather than skipped.
        t="${line#"${line%%[![:space:]]*}"}"
        case "$t" in
            SCRIPT_DIR=*)
                script_dir_n=$((script_dir_n + 1)); script_dir_line="$t" ;;
            PROJECT_ROOT=*)
                project_root_n=$((project_root_n + 1)); project_root_line="$t" ;;
        esac
        if [[ "$t" =~ $any_out_re ]]; then
            out_dir_names+=("${BASH_REMATCH[1]}")
            if [[ "$t" =~ $out_re ]]; then
                case "${BASH_REMATCH[1]}" in
                    TS) ts_suffix="${BASH_REMATCH[2]}" ;;
                    PY) py_suffix="${BASH_REMATCH[2]}" ;;
                esac
            else
                bad_out_dir_lines+=("$t")
            fi
        fi
    done <<< "$src"

    local rc=0
    local shape_hint="  Teach this cross-check the new shape (scripts/check-generated-drift.sh,"
    shape_hint="$shape_hint"$'\n'"  cross_check_output_dirs) in the same change. Do NOT delete the check:"
    shape_hint="$shape_hint"$'\n'"  without it a moved output dir makes every arm below pass vacuously."

    if [ "$script_dir_n" -ne 1 ] || [ "$script_dir_line" != "$EXPECT_SCRIPT_DIR" ]; then
        echo "::error::generate_types.sh derives SCRIPT_DIR in an unrecognized way."
        echo "  expected exactly one: $EXPECT_SCRIPT_DIR"
        echo "  found ($script_dir_n): ${script_dir_line:-<none>}"
        echo "$shape_hint"
        rc=1
    fi

    if [ "$project_root_n" -ne 1 ] || [ "$project_root_line" != "$EXPECT_PROJECT_ROOT" ]; then
        echo "::error::generate_types.sh derives PROJECT_ROOT in an unrecognized way."
        echo "  expected exactly one: $EXPECT_PROJECT_ROOT"
        echo "  found ($project_root_n): ${project_root_line:-<none>}"
        echo "$shape_hint"
        rc=1
    fi

    # Exactly the two output trees this gate inspects — no more, no fewer.
    local names_seen
    names_seen="$(printf '%s\n' ${out_dir_names[@]+"${out_dir_names[@]}"} | LC_ALL=C sort | tr '\n' ' ')"
    names_seen="${names_seen% }"
    if [ "$names_seen" != "PY_OUT_DIR TS_OUT_DIR" ]; then
        echo "::error::generate_types.sh's set of *_OUT_DIR variables is not the set this gate inspects."
        echo "  expected: PY_OUT_DIR TS_OUT_DIR"
        echo "  found:    ${names_seen:-<none>}"
        echo "  A new output tree that no arm of --verify inspects is drift this"
        echo "  gate would never see. Add it to GENERATED_DIRS and commit its"
        echo "  baseline, or remove it from the generator."
        rc=1
    fi

    if [ "${#bad_out_dir_lines[@]}" -gt 0 ]; then
        echo "::error::generate_types.sh assigns an output dir in an unrecognized way."
        printf '  %s\n' "${bad_out_dir_lines[@]}"
        echo "  expected form: <TS|PY>_OUT_DIR=\"\$PROJECT_ROOT/<literal path>\""
        echo "$shape_hint"
        rc=1
    fi

    [ "$rc" -eq 0 ] || return "$rc"

    # Shape proven; now evaluate. SCRIPT_DIR/PROJECT_ROOT matched verbatim
    # above, so the generator's PROJECT_ROOT is exactly dirname(script)/..
    local gen_root repo_root
    gen_root="$(cd "$(dirname "$GENERATE_TYPES_SH")/.." && pwd -P)"
    repo_root="$(pwd -P)"

    local var suffix gate_dir writes inspects
    for var in TS PY; do
        if [ "$var" = TS ]; then
            suffix="$ts_suffix"; gate_dir="$TS_GENERATED_DIR"
        else
            suffix="$py_suffix"; gate_dir="$PY_GENERATED_DIR"
        fi
        writes="$(lexical_normalize "$gen_root/$suffix")"
        inspects="$(lexical_normalize "$repo_root/$gate_dir")"
        if [ "$writes" = "$inspects" ]; then
            echo "preflight: ${var}_OUT_DIR agrees -> $inspects"
        else
            echo "::error::${var}_OUT_DIR disagrees with the tree this gate inspects."
            echo "  generate_types.sh writes to: $writes"
            echo "  this gate inspects:          $inspects"
            echo "  Codegen output would land somewhere unreviewed and every arm of"
            echo "  --verify would pass vacuously. Point GENERATED_DIRS at the new"
            echo "  location (and commit its baseline), or revert the generator."
            echo "  Locally this usually means the checkout directory is not named"
            echo "  'qontinui-schemas' (a git worktree, say). The generator hardcodes"
            echo "  that name, so a regen from here would write into the sibling it"
            echo "  DOES resolve — and this checkout would report a green it never"
            echo "  regenerated. That is a real finding, not a false alarm."
            rc=1
        fi
    done

    return "$rc"
}

# ── --preflight ─────────────────────────────────────────────────────────────
#
# Asserts that regeneration will actually regenerate, INTO THE TREE THIS GATE
# INSPECTS.
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

    assert_repo_root || rc=1

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
            echo "  The arms of --verify have nothing to inspect without it."
            rc=1
        fi
    done

    # The dirs existing says nothing about whether codegen aims at them.
    cross_check_output_dirs || rc=1

    return "$rc"
}

# ── --verify ────────────────────────────────────────────────────────────────
#
# Three arms, because a diff alone covers only one of the ways codegen output
# can move:
#
#   arm 0 (baseline)  — the checked-in copy exists at all. Without it arms 1
#                       and 2 are both meaningless.
#   arm 1 (untracked) — a NEW type appears, so the regen writes a file git does
#                       not track and no diff can see. Also catches a repeat of
#                       the index sweep, one file at a time.
#   arm 2 (diff)      — an EXISTING file changed, or was deleted because its
#                       type was removed, or was added AND staged.
#
# All three run before the function returns, so one invocation reports every
# problem rather than making you fix them one CI round-trip at a time.
verify() {
    local rc=0

    assert_repo_root || rc=1

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
    # `--others` excludes anything already in the index, so a `git add`-ed but
    # uncommitted generated file is invisible here — arm 2 covers that, which
    # is why it diffs against HEAD rather than the index.
    #
    # Split across two statements on purpose. Written as one pipeline with a
    # trailing `|| true`, a FAILURE of `git ls-files` would be swallowed along
    # with grep's no-match exit, leaving `new_files` empty and this arm
    # silently reporting clean — arm 1 no-oping is precisely the bug class this
    # file exists to prevent. Here `git ls-files` runs unguarded (so `set -e`
    # aborts on a real failure) and only grep's exit is tolerated.
    local raw new_files pycache_re
    raw="$(git ls-files --others -- "${GENERATED_DIRS[@]}")"
    # SCOPED to the Python tree. An unanchored `/__pycache__/` filter drops the
    # segment wherever it appears, so `ts/src/generated/__pycache__/Hidden.d.ts`
    # — a path with no legitimate reason to exist — would be silently excused.
    # `$PY_GENERATED_DIR` contains no ERE metacharacters, so it is safe inline.
    pycache_re="^${PY_GENERATED_DIR}/(.*/)?__pycache__/"
    # `|| true` is LOAD-BEARING, not redundant: `grep -v` exits 1 when it
    # prints nothing, which is the normal healthy case, and this script runs
    # under `set -e`. Removing it aborts the job on every clean run.
    new_files="$(printf '%s' "$raw" | grep -Ev "$pycache_re" || true)"
    if [ -n "$new_files" ]; then
        echo "::error::Regeneration produced generated files that are not checked in:"
        printf '%s\n' "$new_files"
        rc=1
    fi

    # arm 2 — committed files that changed, were deleted, or were staged.
    #
    # Against HEAD, not the index. A bare `git diff` compares the working tree
    # to the INDEX, so `git add`-ing drift makes it disappear from this arm
    # while `--others` in arm 1 has already stopped listing it: a staged new
    # file and a staged modification would each exit 0 on genuine drift. CI
    # checkouts have a clean index today, so nothing depends on it — which is
    # the whole problem, since it means one stray `git add` in the regen path
    # would silence this arm with no failing test to notice. `HEAD` makes the
    # arm independent of the index's state.
    #
    # `-I '^#   timestamp:'` ignores hunks whose only changed lines are the
    # `# timestamp: <ISO-8601>` header datamodel-codegen writes at the top of
    # every generated Python file. Those update on every run regardless of
    # schema content, so without the filter this could never go green even on
    # a repo with no real drift. Same trick schema-pg-sql-fresh.yml uses for
    # `-- Dumped by pg_dump version` header noise.
    if ! git diff HEAD --exit-code -I '^#   timestamp:' -- "${GENERATED_DIRS[@]}"; then
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
