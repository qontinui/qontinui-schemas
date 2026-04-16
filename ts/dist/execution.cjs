"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/execution/index.ts
var execution_exports = {};
__export(execution_exports, {
  ActionStatus: () => ActionStatus,
  ActionType: () => ActionType,
  ErrorType: () => ErrorType,
  IssueSeverity: () => IssueSeverity,
  RunStatus: () => RunStatus,
  RunType: () => RunType,
  ScreenshotType: () => ScreenshotType
});
module.exports = __toCommonJS(execution_exports);

// src/execution/_api.ts
var RunType = {
  QA_TEST: "qa_test",
  INTEGRATION_TEST: "integration_test",
  LIVE_AUTOMATION: "live_automation",
  RECORDING: "recording",
  DEBUG: "debug"
};
var RunStatus = {
  PENDING: "pending",
  RUNNING: "running",
  COMPLETED: "completed",
  FAILED: "failed",
  TIMEOUT: "timeout",
  CANCELLED: "cancelled",
  PAUSED: "paused"
};
var ActionStatus = {
  SUCCESS: "success",
  FAILED: "failed",
  TIMEOUT: "timeout",
  SKIPPED: "skipped",
  ERROR: "error",
  PENDING: "pending"
};
var ActionType = {
  // Vision actions
  FIND: "find",
  FIND_ALL: "find_all",
  WAIT_FOR: "wait_for",
  WAIT_UNTIL_GONE: "wait_until_gone",
  // Input actions
  CLICK: "click",
  DOUBLE_CLICK: "double_click",
  RIGHT_CLICK: "right_click",
  TYPE: "type",
  PRESS_KEY: "press_key",
  HOTKEY: "hotkey",
  SCROLL: "scroll",
  DRAG: "drag",
  // State machine actions
  GO_TO_STATE: "go_to_state",
  TRANSITION: "transition",
  VERIFY_STATE: "verify_state",
  // Control flow
  CONDITIONAL: "conditional",
  LOOP: "loop",
  PARALLEL: "parallel",
  SEQUENCE: "sequence",
  // Utility
  WAIT: "wait",
  SCREENSHOT: "screenshot",
  LOG: "log",
  ASSERT: "assert",
  // AI actions
  AI_PROMPT: "ai_prompt",
  RUN_PROMPT_SEQUENCE: "run_prompt_sequence",
  // Custom/plugin
  CUSTOM: "custom"
};
var ErrorType = {
  ELEMENT_NOT_FOUND: "element_not_found",
  TIMEOUT: "timeout",
  ASSERTION_FAILED: "assertion_failed",
  CRASH: "crash",
  NETWORK_ERROR: "network_error",
  VALIDATION_ERROR: "validation_error",
  OTHER: "other"
};
var IssueSeverity = {
  CRITICAL: "critical",
  HIGH: "high",
  MEDIUM: "medium",
  LOW: "low",
  INFORMATIONAL: "informational"
};
var ScreenshotType = {
  ERROR: "error",
  SUCCESS: "success",
  MANUAL: "manual",
  PERIODIC: "periodic",
  ACTION_RESULT: "action_result",
  STATE_VERIFICATION: "state_verification"
};
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  ActionStatus,
  ActionType,
  ErrorType,
  IssueSeverity,
  RunStatus,
  RunType,
  ScreenshotType
});
//# sourceMappingURL=execution.cjs.map