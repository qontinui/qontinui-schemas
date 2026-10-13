#!/usr/bin/env bash
# Test fixture: a STUB of qontinui-runner's src-tauri/scripts/generate_types.sh.
#
# tests/test_check_generated_drift_hook.py points GENERATE_TYPES_SH here so that
# `check-generated-drift.sh --verify` can run end to end with no runner checkout.
# The drift gate never EXECUTES the generator in --verify; it only PARSES these
# assignments (cross_check_output_dirs) to prove the output dirs it inspects are
# the generator's. The forms below are the real generator's, so the cross-check
# passes and a red can only come from the arm under test.
# Running this stub does nothing.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCHEMAS_DIR="${QONTINUI_SCHEMAS_DIR:-$PROJECT_ROOT/../../qontinui-schemas}"
TS_OUT_DIR="${QONTINUI_TS_OUT_DIR:-$SCHEMAS_DIR/ts/src/generated}"
PY_OUT_DIR="${QONTINUI_PY_OUT_DIR:-$SCHEMAS_DIR/src/qontinui_schemas/generated}"
DISCR_SCRIPT="$SCHEMAS_DIR/scripts/add_discriminators.py"
COMPILE_SCRIPT="${QONTINUI_TS_COMPILE_SCRIPT:-$SCHEMAS_DIR/scripts/compile_typescript.mjs}"
exit 0
