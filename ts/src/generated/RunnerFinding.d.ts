/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunnerFindingCodeContext } from './RunnerFindingCodeContext';
import type { RunnerFindingUserInput } from './RunnerFindingUserInput';
import type { TaskRunFindingActionType } from './TaskRunFindingActionType';
import type { TaskRunFindingCategory } from './TaskRunFindingCategory';
import type { TaskRunFindingSeverity } from './TaskRunFindingSeverity';
import type { TaskRunFindingStatus } from './TaskRunFindingStatus';

/**
 * A finding detected by AI analysis (runner-local wire shape).
 *
 * Wire format: serialized via `#[serde(rename_all = "camelCase")]` so all
 * snake_case Rust field names ship as camelCase on the Tauri event channels
 * `finding_detected` and `finding_resolved`. The frontend listener in
 * `services/TauriFindingsListener.ts` MUST read these fields by their
 * camelCase names — reading snake_case silently evaluates to `undefined`.
 *
 * Renamed in the schema registry to `RunnerFinding` to disambiguate from
 * `qontinui_types::verification::Finding`, which has a different shape
 * (`confidence`, `findingType`, `evidence` vs this struct's flat fields).
 */
export interface RunnerFinding {
  actionType: TaskRunFindingActionType;
  categoryId: TaskRunFindingCategory;
  codeContext?: RunnerFindingCodeContext | null;
  description: string;
  detectedAt: string;
  id: string;
  resolution?: string | null;
  resolvedAt?: string | null;
  resolvedInSession?: number | null;
  sessionNum: number;
  severity: TaskRunFindingSeverity;
  signatureHash: string;
  status: TaskRunFindingStatus;
  taskRunId: string;
  title: string;
  updatedAt: string;
  userInput?: RunnerFindingUserInput | null;
  userResponse?: string | null;
  [k: string]: unknown;
}
