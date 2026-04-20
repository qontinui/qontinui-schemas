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

export {
  RunType,
  RunStatus,
  ActionStatus,
  ActionType,
  ErrorType,
  IssueSeverity,
  ScreenshotType
};
//# sourceMappingURL=chunk-YXD4WCYQ.js.map