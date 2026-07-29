/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A canvas panel rendered in the dashboard widget.
 *
 * Wire-format mirror of the runner's `StoredPanel` struct
 * (`qontinui-runner/src-tauri/src/mcp/canvas.rs`). The runner emits this
 * inside [`AppEvent::CanvasUpdate`] via the `canvas-update` Tauri channel.
 * Field names are snake_case to match the existing TS interface in
 * `qontinui-schemas/ts/src/canvas/index.ts`.
 *
 * `data` stays `serde_json::Value` because each `component` type has a
 * different inner shape (Markdown, CodeDiff, Table, …); the per-component
 * data schemas live in the TS module above and are intentionally not
 * modeled as a Rust discriminated union (would balloon the schema for
 * little gain on the Rust side).
 */
export interface CanvasPanel {
  component: string;
  created_at: string;
  data: unknown;
  group?: string | null;
  panel_id: string;
  priority: number;
  size: string;
  task_run_id: string;
  title: string;
  updated_at: string;
  [k: string]: unknown;
}
