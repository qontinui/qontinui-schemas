/**
 * The next-action contract — `@qontinui/shared-types/refusal`.
 *
 * Plan `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
 * Phase D1. Types only: the envelope is generated from the Rust
 * `qontinui_types::refusal` module, whose `Refusal::render()` is the
 * canonical sentence. Readers keep a `default` arm when switching on `code`,
 * `next_action.kind` or `source` — a newer producer sends values this union
 * does not list.
 */

export type { GlossaryTerm } from "../generated/GlossaryTerm";
export type { NextAction } from "../generated/NextAction";
export type { NextActionKind } from "../generated/NextActionKind";
export type { Refusal } from "../generated/Refusal";
export type { RefusalCode } from "../generated/RefusalCode";
export type { RefusalSource } from "../generated/RefusalSource";
