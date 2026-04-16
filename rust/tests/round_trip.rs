//! Round-trip tests for `qontinui-types` DTOs.
//!
//! Each test constructs a representative instance, serializes it to JSON,
//! deserializes it back, and re-serializes. The two JSON strings must match
//! byte-for-byte (serde preserves declaration order, and these tests do the
//! same cycle twice) — if they don't, the DTO has a serde bug.
//!
//! Fixture-driven tests additionally round-trip hand-built JSON files in
//! `qontinui-schemas/tests/fixtures/` to catch shape drift between the wire
//! contract and the Rust types.

use qontinui_types::constraints::*;
use qontinui_types::scheduler::*;
use qontinui_types::workflow::*;
use qontinui_types::workflow_step::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Parse two JSON strings and assert their `serde_json::Value` forms are equal.
/// Useful when comparing serializations of types containing `HashMap`s — where
/// the byte order of keys is non-deterministic but the logical content is
/// stable.
#[allow(dead_code)]
fn assert_json_values_equal(a: &str, b: &str) {
    let va: Value = serde_json::from_str(a).expect("parse first JSON string");
    let vb: Value = serde_json::from_str(b).expect("parse second JSON string");
    assert_eq!(va, vb, "JSON values differ");
}

// ============================================================================
// Scheduler — ScheduleExpression (all four variants, externally tagged with
// `type`/`value`)
// ============================================================================

