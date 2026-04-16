import { defineConfig } from "tsup";

export default defineConfig({
  entry: {
    index: "src/index.ts",
    workflow: "src/workflow/index.ts",
    "task-run": "src/task-run/index.ts",
    execution: "src/execution/index.ts",
    scheduler: "src/scheduler/index.ts",
    library: "src/library/index.ts",
    chat: "src/chat/index.ts",
    "state-machine": "src/state-machine/index.ts",
    constraints: "src/constraints/index.ts",
    geometry: "src/geometry/index.ts",
    "tree-events": "src/tree-events/index.ts",
    accessibility: "src/accessibility/index.ts",
    config: "src/config/index.ts",
    targets: "src/targets/index.ts",
    rag: "src/rag/index.ts",
  },
  format: ["esm", "cjs"],
  dts: true,
  clean: true,
  sourcemap: true,
});
