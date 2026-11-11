# ci-exec-lib-test-binary.awk: print the path of the ci-exec lib unit-test
# binary that a `cargo test` run reported, read from that run's log, or an
# empty line when the log names none.
#
#   awk -f .github/scripts/ci-exec-lib-test-binary.awk <cargo test log>
#
# The answer is the path inside the log's
# `Running unittests src/lib.rs (<dir>/qontinui_ci_exec-<hash>)` line. Both
# legs of rust-ci.yml's "Provisioning env e2e" (the provisioning-env-e2e-linux
# and ci-exec-windows jobs) call this to run the binary their `cargo test` step
# already built, instead of invoking cargo a second time.
#
# * Colour codes are stripped first: dtolnay/rust-toolchain exports
#   CARGO_TERM_COLOR=always.
# * Backslashes become slashes, so one pattern serves the Windows log
#   (`src\lib.rs (target\debug\deps\qontinui_ci_exec-<hash>.exe)`) too.
# * A trailing CR never reaches the answer: the match ends at the closing
#   parenthesis.
# * The whole file is read and the answer printed at END, so nothing upstream
#   of a pipe into this script is ever cut short.
{ gsub(/\033\[[0-9;]*[a-zA-Z]/, ""); gsub(/\\/, "/") }
bin == "" && match($0, /Running unittests src\/lib\.rs \([^)]*\/qontinui_ci_exec-[0-9a-f]+(\.exe)?\)/) {
  bin = substr($0, RSTART, RLENGTH)
  sub(/^Running unittests src\/lib\.rs \(/, "", bin)
  sub(/\)$/, "", bin)
}
END { print bin }
