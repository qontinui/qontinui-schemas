/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Parser type for extracting errors from a managed process's log stream.
 *
 * Mirrors the pre-extraction `crate::error_monitor::types::ParserType` wire
 * format. Note the JavaScript rename: the canonical wire value is
 * `"javascript"` but the serde `alias = "java_script"` is preserved for
 * backwards compatibility with any settings that were written before the
 * rename.
 */
export type ParserType = "python" | "javascript" | "rust" | "generic";
