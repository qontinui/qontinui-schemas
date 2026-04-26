/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ComponentActionInfo } from './ComponentActionInfo';
import type { ElementBbox } from './ElementBbox';
import type { ElementIdentifier } from './ElementIdentifier';
import type { ElementRect } from './ElementRect';
import type { ElementState } from './ElementState';
import type { UIBridgeCapturedToast } from './UIBridgeCapturedToast';
import type { UIBridgeComponent } from './UIBridgeComponent';
import type { UIBridgeElement } from './UIBridgeElement';
import type { UIBridgeModalInfo } from './UIBridgeModalInfo';
import type { UIBridgeModalStack } from './UIBridgeModalStack';
import type { UIBridgeToastContext } from './UIBridgeToastContext';
import type { UIBridgeUndoContext } from './UIBridgeUndoContext';
import type { WorkflowInfo } from './WorkflowInfo';

/**
 * Full snapshot of the UI Bridge state.
 *
 * Captures all registered elements, components, and active workflows
 * at a single point in time. The optional `modalStack` / `toasts` /
 * `undoRedo` fields are populated by the SDK's enricher slot when
 * configured (see `setEnrichers` on web and native registries).
 */
export interface UIBridgeSnapshot {
  /**
   * All registered components.
   */
  components?: UIBridgeComponent[];
  /**
   * Native-only: current navigation route (Expo Router pathname).
   */
  currentRoute?: string | null;
  /**
   * All registered elements.
   */
  elements?: UIBridgeElement[];
  /**
   * Modal/sheet stack (populated when ModalDetector enricher is set).
   */
  modalStack?: UIBridgeModalStack | null;
  /**
   * Native-only: current route segments.
   */
  segments?: string[];
  /**
   * Unix-epoch millisecond timestamp of the snapshot.
   */
  timestamp: number;
  /**
   * Active and recently dismissed toasts (populated by ToastCapture).
   */
  toasts?: UIBridgeToastContext | null;
  /**
   * Undo/redo availability (populated by UndoTracker).
   */
  undoRedo?: UIBridgeUndoContext | null;
  /**
   * Active workflows.
   */
  workflows?: WorkflowInfo[];
}
