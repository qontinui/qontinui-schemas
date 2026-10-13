/**
 * Capture harness for qontinui-schemas' SDK page-health fixtures
 * (rust/tests/fixtures/ui_bridge_sdk/page_health/*.json).
 *
 * Not a test of this SDK: it drives the REAL `diagnosePageHealth` producer
 * (`sdk/page-health`, what `/control/page-health` answers) with EXACTLY the
 * input mapping the in-page handler uses (`server/handlers.ts` `pageHealth`):
 * `elements` handed over as received, `registeredComponents` = the snapshot's
 * `components.length`, `observedAt` = its `timestamp`, `source` =
 * `{ snapshotId }`. It writes the answer's wire bytes (a JSON round-trip, i.e.
 * exactly what goes over HTTP inside the `success` envelope's `data`).
 *
 * Two captures:
 *   - PAGE_HEALTH_POPULATED_OUT: over a recorded genuine SDK snapshot read from
 *     PAGE_HEALTH_SNAPSHOT_IN (a fixture under rust/tests/fixtures/ui_bridge_sdk/).
 *   - PAGE_HEALTH_EMPTY_OUT: over `new UIBridgeRegistry().createSnapshot()` —
 *     an empty registry, zero elements and zero components.
 *
 * `evaluatedAt` is pinned (PAGE_HEALTH_EVALUATED_AT) so the capture is
 * reproducible apart from the empty registry's own `timestamp`/`snapshotId`.
 * The producer version is the SDK's own `__SDK_VERSION__` (vitest defines it
 * from package.json), i.e. the default producer, not an override.
 *
 * Usage (from a ui-bridge checkout with dependencies installed): copy this file
 * to packages/ui-bridge/src/server/, then from packages/ui-bridge run
 *   PAGE_HEALTH_SNAPSHOT_IN=<abs>.json PAGE_HEALTH_POPULATED_OUT=<abs>.json \
 *   PAGE_HEALTH_EMPTY_OUT=<abs>.json PAGE_HEALTH_EVALUATED_AT=<RFC3339> \
 *   npx vitest run src/server/page-health.capture.test.ts
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { UIBridgeRegistry } from '../core/registry';
import { diagnosePageHealth } from './page-health';

type RawSnapshot = {
  elements?: unknown;
  components?: unknown;
  timestamp?: number;
  snapshotId?: unknown;
};

/** The handler's input mapping, verbatim. */
function answer(raw: RawSnapshot, evaluatedAt: string) {
  return diagnosePageHealth(
    {
      elements: raw.elements,
      registeredComponents: Array.isArray(raw.components) ? raw.components.length : null,
      observedAt: typeof raw.timestamp === 'number' ? raw.timestamp : null,
      source: typeof raw.snapshotId === 'string' ? { snapshotId: raw.snapshotId } : null,
    },
    { evaluatedAt }
  );
}

function wire(value: unknown): string {
  return JSON.stringify(JSON.parse(JSON.stringify(value)), null, 2) + '\n';
}

describe('page-health fixture capture', () => {
  it('writes diagnosePageHealth() wire bytes for a populated and an empty registry', () => {
    const snapshotIn = process.env.PAGE_HEALTH_SNAPSHOT_IN;
    const populatedOut = process.env.PAGE_HEALTH_POPULATED_OUT;
    const emptyOut = process.env.PAGE_HEALTH_EMPTY_OUT;
    const evaluatedAt = process.env.PAGE_HEALTH_EVALUATED_AT;
    expect(snapshotIn && populatedOut && emptyOut && evaluatedAt, 'all four env vars must be set').toBeTruthy();

    const populated = answer(JSON.parse(readFileSync(snapshotIn!, 'utf8')) as RawSnapshot, evaluatedAt!);
    expect(populated.status).toBe('measured');
    writeFileSync(populatedOut!, wire(populated));

    const empty = answer(new UIBridgeRegistry().createSnapshot() as RawSnapshot, evaluatedAt!);
    expect(empty.status).toBe('unknown');
    writeFileSync(emptyOut!, wire(empty));
  });
});