#[test]
fn schedule_expression_once_roundtrips() {
    let expr = ScheduleExpression::Once("2026-04-14T03:00:00Z".to_string());
    let json = serde_json::to_string(&expr).unwrap();
    assert_eq!(
        json,
        r#"{"type":"Once","value":"2026-04-14T03:00:00Z"}"#,
        "wire shape must be externally tagged {{type, value}}"
    );
    let back: ScheduleExpression = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn schedule_expression_cron_roundtrips() {
    let expr = ScheduleExpression::Cron("0 3 * * *".to_string());
    let json = serde_json::to_string(&expr).unwrap();
    assert_eq!(json, r#"{"type":"Cron","value":"0 3 * * *"}"#);
    let back: ScheduleExpression = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn schedule_expression_interval_roundtrips() {
    let expr = ScheduleExpression::Interval(60);
    let json = serde_json::to_string(&expr).unwrap();
    assert_eq!(json, r#"{"type":"Interval","value":60}"#);
    let back: ScheduleExpression = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn schedule_expression_condition_roundtrips() {
    let expr = ScheduleExpression::Condition(ConditionScheduleConfig {
        rearm_delay_minutes: 30,
    });
    let json = serde_json::to_string(&expr).unwrap();
    let back: ScheduleExpression = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
    // Confirm the envelope shape.
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "Condition");
    assert_eq!(v["value"]["rearm_delay_minutes"], 30);
}

// ============================================================================
// Scheduler — ScheduledTaskType (all five variants, internally tagged with
// `task_type`)
// ============================================================================

#[test]
fn scheduled_task_type_workflow_roundtrips() {
    let t = ScheduledTaskType::Workflow {
        workflow_name: "Nightly build".to_string(),
        config_path: Some("/configs/build.toml".to_string()),
        monitor_index: Some(0),
        workflow_id: Some("wf-123".to_string()),
    };
    let json = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["task_type"], "Workflow");
    assert_eq!(v["workflow_name"], "Nightly build");
    let back: ScheduledTaskType = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn scheduled_task_type_prompt_roundtrips() {
    let t = ScheduledTaskType::Prompt {
        prompt_id: "prompt-001".to_string(),
        max_sessions: Some(3),
    };
    let json = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["task_type"], "Prompt");
    let back: ScheduledTaskType = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn scheduled_task_type_autofix_roundtrips() {
    let t = ScheduledTaskType::AutoFix {
        check_findings: true,
        force_run: false,
    };
    let json = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["task_type"], "AutoFix");
    let back: ScheduledTaskType = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn scheduled_task_type_watcher_roundtrips() {
    let t = ScheduledTaskType::Watcher {
        watcher_id: "watcher-xyz".to_string(),
    };
    let json = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["task_type"], "Watcher");
    assert_eq!(v["watcher_id"], "watcher-xyz");
    let back: ScheduledTaskType = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn scheduled_task_type_background_capture_roundtrips() {
    let t = ScheduledTaskType::BackgroundCapture {
        monitor_index: Some(1),
        capture_interval_secs: 30,
        capture_on_focus_change: true,
    };
    let json = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["task_type"], "BackgroundCapture");
    let back: ScheduledTaskType = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

// ============================================================================
// Scheduler — ScheduledTaskStatus (snake_case)
// ============================================================================

#[test]
fn scheduled_task_status_snake_case() {
    // Every variant must serialize to its snake_case spelling.
    let cases = [
        (ScheduledTaskStatus::Pending, "\"pending\""),
        (ScheduledTaskStatus::Running, "\"running\""),
        (ScheduledTaskStatus::Completed, "\"completed\""),
        (ScheduledTaskStatus::Failed, "\"failed\""),
        (ScheduledTaskStatus::Skipped, "\"skipped\""),
        (ScheduledTaskStatus::Cancelled, "\"cancelled\""),
    ];
    for (status, expected) in cases {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, expected, "status serialization mismatch");
        let back: ScheduledTaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }
}

// ============================================================================
// Scheduler — ScheduledTask (fully populated + minimally populated)
// ============================================================================

#[test]
fn scheduled_task_fully_populated_roundtrips() {
    let task = ScheduledTask {
        id: "test-id".to_string(),
        name: "Daily cleanup".to_string(),
        description: Some("Clean up stale artifacts.".to_string()),
        enabled: true,
        schedule: ScheduleExpression::Cron("0 3 * * *".to_string()),
        task: ScheduledTaskType::AutoFix {
            check_findings: true,
            force_run: false,
        },
        skip_if_completed: false,
        auto_fix_on_failure: true,
        success_criteria: Some("No leftover temp files.".to_string()),
        created_at: "2026-04-14T00:00:00Z".to_string(),
        modified_at: "2026-04-14T00:00:00Z".to_string(),
        last_run: Some(TaskExecutionRecord {
            execution_id: "exec-1".to_string(),
            session_id: Some("sess-1".to_string()),
            started_at: "2026-04-13T03:00:00Z".to_string(),
            ended_at: Some("2026-04-13T03:05:00Z".to_string()),
            status: ScheduledTaskStatus::Completed,
            success: true,
            error_message: None,
            triggered_auto_fix: false,
            auto_fix_session_id: None,
        }),
        next_run: Some("2026-04-15T03:00:00Z".to_string()),
        conditions: Some(ScheduleConditions {
            require_idle: Some(IdleCondition { enabled: true }),
            require_repo_inactive: None,
            timeout_minutes: Some(30),
        }),
        condition_status: None,
    };
    let json = serde_json::to_string(&task).unwrap();
    let back: ScheduledTask = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn scheduled_task_minimally_populated_roundtrips() {
    let task = ScheduledTask {
        id: "min".to_string(),
        name: "Minimal".to_string(),
        description: None,
        enabled: true,
        schedule: ScheduleExpression::Interval(60),
        task: ScheduledTaskType::AutoFix {
            check_findings: true,
            force_run: false,
        },
        skip_if_completed: false,
        auto_fix_on_failure: false,
        success_criteria: None,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        modified_at: "2026-04-14T00:00:00Z".to_string(),
        last_run: None,
        next_run: None,
        conditions: None,
        condition_status: None,
    };
    let json = serde_json::to_string(&task).unwrap();
    let back: ScheduledTask = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn scheduled_task_minimal_elides_none_fields() {
    // None-valued optional fields must not serialize as `null` — they must be
    // omitted entirely so absence and explicit-null stay distinguishable.
    let task = ScheduledTask {
        id: "x".to_string(),
        name: "X".to_string(),
        description: None,
        enabled: true,
        schedule: ScheduleExpression::Interval(5),
        task: ScheduledTaskType::AutoFix {
            check_findings: true,
            force_run: false,
        },
        skip_if_completed: false,
        auto_fix_on_failure: false,
        success_criteria: None,
        created_at: "t".to_string(),
        modified_at: "t".to_string(),
        last_run: None,
        next_run: None,
        conditions: None,
        condition_status: None,
    };
    let json = serde_json::to_string(&task).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    for absent_field in [
        "description",
        "success_criteria",
        "last_run",
        "next_run",
        "conditions",
        "condition_status",
    ] {
        assert!(
            v.get(absent_field).is_none(),
            "expected `{absent_field}` to be omitted, got: {json}"
        );
    }
    // Re-parse should still yield None for those fields.
    let back: ScheduledTask = serde_json::from_str(&json).unwrap();
    assert!(back.description.is_none());
    assert!(back.last_run.is_none());
    assert!(back.conditions.is_none());
}

// ============================================================================
// Constraints — ConstraintSeverity (snake_case)
// ============================================================================

#[test]
fn constraint_severity_snake_case() {
    let cases = [
        (ConstraintSeverity::Block, "\"block\""),
        (ConstraintSeverity::Warn, "\"warn\""),
        (ConstraintSeverity::Log, "\"log\""),
    ];
    for (sev, expected) in cases {
        let json = serde_json::to_string(&sev).unwrap();
        assert_eq!(json, expected);
        let back: ConstraintSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sev);
    }
}

// ============================================================================
// Constraints — ConstraintCheck (all four variants, internally tagged,
// snake_case)
// ============================================================================

#[test]
fn constraint_check_grep_forbidden_roundtrips() {
    let c = ConstraintCheck::GrepForbidden {
        pattern: "TODO".to_string(),
        file_glob: Some("**/*.rs".to_string()),
    };
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "grep_forbidden");
    assert_eq!(v["pattern"], "TODO");
    let back: ConstraintCheck = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn constraint_check_grep_required_roundtrips() {
    let c = ConstraintCheck::GrepRequired {
        pattern: "Apache-2.0".to_string(),
        file_glob: None,
    };
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "grep_required");
    // file_glob is None → must be omitted (not null).
    assert!(v.get("file_glob").is_none());
    let back: ConstraintCheck = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn constraint_check_file_scope_roundtrips() {
    let c = ConstraintCheck::FileScope {
        allowed_paths: vec!["src/".to_string(), "tests/".to_string()],
    };
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "file_scope");
    let back: ConstraintCheck = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn constraint_check_command_roundtrips() {
    let c = ConstraintCheck::Command {
        cmd: "cargo check".to_string(),
        cwd: Some("src-tauri".to_string()),
        timeout_secs: 60,
    };
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "command");
    assert_eq!(v["timeout_secs"], 60);
    let back: ConstraintCheck = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

// ============================================================================
// Constraints — Constraint (fully populated)
// ============================================================================

#[test]
fn constraint_fully_populated_roundtrips() {
    let c = Constraint {
        id: "project:no-todos".to_string(),
        name: "No TODOs".to_string(),
        description: "Don't leave TODOs in committed code.".to_string(),
        check: ConstraintCheck::GrepForbidden {
            pattern: "TODO".to_string(),
            file_glob: Some("**/*.rs".to_string()),
        },
        severity: ConstraintSeverity::Warn,
        enabled: true,
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: Constraint = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn constraint_check_elides_none_fields() {
    // Make sure Option fields inside internally-tagged variants are omitted
    // rather than emitted as `null`.
    let c = ConstraintCheck::GrepForbidden {
        pattern: "x".to_string(),
        file_glob: None,
    };
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("file_glob").is_none(), "got: {json}");
}

// ============================================================================
// Constraints — ConstraintProposal (both variants)
// ============================================================================

#[test]
fn constraint_proposal_new_constraint_roundtrips() {
    let p = ConstraintProposal::NewConstraint(NewConstraintProposal {
        constraint: Constraint {
            id: "project:no-foo".to_string(),
            name: "No foo".to_string(),
            description: "desc".to_string(),
            check: ConstraintCheck::GrepForbidden {
                pattern: "foo".to_string(),
                file_glob: None,
            },
            severity: ConstraintSeverity::Block,
            enabled: true,
        },
    });
    let json = serde_json::to_string(&p).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "new_constraint");
    let back: ConstraintProposal = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn constraint_proposal_builtin_override_roundtrips() {
    let p = ConstraintProposal::BuiltinOverride(BuiltinOverrideProposal {
        builtin_suffix: "no-secrets".to_string(),
        enabled: false,
        reason: "Handled by pre-commit hook.".to_string(),
    });
    let json = serde_json::to_string(&p).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "builtin_override");
    assert_eq!(v["enabled"], false);
    let back: ConstraintProposal = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

// ============================================================================
// Workflow — LogSourceSelection (untagged: bare string vs object)
// ============================================================================

#[test]
fn log_source_selection_mode_is_bare_string() {
    let sel = LogSourceSelection::Mode(LogSourceMode::Default);
    let json = serde_json::to_string(&sel).unwrap();
    assert_eq!(json, "\"default\"");
    let back: LogSourceSelection = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sel);

    // All three modes round-trip as bare lowercase strings.
    for (mode, expected) in [
        (LogSourceMode::Default, "\"default\""),
        (LogSourceMode::Ai, "\"ai\""),
        (LogSourceMode::All, "\"all\""),
    ] {
        let sel = LogSourceSelection::Mode(mode);
        assert_eq!(serde_json::to_string(&sel).unwrap(), expected);
        let back: LogSourceSelection = serde_json::from_str(expected).unwrap();
        assert_eq!(back, sel);
    }

    // Unknown mode strings are now rejected (narrowing via LogSourceMode enum).
    assert!(serde_json::from_str::<LogSourceSelection>("\"bogus\"").is_err());
}

#[test]
fn log_source_selection_profile_is_object() {
    let sel = LogSourceSelection::Profile {
        profile_id: "abc".to_string(),
    };
    let json = serde_json::to_string(&sel).unwrap();
    assert_eq!(json, r#"{"profile_id":"abc"}"#);
    let back: LogSourceSelection = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sel);
}

#[test]
fn log_source_selection_deserializes_both_wire_shapes() {
    let from_string: LogSourceSelection = serde_json::from_str("\"default\"").unwrap();
    assert_eq!(from_string, LogSourceSelection::Mode(LogSourceMode::Default));

    let from_obj: LogSourceSelection =
        serde_json::from_str(r#"{"profile_id": "abc"}"#).unwrap();
    assert_eq!(
        from_obj,
        LogSourceSelection::Profile {
            profile_id: "abc".to_string()
        }
    );
}

// ============================================================================
// Workflow — UnifiedWorkflow (minimal frame with empty step arrays)
// ============================================================================

#[test]
fn unified_workflow_minimal_frame_roundtrips() {
    // Build via serde_json::from_value so the crate's many `#[serde(default)]`
    // hooks fire — this is the smallest wire payload a runner would accept.
    let raw = json!({
        "name": "Minimal"
    });
    let wf: UnifiedWorkflow = serde_json::from_value(raw).unwrap();
    assert_eq!(wf.name, "Minimal");
    assert!(wf.setup_steps.is_empty());
    assert!(wf.agentic_steps.is_empty());
    assert!(wf.verification_steps.is_empty());
    assert!(wf.completion_steps.is_empty());
    assert!(wf.stages.is_empty());
    // Defaults kick in.
    assert_eq!(wf.category, "general");
    assert_eq!(wf.max_fix_attempts, 3);
    assert_eq!(wf.max_ci_auto_resumes, 10);
    assert!(wf.multi_agent_mode);
    assert!(wf.reflection_mode);
    assert!(wf.ai_reviewed);

    // Round-trip.
    let json1 = serde_json::to_string(&wf).unwrap();
    let back: UnifiedWorkflow = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn unified_workflow_default_log_source_is_elided() {
    // When log_source_selection is the default ("default" mode), it must NOT
    // appear in the serialized output — the `is_default_log_source`
    // skip_serializing_if hook guarantees this.
    let raw = json!({ "name": "x" });
    let wf: UnifiedWorkflow = serde_json::from_value(raw).unwrap();
    let json = serde_json::to_string(&wf).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(
        v.get("log_source_selection").is_none(),
        "default selection leaked: {json}"
    );
}

// ============================================================================
// Fixture-driven tests
// ============================================================================

#[test]
fn constraint_fixture_parses() {
    let raw = include_str!("../../tests/fixtures/constraint_sample.json");
    let parsed: Constraint = serde_json::from_str(raw).expect("parse constraint fixture");
    let re_serialized = serde_json::to_string(&parsed).unwrap();
    let original_value: Value = serde_json::from_str(raw).unwrap();
    let roundtrip_value: Value = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(original_value, roundtrip_value);
}

#[test]
fn scheduled_task_fixture_parses() {
    let raw = include_str!("../../tests/fixtures/scheduled_task_sample.json");
    let parsed: ScheduledTask = serde_json::from_str(raw).expect("parse scheduled task fixture");
    let re_serialized = serde_json::to_string(&parsed).unwrap();
    let original_value: Value = serde_json::from_str(raw).unwrap();
    let roundtrip_value: Value = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(original_value, roundtrip_value);
}

// ============================================================================
// workflow_step — UnifiedStep (all four variants flatten the base fields and
// carry a `type` discriminator)
// ============================================================================

#[test]
fn unified_step_command_roundtrips() {
    let step = UnifiedStep::command(CommandStep {
        base: BaseStepFields {
            id: "s1".into(),
            name: "build".into(),
            ..Default::default()
        },
        phase: CommandStepPhase::Setup,
        mode: Some(CommandMode::Shell),
        command: Some("cargo build".into()),
        ..Default::default()
    });
    let json = serde_json::to_string(&step).unwrap();
    // Flat wire shape: `type` sits beside id/name/phase/mode/command.
    assert!(
        json.contains("\"type\":\"command\""),
        "missing discriminator: {json}"
    );
    assert!(json.contains("\"id\":\"s1\""), "base not flattened: {json}");
    assert!(json.contains("\"phase\":\"setup\""), "phase missing: {json}");
    assert!(json.contains("\"mode\":\"shell\""), "mode missing: {json}");
    assert!(
        json.contains("\"command\":\"cargo build\""),
        "command missing: {json}"
    );

    // Structural confirmation — the object must be flat, not nested under
    // some `data`/`command` key.
    let v: Value = serde_json::from_str(&json).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj["type"], "command");
    assert_eq!(obj["id"], "s1");
    assert_eq!(obj["name"], "build");
    assert!(
        !obj.contains_key("base"),
        "base must be flattened, not nested"
    );

    // Round-trip.
    let back: UnifiedStep = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn unified_step_prompt_roundtrips() {
    let step = UnifiedStep::prompt(PromptStep {
        base: BaseStepFields {
            id: "p1".into(),
            name: "fix-it".into(),
            ..Default::default()
        },
        phase: PromptStepPhase::Agentic,
        content: "Fix the failing test.".into(),
        provider: Some("anthropic".into()),
        ..Default::default()
    });
    let json = serde_json::to_string(&step).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "prompt");
    assert_eq!(v["phase"], "agentic");
    assert_eq!(v["content"], "Fix the failing test.");
    assert_eq!(v["provider"], "anthropic");

    let back: UnifiedStep = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn unified_step_ui_bridge_roundtrips() {
    let step = UnifiedStep::ui_bridge(UiBridgeStep {
        base: BaseStepFields {
            id: "u1".into(),
            name: "open-page".into(),
            ..Default::default()
        },
        phase: UiBridgeStepPhase::Setup,
        action: UiBridgeAction::Navigate,
        url: Some("http://localhost:1420/".into()),
        ..Default::default()
    });
    let json = serde_json::to_string(&step).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "ui_bridge");
    assert_eq!(v["action"], "navigate");
    assert_eq!(v["url"], "http://localhost:1420/");

    let back: UnifiedStep = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn unified_step_workflow_roundtrips() {
    let step = UnifiedStep::workflow(WorkflowStep {
        base: BaseStepFields {
            id: "w1".into(),
            name: "sub".into(),
            ..Default::default()
        },
        phase: WorkflowStepPhase::Completion,
        workflow_id: "wf-abc".into(),
        workflow_name: "Sub Workflow".into(),
    });
    let json = serde_json::to_string(&step).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "workflow");
    assert_eq!(v["phase"], "completion");
    assert_eq!(v["workflow_id"], "wf-abc");
    assert_eq!(v["workflow_name"], "Sub Workflow");

    let back: UnifiedStep = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn unified_step_unknown_type_preserved_as_other() {
    // Runner emits step types beyond the canonical four (e.g. `gate`,
    // `screenshot`, `native_accessibility`). `UnifiedStep` must preserve
    // those verbatim under the `Other` variant rather than failing to parse.
    let raw = r#"{"type":"native_accessibility","id":"n1","name":"audit","extra":42}"#;
    let parsed: UnifiedStep = serde_json::from_str(raw).unwrap();
    assert!(matches!(parsed, UnifiedStep::Other(_)));
    assert_eq!(parsed.step_type(), Some("native_accessibility"));
    assert!(parsed.as_canonical().is_none());

    // Round-trip losslessly.
    let re = serde_json::to_string(&parsed).unwrap();
    let original: Value = serde_json::from_str(raw).unwrap();
    let roundtrip: Value = serde_json::from_str(&re).unwrap();
    assert_eq!(original, roundtrip);
}

#[test]
fn unified_step_canonical_still_parses_typed() {
    // The catch-all must NOT swallow canonical variants — they must still
    // land in `UnifiedStep::Canonical`.
    let raw = r#"{"type":"command","id":"x","name":"y","phase":"setup"}"#;
    let parsed: UnifiedStep = serde_json::from_str(raw).unwrap();
    assert!(matches!(
        parsed,
        UnifiedStep::Canonical(CanonicalStep::Command(_))
    ));
    assert_eq!(parsed.step_type(), Some("command"));
    assert!(parsed.as_canonical().is_some());
}

#[test]
fn unified_step_command_elides_none_and_empty() {
    // Optional fields (None) and empty collections must be omitted from the
    // wire output, not emitted as `null` / `[]` / `{}`.
    let step = UnifiedStep::command(CommandStep {
        base: BaseStepFields {
            id: "s".into(),
            name: "n".into(),
            ..Default::default()
        },
        phase: CommandStepPhase::Setup,
        ..Default::default()
    });
    let json = serde_json::to_string(&step).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    for absent in [
        "mode",
        "command",
        "working_directory",
        "inputs",
        "extract",
        "depends_on",
        "criterion_ids",
        "retry",
        "required",
        "skill_origin",
        "verification_category",
    ] {
        assert!(
            v.get(absent).is_none(),
            "expected `{absent}` to be omitted, got: {json}"
        );
    }
}

#[test]
fn http_method_serializes_uppercase() {
    let json = serde_json::to_string(&HttpMethod::Get).unwrap();
    assert_eq!(json, "\"GET\"");
    let json = serde_json::to_string(&HttpMethod::Post).unwrap();
    assert_eq!(json, "\"POST\"");
    let back: HttpMethod = serde_json::from_str("\"DELETE\"").unwrap();
    assert_eq!(back, HttpMethod::Delete);
}

#[test]
fn api_content_type_mime_strings_roundtrip() {
    let cases = [
        (ApiContentType::ApplicationJson, "\"application/json\""),
        (
            ApiContentType::ApplicationFormUrlEncoded,
            "\"application/x-www-form-urlencoded\"",
        ),
        (ApiContentType::TextPlain, "\"text/plain\""),
        (ApiContentType::None, "\"none\""),
    ];
    for (value, expected) in cases {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, expected);
        let back: ApiContentType = serde_json::from_str(expected).unwrap();
        assert_eq!(back, value);
    }
}

#[test]
fn api_assertion_expected_allows_string_or_number() {
    // TS source: `expected: string | number`. We keep this as
    // `serde_json::Value` so both shapes round-trip.
    let a = ApiAssertion {
        assertion_type: ApiAssertionType::StatusCode,
        expected: json!(200),
        json_path: None,
        header_name: None,
        operator: Some(ApiAssertionOperator::Equals),
    };
    let json = serde_json::to_string(&a).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "status_code");
    assert_eq!(v["expected"], 200);
    assert_eq!(v["operator"], "equals");

    let a_str = ApiAssertion {
        assertion_type: ApiAssertionType::BodyContains,
        expected: json!("hello"),
        json_path: None,
        header_name: None,
        operator: None,
    };
    let json = serde_json::to_string(&a_str).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["expected"], "hello");
}

#[test]
fn unified_workflow_fixture_parses() {
    let raw = include_str!("../../tests/fixtures/unified_workflow_frame_sample.json");
    let parsed: UnifiedWorkflow =
        serde_json::from_str(raw).expect("parse unified workflow fixture");
    let re_serialized = serde_json::to_string(&parsed).unwrap();
    let original_value: Value = serde_json::from_str(raw).unwrap();
    let roundtrip_value: Value = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(original_value, roundtrip_value);
}

// ============================================================================
// ── DB round-trip validation ───
//
// Fixtures under `tests/fixtures/` stand in for real DB-persisted workflow
// rows. These tests pin down that populated step arrays survive a
// JSON → typed → JSON round-trip with no drift, and that the
// `UnifiedStep::Other` fallback preserves runner-specific / forward-compatible
// step types verbatim.
// ============================================================================

#[test]
fn unified_workflow_full_fixture_roundtrips() {
    let raw = include_str!("../../tests/fixtures/unified_workflow_full_sample.json");
    let parsed: UnifiedWorkflow =
        serde_json::from_str(raw).expect("parse full unified workflow fixture");
    let re_serialized = serde_json::to_string(&parsed).unwrap();
    let original_value: Value = serde_json::from_str(raw).unwrap();
    let roundtrip_value: Value = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(
        original_value, roundtrip_value,
        "full-coverage workflow fixture drifted on round-trip"
    );
}

#[test]
fn unknown_step_fixture_roundtrips() {
    let raw = include_str!("../../tests/fixtures/workflow_with_unknown_step_sample.json");
    let parsed: UnifiedWorkflow =
        serde_json::from_str(raw).expect("parse unknown-step unified workflow fixture");
    let re_serialized = serde_json::to_string(&parsed).unwrap();
    let original_value: Value = serde_json::from_str(raw).unwrap();
    let roundtrip_value: Value = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(
        original_value, roundtrip_value,
        "unknown-step workflow fixture drifted on round-trip"
    );
}

#[test]
fn full_fixture_every_step_decodes_as_full_runner_step() {
    // Every step in the full-coverage fixture must decode cleanly into
    // `FullRunnerStep`, its `step_type()` must match the wire `"type"` tag,
    // and it must re-serialize back to the exact same JSON value.
    let raw = include_str!("../../tests/fixtures/unified_workflow_full_sample.json");
    let wf: UnifiedWorkflow = serde_json::from_str(raw).unwrap();

    let phase_arrays: [(&str, &Vec<Value>); 4] = [
        ("setup_steps", &wf.setup_steps),
        ("verification_steps", &wf.verification_steps),
        ("agentic_steps", &wf.agentic_steps),
        ("completion_steps", &wf.completion_steps),
    ];

    let mut seen_types: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (phase, steps) in phase_arrays.iter() {
        for (idx, step) in steps.iter().enumerate() {
            let wire_type = step
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{}[{}]: missing `type` discriminator", phase, idx))
                .to_string();

            // Decode as FullRunnerStep.
            let decoded: FullRunnerStep = serde_json::from_value(step.clone()).unwrap_or_else(|e| {
                panic!(
                    "{}[{}] (type={:?}): failed to decode as FullRunnerStep: {}",
                    phase, idx, wire_type, e
                )
            });

            // Variant's step_type() must match the wire tag.
            assert_eq!(
                decoded.step_type(),
                wire_type.as_str(),
                "{}[{}]: variant step_type() mismatch",
                phase,
                idx
            );

            // Re-serialize and compare as JSON Value.
            let back = serde_json::to_value(&decoded).unwrap();
            assert_eq!(
                &back, step,
                "{}[{}] (type={:?}): round-trip drift",
                phase, idx, wire_type
            );

            seen_types.insert(wire_type);
        }
    }

    // Minimum coverage set — fixture must exercise at least these variants.
    let required: [&str; 11] = [
        "command",
        "prompt",
        "ui_bridge",
        "workflow",
        "code_execution",
        "native_accessibility",
        "restart_process",
        "save_workflow_artifact",
        "ui_bridge_design_audit",
        "ui_bridge_visual_assertion",
        "workflow_fixup",
    ];
    for tag in required.iter() {
        assert!(
            seen_types.contains(*tag),
            "full-coverage fixture missing required step type {:?}; saw {:?}",
            tag,
            seen_types
        );
    }
}

#[test]
fn unknown_fixture_unified_step_fallback() {
    // Every step in the unknown-step fixture must decode as `UnifiedStep`.
    // Canonical tags (`command` / `prompt` / `ui_bridge` / `workflow`) land in
    // `UnifiedStep::Canonical`; everything else lands in `UnifiedStep::Other`
    // and round-trips losslessly.
    let raw = include_str!("../../tests/fixtures/workflow_with_unknown_step_sample.json");
    let wf: UnifiedWorkflow = serde_json::from_str(raw).unwrap();

    let phase_arrays: [(&str, &Vec<Value>); 4] = [
        ("setup_steps", &wf.setup_steps),
        ("verification_steps", &wf.verification_steps),
        ("agentic_steps", &wf.agentic_steps),
        ("completion_steps", &wf.completion_steps),
    ];

    let canonical_tags = ["command", "prompt", "ui_bridge", "workflow"];

    for (phase, steps) in phase_arrays.iter() {
        for (idx, step) in steps.iter().enumerate() {
            let wire_type = step
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{}[{}]: missing `type`", phase, idx))
                .to_string();

            let decoded: UnifiedStep =
                serde_json::from_value(step.clone()).unwrap_or_else(|e| {
                    panic!(
                        "{}[{}] (type={:?}): failed to decode as UnifiedStep: {}",
                        phase, idx, wire_type, e
                    )
                });

            if canonical_tags.contains(&wire_type.as_str()) {
                assert!(
                    matches!(decoded, UnifiedStep::Canonical(_)),
                    "{}[{}] (type={:?}): canonical tag did not land in Canonical variant",
                    phase,
                    idx,
                    wire_type
                );
            } else {
                assert!(
                    matches!(decoded, UnifiedStep::Other(_)),
                    "{}[{}] (type={:?}): non-canonical tag should land in Other variant",
                    phase,
                    idx,
                    wire_type
                );
            }

            // Lossless round-trip.
            let back = serde_json::to_value(&decoded).unwrap();
            assert_eq!(
                &back, step,
                "{}[{}] (type={:?}): UnifiedStep round-trip drift",
                phase, idx, wire_type
            );
        }
    }
}

// ============================================================================
// ── task_run ─────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::task_run::*;

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn task_run_status_snake_case() {
    let cases = [
        (TaskRunStatus::Running, "\"running\""),
        (TaskRunStatus::Complete, "\"complete\""),
        (TaskRunStatus::Failed, "\"failed\""),
        (TaskRunStatus::Stopped, "\"stopped\""),
    ];
    for (status, expected) in cases {
        let json = serde_json::to_string(&status).expect("serialize");
        assert_eq!(json, expected, "TaskRunStatus serialization mismatch");
        let back: TaskRunStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, status);
    }
}

#[test]
fn task_type_snake_case() {
    let cases = [
        (TaskType::Task, "\"task\""),
        (TaskType::Automation, "\"automation\""),
        (TaskType::Scheduled, "\"scheduled\""),
    ];
    for (ty, expected) in cases {
        let json = serde_json::to_string(&ty).expect("serialize");
        assert_eq!(json, expected, "TaskType serialization mismatch");
        let back: TaskType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ty);
    }
}

#[test]
fn task_run_finding_category_snake_case() {
    let cases = [
        (TaskRunFindingCategory::CodeBug, "\"code_bug\""),
        (TaskRunFindingCategory::Security, "\"security\""),
        (TaskRunFindingCategory::Performance, "\"performance\""),
        (TaskRunFindingCategory::Todo, "\"todo\""),
        (TaskRunFindingCategory::Enhancement, "\"enhancement\""),
        (TaskRunFindingCategory::ConfigIssue, "\"config_issue\""),
        (TaskRunFindingCategory::TestIssue, "\"test_issue\""),
        (TaskRunFindingCategory::Documentation, "\"documentation\""),
        (TaskRunFindingCategory::RuntimeIssue, "\"runtime_issue\""),
        (TaskRunFindingCategory::AlreadyFixed, "\"already_fixed\""),
        (
            TaskRunFindingCategory::ExpectedBehavior,
            "\"expected_behavior\"",
        ),
        (TaskRunFindingCategory::DataMigration, "\"data_migration\""),
        (TaskRunFindingCategory::Warning, "\"warning\""),
    ];
    for (cat, expected) in cases {
        let json = serde_json::to_string(&cat).expect("serialize");
        assert_eq!(json, expected, "TaskRunFindingCategory mismatch");
        let back: TaskRunFindingCategory = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, cat);
    }
}

