/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What went wrong, as a stable machine-readable code.
 *
 * Extend by adding a variant (and its row in [`RefusalCode::headline`] and
 * [`RefusalCode::from_wire`]); the tables are exhaustive `match`es, so a new
 * code without a sentence is a compile error.
 */
export type RefusalCode =
  | "workspace_root_unresolved"
  | "sibling_checkout_absent"
  | "endpoint_unresolved"
  | "glossary_term_unknown"
  | "authentication_required"
  | "credential_rejected"
  | "permission_denied"
  | "not_found"
  | "conflict"
  | "invalid_request"
  | "rate_limited"
  | "quota_exceeded"
  | "upstream_unavailable"
  | "upstream_timeout"
  | "device_not_connected"
  | "service_unavailable"
  | "internal_error"
  | "unknown";
