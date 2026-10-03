/**
 * Skill Types
 *
 * A skill is a named, parameterized template that produces pre-configured
 * workflow step(s) when instantiated. Skills sit between raw step types
 * and full workflows:
 *
 *   Raw Step Types  (command, prompt, ui_bridge, workflow)  ← execution primitives
 *        ↑ instantiates
 *   Skills          ("Lint Project", "API Health Check")    ← named capability templates
 *        ↑ composes into
 *   Workflows       (multi-phase verification-agentic loops) ← orchestration
 *
 * Skills are purely a configuration-time abstraction — they produce steps,
 * they do NOT add new runtime behavior.
 */

// Every type here is generated from the runner's `src-tauri/src/skill_types.rs`
// (the skill registry's wire); do not edit by hand — regenerate via
// `qontinui-runner/src-tauri/scripts/generate_types.sh`.
//
// The vocabularies (`SkillCategory`, `SkillParameterType`, `SkillAllowedPhase`,
// `SkillSource`, `SkillApprovalStatus`, `SkillExportContentType`) are
// schema-only enums there, exported here so consumers can narrow against
// them. Only manifest `content_type` is CLOSED on the wire. `category`,
// parameter `type`, `allowed_phases`, `source` and `approval_status` are
// `Vocab | string` — which TypeScript widens to `string` — because the runner
// reads them back from frontmatter, user rows and imported payloads without
// validating them, so the binding says what the wire can carry.

import type { SkillAllowedPhase } from "../generated/SkillAllowedPhase";
import type { SkillTemplate } from "../generated/SkillTemplate";
import type { WorkflowPhase } from "./_api";

export type { SkillCategory } from "../generated/SkillCategory";
export type { SkillAllowedPhase };
export type { SkillParameterType } from "../generated/SkillParameterType";
export type { SkillSource } from "../generated/SkillSource";
export type { SkillApprovalStatus } from "../generated/SkillApprovalStatus";
export type { SkillExportContentType } from "../generated/SkillExportContentType";
export type { SkillParameterDependency } from "../generated/SkillParameterDependency";
export type { SkillPlaybookTrigger } from "../generated/SkillPlaybookTrigger";
export type { SkillAuthor } from "../generated/SkillAuthor";
export type { SkillParameterOption } from "../generated/SkillParameterOption";
export type { SkillParameter } from "../generated/SkillParameter";
export type { SkillRef } from "../generated/SkillRef";
export type { SkillTemplate };
export type { SkillDefinition } from "../generated/SkillDefinition";
export type { SkillOrigin } from "../generated/SkillOrigin";
export type { SkillExportManifest } from "../generated/SkillExportManifest";
export type { SkillExport } from "../generated/SkillExport";
export type { SkillImportResult } from "../generated/SkillImportResult";

// Per-variant names for the `kind`-tagged template union.
export type SingleStepTemplate = Extract<SkillTemplate, { kind: "single_step" }>;
export type MultiStepTemplate = Extract<SkillTemplate, { kind: "multi_step" }>;
export type CompositionTemplate = Extract<SkillTemplate, { kind: "composition" }>;
/** Markdown playbook with domain knowledge injected into AI prompts. */
export type PlaybookTemplate = Extract<SkillTemplate, { kind: "playbook" }>;

// `SkillAllowedPhase` (the runner's schema-only vocabulary for
// `allowed_phases`) must name exactly the workflow phases. Compile-time only:
// adding a phase on either side without the other fails `tsc` here. Unused
// on purpose — instantiating `Assert` is the check.
type Equal<A, B> =
  (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2 ? true : false;
type Assert<T extends true> = T;
type SkillAllowedPhaseIsWorkflowPhase = Assert<Equal<SkillAllowedPhase, WorkflowPhase>>;
