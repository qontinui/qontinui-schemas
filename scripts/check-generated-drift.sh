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
# already checks this repo out (at the ref its resolve-schemas-ref action
# picks) in order to regenerate into it, so
# `bash qontinui-schemas/scripts/check-generated-drift.sh` works verbatim
# there. That coupling is the point: the check then travels on the same branch
# as the bindings it guards, and the two workflows cannot silently diverge on
# the property they exist to enforce. They did diverge once already, within a
# single change, which is what motivated extracting this.
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
# Run from the ROOT of a qontinui-schemas checkout whose DIRECTORY IS NAMED
# `qontinui-schemas`, with a `qontinui-runner` checkout as its sibling. Both
# requirements are real, and both entry points assert them:
#
#   * repo root — every pathspec and path-prefix below is repo-root-relative,
#     and `git ls-files` silently matches nothing when such a pathspec is
#     resolved from a subdirectory.
#   * directory name — generate_types.sh DEFAULTS to
#     `$PROJECT_ROOT/../../qontinui-schemas`, so from a checkout named anything
#     else a regeneration writes into whichever sibling it DOES resolve, and
#     this checkout would report a green it never regenerated. Both CI callers
#     lay the repos out that way. An ad-hoc worktree (`wt-foo/`) does not, and
#     correctly fails — unless you set QONTINUI_SCHEMAS_DIR (below), which
#     makes the name irrelevant.
#
# Environment:
#   GENERATE_TYPES_SH      the generator to cross-check. Defaults to the
#                          sibling layout above.
#   QONTINUI_SCHEMAS_DIR   \
#   QONTINUI_TS_OUT_DIR     > the generator's own overrides. This gate reads
#   QONTINUI_PY_OUT_DIR    /  them and resolves exactly as the generator will,
#                          so a pinned output location is checked against the
#                          tree actually inspected rather than assumed away.

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
# resolves in CI. Both also set this variable explicitly to the same absolute
# path they then execute, so the script this gate PARSES is provably the script
# the workflow RUNS, rather than two independently-written strings that merely
# happen to agree.
GENERATE_TYPES_SH="${GENERATE_TYPES_SH:-../qontinui-runner/src-tauri/scripts/generate_types.sh}"

# ── what the generator is allowed to look like ──────────────────────────────
#
# These are REGEXES, and a SET of them per variable, rather than byte-exact
# strings. Both properties are load-bearing.
#
# `check-drift` is a REQUIRED status check on this repo's main-merge-gates
# ruleset, and schema-drift.yml checks qontinui-runner out with no `ref:` —
# i.e. at runner's DEFAULT BRANCH. So a byte-exact assertion would mean: a
# cosmetic edit to the generator (a trailing space, an inline comment,
# `dirname --`, `pwd -P`) reds only an advisory check on the runner side, lands
# there, and from that moment EVERY PR in this repo fails a required check —
# including release-please's. A whole-repo train freeze, of the same class as
# the 2026-07-05 incident, newly created by the gate meant to prevent one.
#
# Hence: tolerate everything cosmetic, and keep the accepted forms a SET, so a
# catch-up here can teach the gate a new form BEFORE the runner change lands.
# (The reverse direction is already coordinated: the runner-side workflow
# resolves a matching qontinui-schemas ref through its resolve-schemas-ref
# action, so a runner PR can point at a schemas branch that already knows the
# new shape.)
#
# What is asserted is the PROPERTY that licenses computing the generator's
# project root as `dirname(script)/..` — SCRIPT_DIR is the script's own
# directory via BASH_SOURCE, PROJECT_ROOT is its parent — not a byte sequence.
# Matched against the assignment's right-hand side, with trailing comment and
# whitespace already stripped.
EXPECT_SCRIPT_DIR_RHS_FORMS=(
    '^"\$\(cd "\$\(dirname (--[[:space:]]+)*"\$\{BASH_SOURCE\[0\]\}"\)"[[:space:]]*&&[[:space:]]*pwd([[:space:]]+-P)?\)"$'
)
EXPECT_PROJECT_ROOT_RHS_FORMS=(
    '^"\$\(cd "\$\{?SCRIPT_DIR\}?/\.\."[[:space:]]*&&[[:space:]]*pwd([[:space:]]+-P)?\)"$'
)

