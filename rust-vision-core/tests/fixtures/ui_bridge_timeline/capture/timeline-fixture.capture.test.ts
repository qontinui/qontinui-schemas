/**
 * Capture harness for qontinui-schemas `rust-vision-core/tests/timeline_wire.rs`.
 *
 * NOT a ui-bridge test: it lives in qontinui-schemas and is copied into a
 * ui-bridge checkout only to capture a fixture. It drives the SDK's own
 * `ChangeTracker.executeWithDiff` in jsdom (the same harness shape as
 * `src/ai/timeline.phase4.test.ts`) and writes each run's `timeline` — a JSON
 * round-trip, i.e. exactly what `POST /ai/execute-with-diff` puts on the wire —
 * to `$TIMELINE_FIXTURE_OUT`, keyed by scenario.
 *
 *   cp <schemas>/rust-vision-core/tests/fixtures/ui_bridge_timeline/capture/timeline-fixture.capture.test.ts \
 *      <ui-bridge>/packages/ui-bridge/src/ai/
 *   cd <ui-bridge>/packages/ui-bridge && \
 *      TIMELINE_FIXTURE_OUT=<abs path>.json npx vitest run src/ai/timeline-fixture.capture.test.ts
 *
 * The scenarios between them emit every TimelineEvent type, every
 * ExpectedStateOutcome arm, a measured / an absent / an unknown summary field,
 * and a degraded-coverage measured field, so the Rust mirror is exercised on
 * every shape the SDK can send.
 */

import { writeFileSync } from 'node:fs';
import { afterAll, afterEach, beforeEach, expect, it, vi } from 'vitest';
import { ChangeTracker, type ChangeTrackerDeps } from './change-tracker';
import type { ChangeTimeline, SemanticSnapshot } from './types';
import type { ControlSnapshot } from '../control/types';
import type { SemanticSnapshotManager } from './semantic-snapshot';
import type { TimelineNetworkEvent, TimelineStateSource } from './timeline-recorder';
import { __resetGlobalBookmarkStoreForTest } from './bookmarks';

type Callback = (list: { getEntries(): PerformanceEntry[] }) => void;

class FakePerformanceObserver {
  static supportedEntryTypes: string[] = [];
  static live = new Set<FakePerformanceObserver>();
  type: string | null = null;
  constructor(private readonly callback: Callback) {}
  observe(init: { type: string }): void {
    this.type = init.type;
    FakePerformanceObserver.live.add(this);
  }
  disconnect(): void {
    FakePerformanceObserver.live.delete(this);
  }
  takeRecords(): PerformanceEntry[] {
    return [];
  }
  static emit(type: string, entries: Array<Record<string, unknown>>): void {
    for (const obs of FakePerformanceObserver.live) {
      if (obs.type === type) {
        const list = entries as unknown as PerformanceEntry[];
        obs.callback({ getEntries: () => list });
      }
    }
  }
}

function installPerformanceObserver(supported: string[]): void {
  FakePerformanceObserver.supportedEntryTypes = supported;
  FakePerformanceObserver.live.clear();
  vi.stubGlobal('PerformanceObserver', FakePerformanceObserver);
}

let pageUrl = 'http://localhost/fixture';

function domSnapshot(): SemanticSnapshot {
  const ids = Array.from(document.querySelectorAll<HTMLElement>('[data-fixture-id]')).map(
    (el) => el.dataset.fixtureId!
  );
  return {
    timestamp: Date.now(),
    snapshotId: `snap-${ids.length}`,
    page: { url: pageUrl, title: 'Fixture', activeModals: [] },
    elements: ids.map((id) => ({
      id,
      type: 'button',
      label: id,
      tagName: 'div',
      actions: ['click'],
      state: {
        visible: true,
        enabled: true,
        focused: false,
        rect: { x: 0, y: 0, width: 100, height: 30, top: 0, right: 100, bottom: 30, left: 0 },
        textContent: document.querySelector(`[data-fixture-id="${id}"]`)?.textContent ?? '',
      },
      registered: true,
      description: id,
      aliases: [],
      suggestedActions: [],
      category: 'interactive' as const,
    })),
    forms: [],
    activeModals: [],
    summary: 'fixture',
    elementCounts: {},
  } as unknown as SemanticSnapshot;
}

function fixtureDeps(overrides: Partial<ChangeTrackerDeps> = {}): ChangeTrackerDeps {
  return {
    snapshotManager: {
      createSnapshot: vi.fn(() => domSnapshot()),
    } as unknown as SemanticSnapshotManager,
    idleDetector: null,
    createControlSnapshot: vi.fn(() => ({}) as ControlSnapshot),
    executeElementAction: vi.fn().mockResolvedValue({ success: true }),
    executeNLAction: vi.fn().mockResolvedValue({ success: true }),
    refreshElements: vi.fn(),
    ...overrides,
  };
}

function mountPage(): { button: HTMLElement; content: HTMLElement } {
  document.body.innerHTML = '';
  const button = document.createElement('button');
  button.dataset.fixtureId = 'save';
  button.textContent = 'Save';
  const content = document.createElement('main');
  content.dataset.fixtureId = 'content';
  content.textContent = 'Content';
  document.body.append(button, content);
  return { button, content };
}

