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
  },
  format: ["esm", "cjs"],
  dts: true,
  clean: true,
  sourcemap: true,
});