# The root of this repo as the generator computes it: an env override, falling
# back to a literal path under $PROJECT_ROOT. Capture 1 is that fallback suffix.
EXPECT_SCHEMAS_DIR_RHS_FORMS=(
    '^"\$\{QONTINUI_SCHEMAS_DIR:-\$\{?PROJECT_ROOT\}?/([A-Za-z0-9._/-]+)\}"$'
)

# The right-hand side of an output-dir assignment: an env override, falling
# back to a literal path under $SCHEMAS_DIR. Capture 1 is the override's
# TS/PY discriminator (checked against the variable being assigned, so
# `TS_OUT_DIR="${QONTINUI_PY_OUT_DIR:-...}"` cannot slip through), capture 2 is
# the fallback suffix.
EXPECT_OUT_DIR_RHS_FORM='^"\$\{QONTINUI_(TS|PY)_OUT_DIR:-\$\{?SCHEMAS_DIR\}?/([A-Za-z0-9._/-]+)\}"$'

# Every path in generate_types.sh that reaches into THIS repo, keyed by the
# suffix after `$SCHEMAS_DIR/`.
#
# Anchored on that variable rather than on the `*_OUT_DIR` naming convention,
# because the convention is NOT one the generator follows — it also carries
# PER_TYPE_DIR, DISCR_SCRIPT and COMPILE_SCRIPT. A check that only enumerated
# `*_OUT_DIR` variables would wave through a brand-new output tree called
# RUST_GEN_DIR, which is exactly the vacuity being closed. Keyed this way, a
# new path into this repo is caught under ANY variable name, and so is either
# generated tree moving.
#
# Anchoring on `$SCHEMAS_DIR/` rather than on the literal directory name also
# means this keeps working when the generator is pointed at a checkout that is
# not called `qontinui-schemas` via QONTINUI_SCHEMAS_DIR.
EXPECT_SCHEMAS_PATHS=(
    "$TS_GENERATED_DIR"
    "$PY_GENERATED_DIR"
    "scripts/add_discriminators.py"
    "scripts/compile_typescript.mjs"
)

# Any binding of a tracked name, in ANY form.
#
# NOT `^NAME=`. `export NAME=`, `declare -g NAME=`, `readonly NAME=` and
# `NAME+=` are ordinary refactors that a prefix-anchored match does not see at
# all — so a later `export TS_OUT_DIR="$PROJECT_ROOT/elsewhere"` would move the
# output tree while the first, still-matching assignment kept this check
# happily reporting agreement. Captures: [2] keyword prefix (empty for a plain
# assignment), [5] name, [6] `+` for the append form, [7] RHS.
TRACKED_NAMES_RE='SCRIPT_DIR|PROJECT_ROOT|SCHEMAS_DIR|[A-Za-z_][A-Za-z0-9_]*_OUT_DIR'
TRACKED_BIND_RE="^((export|declare|typeset|readonly|local)[[:space:]]+((-[A-Za-z-]+)[[:space:]]+)*)?(${TRACKED_NAMES_RE})(\\+)?=(.*)\$"

usage() {
    echo "usage: $0 --preflight | --verify" >&2
    exit 2
}

[ "$#" -eq 1 ] || usage

# Running from a subdirectory does not error — `git ls-files -- ts/src/generated`
# from `ts/` simply matches nothing — so it would turn arm 1 into a no-op and
# make arm 1's `__pycache__` path-prefix filter meaningless. Assert instead of
# assuming.
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
    local -a parts=()
    IFS='/' read -r -a parts <<< "$path"
    for part in ${parts[@]+"${parts[@]}"}; do
        case "$part" in
            ''|.) ;;
            ..)   out="${out%/*}" ;;
            *)    out="$out/$part" ;;
        esac
    done
    printf '%s' "${out:-/}"
}

