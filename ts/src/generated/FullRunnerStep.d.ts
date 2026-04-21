/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { A11yAction } from './A11yAction';
import type { CheckType } from './CheckType';
import type { CodeExecutionStep } from './CodeExecutionStep';
import type { CommandMode } from './CommandMode';
import type { CommandStep } from './CommandStep';
import type { CommandStepPhase } from './CommandStepPhase';
import type { DagApprovalStep } from './DagApprovalStep';
import type { DagCancelStep } from './DagCancelStep';
import type { DagLoopStep } from './DagLoopStep';
import type { ExecutePlaybookStep } from './ExecutePlaybookStep';
import type { NativeAccessibilityStep } from './NativeAccessibilityStep';
import type { PlaywrightExecutionMode } from './PlaywrightExecutionMode';
import type { PromptStep } from './PromptStep';
import type { PromptStepPhase } from './PromptStepPhase';
import type { RestartProcessStep } from './RestartProcessStep';
import type { RetrySpec } from './RetrySpec';
import type { SaveWorkflowArtifactStep } from './SaveWorkflowArtifactStep';
import type { TestType } from './TestType';
import type { UiBridgeAction } from './UiBridgeAction';
import type { UiBridgeAssertType } from './UiBridgeAssertType';
import type { UiBridgeComparisonMode } from './UiBridgeComparisonMode';
import type { UiBridgeDesignAuditStep } from './UiBridgeDesignAuditStep';
import type { UiBridgeSeverity } from './UiBridgeSeverity';
import type { UiBridgeStep } from './UiBridgeStep';
import type { UiBridgeStepPhase } from './UiBridgeStepPhase';
import type { UiBridgeVisualAssertionStep } from './UiBridgeVisualAssertionStep';
import type { VerificationCategoryKind } from './VerificationCategoryKind';
import type { VgaAction } from './VgaAction';
import type { VgaAutomateStep } from './VgaAutomateStep';
import type { VisualAssertionType } from './VisualAssertionType';
import type { WorkflowFixupMode } from './WorkflowFixupMode';
import type { WorkflowFixupStep } from './WorkflowFixupStep';
import type { WorkflowRefStep } from './WorkflowRefStep';
import type { WorkflowStep } from './WorkflowStep';
import type { WorkflowStepPhase } from './WorkflowStepPhase';

/**
 * Fully typed discriminated union over **all** step variants registered in
 * the runner's `HandlerRegistry`.
 *
 * ## Wire format
 *
 * Internally tagged with `"type"`, matching existing JSON on the wire:
 * ```json
 * {"type": "command", "mode": "shell", "command": "cargo build", ...}
 * {"type": "prompt", "phase": "agentic", "content": "..."}
 * {"type": "ui_bridge", "action": "navigate", "url": "..."}
 * {"type": "code_execution", "code": "print('hello')"}
 * ```
 *
 * ## Step arrays remain `Vec<serde_json::Value>`
 *
 * `UnifiedWorkflow.setup_steps` / `.verification_steps` / `.agentic_steps` /
 * `.completion_steps` stay as `Vec<serde_json::Value>` until the Session 2
 * migration lands. `FullRunnerStep` is available for typed access but is not
 * yet threaded into the workflow frame fields.
 *
 * ## Variant coverage
 *
 * | Variant | Wire tag | Handler |
 * |---------|----------|---------|
 * | `Command` | `"command"` | `CommandHandler` (sub-modes: shell/check/check_group/test) |
 * | `Prompt` | `"prompt"` | `PromptStepHandler` |
 * | `UiBridge` | `"ui_bridge"` | `UiBridgeHandler` (actions: navigate/execute/assert/snapshot/compare/snapshot_assert/action_plan) |
 * | `Workflow` | `"workflow"` | `WorkflowStepHandler` |
 * | `CodeExecution` | `"code_execution"` | `CodeExecutionHandler` |
 * | `ExecutePlaybook` | `"execute_playbook"` | `ExecutePlaybookHandler` |
 * | `NativeAccessibility` | `"native_accessibility"` | `NativeAccessibilityHandler` |
 * | `RestartProcess` | `"restart_process"` | `RestartProcessHandler` |
 * | `SaveWorkflowArtifact` | `"save_workflow_artifact"` | `SaveWorkflowArtifactHandler` |
 * | `WorkflowFixup` | `"workflow_fixup"` | `WorkflowFixupHandler` |
 * | `UiBridgeDesignAudit` | `"ui_bridge_design_audit"` | `UiBridgeDesignAuditHandler` |
 * | `UiBridgeVisualAssertion` | `"ui_bridge_visual_assertion"` | `UiBridgeVisualAssertionHandler` |
 * | `VgaAutomate` | `"vga_automate"` | `VgaAutomateHandler` |
 * | `WorkflowRef` | `"workflow_ref"` | `WorkflowRefHandler` |
 * | `DagCancel` | `"dag_cancel"` | `dag_nodes::DagCancelHandler` |
 * | `DagApproval` | `"dag_approval"` | `dag_nodes::DagApprovalHandler` |
 * | `DagLoop` | `"dag_loop"` | `dag_nodes::DagLoopHandler` |
 *
 * Variant sizes range ~200–672 bytes depending on each step struct's field
 * cardinality. `#[allow(large_enum_variant)]` because the sizes reflect real
 * step shapes; boxing would add heap indirection on every deserialize and
 * dispatch without meaningful savings — no hot path holds dense
 * `Vec<FullRunnerStep>` in memory.
 */
export type FullRunnerStep =
  | (CommandStep & {
      type: "command";
    })
  | (PromptStep & {
      type: "prompt";
    })
  | (UiBridgeStep & {
      type: "ui_bridge";
    })
  | (WorkflowStep & {
      type: "workflow";
    })
  | (CodeExecutionStep & {
      type: "code_execution";
    })
  | (ExecutePlaybookStep & {
      type: "execute_playbook";
    })
  | (NativeAccessibilityStep & {
      type: "native_accessibility";
    })
  | (RestartProcessStep & {
      type: "restart_process";
    })
  | (SaveWorkflowArtifactStep & {
      type: "save_workflow_artifact";
    })
  | (WorkflowFixupStep & {
      type: "workflow_fixup";
    })
  | (UiBridgeDesignAuditStep & {
      type: "ui_bridge_design_audit";
    })
  | (UiBridgeVisualAssertionStep & {
      type: "ui_bridge_visual_assertion";
    })
  | (VgaAutomateStep & {
      type: "vga_automate";
    })
  | (WorkflowRefStep & {
      type: "workflow_ref";
    })
  | (DagCancelStep & {
      type: "dag_cancel";
    })
  | (DagApprovalStep & {
      type: "dag_approval";
    })
  | (DagLoopStep & {
      type: "dag_loop";
    });
