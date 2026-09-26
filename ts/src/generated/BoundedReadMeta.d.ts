/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BoundKind } from "./BoundKind";
import type { FilterNarrowing } from "./FilterNarrowing";

/**
 * The envelope keys every bounded read serves beside its own collection key
 * (plan `2026-09-05-every-bounded-read-is-a-page-that-reads-as-a-corpus`,
 * §3 Layer 2).
 *
 * **Every key is ALWAYS present, and `null` is a value here, not an
 * absence.** This deliberately departs from the crate's
 * `skip_serializing_if = "Option::is_none"` convention: `total: null` says "no
 * count ran", `truncated: null` says "unknown", and `next_cursor: null` says
 * "there is no next page" — each a positive statement a reader must be able
 * to see, not a key it might be missing because of an older server.
 *
 * Enforced on all three sides, because an absent key that silently reads as
 * `null` is exactly the "absent is UNKNOWN, never a default" defect:
 * serialization never skips a field; every `Option` field carries
 * `#[schemars(required, schema_with = "nullable::<T>")]` — `required` because
 * schemars 1 otherwise omits an `Option` from `required` (the generated
 * Pydantic model would then default a missing key to `None`), and
 * `schema_with` because `required` alone makes schemars emit the INNER type's
 * schema, dropping `null` so generated models reject the value this wire
 * sends; and deserialization uses `Option::deserialize`, which refuses a
 * MISSING key instead of defaulting it (serde's derive otherwise treats an
 * absent `Option` as `None`). Pinning tests assert, structurally, that
 * `required` is exactly the field set and that every nullable key admits
 * `null`.
 *
 * `count` and `limit` keep the spellings list doors already serve (coord
 * #1897 put `count` on every agent list door; web proxies and runner pollers
 * read both), so merging this into an existing response rewrites them with
 * the same values rather than moving them.
 */
export interface BoundedReadMeta {
  /**
   * `false` ONLY when the store is unprovisioned: an empty page with
   * `available: false` is UNKNOWN, not "nothing matched".
   */
  available: boolean;
  /**
   * Which kind of bound produced `total` / `truncated`.
   */
  bound_kind: BoundKind;
  /**
   * Rows in this page (the legacy spelling of `shown`).
   */
  count: number;
  /**
   * For a relevance-RANKED read, which can never hand out a cursor: the
   * door that enumerates the same corpus by an immutable sort key (plan
   * `2026-09-05-every-bounded-read-is-a-page-that-reads-as-a-corpus` D8), so
   * a `truncated: true, next_cursor: null` answer still says how to reach
   * the rest. `null` on a keyset walk, whose `next_cursor` is the way on.
   */
  enumerate_via: string | null;
  /**
   * A FILTER the surface narrowed before the read ran, or `null`.
   */
  filter_narrowed: FilterNarrowing | null;
  /**
   * The cap actually applied to this page.
   */
  limit: number;
  /**
   * The opaque token for the next page — pass it back verbatim as
   * `cursor`. `null` iff `truncated` is not `true`, OR the read is a
   * RANKING that cannot be paged — then `truncated` may be `true` with
   * `next_cursor: null`, and `enumerate_via` names the door that walks the
   * corpus by an immutable key.
   */
  next_cursor: string | null;
  /**
   * Rows in this page.
   */
  shown: number;
  /**
   * The exact match count from this page's start position — a number only
   * when `bound_kind` is `exact`, else `null`.
   */
  total: number | null;
  /**
   * Whether matching rows exist beyond this page; `null` when
   * `bound_kind` is `unknown`.
   */
  truncated: boolean | null;
  [k: string]: unknown;
}