#[test]
fn task_run_finding_severity_snake_case() {
    let cases = [
        (TaskRunFindingSeverity::Critical, "\"critical\""),
        (TaskRunFindingSeverity::High, "\"high\""),
        (TaskRunFindingSeverity::Medium, "\"medium\""),
        (TaskRunFindingSeverity::Low, "\"low\""),
        (TaskRunFindingSeverity::Info, "\"info\""),
    ];
    for (sev, expected) in cases {
        let json = serde_json::to_string(&sev).expect("serialize");
        assert_eq!(json, expected);
        let back: TaskRunFindingSeverity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, sev);
    }
}

#[test]
fn task_run_finding_status_snake_case() {
    let cases = [
        (TaskRunFindingStatus::Detected, "\"detected\""),
        (TaskRunFindingStatus::InProgress, "\"in_progress\""),
        (TaskRunFindingStatus::NeedsInput, "\"needs_input\""),
        (TaskRunFindingStatus::Resolved, "\"resolved\""),
        (TaskRunFindingStatus::WontFix, "\"wont_fix\""),
        (TaskRunFindingStatus::Deferred, "\"deferred\""),
    ];
    for (st, expected) in cases {
        let json = serde_json::to_string(&st).expect("serialize");
        assert_eq!(json, expected);
        let back: TaskRunFindingStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, st);
    }
}

