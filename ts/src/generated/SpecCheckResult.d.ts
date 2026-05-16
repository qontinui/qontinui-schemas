/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionMiss } from './AssertionMiss';
import type { AssertionOutcome } from './AssertionOutcome';
import type { AssertionResult } from './AssertionResult';
import type { AssertionSeverityCounts } from './AssertionSeverityCounts';
import type { BridgeFingerprint } from './BridgeFingerprint';
import type { CandidateMiss } from './CandidateMiss';
import type { FieldDiff } from './FieldDiff';
import type { MatchOutcome } from './MatchOutcome';
import type { MatchedElement } from './MatchedElement';
import type { MissReason } from './MissReason';
import type { RecommendedState } from './RecommendedState';
import type { SpecCheckConfidence } from './SpecCheckConfidence';
import type { SpecCheckSummary } from './SpecCheckSummary';
import type { StateMatchResult } from './StateMatchResult';

/**
 * Result of evaluating one or more page specs against a bridge snapshot.
 *
 * `result_schema_version` is the FIRST field per §5.15 — read-time
 * forward migration uses it to route legacy JSONB through
 * `result_migration`. Pre-versioned rows (written before v1 shipped)
 * deserialize as `0`.
 */
export interface SpecCheckResult {
  bridgeFingerprint: BridgeFingerprint;
  /**
   * ISO-8601 UTC. When the spec was hashed and the snapshot indexed.
   */
  evaluatedAt: string;
  /**
   * Page being evaluated.
   */
  pageId: string;
  /**
   * v1 = 1. Missing on the wire (pre-versioned persisted rows) → 0.
   */
  resultSchemaVersion: number;
  /**
   * Caller-minted snapshot ID. Format: `"scs_" + ULID()`. See §5.8.
   */
  snapshotId: string;
  /**
   * Hash of the spec IR document at evaluation time. `"sha256-<hex>"`.
   */
  specContentHash: string;
  /**
   * IR document `version` field (currently `"1.0"`).
   */
  specVersion: string;
  /**
   * One result per `IrState` in the spec.
   */
  stateResults: StateMatchResult[];
  summary: SpecCheckSummary;
  /**
   * Soft signals — currently used for `Stale` from `SnapshotFetchError`.
   */
  warnings?: string[];
}
