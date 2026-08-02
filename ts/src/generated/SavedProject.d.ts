/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A project the user has told the runner about (typically via the setup
 * wizard's project picker, the new-project flow, or the Projects
 * dashboard's "find on disk" discovery).
 *
 * Persisted to `settings.json` under the `saved_projects` key.
 *
 * `id` is the stable key: a project's `path` can move on disk (renamed
 * folder, relocated workspace) but its identity must not. Entries persisted
 * before `id` existed are backfilled with a freshly-minted UUID on first
 * load — `#[serde(default)]` exists only so that load succeeds, never as a
 * legitimate steady-state value.
 *
 * `project_type` is deliberately a free-form `String` so a new framework
 * never requires a schema change.
 * `Default` exists so a producer that only knows a few fields can write
 * `SavedProject { path, name, ..Default::default() }` — the new-project flow
 * and the setup wizard both do. A defaulted `id` is the empty string, which
 * the registry treats as "mint me one" (see the field docs), never as a key.
 */
export interface SavedProject {
  /**
   * Accent colour for the card, as a CSS colour string.
   */
  color?: string | null;
  /**
   * One-line plain-English description ("Website for the pizzeria, with
   * menu + ordering"). Shown on the project card under the name.
   */
  description?: string | null;
  /**
   * Emoji used as the card's visual identity (e.g. "🍕").
   */
  emoji?: string | null;
  /**
   * The project's front page ("http://localhost:3000"). Seeded from the
   * setup wizard's dev-server port detection; user-correctable.
   */
  frontPageUrl?: string | null;
  /**
   * Stable identifier (UUID). Survives a path move; the registry key.
   */
  id: string;
  /**
   * Unix epoch milliseconds of the last time the project was activated.
   */
  lastOpenedMs?: number | null;
  /**
   * Manifest file that identified the project (e.g. "package.json").
   */
  manifest: string;
  /**
   * Human-friendly display name (usually the directory basename).
   */
  name: string;
  /**
   * The user's own free-form notes about the project.
   */
  notes?: string | null;
  /**
   * Absolute path to the project root.
   */
  path: string;
  /**
   * Pinned projects sort first and render as sidebar entries.
   */
  pinned: boolean;
  /**
   * Ids of the `ProcessConfig` entries this project owns.
   */
  processIds: string[];
  /**
   * Framework/language tag, e.g. "react", "python", "rust", "node".
   */
  projectType: string;
  /**
   * `owner/name` GitHub slug, from `repo_detection::detect_repo_slug`.
   */
  repoSlug?: string | null;
  /**
   * Bound Terminal page id.
   *
   * **A hint, not a handle.** Terminal pages persist to the frontend's
   * `instanceStorage` (localStorage, port-namespaced), which Rust cannot
   * read, so this id can name a page that does not exist in the current
   * window. Activation must treat an unknown id as "not created yet" —
   * create a page named after the project and rewrite the id — and must
   * never fail on a dangling value.
   */
  terminalPageId?: string | null;
  /**
   * Terminal zone profile to restore when the project is activated.
   */
  zoneProfile?: string | null;
}
