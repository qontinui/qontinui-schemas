/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from "./RetrySpec";
import type { VerificationCategoryKind } from "./VerificationCategoryKind";
import type { VgaAction } from "./VgaAction";

/**
 * Run a Visual GUI Automation (VGA) action sequence against a target process.
 *
 * Wire tag: `"vga_automate"`.
 *
 * Delegates to the Python `qontinui.vga.worker` via the `python-bridge` IPC.
 * The worker loads the referenced state machine (`state_machine_id` →
 * `runner.vga_state_machines`), focuses `target_process`, and executes each
 * entry in `action_sequence` by grounding the element prompt against a fresh
 * screenshot and dispatching the HAL click/type/wait primitive.
 *
 * The VGA state machine reference serializes as `stateMachineId`. Two
 * aliases are accepted: `state_machine_id`, and `vga_state_machine_id` —
 * the runner's `ExecutionStepConfig` field name, which is what every
 * runner-built step carries. The other runner-side `vga_*` names
 * (`vga_target_process`, `vga_action_sequence`, `vga_timeout_ms`,
 * `vga_async`) are accepted for the same reason.
 */
export interface VgaAutomateStep {
  /**
   * Ordered sequence of VGA actions to execute. Absent or `null` reads as
   * empty.
   */
  actionSequence?: VgaAction[];
  /**
   * Reserved for future async mode. Currently must be `false` (or omitted)
   * — the handler rejects `true` until async mode is implemented.
   */
  async?: boolean | null;
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterionIds?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  dependsOn?: string[];
  /**
   * Extractions published to subsequent steps.
   */
  extract?: {
    [k: string]: string;
  };
  /**
   * If `Some(true)`, a console-error signal from the UI fails this step.
   */
  failOnConsoleErrors?: boolean | null;
  /**
   * Unique identifier for the step. Absent or `null` reads as `""`.
   */
  id: string;
  /**
   * Named input bindings evaluated at step entry.
   */
  inputs?: {
    [k: string]: string;
  };
  /**
   * Display name for the step. Absent or `null` reads as `""`.
   */
  name: string;
  /**
   * Whether this step is required (default: `true` on consumer side).
   */
  required?: boolean | null;
  /**
   * Per-step retry configuration.
   */
  retry?: RetrySpec | null;
  /**
   * Provenance of this step when generated from a skill template.
   *
   * Typed as `serde_json::Value` here to avoid pulling the `skill`
   * dependency chain into this module; the TS side re-imports the typed
   * `SkillOrigin` after regeneration.
   */
  skillOrigin?: {
    [k: string]: unknown;
  };
  /**
   * UUID referencing `runner.vga_state_machines.id` — the persisted state
   * machine that defines the elements this step may click / type into /
   * wait for.
   *
   * `vga_state_machine_id` is the runner's `ExecutionStepConfig` field
   * name — the key every runner-built `vga_automate` step serializes.
   */
  stateMachineId: string;
  /**
   * Target process / window — e.g. `"notepad++.exe"`. Used by the HAL to
   * focus the correct top-level window before each action.
   */
  targetProcess: string;
  /**
   * Overall step timeout in milliseconds. Defaults to `300000` (5 minutes)
   * on the consumer side; bounds `[1000, 3600000]`.
   */
  timeoutMs?: number | null;
  /**
   * Verification depth category.
   */
  verificationCategory?: VerificationCategoryKind | null;
  [k: string]: unknown;
}
