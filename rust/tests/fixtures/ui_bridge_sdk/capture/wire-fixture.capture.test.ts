/**
 * Capture harness for qontinui-schemas' SDK wire fixture
 * (rust/tests/fixtures/ui_bridge_sdk/sdk_main_create_snapshot.json).
 *
 * Not a test of this SDK: it drives the REAL `UIBridgeRegistry.createSnapshot()`
 * (the payload `GET /ui-bridge/control/snapshot` serves) over a registry that
 * exercises every snapshot plane the canonical Rust `UIBridgeSnapshot` types —
 * an element carrying `customActions`, a component with `ComponentActionInfo`
 * actions, a workflow, and the undoRedo enricher — and writes the wire bytes
 * (a JSON round-trip, i.e. exactly what goes over HTTP) to $WIRE_FIXTURE_OUT.
 */
import { writeFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { UIBridgeRegistry } from './registry';

describe('wire fixture capture', () => {
  it('writes createSnapshot() wire bytes', () => {
    const out = process.env.WIRE_FIXTURE_OUT;
    expect(out, 'WIRE_FIXTURE_OUT must be set').toBeTruthy();

    const registry = new UIBridgeRegistry();
    const container = document.createElement('div');
    document.body.appendChild(container);

    const input = document.createElement('textarea');
    input.setAttribute('aria-label', 'Terminal input');
    container.appendChild(input);
    registry.registerElement('terminal-input', input, {
      type: 'input',
      label: 'Terminal input',
      actions: ['focus', 'type'],
      customActions: {
        sendKeys: {
          id: 'sendKeys',
          label: 'Send keys',
          description: 'Write raw key bytes into the PTY',
          handler: () => undefined,
        },
        getScrollback: { id: 'getScrollback', handler: () => '' },
      },
    });

    const btn = document.createElement('button');
    btn.textContent = 'Save';
    container.appendChild(btn);
    registry.registerElement('save-btn', btn, { type: 'button', label: 'Save' });

    registry.registerComponent('editor', {
      name: 'Editor',
      description: 'The editor pane',
      actions: [
        {
          id: 'save',
          label: 'Save',
          description: 'Persist the doc',
          paramSchema: { type: 'object', properties: { force: { type: 'boolean' } } },
          effect: 'write',
          handler: () => undefined,
        },
        { id: 'reset', handler: () => undefined },
      ],
      elementIds: ['save-btn', 'terminal-input'],
    });

    registry.registerWorkflow({
      id: 'save-flow',
      name: 'Save flow',
      description: 'Focus then save',
      steps: [
        { id: 's1', type: 'action', target: 'terminal-input', action: 'focus' },
        { id: 's2', type: 'action', target: 'save-btn', action: 'click' },
      ] as never,
    });

    registry.setEnrichers({
      undoTracker: {
        getSnapshotUndoContext: () => ({
          canUndo: true,
          canRedo: false,
          undoDescription: 'Typing',
          undoDepth: 1,
          redoDepth: 0,
          summary: 'Can undo (Typing).',
        }),
      },
    });

    const wire = JSON.parse(JSON.stringify(registry.createSnapshot()));
    writeFileSync(out as string, JSON.stringify(wire, null, 2) + '\n');
    expect(wire.elements.some((e: { customActions?: unknown[] }) => e.customActions?.length)).toBe(
      true
    );
  });
});