# Strip a CR (CRLF checkout), then leading and trailing whitespace.
#
# The CR strip matters because the shape compare below would otherwise fail on
# a generator checked out with CRLF endings. qontinui-runner pins
# `* text=auto eol=lf` in .gitattributes today, but this gate's correctness
# must not depend on a file in another repo.
#
# Result goes in the global $_T rather than stdout, and the same for
# _strip_trailing_comment below. `x="$(f "$line")"` forks a subshell per call,
# and these run once or twice for every line of the generator — ~500 forks,
# which costs milliseconds on a Linux runner but tens of seconds under MSYS,
# where this gate is also run by hand.
_trim_line() {
    _T="${1%$'\r'}"
    _T="${_T#"${_T%%[![:space:]]*}"}"
    _T="${_T%"${_T##*[![:space:]]}"}"
}

# Strip a trailing ` # comment`, and any whitespace it leaves behind.
#
# Requires whitespace before the `#`, which is what makes this safe to apply to
# a right-hand side: the path charset accepted by EXPECT_OUT_DIR_RHS_FORM has
# no spaces, so this cannot eat part of a real path.
_strip_trailing_comment() {
    _C="${1%%[[:space:]]#*}"
    _C="${_C%"${_C##*[![:space:]]}"}"
}

matches_any() { # matches_any <string> <regex>...
    local s="$1" re
    shift
    for re in "$@"; do
        [[ "$s" =~ $re ]] && return 0
    done
    return 1
}

# ── output-directory cross-check ────────────────────────────────────────────
#
# This gate inspects GENERATED_DIRS. generate_types.sh derives its own output
# dirs from its own location. Nothing but convention links the two, and
# asserting merely that the committed dirs EXIST does not test the link at all:
# if the generator's paths move, the committed dirs still exist, the existence
# test still passes, codegen writes somewhere unreviewed, and all three verify
# arms then pass vacuously — files still tracked (arm 0), nothing new untracked
# (arm 1), nothing changed (arm 2). That is precisely the failure class this
# file exists to eliminate.
#
# So resolve what the generator will ACTUALLY write and require it to equal
# what we inspect. Two independent assertions, because either alone has a gap:
#
#   1. Every path the generator aims INTO THIS REPO, by suffix, must be on
#      EXPECT_SCHEMAS_PATHS. Catches a new output tree under any variable name,
#      and either generated tree moving.
#   2. TS_OUT_DIR and PY_OUT_DIR must RESOLVE to the two dirs this gate
#      inspects. Catches the same tree being reached by a different route.
#
# We do not run the generator (that is a full cargo release build) and we do
# not source it (everything from its `cargo build` line down has side effects).
# We parse it, asserting SHAPE before evaluating anything, and every deviation
# is a hard failure carrying a "teach this check the new shape" message. There
# is deliberately no fallback path: a cross-check that can degrade to "assume
# it's fine" would reintroduce the vacuity it is supposed to close.
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

    # Resolve both roots up front: the schemas-path scan below needs to know
    # what THIS checkout is called.
    local gen_root repo_root rc_path=0
    gen_root="$(cd "$(dirname "$GENERATE_TYPES_SH")/.." && pwd -P)" || rc_path=$?
    repo_root="$(pwd -P)" || rc_path=$?
    if [ "$rc_path" -ne 0 ] || [ -z "$gen_root" ] || [ -z "$repo_root" ]; then
        echo "::error::could not resolve the generator's project root or this repo's root."
        echo "  gen_root='$gen_root' repo_root='$repo_root' (exit $rc_path)"
        return 1
    fi

    local line t name rhs rest seg
    local -a script_dir_rhs=() project_root_rhs=() schemas_dir_rhs=()
    local -a out_dir_names=() bad_bind_lines=() bad_out_dir_lines=() eval_lines=()
    local -a found_schemas_paths=()
    local ts_suffix="" py_suffix="" schemas_suffix=""

    while IFS= read -r line; do
        _trim_line "$line"; t="$_T"
        [ -n "$t" ] || continue

        # A whole-line comment is inert. Skipping it also keeps the
        # schemas-path scan below from tripping over prose mentioning a path.
        case "$t" in \#*) continue ;; esac

        # `eval` can bind a tracked name in a form no static pattern will ever
        # see. Refuse to reason about it rather than guessing.
        if [[ "$t" =~ ^eval([[:space:]]|$) ]] \
           && [[ "$t" =~ (SCRIPT_DIR|PROJECT_ROOT|_OUT_DIR) ]]; then
            eval_lines+=("$t")
        fi

        if [[ "$t" =~ $TRACKED_BIND_RE ]]; then
            name="${BASH_REMATCH[5]}"
            _strip_trailing_comment "${BASH_REMATCH[7]}"; rhs="$_C"
            if [ -n "${BASH_REMATCH[2]}" ] || [ -n "${BASH_REMATCH[6]}" ]; then
                # An `export`/`declare`/... prefix, or the `+=` append form.
                bad_bind_lines+=("$t")
            else
                case "$name" in
                    SCRIPT_DIR)   script_dir_rhs+=("$rhs") ;;
                    PROJECT_ROOT) project_root_rhs+=("$rhs") ;;
                    SCHEMAS_DIR)  schemas_dir_rhs+=("$rhs") ;;
                    *_OUT_DIR)
                        out_dir_names+=("$name")
                        # The override's TS/PY discriminator must match the
                        # variable it is assigned to, so a copy-paste that
                        # crosses them is caught rather than resolved.
                        if [[ "$rhs" =~ $EXPECT_OUT_DIR_RHS_FORM ]] \
                           && [ "${BASH_REMATCH[1]}_OUT_DIR" = "$name" ]; then
                            case "$name" in
                                TS_OUT_DIR) ts_suffix="${BASH_REMATCH[2]}" ;;
                                PY_OUT_DIR) py_suffix="${BASH_REMATCH[2]}" ;;
                            esac
                        else
                            bad_out_dir_lines+=("$t")
                        fi
                        ;;
                esac
            fi
        fi

        # Every reference into THIS repo, whatever variable carries it.
        #
        # `${SCHEMAS_DIR}` is folded to `$SCHEMAS_DIR` first so one marker
        # covers both spellings. The SCHEMAS_DIR assignment line itself never
        # matches, because its own RHS is expressed in terms of PROJECT_ROOT.
        rest="${t//\$\{SCHEMAS_DIR\}/\$SCHEMAS_DIR}"
        while [[ "$rest" == *'$SCHEMAS_DIR/'* ]]; do
            rest="${rest#*'$SCHEMAS_DIR/'}"
            seg="$rest"
            seg="${seg%%\"*}"
            seg="${seg%%\'*}"
            seg="${seg%%\}*}"
            seg="${seg%%[[:space:]]*}"
            seg="${seg%/}"
            [ -n "$seg" ] && found_schemas_paths+=("$seg")
        done
    done <<< "$src"

    local rc=0
    local shape_hint
    shape_hint="  Teach this cross-check the new shape in scripts/check-generated-drift.sh"
    shape_hint="$shape_hint"$'\n'"  — the EXPECT_* arrays near the top accept a SET of forms, so add the new"
    shape_hint="$shape_hint"$'\n'"  one and land it HERE FIRST: schema-drift.yml reads qontinui-runner at its"
    shape_hint="$shape_hint"$'\n'"  default branch, so a single change cannot be green on both sides at once."
    shape_hint="$shape_hint"$'\n'"  Do NOT delete the check: without it a moved output dir makes every arm"
    shape_hint="$shape_hint"$'\n'"  of --verify pass vacuously."

    if [ "${#eval_lines[@]}" -gt 0 ]; then
        echo "::error::generate_types.sh uses \`eval\` on a path variable this gate tracks:"
        printf '  %s\n' "${eval_lines[@]}"
        echo "  A tracked name bound through eval cannot be resolved statically, so"
        echo "  agreement cannot be proven. Use a plain assignment instead."
        rc=1
    fi

    if [ "${#bad_bind_lines[@]}" -gt 0 ]; then
        echo "::error::generate_types.sh binds a tracked path variable in an unsupported form:"
        printf '  %s\n' "${bad_bind_lines[@]}"
        echo "  \`export NAME=\`, \`declare NAME=\`, \`readonly NAME=\` and \`NAME+=\` can"
        echo "  REBIND a path after the assignment this gate resolved, moving codegen"
        echo "  output while the check still reports agreement. Use a single plain"
        echo "  assignment per variable."
        rc=1
    fi

    if [ "${#script_dir_rhs[@]}" -ne 1 ] \
       || ! matches_any "${script_dir_rhs[0]}" "${EXPECT_SCRIPT_DIR_RHS_FORMS[@]}"; then
        echo "::error::generate_types.sh derives SCRIPT_DIR in an unrecognized way."
        echo "  need exactly one plain assignment whose RHS matches an accepted form;"
        echo "  found ${#script_dir_rhs[@]}:"
        printf '    %s\n' ${script_dir_rhs[@]+"${script_dir_rhs[@]}"}
        echo "$shape_hint"
        rc=1
    fi

    if [ "${#project_root_rhs[@]}" -ne 1 ] \
       || ! matches_any "${project_root_rhs[0]}" "${EXPECT_PROJECT_ROOT_RHS_FORMS[@]}"; then
        echo "::error::generate_types.sh derives PROJECT_ROOT in an unrecognized way."
        echo "  need exactly one plain assignment whose RHS matches an accepted form;"
        echo "  found ${#project_root_rhs[@]}:"
        printf '    %s\n' ${project_root_rhs[@]+"${project_root_rhs[@]}"}
        echo "$shape_hint"
        rc=1
    fi

    if [ "${#schemas_dir_rhs[@]}" -ne 1 ] \
       || ! matches_any "${schemas_dir_rhs[0]}" "${EXPECT_SCHEMAS_DIR_RHS_FORMS[@]}"; then
        echo "::error::generate_types.sh derives SCHEMAS_DIR in an unrecognized way."
        echo "  need exactly one plain assignment whose RHS matches an accepted form;"
        echo "  found ${#schemas_dir_rhs[@]}:"
        printf '    %s\n' ${schemas_dir_rhs[@]+"${schemas_dir_rhs[@]}"}
        echo "$shape_hint"
        rc=1
    else
        [[ "${schemas_dir_rhs[0]}" =~ ${EXPECT_SCHEMAS_DIR_RHS_FORMS[0]} ]] \
            && schemas_suffix="${BASH_REMATCH[1]}"
    fi

    if [ "${#bad_out_dir_lines[@]}" -gt 0 ]; then
        echo "::error::generate_types.sh assigns an output dir in an unrecognized way."
        printf '  %s\n' "${bad_out_dir_lines[@]}"
        echo "  expected form: <NAME>_OUT_DIR=\"\$PROJECT_ROOT/<literal path>\""
        echo "$shape_hint"
        rc=1
    fi

    # Exactly the two output-dir variables this gate resolves — no more, no
    # fewer. (The stronger "no NEW tree under any variable name" property is
    # the schemas-path assertion below; this one keeps the resolve step total.)
    local names_seen rc_names=0
    names_seen="$(printf '%s\n' ${out_dir_names[@]+"${out_dir_names[@]}"} \
        | LC_ALL=C sort -u | tr '\n' ' ')" || rc_names=$?
    if [ "$rc_names" -ne 0 ]; then
        echo "::error::could not collate generate_types.sh's *_OUT_DIR names (exit $rc_names)."
        rc=1
    fi
    names_seen="${names_seen% }"
    if [ "$names_seen" != "PY_OUT_DIR TS_OUT_DIR" ]; then
        echo "::error::generate_types.sh's set of *_OUT_DIR variables is not the set this gate resolves."
        echo "  expected: PY_OUT_DIR TS_OUT_DIR"
        echo "  found:    ${names_seen:-<none>}"
        echo "  An output tree this gate never resolves is drift it would never see."
        echo "  Add it to GENERATED_DIRS and commit its baseline, or remove it."
        rc=1
    fi

    # Assertion 1 — every path the generator aims into this repo is known.
    #
    # Guard the EXTRACTOR before comparing its output. An extraction that
    # understands nothing yields the empty set, and the empty set compares
    # equal to an empty expectation — so a parser that silently stops matching
    # would pass vacuously. That is the "absence reads as OK" family this whole
    # file exists to eliminate, and it is not hypothetical: the first cut of
    # this check keyed on the literal repo directory name, the generator moved
    # to `$SCHEMAS_DIR/`, and the extractor went to zero matches. It failed
    # closed then only because the expectation happened to be non-empty.
    # The generator unavoidably references this repo, so zero is always a bug
    # in the extractor, never a true observation about the generator.
    if [ "${#found_schemas_paths[@]}" -eq 0 ]; then
        echo "::error::found NO references into this repo in generate_types.sh."
        echo "  It must have some — it writes the bindings this gate inspects — so"
        echo "  this is the EXTRACTOR failing to understand the script's shape, not"
        echo "  the generator having none. Reported separately from a set mismatch"
        echo "  because an empty extraction would otherwise compare equal to an"
        echo "  empty expectation and pass vacuously."
        echo "  The extractor keys on \`\$SCHEMAS_DIR/\` and \`\${SCHEMAS_DIR}/\`."
        echo "$shape_hint"
        rc=1
    fi

    local want got p rc_sets=0
    want="$(printf '%s\n' "${EXPECT_SCHEMAS_PATHS[@]}" | LC_ALL=C sort -u)" || rc_sets=$?
    got="$(printf '%s\n' ${found_schemas_paths[@]+"${found_schemas_paths[@]}"} \
        | LC_ALL=C sort -u)" || rc_sets=$?
    if [ "$rc_sets" -ne 0 ]; then
        echo "::error::could not collate generate_types.sh's paths into this repo (exit $rc_sets)."
        rc=1
    elif [ "$want" = "$got" ]; then
        # Echo the extraction on success too. A silent pass cannot be
        # distinguished from a pass over nothing when reading a CI log.
        echo "cross-check: ${#found_schemas_paths[@]} reference(s) into this repo, all known:"
        while IFS= read -r p; do
            [ -n "$p" ] && echo "cross-check:   $p"
        done <<< "$got"
    else
        echo "::error::generate_types.sh's set of paths into this repo changed."
        echo "  expected:"
        printf '    %s\n' "${EXPECT_SCHEMAS_PATHS[@]}"
        echo "  found:"
        while IFS= read -r p; do
            [ -n "$p" ] && echo "    $p"
        done <<< "$got"
        echo "  A path into this repo that this gate does not know about is either a"
        echo "  new generated tree no arm inspects, or an existing tree that moved."
        echo "  Update EXPECT_SCHEMAS_PATHS (and GENERATED_DIRS if it is an output"
        echo "  tree, committing its baseline)."
        echo "$shape_hint"
        rc=1
    fi

    [ "$rc" -eq 0 ] || return "$rc"

    # Assertion 2 — shape proven; now evaluate. SCRIPT_DIR/PROJECT_ROOT matched
    # an accepted form above, so the generator's PROJECT_ROOT is exactly
    # dirname(script)/.. — which is what licenses resolving the suffixes
    # against gen_root.
    # The generator's three QONTINUI_* overrides are part of its contract, so
    # resolve the SAME way it will: override if set, documented default if not.
    # Reading them here is not merely cosmetic — if a workflow ever sets one at
    # job level, the static default is NOT what codegen would use, and a
    # cross-check that ignored them would prove the wrong thing.
    #
    # Residual gap, stated plainly: an override set only on the regen STEP is
    # invisible to this step, so what is proven is "the paths that apply in
    # THIS environment", not "in every environment".
    local schemas_root
    if [ -n "${QONTINUI_SCHEMAS_DIR:-}" ]; then
        schemas_root="$QONTINUI_SCHEMAS_DIR"
        echo "cross-check: QONTINUI_SCHEMAS_DIR override in effect -> $schemas_root"
    else
        schemas_root="$gen_root/$schemas_suffix"
    fi
    case "$schemas_root" in /*) ;; *) schemas_root="$repo_root/$schemas_root" ;; esac

    local var suffix gate_dir writes inspects override
    for var in TS PY; do
        if [ "$var" = TS ]; then
            suffix="$ts_suffix"; gate_dir="$TS_GENERATED_DIR"
            override="${QONTINUI_TS_OUT_DIR:-}"
        else
            suffix="$py_suffix"; gate_dir="$PY_GENERATED_DIR"
            override="${QONTINUI_PY_OUT_DIR:-}"
        fi
        if [ -n "$override" ]; then
            echo "cross-check: QONTINUI_${var}_OUT_DIR override in effect"
            case "$override" in /*) ;; *) override="$repo_root/$override" ;; esac
            writes="$(lexical_normalize "$override")"
        else
            writes="$(lexical_normalize "$schemas_root/$suffix")"
        fi
        inspects="$(lexical_normalize "$repo_root/$gate_dir")"
        if [ "$writes" = "$inspects" ]; then
            echo "cross-check: ${var}_OUT_DIR agrees -> $inspects"
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
# It ALSO re-runs the output-dir cross-check. That is not redundant with
# --preflight: the arms are what post the required check status, and a gate
# whose anti-vacuity property lives in a SEPARATE workflow step is exactly the
# "a comment asking humans to keep them in step is not a mechanism" problem
# this file was extracted to solve — one caller could simply omit the preflight
# step and every arm below would go vacuous with nothing to notice. Run alone,
# `--verify` would otherwise exit 0 with the generator's output location
# entirely unknown. It costs one `cat`.
#
# Everything runs before the function returns, so one invocation reports every
# problem rather than making you fix them one CI round-trip at a time.
verify() {
    local rc=0

    assert_repo_root || rc=1
    cross_check_output_dirs || rc=1

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
    # Both substitutions below have their exit status checked EXPLICITLY rather
    # than left to `set -e`. Arm 1 silently reporting clean is precisely the bug
    # class this file exists to prevent, and a swallowed failure here is
    # indistinguishable from "no untracked files".
    local raw new_files pycache_re rc_ls=0 rc_grep=0
    set +e
    raw="$(git ls-files --others -- "${GENERATED_DIRS[@]}")"
    rc_ls=$?
    set -e
    if [ "$rc_ls" -ne 0 ]; then
        echo "::error::arm 1 could not list untracked files (git ls-files exit $rc_ls)."
        echo "  Arm 1 cannot be trusted, so this fails rather than reporting clean."
        rc=1
    fi
    # SCOPED to the Python tree. An unanchored `/__pycache__/` filter drops the
    # segment wherever it appears, so `ts/src/generated/__pycache__/Hidden.d.ts`
    # — a path with no legitimate reason to exist — would be silently excused.
    pycache_re="^${PY_GENERATED_DIR}/(.*/)?__pycache__/"
    # grep exits 1 on no-match (the normal healthy case) and 2 on a real error
    # such as an invalid regex. A bare `|| true` would swallow BOTH, leaving
    # new_files empty and this arm reporting clean while drift exists. The
    # comment justifying `|| true` here used to cite only the no-match exit,
    # which is the narrower half of what it actually suppressed.
    #
    # `printf '%s'` without a trailing newline is load-bearing: with `\n`, an
    # empty `raw` becomes one blank line, which `grep -Ev` happily prints, so
    # new_files would be non-empty and every clean run would red.
    set +e
    new_files="$(printf '%s' "$raw" | grep -Ev "$pycache_re")"
    rc_grep=$?
    set -e
    if [ "$rc_grep" -gt 1 ]; then
        echo "::error::arm 1's __pycache__ filter failed (grep exit $rc_grep)."
        echo "  Arm 1 cannot be trusted, so this fails rather than reporting clean."
        rc=1
    fi
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
    # `-- Dumped by pg_dump version` header noise. Verified still to suppress
    # exactly that, and nothing more, under `HEAD`.
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
