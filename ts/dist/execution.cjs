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
var RunType = /* @__PURE__ */ ((RunType2) => {
  RunType2["QA_TEST"] = "qa_test";
  RunType2["INTEGRATION_TEST"] = "integration_test";
  RunType2["LIVE_AUTOMATION"] = "live_automation";
  RunType2["RECORDING"] = "recording";
  RunType2["DEBUG"] = "debug";
  return RunType2;
})(RunType || {});
var RunStatus = /* @__PURE__ */ ((RunStatus2) => {
  RunStatus2["PENDING"] = "pending";
  RunStatus2["RUNNING"] = "running";
  RunStatus2["COMPLETED"] = "completed";
  RunStatus2["FAILED"] = "failed";
  RunStatus2["TIMEOUT"] = "timeout";
  RunStatus2["CANCELLED"] = "cancelled";
  RunStatus2["PAUSED"] = "paused";
  return RunStatus2;
})(RunStatus || {});
var ActionStatus = /* @__PURE__ */ ((ActionStatus2) => {
  ActionStatus2["SUCCESS"] = "success";
  ActionStatus2["FAILED"] = "failed";
  ActionStatus2["TIMEOUT"] = "timeout";
  ActionStatus2["SKIPPED"] = "skipped";
  ActionStatus2["ERROR"] = "error";
  ActionStatus2["PENDING"] = "pending";
  return ActionStatus2;
})(ActionStatus || {});
var ActionType = /* @__PURE__ */ ((ActionType2) => {
  ActionType2["FIND"] = "find";
  ActionType2["FIND_ALL"] = "find_all";
  ActionType2["WAIT_FOR"] = "wait_for";
  ActionType2["WAIT_UNTIL_GONE"] = "wait_until_gone";
  ActionType2["CLICK"] = "click";
  ActionType2["DOUBLE_CLICK"] = "double_click";
  ActionType2["RIGHT_CLICK"] = "right_click";
  ActionType2["TYPE"] = "type";
  ActionType2["PRESS_KEY"] = "press_key";
  ActionType2["HOTKEY"] = "hotkey";
  ActionType2["SCROLL"] = "scroll";
  ActionType2["DRAG"] = "drag";
  ActionType2["GO_TO_STATE"] = "go_to_state";
  ActionType2["TRANSITION"] = "transition";
  ActionType2["VERIFY_STATE"] = "verify_state";
  ActionType2["CONDITIONAL"] = "conditional";
  ActionType2["LOOP"] = "loop";
  ActionType2["PARALLEL"] = "parallel";
  ActionType2["SEQUENCE"] = "sequence";
  ActionType2["WAIT"] = "wait";
  ActionType2["SCREENSHOT"] = "screenshot";
  ActionType2["LOG"] = "log";
  ActionType2["ASSERT"] = "assert";
  ActionType2["AI_PROMPT"] = "ai_prompt";
  ActionType2["RUN_PROMPT_SEQUENCE"] = "run_prompt_sequence";
  ActionType2["CUSTOM"] = "custom";
  return ActionType2;
})(ActionType || {});
var ErrorType = /* @__PURE__ */ ((ErrorType2) => {
  ErrorType2["ELEMENT_NOT_FOUND"] = "element_not_found";
  ErrorType2["TIMEOUT"] = "timeout";
  ErrorType2["ASSERTION_FAILED"] = "assertion_failed";
  ErrorType2["CRASH"] = "crash";
  ErrorType2["NETWORK_ERROR"] = "network_error";
  ErrorType2["VALIDATION_ERROR"] = "validation_error";
  ErrorType2["OTHER"] = "other";
  return ErrorType2;
})(ErrorType || {});
var IssueSeverity = /* @__PURE__ */ ((IssueSeverity2) => {
  IssueSeverity2["CRITICAL"] = "critical";
  IssueSeverity2["HIGH"] = "high";
  IssueSeverity2["MEDIUM"] = "medium";
  IssueSeverity2["LOW"] = "low";
  IssueSeverity2["INFORMATIONAL"] = "informational";
  return IssueSeverity2;
})(IssueSeverity || {});
var ScreenshotType = /* @__PURE__ */ ((ScreenshotType2) => {
  ScreenshotType2["ERROR"] = "error";
  ScreenshotType2["SUCCESS"] = "success";
  ScreenshotType2["MANUAL"] = "manual";
  ScreenshotType2["PERIODIC"] = "periodic";
  ScreenshotType2["ACTION_RESULT"] = "action_result";
  ScreenshotType2["STATE_VERIFICATION"] = "state_verification";
  return ScreenshotType2;
})(ScreenshotType || {});
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