/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TicketSource } from './TicketSource';

/**
 * Configuration for a ticket provider watcher.
 *
 * Persisted in `workflow_triggers.trigger_config` and
 * `ticket_provider_configs.config_json` so that watcher processes can be
 * reconstructed across runner restarts.
 *
 * **Security**: `api_token` is a secret. This struct serializes it so that
 * the same shape works for at-rest persistence, but MCP / HTTP API
 * responses exposing this config to end-user UIs MUST redact the token
 * before returning. See the module-level doc for the full policy.
 */
export interface TicketProviderConfig {
  /**
   * Labels / filters that mark a ticket as "actionable" (i.e. eligible to
   * spawn a workflow task). All listed labels must be present for a match.
   */
  actionableLabels?: string[];
  /**
   * API token for the provider. **Secret** — see security note above.
   * Persisted in the DB so the watcher can reconstruct a provider across
   * restarts; redact before exposing over UI-facing APIs.
   */
  apiToken: string;
  /**
   * Poll interval in seconds. Default: 60.
   */
  pollIntervalSeconds: number;
  source: TicketSource;
  /**
   * Provider-specific target:
   * - GitHub: `"owner/repo"`
   * - Linear: team key (e.g. `"ENG"`)
   * - Jira: project key
   */
  target: string;
  /**
   * Whether to update the remote ticket's state when the spawned task
   * completes. Default: true.
   */
  updateOnCompletion: boolean;
  /**
   * ID of the workflow to spawn for matched tickets.
   */
  workflowId: string;
  [k: string]: unknown;
}
