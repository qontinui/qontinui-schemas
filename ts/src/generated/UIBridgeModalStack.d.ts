/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { UIBridgeModalInfo } from './UIBridgeModalInfo';

/**
 * Modal stack context attached to a snapshot when the SDK has a
 * `ModalDetector` enricher configured.
 */
export interface UIBridgeModalStack {
  count: number;
  hasBlockingModal: boolean;
  modals?: UIBridgeModalInfo[];
  topModal?: UIBridgeModalInfo | null;
}
