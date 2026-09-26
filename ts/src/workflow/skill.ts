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
// `qontinui-runner/src-tauri/scripts/generate_types.sh`. Closed vocabularies
// (`SkillCategory`, parameter `type`, `allowed_phases`, `source`,
// `approval_status`, manifest `content_type`) are schema-only enums there:
// the Rust fields stay lenient strings on the way in, the schema names what
// the runner produces.

import type { SkillTemplate } from "../generated/SkillTemplate";

export type { SkillCategory } from "../generated/SkillCategory";
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
