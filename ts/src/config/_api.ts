/**
 * Config Types — AI context + workflow category
 *
 * Source of truth: qontinui-schemas/rust/src/config.rs.
 *
 * Wire convention: these types predate the snake_case migration and use
 * camelCase field aliases on the wire (`taskMentions`, `autoInclude`,
 * `createdAt`, `modifiedAt`, `automationEnabled`). Consumers see camelCase.
 */

export type { Context } from "../generated/Context";
export type { ContextAutoInclude } from "../generated/ContextAutoInclude";
export type { Category } from "../generated/Category";