#[test]
fn task_run_finding_action_type_snake_case() {
    let cases = [
        (TaskRunFindingActionType::AutoFix, "\"auto_fix\""),
        (
            TaskRunFindingActionType::NeedsUserInput,
            "\"needs_user_input\"",
        ),
        (TaskRunFindingActionType::Manual, "\"manual\""),
        (
            TaskRunFindingActionType::Informational,
            "\"informational\"",
        ),
    ];
    for (at, expected) in cases {
        let json = serde_json::to_string(&at).expect("serialize");
        assert_eq!(json, expected);
        let back: TaskRunFindingActionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, at);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_task_run_backend() -> TaskRunBackend {
    TaskRunBackend {
        id: "run-001".to_string(),
        project_id: Some("proj-alpha".to_string()),
        created_by_user_id: Some("user-7".to_string()),
        runner_id: Some("runner-primary".to_string()),
        task_name: "Nightly verify".to_string(),
        prompt: "Run the full verification suite and report".to_string(),
        status: TaskRunStatus::Complete,
        sessions_count: 3,
        max_sessions: Some(5),
        auto_continue: true,
        output_summary: Some("3 sessions, all green".to_string()),
        full_output_stored: true,
        error_message: None,
        duration_seconds: Some(912),
        created_at: "2026-04-14T01:00:00Z".to_string(),
        updated_at: "2026-04-14T01:15:12Z".to_string(),
        completed_at: Some("2026-04-14T01:15:12Z".to_string()),
    }
}

fn sample_task_run_finding() -> TaskRunFinding {
    TaskRunFinding {
        id: "find-1".to_string(),
        task_run_id: "run-001".to_string(),
        category: TaskRunFindingCategory::CodeBug,
        severity: TaskRunFindingSeverity::High,
        status: TaskRunFindingStatus::Detected,
        action_type: TaskRunFindingActionType::AutoFix,
        signature_hash: Some("deadbeef".to_string()),
        title: "Unused variable `foo`".to_string(),
        description: "Variable `foo` is assigned but never read.".to_string(),
        resolution: None,
        file_path: Some("src/main.rs".to_string()),
        line_number: Some(42),
        column_number: Some(9),
        code_snippet: Some("let foo = 1;".to_string()),
        detected_in_session: 2,
        resolved_in_session: None,
        needs_input: false,
        question: None,
        input_options: None,
        user_response: None,
        detected_at: "2026-04-14T01:05:00Z".to_string(),
        resolved_at: None,
        updated_at: "2026-04-14T01:05:00Z".to_string(),
    }
}

fn sample_finding_summary() -> TaskRunFindingSummary {
    let mut by_category = HashMap::new();
    by_category.insert("code_bug".to_string(), 2);
    by_category.insert("todo".to_string(), 1);
    let mut by_severity = HashMap::new();
    by_severity.insert("high".to_string(), 1);
    by_severity.insert("low".to_string(), 2);
    let mut by_status = HashMap::new();
    by_status.insert("detected".to_string(), 2);
    by_status.insert("resolved".to_string(), 1);
    TaskRunFindingSummary {
        by_category,
        by_severity,
        by_status,
        total: 3,
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn task_run_fully_populated_roundtrips() {
    let tr = TaskRun {
        id: "run-abc".to_string(),
        task_name: "Refactor error paths".to_string(),
        prompt: Some("Please refactor error handling in foo.rs".to_string()),
        task_type: TaskType::Task,
        config_id: Some("cfg-01".to_string()),
        workflow_name: Some("refactor-flow".to_string()),
        status: TaskRunStatus::Running,
        sessions_count: 1,
        max_sessions: Some(4),
        auto_continue: true,
        output_log: "Session 1 starting...\n".to_string(),
        error_message: None,
        summary: Some("Refactor in progress".to_string()),
        goal_achieved: Some(false),
        remaining_work: Some("Tests still failing".to_string()),
        summary_generated_at: Some("2026-04-14T02:00:00Z".to_string()),
        created_at: "2026-04-14T00:30:00Z".to_string(),
        updated_at: "2026-04-14T02:00:00Z".to_string(),
        completed_at: None,
    };
    let json = serde_json::to_string(&tr).expect("serialize");
    let back: TaskRun = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("serialize round");
    assert_eq!(json, json2);
}

#[test]
fn task_run_minimal_elides_none_fields() {
    let tr = TaskRun {
        id: "min".to_string(),
        task_name: "Minimal".to_string(),
        prompt: None,
        task_type: TaskType::Automation,
        config_id: None,
        workflow_name: None,
        status: TaskRunStatus::Running,
        sessions_count: 0,
        max_sessions: None,
        auto_continue: false,
        output_log: String::new(),
        error_message: None,
        summary: None,
        goal_achieved: None,
        remaining_work: None,
        summary_generated_at: None,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:00:00Z".to_string(),
        completed_at: None,
    };
    let json = serde_json::to_string(&tr).expect("serialize");
    assert!(!json.contains("null"), "none fields must be elided: {json}");
    assert!(!json.contains("\"prompt\""));
    assert!(!json.contains("\"completed_at\""));
    let back: TaskRun = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_backend_fully_populated_roundtrips() {
    let b = sample_task_run_backend();
    let json = serde_json::to_string(&b).expect("serialize");
    let back: TaskRunBackend = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_session_fully_populated_roundtrips() {
    let s = TaskRunSession {
        id: "sess-1".to_string(),
        task_run_id: "run-001".to_string(),
        session_number: 2,
        started_at: "2026-04-14T01:00:00Z".to_string(),
        ended_at: Some("2026-04-14T01:05:00Z".to_string()),
        duration_seconds: Some(300),
        output_summary: Some("Session 2 summary".to_string()),
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let back: TaskRunSession = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_finding_fully_populated_roundtrips() {
    let f = TaskRunFinding {
        id: "f-1".to_string(),
        task_run_id: "t-1".to_string(),
        category: TaskRunFindingCategory::Security,
        severity: TaskRunFindingSeverity::Critical,
        status: TaskRunFindingStatus::NeedsInput,
        action_type: TaskRunFindingActionType::NeedsUserInput,
        signature_hash: Some("hash-abc".to_string()),
        title: "Hardcoded API key".to_string(),
        description: "Found an API key in a committed file.".to_string(),
        resolution: Some("Rotate and move to env.".to_string()),
        file_path: Some("config/app.rs".to_string()),
        line_number: Some(17),
        column_number: Some(5),
        code_snippet: Some("const KEY: &str = \"sk-test\";".to_string()),
        detected_in_session: 1,
        resolved_in_session: Some(2),
        needs_input: true,
        question: Some("Confirm rotation?".to_string()),
        input_options: Some(vec!["yes".to_string(), "no".to_string()]),
        user_response: Some("yes".to_string()),
        detected_at: "2026-04-14T02:00:00Z".to_string(),
        resolved_at: Some("2026-04-14T02:10:00Z".to_string()),
        updated_at: "2026-04-14T02:10:00Z".to_string(),
    };
    let json = serde_json::to_string(&f).expect("serialize");
    let back: TaskRunFinding = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_finding_summary_roundtrips() {
    let s = sample_finding_summary();
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: TaskRunFindingSummary = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["total"], 3);
}

#[test]
fn task_run_backend_detail_flattens_task() {
    let detail = TaskRunBackendDetail {
        base: sample_task_run_backend(),
        sessions: vec![TaskRunSession {
            id: "sess-1".to_string(),
            task_run_id: "run-001".to_string(),
            session_number: 1,
            started_at: "2026-04-14T01:00:00Z".to_string(),
            ended_at: Some("2026-04-14T01:05:00Z".to_string()),
            duration_seconds: Some(300),
            output_summary: Some("done".to_string()),
        }],
        findings: vec![sample_task_run_finding()],
        finding_summary: sample_finding_summary(),
    };
    let json = serde_json::to_string(&detail).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // `#[serde(flatten)]` should hoist TaskRunBackend fields to the top level.
    assert_eq!(v["id"], "run-001");
    assert_eq!(v["task_name"], "Nightly verify");
    assert!(v.get("task").is_none(), "flatten should inline the task");
    assert!(v["sessions"].is_array());
    assert!(v["findings"].is_array());
    assert_eq!(v["finding_summary"]["total"], 3);
    let back: TaskRunBackendDetail = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json, &json2);
}

#[test]
fn task_run_create_fully_populated_roundtrips() {
    let c = TaskRunCreate {
        id: Some("client-id".to_string()),
        project_id: Some("proj-1".to_string()),
        runner_id: Some("runner-1".to_string()),
        task_name: "Create me".to_string(),
        prompt: Some("do stuff".to_string()),
        max_sessions: Some(3),
        auto_continue: Some(true),
        task_type: Some(TaskType::Task),
        config_id: Some("cfg-1".to_string()),
        workflow_name: Some("wf-1".to_string()),
        execution_steps_json: Some("[]".to_string()),
        log_sources_json: Some("{}".to_string()),
    };
    let json = serde_json::to_string(&c).expect("serialize");
    let back: TaskRunCreate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_create_default_elides_all_options() {
    let c = TaskRunCreate {
        task_name: "only-required".to_string(),
        ..Default::default()
    };
    let json = serde_json::to_string(&c).expect("serialize");
    // Only the required `task_name` should appear on the wire.
    assert_eq!(json, r#"{"task_name":"only-required"}"#);
    let back: TaskRunCreate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_update_roundtrips() {
    let u = TaskRunUpdate {
        status: Some(TaskRunStatus::Complete),
        sessions_count: Some(4),
        output_summary: Some("done".to_string()),
        full_output: Some("log\nlines\n".to_string()),
        full_output_stored: Some(true),
        error_message: None,
        duration_seconds: Some(600),
        completed_at: Some("2026-04-14T02:15:00Z".to_string()),
    };
    let json = serde_json::to_string(&u).expect("serialize");
    let back: TaskRunUpdate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
    assert!(!json.contains("error_message"));
}

#[test]
fn task_run_finding_create_roundtrips() {
    let c = TaskRunFindingCreate {
        id: Some("f-new".to_string()),
        category: TaskRunFindingCategory::Enhancement,
        severity: TaskRunFindingSeverity::Low,
        status: Some(TaskRunFindingStatus::Detected),
        action_type: Some(TaskRunFindingActionType::Manual),
        signature_hash: Some("abc".to_string()),
        title: "Add docs".to_string(),
        description: "Public API `bar` has no docstring.".to_string(),
        resolution: None,
        file_path: Some("src/bar.rs".to_string()),
        line_number: Some(1),
        column_number: None,
        code_snippet: Some("pub fn bar() {}".to_string()),
        detected_in_session: 1,
        needs_input: Some(false),
        question: None,
        input_options: None,
    };
    let json = serde_json::to_string(&c).expect("serialize");
    let back: TaskRunFindingCreate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_finding_update_roundtrips() {
    let u = TaskRunFindingUpdate {
        status: Some(TaskRunFindingStatus::Resolved),
        resolution: Some("Fixed in commit abc123".to_string()),
        resolved_in_session: Some(3),
        resolved_at: Some("2026-04-14T03:00:00Z".to_string()),
        user_response: Some("ack".to_string()),
    };
    let json = serde_json::to_string(&u).expect("serialize");
    let back: TaskRunFindingUpdate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn run_prompt_response_data_roundtrips() {
    let d = RunPromptResponseData {
        output: Some("hello".to_string()),
        response: Some("world".to_string()),
    };
    let json = serde_json::to_string(&d).expect("serialize");
    let back: RunPromptResponseData = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn run_prompt_response_roundtrips() {
    let r = RunPromptResponse {
        success: true,
        task_run_id: Some("run-1".to_string()),
        session_id: Some("sess-1".to_string()),
        state_file: Some("/tmp/state.json".to_string()),
        log_file: Some("/tmp/log.txt".to_string()),
        pid: Some(12345),
        error: None,
        output: Some("immediate output".to_string()),
        data: Some(RunPromptResponseData {
            output: Some("out".to_string()),
            response: Some("resp".to_string()),
        }),
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: RunPromptResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn run_prompt_request_roundtrips() {
    let r = RunPromptRequest {
        name: "Review PR".to_string(),
        content: "Please review the attached PR and summarize changes".to_string(),
        max_sessions: Some(2),
        display_prompt: Some("Review PR #42".to_string()),
        timeout_seconds: Some(600),
        context: Some("CI is red".to_string()),
        image_paths: Some(vec!["/tmp/a.png".to_string(), "/tmp/b.png".to_string()]),
        video_paths: Some(vec!["/tmp/c.mp4".to_string()]),
        trace_path: Some("/tmp/trace.jsonl".to_string()),
        max_video_frames: Some(20),
        max_trace_screenshots: Some(5),
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: RunPromptRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn create_task_run_request_roundtrips() {
    let r = CreateTaskRunRequest {
        task_name: "Make tests".to_string(),
        prompt: Some("Write tests for foo".to_string()),
        task_type: Some(TaskType::Task),
        config_id: Some("cfg-1".to_string()),
        workflow_name: Some("wf-1".to_string()),
        max_sessions: Some(3),
        auto_continue: Some(false),
        execution_steps_json: Some("[{\"step\":1}]".to_string()),
        log_sources_json: Some("{}".to_string()),
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: CreateTaskRunRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_filters_roundtrips() {
    let f = TaskRunFilters {
        project_id: Some("proj-1".to_string()),
        status: Some(TaskRunStatus::Running),
        start_date: Some("2026-04-01T00:00:00Z".to_string()),
        end_date: Some("2026-04-14T00:00:00Z".to_string()),
        offset: Some(0),
        limit: Some(50),
    };
    let json = serde_json::to_string(&f).expect("serialize");
    let back: TaskRunFilters = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_finding_filters_roundtrips() {
    let f = TaskRunFindingFilters {
        category: Some(TaskRunFindingCategory::Warning),
        severity: Some(TaskRunFindingSeverity::Medium),
        status: Some(TaskRunFindingStatus::InProgress),
    };
    let json = serde_json::to_string(&f).expect("serialize");
    let back: TaskRunFindingFilters = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn pagination_roundtrips() {
    let p = Pagination {
        total: 137,
        limit: 50,
        offset: 100,
        has_more: false,
    };
    let json = serde_json::to_string(&p).expect("serialize");
    let back: Pagination = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["total"], 137);
    assert_eq!(v["has_more"], false);
}

#[test]
fn task_run_list_response_roundtrips() {
    let resp = TaskRunListResponse {
        tasks: vec![sample_task_run_backend()],
        pagination: Pagination {
            total: 1,
            limit: 50,
            offset: 0,
            has_more: false,
        },
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: TaskRunListResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_findings_list_response_roundtrips() {
    let resp = TaskRunFindingsListResponse {
        findings: vec![sample_task_run_finding()],
        summary: sample_finding_summary(),
    };
    let json1 = serde_json::to_string(&resp).expect("serialize");
    let back: TaskRunFindingsListResponse =
        serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn findings_summary_roundtrips() {
    let mut by_severity = HashMap::new();
    by_severity.insert("critical".to_string(), 1);
    by_severity.insert("medium".to_string(), 3);
    let mut by_category = HashMap::new();
    by_category.insert("security".to_string(), 1);
    by_category.insert("todo".to_string(), 3);
    let mut by_status = HashMap::new();
    by_status.insert("detected".to_string(), 4);
    let s = FindingsSummary {
        total: 4,
        by_severity,
        by_category,
        by_status,
        recent: vec![sample_task_run_finding()],
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: FindingsSummary = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["total"], 4);
    assert_eq!(v["recent"][0]["id"], "find-1");
}

#[test]
fn check_issue_detail_roundtrips() {
    let c = CheckIssueDetail {
        file: "src/main.rs".to_string(),
        line: Some(10),
        column: Some(4),
        code: Some("E0001".to_string()),
        message: "unused import".to_string(),
        severity: "warning".to_string(),
        fixable: true,
    };
    let json = serde_json::to_string(&c).expect("serialize");
    let back: CheckIssueDetail = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn individual_check_result_roundtrips() {
    let r = IndividualCheckResult {
        name: "eslint".to_string(),
        status: "passed".to_string(),
        duration_ms: 1250,
        issues_found: 2,
        issues_fixed: 1,
        files_checked: 42,
        error_message: None,
        output: Some("eslint output".to_string()),
        issues: vec![CheckIssueDetail {
            file: "src/a.ts".to_string(),
            line: Some(1),
            column: Some(1),
            code: Some("no-unused".to_string()),
            message: "unused var".to_string(),
            severity: "warning".to_string(),
            fixable: true,
        }],
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: IndividualCheckResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn verification_step_details_roundtrips() {
    let d = VerificationStepDetails {
        step_id: "step-01".to_string(),
        phase: "verification".to_string(),
        stdout: Some("stdout lines".to_string()),
        stderr: Some("stderr lines".to_string()),
        assertions_passed: Some(9),
        assertions_total: Some(10),
        console_output: Some("console".to_string()),
        page_snapshot: Some("<html></html>".to_string()),
        exit_code: Some(0),
        check_results: Some(vec![IndividualCheckResult {
            name: "tsc".to_string(),
            status: "passed".to_string(),
            duration_ms: 800,
            issues_found: 0,
            issues_fixed: 0,
            files_checked: 10,
            error_message: None,
            output: None,
            issues: vec![],
        }]),
    };
    let json = serde_json::to_string(&d).expect("serialize");
    let back: VerificationStepDetails = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn step_execution_config_preserves_extra_keys() {
    // Put an unknown key on the wire; StepExecutionConfig#extra must capture
    // it via `#[serde(flatten)]` and preserve it through round-trip.
    let wire = r#"{"action_type":"click","target_image_id":"img-7","retries":3,"custom":{"a":1}}"#;
    let cfg: StepExecutionConfig =
        serde_json::from_str(wire).expect("deserialize with extra keys");
    assert_eq!(cfg.action_type.as_deref(), Some("click"));
    assert_eq!(cfg.target_image_id.as_deref(), Some("img-7"));
    assert_eq!(cfg.extra.get("retries"), Some(&json!(3)));
    assert_eq!(cfg.extra.get("custom"), Some(&json!({"a": 1})));
    let round = serde_json::to_string(&cfg).expect("serialize");
    // HashMap ordering isn't stable; compare as Value.
    assert_json_values_equal(wire, &round);
}

#[test]
fn verification_step_result_roundtrips() {
    let r = VerificationStepResult {
        step_index: 3,
        step_type: "action".to_string(),
        step_name: "Click submit".to_string(),
        step_id: Some("step-03".to_string()),
        success: true,
        error: None,
        screenshot_path: Some("/tmp/shot.png".to_string()),
        started_at: Some("2026-04-14T04:00:00Z".to_string()),
        ended_at: Some("2026-04-14T04:00:02Z".to_string()),
        duration_ms: 2000,
        config: StepExecutionConfig {
            action_type: Some("click".to_string()),
            target_image_id: None,
            target_image_name: None,
            check_type: None,
            timeout_seconds: Some(10),
            extra: HashMap::new(),
        },
        verification_details: None,
        output_data: Some({
            let mut m = HashMap::new();
            m.insert("clicked".to_string(), json!(true));
            m
        }),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: VerificationStepResult = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn gate_evaluation_result_roundtrips() {
    let g = GateEvaluationResult {
        gate_name: "smoke".to_string(),
        required_step_ids: vec!["s1".to_string(), "s2".to_string()],
        passed_step_ids: vec!["s1".to_string()],
        failed_step_ids: vec!["s2".to_string()],
        missing_step_ids: vec![],
        passed: false,
    };
    let json = serde_json::to_string(&g).expect("serialize");
    let back: GateEvaluationResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn verification_phase_result_roundtrips() {
    let p = VerificationPhaseResult {
        iteration: 1,
        all_passed: false,
        total_steps: 5,
        passed_steps: 4,
        failed_steps: 1,
        skipped_steps: 0,
        total_duration_ms: 8500,
        step_results: vec![],
        critical_failure: false,
        gate_results: vec![GateEvaluationResult {
            gate_name: "all".to_string(),
            required_step_ids: vec!["s1".to_string()],
            passed_step_ids: vec!["s1".to_string()],
            failed_step_ids: vec![],
            missing_step_ids: vec![],
            passed: true,
        }],
        gate_based_evaluation: true,
    };
    let json = serde_json::to_string(&p).expect("serialize");
    let back: VerificationPhaseResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn verification_result_response_roundtrips() {
    let r = VerificationResultResponse {
        id: "vr-1".to_string(),
        task_run_id: "run-abc".to_string(),
        iteration: 2,
        all_passed: true,
        total_steps: 7,
        passed_steps: 7,
        failed_steps: 0,
        skipped_steps: 0,
        total_duration_ms: 12_340,
        critical_failure: false,
        result_json: VerificationPhaseResult {
            iteration: 2,
            all_passed: true,
            total_steps: 7,
            passed_steps: 7,
            failed_steps: 0,
            skipped_steps: 0,
            total_duration_ms: 12_340,
            step_results: vec![],
            critical_failure: false,
            gate_results: vec![],
            gate_based_evaluation: false,
        },
        created_at: "2026-04-14T05:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: VerificationResultResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn verification_results_list_response_roundtrips() {
    let resp = VerificationResultsListResponse {
        task_run_id: "run-abc".to_string(),
        results: vec![],
        count: 0,
        passed_iterations: 0,
        failed_iterations: 0,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: VerificationResultsListResponse =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn task_run_finding_response_type_alias_roundtrips() {
    // TaskRunFindingResponse is a type alias for TaskRunFinding — the wire
    // shape must be identical.
    let f = sample_task_run_finding();
    let json = serde_json::to_string(&f).expect("serialize as TaskRunFinding");
    let back: TaskRunFindingResponse =
        serde_json::from_str(&json).expect("deserialize as response alias");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_eq!(json, json2);
}

// ============================================================================
// ── state_machine ────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::state_machine::*;

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn standard_action_type_serialization() {
    let cases = [
        (StandardActionType::Click, "\"click\""),
        (StandardActionType::DoubleClick, "\"doubleClick\""),
        (StandardActionType::RightClick, "\"rightClick\""),
        (StandardActionType::Type, "\"type\""),
        (StandardActionType::Clear, "\"clear\""),
        (StandardActionType::Select, "\"select\""),
        (StandardActionType::Focus, "\"focus\""),
        (StandardActionType::Blur, "\"blur\""),
        (StandardActionType::Hover, "\"hover\""),
        (StandardActionType::Scroll, "\"scroll\""),
        (StandardActionType::Check, "\"check\""),
        (StandardActionType::Uncheck, "\"uncheck\""),
        (StandardActionType::Toggle, "\"toggle\""),
        (StandardActionType::SetValue, "\"setValue\""),
        (StandardActionType::Drag, "\"drag\""),
        (StandardActionType::Submit, "\"submit\""),
        (StandardActionType::Reset, "\"reset\""),
        (StandardActionType::Wait, "\"wait\""),
        (StandardActionType::Navigate, "\"navigate\""),
    ];
    for (action, expected) in cases {
        let json = serde_json::to_string(&action).expect("serialize");
        assert_eq!(json, expected, "StandardActionType mismatch");
        let back: StandardActionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, action);
    }
}

#[test]
fn scroll_direction_lowercase() {
    let cases = [
        (ScrollDirection::Up, "\"up\""),
        (ScrollDirection::Down, "\"down\""),
        (ScrollDirection::Left, "\"left\""),
        (ScrollDirection::Right, "\"right\""),
    ];
    for (sd, expected) in cases {
        let json = serde_json::to_string(&sd).expect("serialize");
        assert_eq!(json, expected);
        let back: ScrollDirection = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, sd);
    }
}

#[test]
fn mouse_button_lowercase() {
    let cases = [
        (MouseButton::Left, "\"left\""),
        (MouseButton::Right, "\"right\""),
        (MouseButton::Middle, "\"middle\""),
    ];
    for (mb, expected) in cases {
        let json = serde_json::to_string(&mb).expect("serialize");
        assert_eq!(json, expected);
        let back: MouseButton = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, mb);
    }
}

#[test]
fn initial_states_source_lowercase() {
    let cases = [
        (InitialStatesSource::Defaults, "\"defaults\""),
        (InitialStatesSource::Workflow, "\"workflow\""),
        (InitialStatesSource::Override, "\"override\""),
    ];
    for (src, expected) in cases {
        let json = serde_json::to_string(&src).expect("serialize");
        assert_eq!(json, expected);
        let back: InitialStatesSource = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, src);
    }
}

#[test]
fn discovery_strategy_lowercase() {
    let cases = [
        (DiscoveryStrategy::Auto, "\"auto\""),
        (DiscoveryStrategy::Fingerprint, "\"fingerprint\""),
    ];
    for (ds, expected) in cases {
        let json = serde_json::to_string(&ds).expect("serialize");
        assert_eq!(json, expected);
        let back: DiscoveryStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ds);
    }
}

#[test]
fn transition_action_value_untagged() {
    // Single variant: wire is a bare string.
    let single = TransitionActionValue::Single("hello".to_string());
    let json = serde_json::to_string(&single).expect("serialize Single");
    assert_eq!(json, "\"hello\"");
    let back: TransitionActionValue = serde_json::from_str(&json).expect("deserialize Single");
    assert!(matches!(back, TransitionActionValue::Single(ref s) if s == "hello"));

    // Multiple variant: wire is a string array.
    let multi = TransitionActionValue::Multiple(vec!["a".to_string(), "b".to_string()]);
    let json = serde_json::to_string(&multi).expect("serialize Multiple");
    assert_eq!(json, r#"["a","b"]"#);
    let back: TransitionActionValue = serde_json::from_str(&json).expect("deserialize Multiple");
    assert!(matches!(back, TransitionActionValue::Multiple(ref v) if v.len() == 2));
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_state_machine_config() -> StateMachineConfig {
    StateMachineConfig {
        id: "cfg-1".to_string(),
        name: "My app".to_string(),
        description: Some("Demo config".to_string()),
        render_count: 12,
        element_count: 34,
        include_html_ids: true,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T01:00:00Z".to_string(),
    }
}

fn sample_domain_knowledge() -> DomainKnowledge {
    DomainKnowledge {
        id: "dk-1".to_string(),
        title: "Login flow".to_string(),
        content: "Users enter email then password.".to_string(),
        tags: vec!["auth".to_string(), "login".to_string()],
    }
}

fn sample_state_machine_state() -> StateMachineState {
    let mut extra_metadata = HashMap::new();
    extra_metadata.insert("source".to_string(), json!("discovery"));
    StateMachineState {
        id: "state-uuid-1".to_string(),
        config_id: "cfg-1".to_string(),
        state_id: "home".to_string(),
        name: "Home Page".to_string(),
        description: Some("Landing page".to_string()),
        element_ids: vec!["el-1".to_string(), "el-2".to_string()],
        render_ids: vec!["r-1".to_string()],
        confidence: 0.92,
        acceptance_criteria: vec!["has header".to_string()],
        extra_metadata,
        domain_knowledge: vec![sample_domain_knowledge()],
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:05:00Z".to_string(),
    }
}

fn sample_transition_action() -> TransitionAction {
    TransitionAction {
        action_type: StandardActionType::Click,
        target: Some("submit-btn".to_string()),
        text: None,
        clear_first: None,
        type_delay: None,
        value: None,
        select_by_label: None,
        url: None,
        delay_ms: None,
        scroll_direction: None,
        scroll_amount: None,
        drag_target: None,
        drag_target_position: None,
        drag_steps: None,
        drag_hold_delay: None,
        drag_html5: None,
        button: Some(MouseButton::Left),
        position: Some(Point { x: 10.0, y: 20.0 }),
    }
}

fn sample_state_machine_transition() -> StateMachineTransition {
    let mut extra_metadata = HashMap::new();
    extra_metadata.insert("source".to_string(), json!("editor"));
    StateMachineTransition {
        id: "trans-uuid-1".to_string(),
        config_id: "cfg-1".to_string(),
        transition_id: "go-to-login".to_string(),
        name: "Go to login".to_string(),
        from_states: vec!["home".to_string()],
        activate_states: vec!["login".to_string()],
        exit_states: vec!["home".to_string()],
        actions: vec![sample_transition_action()],
        path_cost: 1.0,
        stays_visible: false,
        extra_metadata,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:10:00Z".to_string(),
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn point_roundtrips() {
    let p = Point { x: 1.5, y: -3.25 };
    let json = serde_json::to_string(&p).expect("serialize");
    assert_eq!(json, r#"{"x":1.5,"y":-3.25}"#);
    let back: Point = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(p, back);
}

#[test]
fn transition_action_fully_populated_roundtrips() {
    let a = TransitionAction {
        action_type: StandardActionType::Drag,
        target: Some("el-1".to_string()),
        text: Some("hello".to_string()),
        clear_first: Some(true),
        type_delay: Some(25.0),
        value: Some(TransitionActionValue::Multiple(vec![
            "a".to_string(),
            "b".to_string(),
        ])),
        select_by_label: Some(false),
        url: Some("https://example.com".to_string()),
        delay_ms: Some(500.0),
        scroll_direction: Some(ScrollDirection::Down),
        scroll_amount: Some(200.0),
        drag_target: Some("el-2".to_string()),
        drag_target_position: Some("center".to_string()),
        drag_steps: Some(8.0),
        drag_hold_delay: Some(50.0),
        drag_html5: Some(true),
        button: Some(MouseButton::Right),
        position: Some(Point { x: 5.0, y: 7.5 }),
    };
    let json = serde_json::to_string(&a).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // action_type is renamed to "type" on the wire.
    assert_eq!(v["type"], "drag");
    assert!(v.get("action_type").is_none());
    let back: TransitionAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn transition_action_minimal_elides_none_fields() {
    let a = TransitionAction {
        action_type: StandardActionType::Click,
        target: None,
        text: None,
        clear_first: None,
        type_delay: None,
        value: None,
        select_by_label: None,
        url: None,
        delay_ms: None,
        scroll_direction: None,
        scroll_amount: None,
        drag_target: None,
        drag_target_position: None,
        drag_steps: None,
        drag_hold_delay: None,
        drag_html5: None,
        button: None,
        position: None,
    };
    let json = serde_json::to_string(&a).expect("serialize");
    // Only the renamed `type` key should remain.
    assert_eq!(json, r#"{"type":"click"}"#);
    let back: TransitionAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn domain_knowledge_roundtrips() {
    let dk = sample_domain_knowledge();
    let json = serde_json::to_string(&dk).expect("serialize");
    let back: DomainKnowledge = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
    assert_eq!(dk, back);
}

#[test]
fn state_machine_config_roundtrips() {
    let c = sample_state_machine_config();
    let json = serde_json::to_string(&c).expect("serialize");
    let back: StateMachineConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(c, back);
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn state_machine_config_create_roundtrips() {
    let c = StateMachineConfigCreate {
        name: "New config".to_string(),
        description: Some("Created from UI".to_string()),
    };
    let json = serde_json::to_string(&c).expect("serialize");
    let back: StateMachineConfigCreate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
    assert_eq!(c, back);
}

#[test]
fn state_machine_config_update_roundtrips() {
    let u = StateMachineConfigUpdate {
        name: Some("Renamed".to_string()),
        description: None,
    };
    let json = serde_json::to_string(&u).expect("serialize");
    assert!(!json.contains("description"));
    let back: StateMachineConfigUpdate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
    assert_eq!(u, back);
}

#[test]
fn state_machine_config_full_flattens_config() {
    let full = StateMachineConfigFull {
        config: sample_state_machine_config(),
        states: vec![sample_state_machine_state()],
        transitions: vec![sample_state_machine_transition()],
    };
    let json = serde_json::to_string(&full).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // Base config fields must be flattened to the top level.
    assert_eq!(v["id"], "cfg-1");
    assert_eq!(v["name"], "My app");
    assert!(v.get("config").is_none(), "flatten should inline config");
    assert!(v["states"].is_array());
    assert!(v["transitions"].is_array());
    let back: StateMachineConfigFull = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json, &json2);
}

#[test]
fn state_machine_state_roundtrips() {
    let s = sample_state_machine_state();
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: StateMachineState = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_machine_state_create_roundtrips() {
    let mut extra = HashMap::new();
    extra.insert("source".to_string(), json!("manual"));
    let c = StateMachineStateCreate {
        state_id: Some("home".to_string()),
        name: "Home".to_string(),
        description: Some("Landing".to_string()),
        element_ids: Some(vec!["el-1".to_string()]),
        render_ids: Some(vec!["r-1".to_string()]),
        confidence: Some(0.8),
        acceptance_criteria: Some(vec!["contains header".to_string()]),
        extra_metadata: Some(extra),
        domain_knowledge: Some(vec![sample_domain_knowledge()]),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let back: StateMachineStateCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_machine_state_update_roundtrips() {
    let u = StateMachineStateUpdate {
        name: Some("New name".to_string()),
        description: None,
        element_ids: Some(vec!["el-2".to_string()]),
        render_ids: None,
        confidence: Some(0.99),
        acceptance_criteria: None,
        extra_metadata: None,
        domain_knowledge: None,
    };
    let json = serde_json::to_string(&u).expect("serialize");
    let back: StateMachineStateUpdate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn state_machine_transition_roundtrips() {
    let t = sample_state_machine_transition();
    let json1 = serde_json::to_string(&t).expect("serialize");
    let back: StateMachineTransition = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_machine_transition_create_roundtrips() {
    let mut extra = HashMap::new();
    extra.insert("note".to_string(), json!("from UI"));
    let c = StateMachineTransitionCreate {
        name: "Click submit".to_string(),
        from_states: vec!["login".to_string()],
        activate_states: vec!["home".to_string()],
        exit_states: vec!["login".to_string()],
        actions: vec![sample_transition_action()],
        path_cost: Some(2.5),
        stays_visible: Some(false),
        extra_metadata: Some(extra),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let back: StateMachineTransitionCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_machine_transition_update_roundtrips() {
    let u = StateMachineTransitionUpdate {
        name: Some("Renamed".to_string()),
        from_states: Some(vec!["a".to_string()]),
        activate_states: Some(vec!["b".to_string()]),
        exit_states: None,
        actions: None,
        path_cost: Some(1.0),
        stays_visible: Some(true),
        extra_metadata: None,
    };
    let json = serde_json::to_string(&u).expect("serialize");
    let back: StateMachineTransitionUpdate =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn pathfinding_request_roundtrips() {
    let r = PathfindingRequest {
        from_states: vec!["home".to_string()],
        target_states: vec!["profile".to_string(), "settings".to_string()],
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: PathfindingRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn pathfinding_step_roundtrips() {
    let s = PathfindingStep {
        transition_id: "t-1".to_string(),
        transition_name: "Click home link".to_string(),
        from_states: vec!["profile".to_string()],
        activate_states: vec!["home".to_string()],
        exit_states: vec!["profile".to_string()],
        path_cost: 1.0,
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let back: PathfindingStep = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn pathfinding_result_roundtrips() {
    let r = PathfindingResult {
        found: true,
        steps: vec![PathfindingStep {
            transition_id: "t-1".to_string(),
            transition_name: "Go home".to_string(),
            from_states: vec!["login".to_string()],
            activate_states: vec!["home".to_string()],
            exit_states: vec!["login".to_string()],
            path_cost: 1.0,
        }],
        total_cost: 1.0,
        error: None,
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: PathfindingResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn transition_execution_result_roundtrips() {
    let r = TransitionExecutionResult {
        success: false,
        transition_id: "t-broken".to_string(),
        active_states: vec!["home".to_string()],
        error: Some("element not found".to_string()),
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: TransitionExecutionResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn navigation_result_recursive_roundtrips() {
    // Outer result has two nested sub-results, exercising the recursive
    // `results: Option<Vec<NavigationResult>>` case.
    let inner_a = NavigationResult {
        success: true,
        path: vec!["home".to_string(), "profile".to_string()],
        active_states: vec!["profile".to_string()],
        target_state: Some("profile".to_string()),
        results: None,
        error: None,
    };
    let inner_b = NavigationResult {
        success: false,
        path: vec!["home".to_string()],
        active_states: vec!["home".to_string()],
        target_state: Some("settings".to_string()),
        results: None,
        error: Some("path not found".to_string()),
    };
    let outer = NavigationResult {
        success: false,
        path: vec![],
        active_states: vec!["home".to_string()],
        target_state: None,
        results: Some(vec![inner_a, inner_b]),
        error: Some("one sub-target failed".to_string()),
    };
    let json = serde_json::to_string(&outer).expect("serialize");
    let back: NavigationResult = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_eq!(json, json2);
    let v: Value = serde_json::from_str(&json).expect("parse");
    assert_eq!(v["results"].as_array().map(|a| a.len()), Some(2));
    assert_eq!(v["results"][0]["target_state"], "profile");
}

#[test]
fn active_states_result_roundtrips() {
    let r = ActiveStatesResult {
        success: true,
        active_states: vec!["home".to_string()],
        current_state: Some("home".to_string()),
        state_history: Some(vec!["login".to_string(), "home".to_string()]),
        error: None,
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: ActiveStatesResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn transition_info_roundtrips() {
    let t = TransitionInfo {
        id: "t-1".to_string(),
        from_state: "home".to_string(),
        to_state: Some("profile".to_string()),
        workflows: vec!["wf-1".to_string()],
    };
    let json = serde_json::to_string(&t).expect("serialize");
    let back: TransitionInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn available_transitions_result_roundtrips() {
    let r = AvailableTransitionsResult {
        success: true,
        transitions: vec![TransitionInfo {
            id: "t-1".to_string(),
            from_state: "home".to_string(),
            to_state: None,
            workflows: vec![],
        }],
        current_state: Some("home".to_string()),
        message: Some("1 transition available".to_string()),
        error: None,
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: AvailableTransitionsResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn initial_state_ref_roundtrips() {
    let i = InitialStateRef {
        id: "s-1".to_string(),
        name: "Home".to_string(),
    };
    let json = serde_json::to_string(&i).expect("serialize");
    assert_eq!(json, r#"{"id":"s-1","name":"Home"}"#);
    let back: InitialStateRef = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn resolved_initial_states_roundtrips() {
    let r = ResolvedInitialStates {
        state_ids: vec!["s-1".to_string(), "s-2".to_string()],
        source: InitialStatesSource::Workflow,
        states: Some(vec![
            InitialStateRef {
                id: "s-1".to_string(),
                name: "Home".to_string(),
            },
            InitialStateRef {
                id: "s-2".to_string(),
                name: "Login".to_string(),
            },
        ]),
        workflow_id: Some("wf-123".to_string()),
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // Check renames: state_ids → stateIds, workflow_id → workflowId.
    assert!(v["stateIds"].is_array());
    assert_eq!(v["workflowId"], "wf-123");
    assert!(v.get("state_ids").is_none());
    assert!(v.get("workflow_id").is_none());
    let back: ResolvedInitialStates = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn resolved_initial_states_result_roundtrips() {
    let r = ResolvedInitialStatesResult {
        success: true,
        state_ids: vec!["s-1".to_string()],
        source: InitialStatesSource::Defaults,
        states: vec![InitialStateRef {
            id: "s-1".to_string(),
            name: "Home".to_string(),
        }],
        workflow_id: String::new(),
        error: None,
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    assert!(v["stateIds"].is_array());
    assert_eq!(v["workflowId"], "");
    let back: ResolvedInitialStatesResult =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn state_node_data_fully_populated_roundtrips() {
    let mut thumbs = HashMap::new();
    thumbs.insert("el-1".to_string(), "data:image/png;base64,AAAA".to_string());
    let n = StateNodeData {
        state_id: "home".to_string(),
        name: "Home".to_string(),
        element_count: 4,
        confidence: 0.95,
        element_ids: vec!["el-1".to_string(), "el-2".to_string()],
        description: Some("Landing page".to_string()),
        is_blocking: false,
        is_selected: true,
        is_initial: true,
        outgoing_count: Some(3),
        incoming_count: Some(1),
        is_drop_target: Some(false),
        element_thumbnails: Some(thumbs),
    };
    let json1 = serde_json::to_string(&n).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    // camelCase renames are in effect.
    assert_eq!(v["stateId"], "home");
    assert_eq!(v["elementCount"], 4);
    assert_eq!(v["elementIds"].as_array().map(|a| a.len()), Some(2));
    assert_eq!(v["isInitial"], true);
    assert_eq!(v["outgoingCount"], 3);
    let back: StateNodeData = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_node_data_description_none_serializes_as_explicit_null() {
    // The `description` field deliberately omits `skip_serializing_if` so that
    // `None` serializes as an explicit `null` on the wire.
    let n = StateNodeData {
        state_id: "home".to_string(),
        name: "Home".to_string(),
        element_count: 0,
        confidence: 0.5,
        element_ids: vec![],
        description: None,
        is_blocking: false,
        is_selected: false,
        is_initial: false,
        outgoing_count: None,
        incoming_count: None,
        is_drop_target: None,
        element_thumbnails: None,
    };
    let json = serde_json::to_string(&n).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    assert!(
        v.get("description").is_some(),
        "description must be present on the wire (as null), not elided"
    );
    assert!(
        v["description"].is_null(),
        "None must serialize as explicit null, got {}",
        v["description"]
    );
    // Optional camelCase fields marked with skip_serializing_if should be absent.
    assert!(v.get("outgoingCount").is_none());
    assert!(v.get("incomingCount").is_none());
    assert!(v.get("isDropTarget").is_none());
    assert!(v.get("elementThumbnails").is_none());
    let back: StateNodeData = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn transition_edge_data_roundtrips() {
    let e = TransitionEdgeData {
        transition_id: "t-1".to_string(),
        name: "Go home".to_string(),
        path_cost: 1.25,
        action_count: 2,
        action_types: vec![StandardActionType::Click, StandardActionType::Type],
        is_highlighted: true,
        stays_visible: false,
        first_action_target: Some("home-link".to_string()),
    };
    let json = serde_json::to_string(&e).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // camelCase renames in effect.
    assert_eq!(v["transitionId"], "t-1");
    assert_eq!(v["pathCost"], 1.25);
    assert_eq!(v["actionCount"], 2);
    assert_eq!(v["actionTypes"][0], "click");
    assert_eq!(v["actionTypes"][1], "type");
    assert_eq!(v["firstActionTarget"], "home-link");
    let back: TransitionEdgeData = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn state_machine_export_format_roundtrips() {
    let mut state_payload = HashMap::new();
    state_payload.insert("name".to_string(), json!("Home"));
    state_payload.insert("element_ids".to_string(), json!(["el-1"]));
    let mut states = HashMap::new();
    states.insert("home".to_string(), state_payload);

    let mut transition_payload = HashMap::new();
    transition_payload.insert("name".to_string(), json!("Click login"));
    let mut transitions = HashMap::new();
    transitions.insert("t-1".to_string(), transition_payload);

    let mut config = HashMap::new();
    config.insert("name".to_string(), json!("My app"));
    config.insert("version".to_string(), json!(1));

    let exp = StateMachineExportFormat {
        states,
        transitions,
        config,
    };
    let json1 = serde_json::to_string(&exp).expect("serialize");
    let back: StateMachineExportFormat = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

// ============================================================================
// Execution — RunType / RunStatus enum snake_case wire form
// ============================================================================

use qontinui_types::execution::{RunStatus, RunType};

#[test]
fn run_type_qa_test_snake_case() {
    let json = serde_json::to_string(&RunType::QaTest).expect("serialize");
    assert_eq!(json, "\"qa_test\"", "RunType::QaTest wire form");
    let back: RunType = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, RunType::QaTest);
}

#[test]
fn run_status_completed_snake_case() {
    let json = serde_json::to_string(&RunStatus::Completed).expect("serialize");
    assert_eq!(json, "\"completed\"", "RunStatus::Completed wire form");
    let back: RunStatus = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, RunStatus::Completed);
}