function stateSource(declared: string[]): TimelineStateSource & {
  activate(id: string): void;
  deactivate(id: string): void;
} {
  const active = new Set<string>();
  const listeners = new Set<(e: { stateId: string; active: boolean }) => void>();
  return {
    getActiveStates: () => [...active],
    isDeclared: (id) => declared.includes(id),
    subscribe(cb) {
      listeners.add(cb);
      return () => listeners.delete(cb);
    },
    activate(id) {
      active.add(id);
      listeners.forEach((l) => l({ stateId: id, active: true }));
    },
    deactivate(id) {
      active.delete(id);
      listeners.forEach((l) => l({ stateId: id, active: false }));
    },
  };
}

function animationEvent(type: string, name: string, elapsed?: number): Event {
  const ev = new Event(type, { bubbles: true });
  Object.assign(ev, { animationName: name, elapsedTime: elapsed });
  return ev;
}

const captured: Record<string, ChangeTimeline> = {};

function keep(name: string, timeline: ChangeTimeline | undefined): void {
  expect(timeline).toBeDefined();
  // A JSON round-trip: exactly the bytes the HTTP route sends.
  captured[name] = JSON.parse(JSON.stringify(timeline));
}

beforeEach(() => {
  __resetGlobalBookmarkStoreForTest();
  pageUrl = 'http://localhost/fixture';
  mountPage();
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.innerHTML = '';
});

afterAll(() => {
  const out = process.env.TIMELINE_FIXTURE_OUT;
  if (!out) throw new Error('TIMELINE_FIXTURE_OUT is required');
  writeFileSync(out, JSON.stringify(captured, null, 2) + '\n');
});

it('rich: every event type, entered / not-entered / undeclared / already-active states', async () => {
  installPerformanceObserver(['layout-shift', 'longtask']);
  const { button, content } = mountPage();
  const hint = document.createElement('p');
  hint.dataset.fixtureId = 'hint';
  hint.textContent = 'Unsaved changes';
  document.body.append(hint);
  const states = stateSource(['saved-toast', 'error-banner', 'editor', 'spinner']);
  states.activate('editor');
  let networkListener: ((e: TimelineNetworkEvent) => void) | null = null;
  const subscribeNetworkEvents = vi.fn((cb: (e: TimelineNetworkEvent) => void) => {
    networkListener = cb;
    return () => {
      networkListener = null;
    };
  });
  const executeElementAction = vi.fn(async () => {
    button.textContent = 'Saving…';
    states.activate('spinner');
    const request = { id: 'r1', url: '/api/save?draft=1', method: 'POST', startedAt: Date.now() };
    networkListener?.({ type: 'request-start', timestamp: Date.now(), entry: { request } });
    button.dispatchEvent(animationEvent('animationstart', 'pulse'));
    content.dispatchEvent(animationEvent('animationstart', 'slide-in'));
    FakePerformanceObserver.emit('longtask', [{ startTime: performance.now(), duration: 64 }]);
    setTimeout(() => content.dispatchEvent(animationEvent('animationend', 'slide-in', 0.2)), 30);
    setTimeout(() => {
      networkListener?.({
        type: 'request-complete',
        timestamp: Date.now(),
        entry: { request, response: { statusCode: 200, durationMs: 80 } },
      });
      states.deactivate('spinner');
      states.activate('saved-toast');
      const banner = document.createElement('div');
      banner.dataset.fixtureId = 'banner';
      banner.textContent = 'Saved!';
      document.body.insertBefore(banner, content);
      hint.remove();
      pageUrl = 'http://localhost/fixture/saved';
      FakePerformanceObserver.emit('layout-shift', [
        {
          startTime: performance.now(),
          value: 0.12,
          hadRecentInput: false,
          sources: [{ node: content }],
        },
      ]);
    }, 80);
    return { success: true };
  });
  const result = await new ChangeTracker(
    fixtureDeps({ executeElementAction, subscribeNetworkEvents, states })
  ).executeWithDiff({
    elementAction: { elementId: 'save', action: 'click' },
    settleMinStable: 200,
    timelineInterval: 20,
    settleTimeout: 4000,
    expectStates: ['saved-toast', 'error-banner', 'editor', 'undeclared-state'],
    withinMs: 400,
  });
  keep('rich', result.timeline);
}, 30_000);

it('no PerformanceObserver: layout-shift and long-task fields are unknown', async () => {
  vi.stubGlobal('PerformanceObserver', undefined);
  const result = await new ChangeTracker(fixtureDeps()).executeWithDiff({
    elementAction: { elementId: 'save', action: 'click' },
    settleMinStable: 20,
    timelineInterval: 10,
  });
  keep('noPerformanceObserver', result.timeline);
});

it('never settles: settleMs is unknown{no_event_before_timeout}', async () => {
  installPerformanceObserver(['layout-shift', 'longtask']);
  let n = 0;
  const result = await new ChangeTracker(
    fixtureDeps({
      snapshotManager: {
        createSnapshot: vi.fn(() => {
          const el = document.createElement('div');
          el.dataset.fixtureId = `churn-${n++}`;
          document.body.append(el);
          return domSnapshot();
        }),
      } as unknown as SemanticSnapshotManager,
    })
  ).executeWithDiff({
    elementAction: { elementId: 'save', action: 'click' },
    settleMinStable: 50,
    timelineInterval: 10,
    settleTimeout: 120,
  });
  keep('neverSettles', result.timeline);
});

it('static page: nothing responds', async () => {
  installPerformanceObserver(['layout-shift', 'longtask']);
  const result = await new ChangeTracker(fixtureDeps()).executeWithDiff({
    elementAction: { elementId: 'save', action: 'click' },
    settleMinStable: 20,
    timelineInterval: 10,
  });
  keep('staticPage', result.timeline);
});
