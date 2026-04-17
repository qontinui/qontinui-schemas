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
use qontinui_types::orchestration_config::*;
use qontinui_types::process_management::*;
use qontinui_types::scheduler::*;
use qontinui_types::terminal::*;
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
// ── geometry ─────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::geometry::{
    CoordinateSystem, Coordinates, Monitor, MonitorPosition, Region, VirtualDesktop,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn coordinate_system_snake_case() {
    let cases = [
        (CoordinateSystem::Screen, "\"screen\""),
        (CoordinateSystem::Virtual, "\"virtual\""),
        (CoordinateSystem::MonitorRelative, "\"monitor_relative\""),
    ];
    for (cs, expected) in cases {
        let json = serde_json::to_string(&cs).expect("serialize");
        assert_eq!(json, expected, "CoordinateSystem wire form mismatch");
        let back: CoordinateSystem = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, cs);
    }
}

#[test]
fn monitor_position_lowercase() {
    let cases = [
        (MonitorPosition::Left, "\"left\""),
        (MonitorPosition::Center, "\"center\""),
        (MonitorPosition::Right, "\"right\""),
    ];
    for (p, expected) in cases {
        let json = serde_json::to_string(&p).expect("serialize");
        assert_eq!(json, expected);
        let back: MonitorPosition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, p);
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn coordinates_monitor_relative_roundtrips() {
    let c = Coordinates {
        x: 100,
        y: 200,
        system: Some(CoordinateSystem::MonitorRelative),
        monitor_index: Some(1),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["x"], 100);
    assert_eq!(v["y"], 200);
    assert_eq!(v["system"], "monitor_relative");
    assert_eq!(v["monitor_index"], 1);
    let back: Coordinates = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn coordinates_minimal_roundtrips() {
    let c = Coordinates {
        x: -50,
        y: 0,
        system: None,
        monitor_index: None,
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert!(v.get("system").is_none(), "None system must be omitted");
    assert!(v.get("monitor_index").is_none());
    let back: Coordinates = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn region_virtual_roundtrips() {
    let r = Region {
        x: 10,
        y: 20,
        width: 800,
        height: 600,
        system: Some(CoordinateSystem::Virtual),
        monitor_index: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["system"], "virtual");
    assert_eq!(v["width"], 800);
    let back: Region = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn monitor_fully_populated_roundtrips() {
    let m = Monitor {
        index: 0,
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        position: MonitorPosition::Center,
        is_primary: true,
        scale_factor: 1.5,
        name: Some("DELL U2720Q".to_string()),
    };
    let json1 = serde_json::to_string(&m).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["position"], "center");
    assert_eq!(v["is_primary"], true);
    assert_eq!(v["scale_factor"], 1.5);
    assert_eq!(v["name"], "DELL U2720Q");
    let back: Monitor = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn virtual_desktop_multimon_roundtrips() {
    let vd = VirtualDesktop {
        monitors: vec![
            Monitor {
                index: 0,
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                position: MonitorPosition::Left,
                is_primary: true,
                scale_factor: 1.0,
                name: None,
            },
            Monitor {
                index: 1,
                x: 1920,
                y: 0,
                width: 2560,
                height: 1440,
                position: MonitorPosition::Right,
                is_primary: false,
                scale_factor: 1.25,
                name: Some("Secondary".to_string()),
            },
        ],
    };
    let json1 = serde_json::to_string(&vd).expect("serialize");
    let back: VirtualDesktop = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

// ============================================================================
// ── config ───────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::config::{Category, Context, ContextAutoInclude};

#[test]
fn context_auto_include_camelcase_roundtrips() {
    let c = ContextAutoInclude {
        task_mentions: Some(vec!["schema".to_string(), "flow".to_string()]),
        action_types: Some(vec!["CLICK".to_string(), "FIND".to_string()]),
        error_patterns: Some(vec!["E\\d{4}".to_string()]),
        file_patterns: Some(vec!["*.rs".to_string(), "src/api/**".to_string()]),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    // camelCase rename checks — wire form must match Python consumers.
    assert_eq!(v["taskMentions"][0], "schema");
    assert_eq!(v["actionTypes"][1], "FIND");
    assert_eq!(v["errorPatterns"][0], "E\\d{4}");
    assert_eq!(v["filePatterns"][0], "*.rs");
    assert!(
        v.get("task_mentions").is_none(),
        "snake_case field must not appear on the wire"
    );
    let back: ContextAutoInclude = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn context_fully_populated_roundtrips() {
    let c = Context {
        id: "ctx-schema-flow".to_string(),
        name: "Schema Flow".to_string(),
        content: "# Context\n\nRust is the source of truth.".to_string(),
        category: Some("architecture".to_string()),
        tags: vec!["schemas".to_string(), "types".to_string()],
        auto_include: Some(ContextAutoInclude {
            task_mentions: Some(vec!["schema".to_string()]),
            action_types: None,
            error_patterns: None,
            file_patterns: None,
        }),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-04-14T12:30:00Z".to_string(),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["createdAt"], "2026-01-01T00:00:00Z");
    assert_eq!(v["modifiedAt"], "2026-04-14T12:30:00Z");
    assert_eq!(v["autoInclude"]["taskMentions"][0], "schema");
    let back: Context = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn category_automation_enabled_default_true() {
    // The Python-side default is automationEnabled=true — exercise the default
    // by deserializing JSON that omits the field.
    let json_no_flag = r#"{"name":"Main"}"#;
    let c: Category = serde_json::from_str(json_no_flag).expect("deserialize");
    assert!(
        c.automation_enabled,
        "default_true must apply when field absent"
    );

    let c2 = Category {
        name: "Testing".to_string(),
        automation_enabled: false,
    };
    let json1 = serde_json::to_string(&c2).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["automationEnabled"], false);
    assert_eq!(v["name"], "Testing");
    let back: Category = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

// ============================================================================
// ── accessibility ────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::accessibility::{
    AccessibilityBackend, AccessibilityBounds, AccessibilityNode, AccessibilityRole,
    AccessibilitySelector, AccessibilitySnapshot, AccessibilityState, RoleCriterion,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn accessibility_role_snake_case_representative_variants() {
    let cases = [
        (AccessibilityRole::Button, "\"button\""),
        (AccessibilityRole::Textbox, "\"textbox\""),
        (AccessibilityRole::StaticText, "\"static_text\""),
        (AccessibilityRole::None, "\"none\""),
        (AccessibilityRole::Menuitemcheckbox, "\"menuitemcheckbox\""),
        (AccessibilityRole::Application, "\"application\""),
        (AccessibilityRole::Treeitem, "\"treeitem\""),
        (AccessibilityRole::Unknown, "\"unknown\""),
        (AccessibilityRole::Splitbutton, "\"splitbutton\""),
    ];
    for (role, expected) in cases {
        let json = serde_json::to_string(&role).expect("serialize");
        assert_eq!(json, expected, "AccessibilityRole wire form mismatch");
        let back: AccessibilityRole = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, role);
    }
}

#[test]
fn accessibility_backend_lowercase() {
    let cases = [
        (AccessibilityBackend::Auto, "\"auto\""),
        (AccessibilityBackend::Cdp, "\"cdp\""),
        (AccessibilityBackend::Uia, "\"uia\""),
        (AccessibilityBackend::Atspi, "\"atspi\""),
        (AccessibilityBackend::Ax, "\"ax\""),
        (AccessibilityBackend::None, "\"none\""),
    ];
    for (b, expected) in cases {
        let json = serde_json::to_string(&b).expect("serialize");
        assert_eq!(json, expected);
        let back: AccessibilityBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, b);
    }
}

#[test]
fn role_criterion_untagged_single_and_any() {
    // Single variant: wire is a bare string (well, a role-enum JSON value).
    let single = RoleCriterion::Single(AccessibilityRole::Button);
    let json = serde_json::to_string(&single).expect("serialize");
    assert_eq!(
        json, "\"button\"",
        "untagged Single must serialize as bare role"
    );
    let back: RoleCriterion = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        back,
        RoleCriterion::Single(AccessibilityRole::Button)
    ));

    // Any variant: wire is a JSON array of roles.
    let any = RoleCriterion::Any(vec![
        AccessibilityRole::Button,
        AccessibilityRole::Link,
        AccessibilityRole::Menuitem,
    ]);
    let json = serde_json::to_string(&any).expect("serialize");
    assert_eq!(json, r#"["button","link","menuitem"]"#);
    let back: RoleCriterion = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(back, RoleCriterion::Any(ref v) if v.len() == 3));
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn accessibility_state_fully_populated_roundtrips() {
    let s = AccessibilityState {
        is_focused: true,
        is_disabled: false,
        is_hidden: false,
        is_expanded: Some(true),
        is_selected: Some(false),
        is_checked: Some(true),
        is_pressed: None,
        is_readonly: false,
        is_required: true,
        is_multiselectable: false,
        is_editable: true,
        is_focusable: true,
        is_modal: false,
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["is_focused"], true);
    assert_eq!(v["is_expanded"], true);
    assert!(
        v.get("is_pressed").is_none(),
        "None tri-state must be omitted"
    );
    let back: AccessibilityState = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn accessibility_bounds_roundtrips() {
    let b = AccessibilityBounds {
        x: 100,
        y: 200,
        width: 320,
        height: 48,
    };
    let json1 = serde_json::to_string(&b).expect("serialize");
    let back: AccessibilityBounds = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

fn sample_a11y_button_node() -> AccessibilityNode {
    AccessibilityNode {
        ref_id: "@e1".to_string(),
        role: AccessibilityRole::Button,
        name: Some("Submit".to_string()),
        value: None,
        description: Some("Send the form".to_string()),
        bounds: Some(AccessibilityBounds {
            x: 10,
            y: 20,
            width: 80,
            height: 32,
        }),
        state: AccessibilityState {
            is_focusable: true,
            is_focused: true,
            ..Default::default()
        },
        is_interactive: true,
        level: None,
        automation_id: Some("submit-btn".to_string()),
        class_name: Some("btn btn-primary".to_string()),
        html_tag: Some("button".to_string()),
        url: None,
        children: vec![],
    }
}

#[test]
fn accessibility_node_ref_id_renamed_to_ref() {
    let n = sample_a11y_button_node();
    let json1 = serde_json::to_string(&n).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    // `ref_id` must appear as `ref` on the wire.
    assert_eq!(v["ref"], "@e1");
    assert!(v.get("ref_id").is_none());
    assert_eq!(v["role"], "button");
    assert_eq!(v["automation_id"], "submit-btn");
    assert_eq!(v["html_tag"], "button");
    let back: AccessibilityNode = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn accessibility_node_tree_roundtrips() {
    // Parent with one child — exercises recursion.
    let child = sample_a11y_button_node();
    let parent = AccessibilityNode {
        ref_id: "@e0".to_string(),
        role: AccessibilityRole::Form,
        name: Some("Login form".to_string()),
        value: None,
        description: None,
        bounds: None,
        state: AccessibilityState::default(),
        is_interactive: false,
        level: Some(1),
        automation_id: None,
        class_name: None,
        html_tag: Some("form".to_string()),
        url: None,
        children: vec![child],
    };
    let json1 = serde_json::to_string(&parent).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["children"][0]["ref"], "@e1");
    let back: AccessibilityNode = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn accessibility_snapshot_roundtrips() {
    let s = AccessibilitySnapshot {
        root: sample_a11y_button_node(),
        timestamp: 1_713_033_600.5,
        backend: AccessibilityBackend::Cdp,
        url: Some("https://app.example.com/login".to_string()),
        title: Some("Login — Example App".to_string()),
        total_nodes: 142,
        interactive_nodes: 37,
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["backend"], "cdp");
    assert_eq!(v["total_nodes"], 142);
    let back: AccessibilitySnapshot = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn accessibility_selector_with_ancestor_roundtrips() {
    let inner = AccessibilitySelector {
        role: Some(RoleCriterion::Single(AccessibilityRole::Form)),
        automation_id: Some("login-form".to_string()),
        case_sensitive: true,
        ..Default::default()
    };
    let sel = AccessibilitySelector {
        role: Some(RoleCriterion::Any(vec![
            AccessibilityRole::Button,
            AccessibilityRole::Link,
        ])),
        name: None,
        name_contains: Some("Submit".to_string()),
        name_pattern: None,
        value: None,
        value_contains: None,
        automation_id: None,
        class_name: None,
        html_tag: None,
        state: Some(AccessibilityState {
            is_focusable: true,
            ..Default::default()
        }),
        is_interactive: Some(true),
        ancestor: Some(Box::new(inner)),
        max_depth: Some(10),
        case_sensitive: false,
    };
    let json1 = serde_json::to_string(&sel).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    // role must serialize as an untagged array for the Any variant.
    assert!(v["role"].is_array(), "RoleCriterion::Any must be an array");
    assert_eq!(v["role"][0], "button");
    assert_eq!(v["ancestor"]["role"], "form");
    let back: AccessibilitySelector = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

// ============================================================================
// ── targets ──────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::targets::{
    AccessibilityRoleCriterion as TargetsRoleCriterion, MatchAdjustment, MatchMethod, OcrEngine,
    PatternOptions, PollingConfig, SearchOptions, SearchStrategy, TargetConfig, TextMatchType,
    TextSearchOptions,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn search_strategy_uppercase() {
    let cases = [
        (SearchStrategy::First, "\"FIRST\""),
        (SearchStrategy::All, "\"ALL\""),
        (SearchStrategy::Best, "\"BEST\""),
        (SearchStrategy::Each, "\"EACH\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: SearchStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn match_method_uppercase() {
    let cases = [
        (MatchMethod::Correlation, "\"CORRELATION\""),
        (MatchMethod::CorrelationNormed, "\"CORRELATION_NORMED\""),
        (MatchMethod::SquaredDifference, "\"SQUARED_DIFFERENCE\""),
        (
            MatchMethod::SquaredDifferenceNormed,
            "\"SQUARED_DIFFERENCE_NORMED\"",
        ),
    ];
    for (m, expected) in cases {
        let json = serde_json::to_string(&m).expect("serialize");
        assert_eq!(json, expected);
        let back: MatchMethod = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, m);
    }
}

#[test]
fn ocr_engine_uppercase() {
    let cases = [
        (OcrEngine::Tesseract, "\"TESSERACT\""),
        (OcrEngine::EasyOcr, "\"EASYOCR\""),
        (OcrEngine::PaddleOcr, "\"PADDLEOCR\""),
        (OcrEngine::Native, "\"NATIVE\""),
    ];
    for (o, expected) in cases {
        let json = serde_json::to_string(&o).expect("serialize");
        assert_eq!(json, expected);
        let back: OcrEngine = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, o);
    }
}

#[test]
fn text_match_type_uppercase_with_underscores() {
    let cases = [
        (TextMatchType::Exact, "\"EXACT\""),
        (TextMatchType::Contains, "\"CONTAINS\""),
        (TextMatchType::StartsWith, "\"STARTS_WITH\""),
        (TextMatchType::EndsWith, "\"ENDS_WITH\""),
        (TextMatchType::Regex, "\"REGEX\""),
        (TextMatchType::Fuzzy, "\"FUZZY\""),
    ];
    for (t, expected) in cases {
        let json = serde_json::to_string(&t).expect("serialize");
        assert_eq!(json, expected);
        let back: TextMatchType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, t);
    }
}

#[test]
fn targets_accessibility_role_criterion_untagged() {
    let single = TargetsRoleCriterion::Single("button".to_string());
    let json = serde_json::to_string(&single).expect("serialize");
    assert_eq!(json, "\"button\"");
    let back: TargetsRoleCriterion = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(back, TargetsRoleCriterion::Single(ref s) if s == "button"));

    let any = TargetsRoleCriterion::Any(vec!["button".to_string(), "link".to_string()]);
    let json = serde_json::to_string(&any).expect("serialize");
    assert_eq!(json, r#"["button","link"]"#);
    let back: TargetsRoleCriterion = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(back, TargetsRoleCriterion::Any(ref v) if v.len() == 2));
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn polling_config_camelcase_roundtrips() {
    let p = PollingConfig {
        interval: Some(500),
        max_attempts: Some(12),
    };
    let json1 = serde_json::to_string(&p).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["interval"], 500);
    assert_eq!(v["maxAttempts"], 12);
    assert!(v.get("max_attempts").is_none());
    let back: PollingConfig = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn pattern_options_partial_roundtrips() {
    // Exercise a representative subset — PatternOptions has 17 optional fields;
    // populating a realistic subset is more informative than every field.
    let p = PatternOptions {
        match_method: Some(MatchMethod::CorrelationNormed),
        scale_invariant: Some(true),
        min_scale: Some(0.8),
        max_scale: Some(1.2),
        scale_step: Some(0.05),
        use_grayscale: Some(true),
        use_color_reduction: Some(false),
        color_tolerance: Some(25.0),
        use_edges: Some(false),
        nms_threshold: Some(0.3),
        ..Default::default()
    };
    let json1 = serde_json::to_string(&p).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["matchMethod"], "CORRELATION_NORMED");
    assert_eq!(v["scaleInvariant"], true);
    assert_eq!(v["useGrayscale"], true);
    assert_eq!(v["colorTolerance"], 25.0);
    assert_eq!(v["nmsThreshold"], 0.3);
    // Absent fields must be skipped entirely.
    assert!(v.get("minRotation").is_none());
    let back: PatternOptions = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn match_adjustment_camelcase_roundtrips() {
    let m = MatchAdjustment {
        target_position: Some("TOP_LEFT".to_string()),
        target_offset: Some(Coordinates {
            x: 5,
            y: 10,
            system: None,
            monitor_index: None,
        }),
        add_w: Some(4),
        add_h: Some(2),
        absolute_w: None,
        absolute_h: None,
        add_x: Some(-1),
        add_y: None,
    };
    let json1 = serde_json::to_string(&m).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["targetPosition"], "TOP_LEFT");
    assert_eq!(v["targetOffset"]["x"], 5);
    assert_eq!(v["addW"], 4);
    assert_eq!(v["addX"], -1);
    assert!(v.get("absoluteW").is_none());
    assert!(v.get("addY").is_none());
    let back: MatchAdjustment = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn search_options_nested_roundtrips() {
    let so = SearchOptions {
        similarity: Some(0.85),
        timeout: Some(5000),
        search_regions: Some(vec![Region {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            system: None,
            monitor_index: None,
        }]),
        strategy: Some(SearchStrategy::Best),
        use_defined_region: Some(false),
        max_matches_to_act_on: Some(3),
        min_matches: Some(1),
        max_matches: Some(10),
        polling: Some(PollingConfig {
            interval: Some(250),
            max_attempts: Some(20),
        }),
        pattern: Some(PatternOptions {
            match_method: Some(MatchMethod::Correlation),
            ..Default::default()
        }),
        adjustment: None,
        capture_image: Some(true),
    };
    let json1 = serde_json::to_string(&so).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["similarity"], 0.85);
    assert_eq!(v["searchStrategy"], "BEST");
    assert_eq!(v["maxMatchesToActOn"], 3);
    assert_eq!(v["captureImage"], true);
    assert!(v.get("adjustment").is_none());
    let back: SearchOptions = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn text_search_options_ocr_tesseract_roundtrips() {
    let t = TextSearchOptions {
        ocr_engine: Some(OcrEngine::Tesseract),
        language: Some("eng".to_string()),
        whitelist_chars: Some("0123456789.".to_string()),
        blacklist_chars: None,
        match_type: Some(TextMatchType::Fuzzy),
        case_sensitive: Some(false),
        ignore_whitespace: Some(true),
        normalize_unicode: Some(true),
        fuzzy_threshold: Some(0.8),
        edit_distance: Some(2),
        preprocessing: Some(vec!["grayscale".to_string(), "threshold".to_string()]),
        scale_factor: Some(2.0),
        psm_mode: Some(6),
        oem_mode: Some(3),
        confidence_threshold: Some(0.75),
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["ocrEngine"], "TESSERACT");
    assert_eq!(v["matchType"], "FUZZY");
    assert_eq!(v["caseSensitive"], false);
    assert_eq!(v["fuzzyThreshold"], 0.8);
    assert_eq!(v["psmMode"], 6);
    assert_eq!(v["confidenceThreshold"], 0.75);
    let back: TextSearchOptions = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn target_config_image_variant_roundtrips() {
    let t = TargetConfig::Image {
        image_ids: vec!["img-1".to_string(), "img-2".to_string()],
        search_options: Some(SearchOptions {
            similarity: Some(0.9),
            ..Default::default()
        }),
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["type"], "image");
    assert_eq!(v["imageIds"][0], "img-1");
    assert_eq!(v["searchOptions"]["similarity"], 0.9);
    let back: TargetConfig = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn target_config_state_image_variant_roundtrips() {
    let t = TargetConfig::StateImage {
        state_id: "home".to_string(),
        image_ids: vec!["logo".to_string()],
        state_name: Some("Home Page".to_string()),
        image_names: Some(vec!["Company logo".to_string()]),
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["type"], "stateImage");
    assert_eq!(v["stateId"], "home");
    assert_eq!(v["imageIds"][0], "logo");
    assert_eq!(v["stateName"], "Home Page");
    let back: TargetConfig = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn target_config_unit_variants_roundtrip() {
    // Unit-like variants carry only the `type` discriminator.
    for (t, tag) in [
        (TargetConfig::CurrentPosition, "currentPosition"),
        (TargetConfig::LastFindResult, "lastFindResult"),
        (TargetConfig::AllResults, "allResults"),
    ] {
        let json = serde_json::to_string(&t).expect("serialize");
        let v: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(v["type"], tag);
        let back: TargetConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(json, serde_json::to_string(&back).expect("re-serialize"));
    }
}

#[test]
fn target_config_accessibility_variant_camelcase_roundtrips() {
    let t = TargetConfig::Accessibility {
        r#ref: Some("@e7".to_string()),
        role: Some(TargetsRoleCriterion::Any(vec![
            "button".to_string(),
            "link".to_string(),
        ])),
        name: None,
        name_contains: Some("Submit".to_string()),
        is_interactive: Some(true),
        capture_first: true,
        cdp_host: "localhost".to_string(),
        cdp_port: 9222,
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["type"], "accessibility");
    assert_eq!(v["ref"], "@e7");
    assert!(v["role"].is_array());
    assert_eq!(v["nameContains"], "Submit");
    assert_eq!(v["isInteractive"], true);
    assert_eq!(v["captureFirst"], true);
    assert_eq!(v["cdpHost"], "localhost");
    assert_eq!(v["cdpPort"], 9222);
    let back: TargetConfig = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn target_config_result_by_image_roundtrips() {
    let t = TargetConfig::ResultByImage {
        image_id: "img-xyz".to_string(),
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["type"], "resultByImage");
    assert_eq!(v["imageId"], "img-xyz");
    let back: TargetConfig = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

// ============================================================================
// ── execution ────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::execution::{
    ActionExecutionCreate, ActionExecutionResponse, ActionStatus, ActionType as ExecActionType,
    CoverageData, ErrorType, ExecutionIssueCreate, ExecutionIssueResponse, ExecutionRunComplete,
    ExecutionRunCompleteResponse, ExecutionRunCreate, ExecutionRunResponse, ExecutionScreenshotCreate,
    ExecutionScreenshotResponse, ExecutionStats, IssueSeverity, LLMMetrics,
    MatchLocation as ExecMatchLocation, RunStatus, RunType, RunnerMetadata, ScreenshotAnnotation,
    ScreenshotAnnotationShape, ScreenshotType, WorkflowMetadata,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn run_type_all_variants_snake_case() {
    let cases = [
        (RunType::QaTest, "\"qa_test\""),
        (RunType::IntegrationTest, "\"integration_test\""),
        (RunType::LiveAutomation, "\"live_automation\""),
        (RunType::Recording, "\"recording\""),
        (RunType::Debug, "\"debug\""),
    ];
    for (t, expected) in cases {
        let json = serde_json::to_string(&t).expect("serialize");
        assert_eq!(json, expected, "RunType wire form mismatch");
        let back: RunType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, t);
    }
}

#[test]
fn run_status_all_variants_snake_case() {
    let cases = [
        (RunStatus::Pending, "\"pending\""),
        (RunStatus::Running, "\"running\""),
        (RunStatus::Completed, "\"completed\""),
        (RunStatus::Failed, "\"failed\""),
        (RunStatus::Timeout, "\"timeout\""),
        (RunStatus::Cancelled, "\"cancelled\""),
        (RunStatus::Paused, "\"paused\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: RunStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn action_status_all_variants_snake_case() {
    let cases = [
        (ActionStatus::Success, "\"success\""),
        (ActionStatus::Failed, "\"failed\""),
        (ActionStatus::Timeout, "\"timeout\""),
        (ActionStatus::Skipped, "\"skipped\""),
        (ActionStatus::Error, "\"error\""),
        (ActionStatus::Pending, "\"pending\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: ActionStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn execution_action_type_representative_variants_snake_case() {
    let cases = [
        (ExecActionType::Find, "\"find\""),
        (ExecActionType::FindAll, "\"find_all\""),
        (ExecActionType::WaitUntilGone, "\"wait_until_gone\""),
        (ExecActionType::DoubleClick, "\"double_click\""),
        (ExecActionType::RightClick, "\"right_click\""),
        (ExecActionType::PressKey, "\"press_key\""),
        (ExecActionType::GoToState, "\"go_to_state\""),
        (ExecActionType::VerifyState, "\"verify_state\""),
        (ExecActionType::AiPrompt, "\"ai_prompt\""),
        (ExecActionType::RunPromptSequence, "\"run_prompt_sequence\""),
        (ExecActionType::Custom, "\"custom\""),
    ];
    for (a, expected) in cases {
        let json = serde_json::to_string(&a).expect("serialize");
        assert_eq!(json, expected, "execution::ActionType wire form");
        let back: ExecActionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, a);
    }
}

#[test]
fn error_type_all_variants_snake_case() {
    let cases = [
        (ErrorType::ElementNotFound, "\"element_not_found\""),
        (ErrorType::Timeout, "\"timeout\""),
        (ErrorType::AssertionFailed, "\"assertion_failed\""),
        (ErrorType::Crash, "\"crash\""),
        (ErrorType::NetworkError, "\"network_error\""),
        (ErrorType::ValidationError, "\"validation_error\""),
        (ErrorType::Other, "\"other\""),
    ];
    for (e, expected) in cases {
        let json = serde_json::to_string(&e).expect("serialize");
        assert_eq!(json, expected);
        let back: ErrorType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, e);
    }
}

#[test]
fn issue_severity_all_variants_snake_case() {
    let cases = [
        (IssueSeverity::Critical, "\"critical\""),
        (IssueSeverity::High, "\"high\""),
        (IssueSeverity::Medium, "\"medium\""),
        (IssueSeverity::Low, "\"low\""),
        (IssueSeverity::Informational, "\"informational\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: IssueSeverity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn screenshot_type_all_variants_snake_case() {
    let cases = [
        (ScreenshotType::Error, "\"error\""),
        (ScreenshotType::Success, "\"success\""),
        (ScreenshotType::Manual, "\"manual\""),
        (ScreenshotType::Periodic, "\"periodic\""),
        (ScreenshotType::ActionResult, "\"action_result\""),
        (ScreenshotType::StateVerification, "\"state_verification\""),
    ];
    for (t, expected) in cases {
        let json = serde_json::to_string(&t).expect("serialize");
        assert_eq!(json, expected);
        let back: ScreenshotType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, t);
    }
}

#[test]
fn screenshot_annotation_shape_snake_case() {
    let cases = [
        (ScreenshotAnnotationShape::Box, "\"box\""),
        (ScreenshotAnnotationShape::Circle, "\"circle\""),
        (ScreenshotAnnotationShape::Arrow, "\"arrow\""),
        (ScreenshotAnnotationShape::Text, "\"text\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: ScreenshotAnnotationShape = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_runner_metadata() -> RunnerMetadata {
    let mut extra = HashMap::new();
    extra.insert("build".to_string(), json!("release"));
    extra.insert("ci".to_string(), json!(false));
    RunnerMetadata {
        runner_version: "1.4.2".to_string(),
        os: "windows".to_string(),
        hostname: "dev-box-01".to_string(),
        screen_resolution: Some("1920x1080".to_string()),
        cpu_info: Some("AMD Ryzen 9 7950X".to_string()),
        memory_mb: Some(65_536),
        extra: Some(extra),
    }
}

fn sample_workflow_metadata() -> WorkflowMetadata {
    WorkflowMetadata {
        workflow_id: "wf-login".to_string(),
        workflow_name: "Login smoke test".to_string(),
        workflow_version: Some("v3".to_string()),
        total_states: Some(8),
        total_transitions: Some(14),
        tags: Some(vec!["smoke".to_string(), "login".to_string()]),
        description: Some("Happy path login + logout".to_string()),
        initial_state_ids: Some(vec!["landing".to_string()]),
    }
}

fn sample_execution_stats() -> ExecutionStats {
    ExecutionStats {
        total_actions: 42,
        successful_actions: 38,
        failed_actions: 3,
        timeout_actions: 1,
        skipped_actions: 0,
        total_duration_ms: 91_250,
        avg_action_duration_ms: Some(2172.6),
        total_tokens_input: Some(3200),
        total_tokens_output: Some(1150),
        total_cost_usd: Some(0.042),
        llm_action_count: Some(4),
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn runner_metadata_roundtrips() {
    let m = sample_runner_metadata();
    let json1 = serde_json::to_string(&m).expect("serialize");
    let back: RunnerMetadata = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn workflow_metadata_roundtrips() {
    let m = sample_workflow_metadata();
    let json1 = serde_json::to_string(&m).expect("serialize");
    let back: WorkflowMetadata = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_stats_roundtrips() {
    let s = sample_execution_stats();
    let json1 = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["total_actions"], 42);
    assert_eq!(v["successful_actions"], 38);
    let back: ExecutionStats = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn coverage_data_fully_populated_roundtrips() {
    let mut state_counts = HashMap::new();
    state_counts.insert("landing".to_string(), 1u32);
    state_counts.insert("logged_in".to_string(), 2u32);
    let mut trans_counts = HashMap::new();
    trans_counts.insert("landing->login".to_string(), 1u32);
    trans_counts.insert("login->logged_in".to_string(), 1u32);
    let c = CoverageData {
        coverage_percentage: 75.0,
        states_covered: 6,
        total_states: 8,
        transitions_covered: 10,
        total_transitions: 14,
        uncovered_states: Some(vec!["error".to_string(), "settings".to_string()]),
        uncovered_transitions: Some(vec!["login->error".to_string()]),
        state_visit_counts: Some(state_counts),
        transition_execution_counts: Some(trans_counts),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let back: CoverageData = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn llm_metrics_roundtrips() {
    let mut params = HashMap::new();
    params.insert("temperature".to_string(), json!(0.2));
    params.insert("max_tokens".to_string(), json!(1024));
    let m = LLMMetrics {
        model: Some("claude-opus-4-6".to_string()),
        provider: Some("anthropic".to_string()),
        tokens_input: Some(2048),
        tokens_output: Some(512),
        tokens_total: Some(2560),
        cost_usd: Some(0.018),
        generation_params: Some(params),
    };
    let json1 = serde_json::to_string(&m).expect("serialize");
    let back: LLMMetrics = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_run_create_roundtrips() {
    let mut cfg = HashMap::new();
    cfg.insert("headless".to_string(), json!(true));
    cfg.insert("retries".to_string(), json!(2));
    let c = ExecutionRunCreate {
        project_id: "proj-alpha".to_string(),
        run_type: RunType::QaTest,
        run_name: "Smoke test 2026-04-14".to_string(),
        description: Some("Nightly QA smoke suite".to_string()),
        runner_metadata: sample_runner_metadata(),
        workflow_metadata: Some(sample_workflow_metadata()),
        configuration: Some(cfg),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["run_type"], "qa_test");
    assert_eq!(v["project_id"], "proj-alpha");
    let back: ExecutionRunCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_run_response_roundtrips() {
    let r = ExecutionRunResponse {
        run_id: "run-789".to_string(),
        project_id: "proj-alpha".to_string(),
        run_type: RunType::LiveAutomation,
        run_name: "Prod check".to_string(),
        status: RunStatus::Running,
        started_at: "2026-04-14T03:00:00Z".to_string(),
        ended_at: None,
        duration_seconds: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["status"], "running");
    assert!(v.get("ended_at").is_none());
    let back: ExecutionRunResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn exec_match_location_roundtrips() {
    let m = ExecMatchLocation {
        x: 420,
        y: 130,
        width: Some(80),
        height: Some(24),
    };
    let json1 = serde_json::to_string(&m).expect("serialize");
    let back: ExecMatchLocation = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn action_execution_create_roundtrips() {
    let mut input_data = HashMap::new();
    input_data.insert("text".to_string(), json!("hello world"));
    let a = ActionExecutionCreate {
        sequence_number: 5,
        action_type: ExecActionType::Click,
        action_name: "Click Submit".to_string(),
        status: ActionStatus::Success,
        started_at: "2026-04-14T03:00:05.000Z".to_string(),
        completed_at: "2026-04-14T03:00:05.122Z".to_string(),
        duration_ms: 122,
        from_state: Some("form".to_string()),
        to_state: Some("submitted".to_string()),
        active_states: Some(vec!["form".to_string()]),
        pattern_id: Some("pat-submit".to_string()),
        pattern_name: Some("Submit button".to_string()),
        confidence_score: Some(0.97),
        match_location: Some(ExecMatchLocation {
            x: 200,
            y: 400,
            width: Some(80),
            height: Some(32),
        }),
        error_message: None,
        error_type: None,
        error_stack: None,
        screenshot_id: Some("scr-42".to_string()),
        parent_action_id: None,
        input_data: Some(input_data),
        output_data: None,
        metadata: None,
        llm_metrics: None,
        span_type: Some("tool".to_string()),
        trace_id: Some("trc-xyz".to_string()),
        parent_id: None,
    };
    let json1 = serde_json::to_string(&a).expect("serialize");
    let back: ActionExecutionCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn action_execution_response_roundtrips() {
    let r = ActionExecutionResponse {
        recorded: 3,
        run_id: "run-789".to_string(),
        action_ids: Some(vec![
            "act-1".to_string(),
            "act-2".to_string(),
            "act-3".to_string(),
        ]),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ActionExecutionResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn screenshot_annotation_roundtrips() {
    let a = ScreenshotAnnotation {
        shape: ScreenshotAnnotationShape::Box,
        x: 100,
        y: 120,
        width: Some(40),
        height: Some(40),
        label: Some("Focused button".to_string()),
        color: Some("#FF5500".to_string()),
    };
    let json1 = serde_json::to_string(&a).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    // The `shape` field is renamed to `type` on the wire.
    assert_eq!(v["type"], "box");
    assert!(v.get("shape").is_none());
    let back: ScreenshotAnnotation = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_screenshot_create_roundtrips() {
    let c = ExecutionScreenshotCreate {
        screenshot_id: "scr-42".to_string(),
        sequence_number: 7,
        screenshot_type: ScreenshotType::ActionResult,
        timestamp: "2026-04-14T03:00:05.000Z".to_string(),
        width: 1920,
        height: 1080,
        action_sequence_number: Some(5),
        state: Some("form".to_string()),
        active_states: Some(vec!["form".to_string()]),
        annotations: Some(vec![ScreenshotAnnotation {
            shape: ScreenshotAnnotationShape::Arrow,
            x: 10,
            y: 10,
            width: None,
            height: None,
            label: None,
            color: Some("#00AAFF".to_string()),
        }]),
        metadata: None,
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["screenshot_type"], "action_result");
    let back: ExecutionScreenshotCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_screenshot_response_roundtrips() {
    let r = ExecutionScreenshotResponse {
        screenshot_id: "scr-42".to_string(),
        run_id: "run-789".to_string(),
        image_url: "https://storage.example.com/scr/42.png".to_string(),
        thumbnail_url: Some("https://storage.example.com/scr/42_thumb.png".to_string()),
        uploaded_at: "2026-04-14T03:00:05.250Z".to_string(),
        file_size_bytes: 183_421,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ExecutionScreenshotResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_issue_create_roundtrips() {
    let i = ExecutionIssueCreate {
        title: "Button label drifted".to_string(),
        description: "Submit button now reads 'Send' instead of 'Submit'".to_string(),
        severity: IssueSeverity::Medium,
        issue_type: "visual_regression".to_string(),
        action_sequence_number: Some(5),
        state: Some("form".to_string()),
        screenshot_ids: Some(vec!["scr-42".to_string()]),
        reproduction_steps: Some(vec![
            "Navigate to login form".to_string(),
            "Observe button label".to_string(),
        ]),
        expected_behavior: Some("Button reads 'Submit'".to_string()),
        actual_behavior: Some("Button reads 'Send'".to_string()),
        metadata: None,
    };
    let json1 = serde_json::to_string(&i).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["severity"], "medium");
    let back: ExecutionIssueCreate = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_issue_response_roundtrips() {
    let r = ExecutionIssueResponse {
        recorded: 2,
        run_id: "run-789".to_string(),
        issue_ids: Some(vec!["iss-1".to_string(), "iss-2".to_string()]),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ExecutionIssueResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_run_complete_roundtrips() {
    let c = ExecutionRunComplete {
        status: RunStatus::Completed,
        ended_at: "2026-04-14T03:15:12Z".to_string(),
        stats: sample_execution_stats(),
        coverage: Some(CoverageData {
            coverage_percentage: 85.7,
            states_covered: 6,
            total_states: 7,
            transitions_covered: 12,
            total_transitions: 14,
            uncovered_states: None,
            uncovered_transitions: None,
            state_visit_counts: None,
            transition_execution_counts: None,
        }),
        summary: Some("All critical paths passed".to_string()),
        error_message: None,
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["status"], "completed");
    assert_eq!(v["coverage"]["coverage_percentage"], 85.7);
    let back: ExecutionRunComplete = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_run_complete_response_roundtrips() {
    let r = ExecutionRunCompleteResponse {
        run_id: "run-789".to_string(),
        status: RunStatus::Failed,
        started_at: "2026-04-14T03:00:00Z".to_string(),
        ended_at: "2026-04-14T03:05:42Z".to_string(),
        duration_seconds: 342.5,
        stats: sample_execution_stats(),
        coverage: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["duration_seconds"], 342.5);
    assert!(v.get("coverage").is_none());
    let back: ExecutionRunCompleteResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

// ─── Additional types ported from api/execution.py ───────────────────────────

use qontinui_types::execution::{
    ActionExecutionBatch, ActionExecutionBatchResponse, ActionExecutionListResponse,
    ActionReliabilityStats, CostTrendDataPoint, CostTrendResponse, ExecutionIssueBatch,
    ExecutionIssueBatchResponse, ExecutionIssueDetail, ExecutionIssueListResponse,
    ExecutionIssueUpdate, ExecutionRunDetail, ExecutionRunListResponse, ExecutionTrendDataPoint,
    ExecutionTrendResponse, HistoricalActionQuery, HistoricalActionResult, IssueSource,
    IssueStatus, IssueType, LLMCostSummary, ModelCostBreakdown, PlaybackFrameRequest,
    VisualComparisonResult,
};
use qontinui_types::task_run::Pagination as TrPagination;

#[test]
fn issue_status_all_variants_snake_case() {
    let cases = [
        (IssueStatus::New, "\"new\""),
        (IssueStatus::Open, "\"open\""),
        (IssueStatus::InProgress, "\"in_progress\""),
        (IssueStatus::Resolved, "\"resolved\""),
        (IssueStatus::Closed, "\"closed\""),
        (IssueStatus::WontFix, "\"wont_fix\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected, "IssueStatus wire form");
        let back: IssueStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn issue_type_all_variants_snake_case() {
    let cases = [
        (IssueType::Functional, "\"functional\""),
        (IssueType::Visual, "\"visual\""),
        (IssueType::Performance, "\"performance\""),
        (IssueType::Crash, "\"crash\""),
        (IssueType::Timeout, "\"timeout\""),
        (IssueType::Assertion, "\"assertion\""),
        (IssueType::StateMismatch, "\"state_mismatch\""),
        (IssueType::ElementNotFound, "\"element_not_found\""),
        (IssueType::AiDetected, "\"ai_detected\""),
        (IssueType::Other, "\"other\""),
    ];
    for (t, expected) in cases {
        let json = serde_json::to_string(&t).expect("serialize");
        assert_eq!(json, expected, "IssueType wire form");
        let back: IssueType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, t);
    }
}

#[test]
fn issue_source_all_variants_snake_case() {
    let cases = [
        (IssueSource::Automation, "\"automation\""),
        (IssueSource::AiAnalysis, "\"ai_analysis\""),
        (IssueSource::VisualRegression, "\"visual_regression\""),
        (IssueSource::UserReported, "\"user_reported\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected, "IssueSource wire form");
        let back: IssueSource = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn action_execution_batch_roundtrips() {
    let a = ActionExecutionCreate {
        sequence_number: 1,
        action_type: ExecActionType::Click,
        action_name: "Click Submit".to_string(),
        status: ActionStatus::Success,
        started_at: "2026-04-14T03:00:05.000Z".to_string(),
        completed_at: "2026-04-14T03:00:05.122Z".to_string(),
        duration_ms: 122,
        from_state: None,
        to_state: None,
        active_states: None,
        pattern_id: None,
        pattern_name: None,
        confidence_score: None,
        match_location: None,
        error_message: None,
        error_type: None,
        error_stack: None,
        screenshot_id: None,
        parent_action_id: None,
        input_data: None,
        output_data: None,
        metadata: None,
        llm_metrics: None,
        span_type: None,
        trace_id: None,
        parent_id: None,
    };
    let batch = ActionExecutionBatch {
        actions: vec![a.clone(), a],
    };
    let json1 = serde_json::to_string(&batch).expect("serialize");
    let back: ActionExecutionBatch = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn action_execution_batch_response_roundtrips() {
    let r = ActionExecutionBatchResponse {
        run_id: "run-7".to_string(),
        actions_recorded: 3,
        action_ids: vec!["a-1".to_string(), "a-2".to_string(), "a-3".to_string()],
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ActionExecutionBatchResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_issue_batch_roundtrips() {
    let i = ExecutionIssueCreate {
        title: "T".to_string(),
        description: "D".to_string(),
        severity: IssueSeverity::Low,
        issue_type: "functional".to_string(),
        action_sequence_number: None,
        state: None,
        screenshot_ids: None,
        reproduction_steps: None,
        expected_behavior: None,
        actual_behavior: None,
        metadata: None,
    };
    let b = ExecutionIssueBatch {
        issues: vec![i.clone(), i],
    };
    let json1 = serde_json::to_string(&b).expect("serialize");
    let back: ExecutionIssueBatch = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_issue_batch_response_roundtrips() {
    let r = ExecutionIssueBatchResponse {
        run_id: "run-7".to_string(),
        issues_recorded: 2,
        issue_ids: vec!["i-1".to_string(), "i-2".to_string()],
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ExecutionIssueBatchResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_issue_update_roundtrips() {
    let u = ExecutionIssueUpdate {
        status: Some(IssueStatus::InProgress),
        severity: Some(IssueSeverity::High),
        assigned_to_user_id: Some("user-42".to_string()),
        resolution_notes: Some("Investigating".to_string()),
    };
    let json1 = serde_json::to_string(&u).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["status"], "in_progress");
    assert_eq!(v["severity"], "high");
    let back: ExecutionIssueUpdate = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_issue_update_empty_omits_all_fields() {
    let u = ExecutionIssueUpdate::default();
    let json1 = serde_json::to_string(&u).expect("serialize");
    assert_eq!(json1, "{}");
    let back: ExecutionIssueUpdate = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(back, u);
}

#[test]
fn visual_comparison_result_roundtrips() {
    let v = VisualComparisonResult {
        comparison_id: "cmp-1".to_string(),
        baseline_id: Some("base-7".to_string()),
        similarity_score: 0.942,
        threshold: 0.95,
        passed: false,
        diff_image_url: Some("https://example.com/diff.png".to_string()),
        diff_region_count: 3,
    };
    let json1 = serde_json::to_string(&v).expect("serialize");
    let back: VisualComparisonResult = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_run_detail_roundtrips() {
    let mut cfg = HashMap::new();
    cfg.insert("headless".to_string(), json!(true));
    let d = ExecutionRunDetail {
        run_id: "run-1".to_string(),
        project_id: "proj-a".to_string(),
        run_type: RunType::QaTest,
        run_name: "Nightly".to_string(),
        status: RunStatus::Completed,
        started_at: "2026-04-14T03:00:00Z".to_string(),
        ended_at: Some("2026-04-14T03:10:00Z".to_string()),
        duration_seconds: Some(600.0),
        description: Some("Scheduled nightly smoke".to_string()),
        runner_metadata: sample_runner_metadata(),
        workflow_metadata: Some(sample_workflow_metadata()),
        configuration: cfg,
        stats: sample_execution_stats(),
        coverage: None,
        created_at: "2026-04-14T02:59:59Z".to_string(),
        updated_at: Some("2026-04-14T03:10:01Z".to_string()),
    };
    let json1 = serde_json::to_string(&d).expect("serialize");
    let back: ExecutionRunDetail = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_issue_detail_roundtrips() {
    let d = ExecutionIssueDetail {
        id: "iss-1".to_string(),
        run_id: "run-1".to_string(),
        issue_type: IssueType::Visual,
        severity: IssueSeverity::Medium,
        status: IssueStatus::Open,
        source: IssueSource::VisualRegression,
        title: "Button drift".to_string(),
        description: "Label changed".to_string(),
        state_name: Some("form".to_string()),
        screenshot_count: 1,
        created_at: "2026-04-14T03:05:00Z".to_string(),
        updated_at: "2026-04-14T03:06:00Z".to_string(),
        action_sequence_number: Some(5),
        reproduction_steps: vec!["Open form".to_string(), "Check label".to_string()],
        screenshots: vec![],
        error_details: HashMap::new(),
        metadata: HashMap::new(),
        assigned_to: None,
        resolution_notes: None,
    };
    let json1 = serde_json::to_string(&d).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["source"], "visual_regression");
    assert_eq!(v["status"], "open");
    let back: ExecutionIssueDetail = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_run_list_response_roundtrips() {
    let resp = ExecutionRunListResponse {
        runs: vec![ExecutionRunResponse {
            run_id: "r-1".to_string(),
            project_id: "p-1".to_string(),
            run_type: RunType::Debug,
            run_name: "debug".to_string(),
            status: RunStatus::Completed,
            started_at: "2026-04-14T03:00:00Z".to_string(),
            ended_at: None,
            duration_seconds: None,
        }],
        pagination: TrPagination {
            total: 1,
            limit: 20,
            offset: 0,
            has_more: false,
        },
    };
    let json1 = serde_json::to_string(&resp).expect("serialize");
    let back: ExecutionRunListResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn action_execution_list_response_roundtrips() {
    let resp = ActionExecutionListResponse {
        actions: vec![ActionExecutionResponse {
            recorded: 1,
            run_id: "r-1".to_string(),
            action_ids: Some(vec!["a-1".to_string()]),
        }],
        pagination: TrPagination {
            total: 1,
            limit: 10,
            offset: 0,
            has_more: false,
        },
    };
    let json1 = serde_json::to_string(&resp).expect("serialize");
    let back: ActionExecutionListResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_issue_list_response_roundtrips() {
    let mut summary = HashMap::new();
    summary.insert("critical".to_string(), json!(0));
    summary.insert("high".to_string(), json!(2));
    let resp = ExecutionIssueListResponse {
        issues: vec![ExecutionIssueResponse {
            recorded: 1,
            run_id: "r-1".to_string(),
            issue_ids: Some(vec!["i-1".to_string()]),
        }],
        pagination: TrPagination {
            total: 1,
            limit: 10,
            offset: 0,
            has_more: false,
        },
        summary,
    };
    let json1 = serde_json::to_string(&resp).expect("serialize");
    let back: ExecutionIssueListResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn action_reliability_stats_roundtrips() {
    let mut err = HashMap::new();
    err.insert("type".to_string(), json!("timeout"));
    err.insert("count".to_string(), json!(4));
    let s = ActionReliabilityStats {
        action_name: "Click Submit".to_string(),
        action_type: ExecActionType::Click,
        total_executions: 100,
        successful_executions: 96,
        failed_executions: 4,
        success_rate: 96.0,
        avg_duration_ms: 150,
        p50_duration_ms: 120,
        p95_duration_ms: 310,
        common_errors: vec![err],
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: ActionReliabilityStats = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn execution_trend_response_roundtrips() {
    let r = ExecutionTrendResponse {
        project_id: "proj-a".to_string(),
        run_type: Some(RunType::QaTest),
        start_date: "2026-04-01".to_string(),
        end_date: "2026-04-14".to_string(),
        granularity: "daily".to_string(),
        data_points: vec![ExecutionTrendDataPoint {
            date: "2026-04-01".to_string(),
            runs_count: 5,
            success_rate: 80.0,
            avg_duration_seconds: 120,
            total_actions: 300,
            issues_count: 2,
        }],
        overall_stats: HashMap::new(),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["granularity"], "daily");
    assert_eq!(v["run_type"], "qa_test");
    let back: ExecutionTrendResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn llm_cost_summary_roundtrips() {
    let s = LLMCostSummary {
        run_id: "run-1".to_string(),
        total_tokens_input: 1000,
        total_tokens_output: 500,
        total_cost_usd: 0.025,
        llm_action_count: 3,
        per_model: vec![ModelCostBreakdown {
            model: "claude-opus-4-6".to_string(),
            provider: Some("anthropic".to_string()),
            tokens_input: 1000,
            tokens_output: 500,
            cost_usd: 0.025,
            action_count: 3,
        }],
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: LLMCostSummary = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn cost_trend_response_roundtrips() {
    let r = CostTrendResponse {
        project_id: "proj-a".to_string(),
        start_date: "2026-04-01".to_string(),
        end_date: "2026-04-14".to_string(),
        granularity: "weekly".to_string(),
        data_points: vec![CostTrendDataPoint {
            date: "2026-04-01".to_string(),
            tokens_input: 1000,
            tokens_output: 500,
            cost_usd: 0.02,
            llm_action_count: 2,
            runs_count: 1,
        }],
        overall_stats: HashMap::new(),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: CostTrendResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn historical_action_query_roundtrips() {
    let q = HistoricalActionQuery {
        action_type: Some(ExecActionType::Click),
        action_name: Some("Click Submit".to_string()),
        state_name: None,
        success_only: true,
        project_id: Some("p-1".to_string()),
        workflow_id: Some("wf-login".to_string()),
        limit: 25,
    };
    let json1 = serde_json::to_string(&q).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["action_type"], "click");
    assert_eq!(v["success_only"], true);
    let back: HistoricalActionQuery = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn historical_action_result_roundtrips() {
    let mut inp = HashMap::new();
    inp.insert("x".to_string(), json!(10));
    let mut out = HashMap::new();
    out.insert("clicked".to_string(), json!(true));
    let r = HistoricalActionResult {
        id: "a-1".to_string(),
        action_type: ExecActionType::Click,
        action_name: "Click".to_string(),
        status: ActionStatus::Success,
        from_state: Some("form".to_string()),
        to_state: Some("submitted".to_string()),
        input_data: inp,
        output_data: out,
        duration_ms: 120,
        screenshot_url: Some("https://example.com/s.png".to_string()),
        has_screenshot: true,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: HistoricalActionResult = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn playback_frame_request_roundtrips() {
    let r = PlaybackFrameRequest {
        action_ids: vec!["a-1".to_string(), "a-2".to_string(), "a-3".to_string()],
        include_screenshots: false,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: PlaybackFrameRequest = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

// ============================================================================
// ── rag ──────────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::rag::{
    BatchComputeEmbeddingRequest, BatchComputeEmbeddingResponse, BatchEmbeddingResult,
    BoundingBox, ComputeEmbeddingRequest, ComputeEmbeddingResponse, ComputeTextEmbeddingRequest,
    ComputeTextEmbeddingResponse, ElementType, EmbeddedElement, EmbeddingItem,
    EmbeddingListResponse, EmbeddingResultItem, EmbeddingResultsRequest, EmbeddingResultsResponse,
    ExportResult, GUIElementChunk, JobItem, JobListResponse, JobStatus, JobSummary,
    RAGDashboardStats, RagCompletionEvent, RagProcessingStatus, RagProgressEvent,
    SearchResultItem, SemanticSearchRequest, SemanticSearchResponse, StateFilterItem,
    StatesResponse, VectorSearchResult,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn job_status_all_variants_snake_case() {
    let cases = [
        (JobStatus::Pending, "\"pending\""),
        (JobStatus::InProgress, "\"in_progress\""),
        (JobStatus::Completed, "\"completed\""),
        (JobStatus::Failed, "\"failed\""),
        (JobStatus::Cancelled, "\"cancelled\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: JobStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn rag_processing_status_all_variants_snake_case() {
    let cases = [
        (RagProcessingStatus::NotStarted, "\"not_started\""),
        (RagProcessingStatus::InProgress, "\"in_progress\""),
        (RagProcessingStatus::Completed, "\"completed\""),
        (RagProcessingStatus::Failed, "\"failed\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: RagProcessingStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_embedding_item() -> EmbeddingItem {
    let mut meta = HashMap::new();
    meta.insert("capture_reason".to_string(), json!("discovery"));
    EmbeddingItem {
        id: "emb-001".to_string(),
        pattern_id: "pat-001".to_string(),
        pattern_name: Some("Login button".to_string()),
        state_id: "state-login".to_string(),
        state_name: "Login page".to_string(),
        image_id: "img-001".to_string(),
        image_storage_path: "projects/proj-alpha/images/img-001.png".to_string(),
        image_url: Some("https://storage.example.com/proj-alpha/img-001.png".to_string()),
        embedding_model: "clip-vit-b-32".to_string(),
        embedding_version: "2026-01-01".to_string(),
        image_width: 80,
        image_height: 32,
        text_description: Some("Blue rounded button labeled Login".to_string()),
        has_text_embedding: true,
        pattern_metadata: meta,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T01:00:00Z".to_string(),
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn compute_text_embedding_request_default_model() {
    // Default model applies when the field is omitted on input.
    let json_no_model = r#"{"text":"login button"}"#;
    let r: ComputeTextEmbeddingRequest = serde_json::from_str(json_no_model).expect("deserialize");
    assert_eq!(r.model, "clip", "default_clip_model must apply");

    let r2 = ComputeTextEmbeddingRequest {
        text: "submit form".to_string(),
        model: "minilm".to_string(),
    };
    let json1 = serde_json::to_string(&r2).expect("serialize");
    let back: ComputeTextEmbeddingRequest = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn compute_text_embedding_response_roundtrips() {
    let r = ComputeTextEmbeddingResponse {
        success: true,
        embedding: Some(vec![0.1, -0.2, 0.33, 0.04]),
        embedding_dim: 384,
        processing_time_ms: 12.7,
        error: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ComputeTextEmbeddingResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn compute_embedding_request_roundtrips() {
    let r = ComputeEmbeddingRequest {
        image_data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB".to_string(),
        compute_text_embedding: true,
        text_description: Some("Blue button".to_string()),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ComputeEmbeddingRequest = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn compute_embedding_response_roundtrips() {
    let r = ComputeEmbeddingResponse {
        success: true,
        image_embedding: Some(vec![0.12, 0.34]),
        text_embedding: Some(vec![0.55, 0.66]),
        text_description: Some("Blue button".to_string()),
        ocr_text: Some("Submit".to_string()),
        ocr_confidence: Some(0.94),
        processing_time_ms: 42.0,
        error: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ComputeEmbeddingResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn batch_compute_embedding_request_defaults_true() {
    // compute_text_embeddings / extract_ocr default to true when absent.
    let json_min =
        r#"{"images":[{"id":"a","image_data":"AAAA"},{"id":"b","image_data":"BBBB"}]}"#;
    let r: BatchComputeEmbeddingRequest = serde_json::from_str(json_min).expect("deserialize");
    assert!(r.compute_text_embeddings);
    assert!(r.extract_ocr);

    let r2 = BatchComputeEmbeddingRequest {
        images: vec![{
            let mut img = HashMap::new();
            img.insert("id".to_string(), json!("a"));
            img.insert("image_data".to_string(), json!("AAAA"));
            img
        }],
        compute_text_embeddings: false,
        extract_ocr: false,
    };
    let json1 = serde_json::to_string(&r2).expect("serialize");
    let back: BatchComputeEmbeddingRequest = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn batch_embedding_result_roundtrips() {
    let r = BatchEmbeddingResult {
        id: "img-a".to_string(),
        success: true,
        image_embedding: Some(vec![0.1, 0.2, 0.3]),
        text_embedding: Some(vec![0.4, 0.5, 0.6]),
        text_description: Some("Header logo".to_string()),
        ocr_text: Some("Brand".to_string()),
        ocr_confidence: Some(0.88),
        error: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: BatchEmbeddingResult = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn batch_compute_embedding_response_roundtrips() {
    let r = BatchComputeEmbeddingResponse {
        success: true,
        results: vec![BatchEmbeddingResult {
            id: "img-a".to_string(),
            success: true,
            image_embedding: Some(vec![0.1, 0.2]),
            text_embedding: None,
            text_description: None,
            ocr_text: None,
            ocr_confidence: None,
            error: None,
        }],
        total_processed: 1,
        successful: 1,
        failed: 0,
        processing_time_ms: 81.5,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: BatchComputeEmbeddingResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn embedding_result_item_roundtrips() {
    let r = EmbeddingResultItem {
        state_image_id: "si-001".to_string(),
        success: true,
        image_embedding: Some(vec![0.1; 4]),
        text_embedding: Some(vec![0.2; 4]),
        text_description: Some("Logo".to_string()),
        ocr_text: None,
        ocr_confidence: None,
        error: None,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: EmbeddingResultItem = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn embedding_results_request_roundtrips() {
    let r = EmbeddingResultsRequest {
        project_id: "proj-alpha".to_string(),
        results: vec![EmbeddingResultItem {
            state_image_id: "si-001".to_string(),
            success: true,
            image_embedding: None,
            text_embedding: None,
            text_description: None,
            ocr_text: None,
            ocr_confidence: None,
            error: Some("model not loaded".to_string()),
        }],
        total_processed: 1,
        successful: 0,
        failed: 1,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: EmbeddingResultsRequest = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn embedding_results_response_roundtrips() {
    let r = EmbeddingResultsResponse {
        success: true,
        message: "Applied 3 embeddings".to_string(),
        applied: 3,
        failed: 0,
        not_found: 1,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: EmbeddingResultsResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn rag_progress_event_roundtrips() {
    let e = RagProgressEvent {
        project_id: "proj-alpha".to_string(),
        status: RagProcessingStatus::InProgress,
        message: "Processing images 42 / 100".to_string(),
        percent: Some(42.0),
        elements_processed: Some(42),
        total_elements: Some(100),
        error: None,
    };
    let json1 = serde_json::to_string(&e).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["status"], "in_progress");
    assert_eq!(v["percent"], 42.0);
    let back: RagProgressEvent = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn rag_completion_event_roundtrips() {
    let e = RagCompletionEvent {
        project_id: "proj-alpha".to_string(),
        success: true,
        results: vec![EmbeddingResultItem {
            state_image_id: "si-001".to_string(),
            success: true,
            image_embedding: None,
            text_embedding: None,
            text_description: None,
            ocr_text: None,
            ocr_confidence: None,
            error: None,
        }],
        total_processed: 1,
        successful: 1,
        failed: 0,
        web_sync_success: Some(true),
        web_sync_error: None,
    };
    let json1 = serde_json::to_string(&e).expect("serialize");
    let back: RagCompletionEvent = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn job_summary_roundtrips() {
    let j = JobSummary {
        id: "job-001".to_string(),
        status: JobStatus::InProgress,
        progress_percent: 66.7,
        total_patterns: 150,
        processed_patterns: 100,
        started_at: Some("2026-04-14T00:00:00Z".to_string()),
        error_message: None,
    };
    let json1 = serde_json::to_string(&j).expect("serialize");
    let back: JobSummary = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn rag_dashboard_stats_with_active_job_roundtrips() {
    let s = RAGDashboardStats {
        total_embeddings: 12_500,
        total_states: 140,
        total_patterns: 500,
        last_sync_at: Some("2026-04-14T02:30:00Z".to_string()),
        active_job: Some(JobSummary {
            id: "job-001".to_string(),
            status: JobStatus::InProgress,
            progress_percent: 25.0,
            total_patterns: 200,
            processed_patterns: 50,
            started_at: Some("2026-04-14T03:00:00Z".to_string()),
            error_message: None,
        }),
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: RAGDashboardStats = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn embedding_item_roundtrips() {
    let e = sample_embedding_item();
    let json1 = serde_json::to_string(&e).expect("serialize");
    let back: EmbeddingItem = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn embedding_list_response_roundtrips() {
    let r = EmbeddingListResponse {
        items: vec![sample_embedding_item()],
        total: 1,
        page: 1,
        limit: 50,
        has_more: false,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: EmbeddingListResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn job_item_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("trigger".to_string(), json!("manual"));
    let j = JobItem {
        id: "job-001".to_string(),
        status: JobStatus::Completed,
        total_patterns: 200,
        processed_patterns: 200,
        progress_percent: 100.0,
        error_message: None,
        retry_count: 0,
        max_retries: 3,
        job_metadata: meta,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        started_at: Some("2026-04-14T00:00:05Z".to_string()),
        completed_at: Some("2026-04-14T00:12:30Z".to_string()),
    };
    let json1 = serde_json::to_string(&j).expect("serialize");
    let back: JobItem = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn job_list_response_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("trigger".to_string(), json!("schedule"));
    let r = JobListResponse {
        items: vec![JobItem {
            id: "job-002".to_string(),
            status: JobStatus::Failed,
            total_patterns: 200,
            processed_patterns: 50,
            progress_percent: 25.0,
            error_message: Some("clip model unavailable".to_string()),
            retry_count: 2,
            max_retries: 3,
            job_metadata: meta,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            started_at: None,
            completed_at: None,
        }],
        total: 1,
        page: 1,
        limit: 50,
        has_more: false,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: JobListResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn semantic_search_request_defaults() {
    let json_min = r#"{"query":"login button"}"#;
    let r: SemanticSearchRequest = serde_json::from_str(json_min).expect("deserialize");
    assert_eq!(r.limit, 20, "default_search_limit must apply");
    assert!(
        (r.min_similarity - 0.2).abs() < 1e-9,
        "default_min_similarity must apply"
    );

    let r2 = SemanticSearchRequest {
        query: "blue button".to_string(),
        limit: 50,
        min_similarity: 0.35,
        state_filter: Some("state-login".to_string()),
    };
    let json1 = serde_json::to_string(&r2).expect("serialize");
    let back: SemanticSearchRequest = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn search_result_item_roundtrips() {
    let r = SearchResultItem {
        embedding: sample_embedding_item(),
        similarity_score: 0.87,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: SearchResultItem = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn semantic_search_response_roundtrips() {
    let r = SemanticSearchResponse {
        results: vec![SearchResultItem {
            embedding: sample_embedding_item(),
            similarity_score: 0.91,
        }],
        query: "blue button".to_string(),
        total_found: 1,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: SemanticSearchResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_filter_item_and_states_response_roundtrip() {
    let r = StatesResponse {
        states: vec![
            StateFilterItem {
                state_id: "state-login".to_string(),
                state_name: "Login page".to_string(),
                count: 42,
            },
            StateFilterItem {
                state_id: "state-home".to_string(),
                state_name: "Home page".to_string(),
                count: 18,
            },
        ],
        count: 2,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: StatesResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

// ─── GUI element chunking (rag/models.py) ───────────────────────────────────

#[test]
fn element_type_all_variants_snake_case() {
    let cases = [
        (ElementType::Button, "\"button\""),
        (ElementType::IconButton, "\"icon_button\""),
        (ElementType::ToggleButton, "\"toggle_button\""),
        (ElementType::DropdownButton, "\"dropdown_button\""),
        (ElementType::TextInput, "\"text_input\""),
        (ElementType::SearchInput, "\"search_input\""),
        (ElementType::PasswordInput, "\"password_input\""),
        (ElementType::Textarea, "\"textarea\""),
        (ElementType::Checkbox, "\"checkbox\""),
        (ElementType::RadioButton, "\"radio_button\""),
        (ElementType::Dropdown, "\"dropdown\""),
        (ElementType::Combobox, "\"combobox\""),
        (ElementType::Slider, "\"slider\""),
        (ElementType::Link, "\"link\""),
        (ElementType::Tab, "\"tab\""),
        (ElementType::MenuItem, "\"menu_item\""),
        (ElementType::Breadcrumb, "\"breadcrumb\""),
        (ElementType::Modal, "\"modal\""),
        (ElementType::Dialog, "\"dialog\""),
        (ElementType::Panel, "\"panel\""),
        (ElementType::Card, "\"card\""),
        (ElementType::Icon, "\"icon\""),
        (ElementType::Image, "\"image\""),
        (ElementType::Label, "\"label\""),
        (ElementType::Badge, "\"badge\""),
        (ElementType::Tooltip, "\"tooltip\""),
        (ElementType::TableCell, "\"table_cell\""),
        (ElementType::TableHeader, "\"table_header\""),
        (ElementType::ListItem, "\"list_item\""),
        (ElementType::Progress, "\"progress\""),
        (ElementType::Spinner, "\"spinner\""),
        (ElementType::Unknown, "\"unknown\""),
    ];
    for (e, expected) in cases {
        let json = serde_json::to_string(&e).expect("serialize");
        assert_eq!(json, expected);
        let back: ElementType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, e);
    }
}

#[test]
fn bounding_box_roundtrips() {
    let b = BoundingBox {
        x: 10,
        y: 20,
        width: 80,
        height: 32,
    };
    let json1 = serde_json::to_string(&b).expect("serialize");
    let back: BoundingBox = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
    assert_eq!(back, b);
}

fn sample_gui_element_chunk() -> GUIElementChunk {
    GUIElementChunk {
        id: "elem-001".to_string(),
        created_at: Some("2026-04-14T00:00:00Z".to_string()),
        updated_at: Some("2026-04-14T01:00:00Z".to_string()),
        source_app: "calculator".to_string(),
        source_state_id: Some("state-main".to_string()),
        source_screenshot_id: Some("ss-001".to_string()),
        extraction_method: "auto".to_string(),
        bounding_box: Some(BoundingBox {
            x: 100,
            y: 200,
            width: 80,
            height: 32,
        }),
        width: 80,
        height: 32,
        aspect_ratio: 2.5,
        area: 2560,
        position_quadrant: "top-left".to_string(),
        dominant_colors: vec![vec![255, 0, 0], vec![0, 0, 255]],
        color_histogram: vec![10, 20, 30],
        average_brightness: 0.75,
        contrast_ratio: 4.5,
        edge_density: 0.3,
        has_text: true,
        ocr_text: "Submit".to_string(),
        ocr_confidence: 0.95,
        text_length: 6,
        element_type: ElementType::Button,
        element_subtype: "primary".to_string(),
        is_interactive: true,
        interaction_type: "click".to_string(),
        visual_state: "normal".to_string(),
        is_enabled: true,
        is_selected: false,
        is_focused: false,
        parent_region: Some("toolbar".to_string()),
        depth_in_hierarchy: 2,
        sibling_count: 3,
        platform: "windows".to_string(),
        text_embedding: Some(vec![0.1, 0.2, 0.3]),
        text_description: "Blue submit button".to_string(),
        image_embedding: Some(vec![0.4, 0.5, 0.6]),
        state_id: Some("state-main".to_string()),
        state_name: "Main screen".to_string(),
        is_defining_element: true,
        is_optional_element: false,
        similarity_threshold: 0.85,
        is_fixed_position: true,
        is_shared: false,
        probability: 0.99,
        search_region_id: Some("region-toolbar".to_string()),
        semantic_role: "submit".to_string(),
        semantic_action: "submit_form".to_string(),
        style_family: "fluent".to_string(),
    }
}

#[test]
fn gui_element_chunk_roundtrips() {
    let e = sample_gui_element_chunk();
    let json1 = serde_json::to_string(&e).expect("serialize");
    let back: GUIElementChunk = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn gui_element_chunk_defaults_apply() {
    let json_min = r#"{"id":"elem-min"}"#;
    let e: GUIElementChunk = serde_json::from_str(json_min).expect("deserialize");
    assert_eq!(e.extraction_method, "manual");
    assert_eq!(e.element_type, ElementType::Unknown);
    assert_eq!(e.visual_state, "normal");
    assert!(e.is_enabled);
    assert!((e.similarity_threshold - 0.8).abs() < 1e-9);
    assert!((e.probability - 1.0).abs() < 1e-9);
}

#[test]
fn embedded_element_roundtrips() {
    let e = EmbeddedElement {
        element: sample_gui_element_chunk(),
        text_embedding: Some(vec![0.7, 0.8]),
        image_embedding: Some(vec![0.9, 1.0]),
        embedding_model: "clip-vit-b-32".to_string(),
        embedding_timestamp: Some("2026-04-14T02:00:00Z".to_string()),
    };
    let json1 = serde_json::to_string(&e).expect("serialize");
    let back: EmbeddedElement = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn vector_search_result_roundtrips() {
    let r = VectorSearchResult {
        element: sample_gui_element_chunk(),
        score: 0.92,
        distance: 0.08,
        rank: 1,
        matched_on: "hybrid".to_string(),
        search_type: "hybrid".to_string(),
        query_text: "blue submit button".to_string(),
        query_timestamp: Some("2026-04-14T03:00:00Z".to_string()),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: VectorSearchResult = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn vector_search_result_defaults() {
    let json_min = r#"{"element":{"id":"e1"},"score":0.5}"#;
    let r: VectorSearchResult = serde_json::from_str(json_min).expect("deserialize");
    assert_eq!(r.matched_on, "text");
    assert_eq!(r.search_type, "text");
    assert_eq!(r.rank, 0);
}

#[test]
fn export_result_roundtrips() {
    let r = ExportResult {
        success: true,
        exported_count: 42,
        failed_count: 2,
        skipped_count: 5,
        errors: vec!["timeout on elem-99".to_string()],
        warnings: vec!["low confidence on elem-33".to_string()],
        export_timestamp: Some("2026-04-14T04:00:00Z".to_string()),
        export_path: "/tmp/export/rag_elements.json".to_string(),
        format: "json".to_string(),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ExportResult = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn export_result_default_format() {
    let json_min = r#"{"success":false}"#;
    let r: ExportResult = serde_json::from_str(json_min).expect("deserialize");
    assert_eq!(r.format, "json");
    assert_eq!(r.exported_count, 0);
}

// ============================================================================
// ── tree_events ──────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::tree_events::{
    ActionType as TreeActionType, DisplayNode, ExecutionTreeResponse,
    MatchLocation as TreeMatchLocation, NodeMetadata, NodeStatus, NodeType, Outcome, PathElement,
    RuntimeData, StateContext, TimingInfo, TopMatch, TreeEvent, TreeEventCreate,
    TreeEventListResponse, TreeEventResponse, TreeEventType, TreeNode,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn node_type_lowercase() {
    let cases = [
        (NodeType::Workflow, "\"workflow\""),
        (NodeType::Action, "\"action\""),
        (NodeType::Transition, "\"transition\""),
    ];
    for (t, expected) in cases {
        let json = serde_json::to_string(&t).expect("serialize");
        assert_eq!(json, expected);
        let back: NodeType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, t);
    }
}

#[test]
fn node_status_lowercase() {
    let cases = [
        (NodeStatus::Pending, "\"pending\""),
        (NodeStatus::Running, "\"running\""),
        (NodeStatus::Success, "\"success\""),
        (NodeStatus::Failed, "\"failed\""),
    ];
    for (s, expected) in cases {
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, expected);
        let back: NodeStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }
}

#[test]
fn tree_event_type_all_variants_snake_case() {
    let cases = [
        (TreeEventType::WorkflowStarted, "\"workflow_started\""),
        (TreeEventType::WorkflowCompleted, "\"workflow_completed\""),
        (TreeEventType::WorkflowFailed, "\"workflow_failed\""),
        (TreeEventType::ActionStarted, "\"action_started\""),
        (TreeEventType::ActionCompleted, "\"action_completed\""),
        (TreeEventType::ActionFailed, "\"action_failed\""),
        (TreeEventType::TransitionStarted, "\"transition_started\""),
        (
            TreeEventType::TransitionCompleted,
            "\"transition_completed\"",
        ),
        (TreeEventType::TransitionFailed, "\"transition_failed\""),
    ];
    for (t, expected) in cases {
        let json = serde_json::to_string(&t).expect("serialize");
        assert_eq!(json, expected);
        let back: TreeEventType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, t);
    }
}

#[test]
fn tree_events_action_type_representative_variants() {
    // SCREAMING_SNAKE_CASE wire form preserved via explicit per-variant renames.
    let cases = [
        (TreeActionType::Find, "\"FIND\""),
        (TreeActionType::FindStateImage, "\"FIND_STATE_IMAGE\""),
        (TreeActionType::DoubleClick, "\"DOUBLE_CLICK\""),
        (TreeActionType::MouseMove, "\"MOUSE_MOVE\""),
        (TreeActionType::KeyPress, "\"KEY_PRESS\""),
        (TreeActionType::TryCatch, "\"TRY_CATCH\""),
        (TreeActionType::GoToState, "\"GO_TO_STATE\""),
        (TreeActionType::RunWorkflow, "\"RUN_WORKFLOW\""),
        (TreeActionType::CodeBlock, "\"CODE_BLOCK\""),
        (TreeActionType::Custom, "\"CUSTOM\""),
    ];
    for (a, expected) in cases {
        let json = serde_json::to_string(&a).expect("serialize");
        assert_eq!(json, expected, "tree_events::ActionType wire form");
        let back: TreeActionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, a);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_tree_match_location() -> TreeMatchLocation {
    TreeMatchLocation {
        x: 120,
        y: 240,
        w: Some(64),
        h: Some(32),
    }
}

fn sample_top_match() -> TopMatch {
    TopMatch {
        confidence: 0.93,
        location: sample_tree_match_location(),
        dimensions: Some(TreeMatchLocation {
            x: 0,
            y: 0,
            w: Some(64),
            h: Some(32),
        }),
    }
}

fn sample_runtime_data() -> RuntimeData {
    RuntimeData {
        image_id: Some("img-001".to_string()),
        found: Some(true),
        confidence: Some(0.93),
        location: Some(sample_tree_match_location()),
        dimensions: Some(TreeMatchLocation {
            x: 0,
            y: 0,
            w: Some(64),
            h: Some(32),
        }),
        match_method: Some("CORRELATION".to_string()),
        top_matches: Some(vec![sample_top_match()]),
        clicked_at: Some(sample_tree_match_location()),
        button: Some("left".to_string()),
        target_type: Some("image".to_string()),
        ..Default::default()
    }
}

fn sample_node_metadata() -> NodeMetadata {
    let mut cfg = HashMap::new();
    cfg.insert("imageIds".to_string(), json!(["img-001"]));
    cfg.insert("similarity".to_string(), json!(0.85));
    NodeMetadata {
        config: Some(cfg),
        is_expandable: false,
        is_inline: false,
        runtime: Some(sample_runtime_data()),
        state_context: Some(StateContext {
            active_before: vec!["home".to_string()],
            active_after: vec!["login".to_string()],
            changed: true,
            activated: vec!["login".to_string()],
            deactivated: vec!["home".to_string()],
        }),
        timing: Some(TimingInfo {
            start_time: "2026-04-14T03:00:00.000Z".to_string(),
            end_time: Some("2026-04-14T03:00:00.120Z".to_string()),
            duration_ms: Some(120.0),
        }),
        outcome: Some(Outcome {
            success: true,
            error: None,
            retry_count: 0,
        }),
        screenshot_reference: Some("screenshots/s-1.png".to_string()),
        visual_debug_reference: None,
    }
}

fn sample_tree_node() -> TreeNode {
    TreeNode {
        id: "n-1".to_string(),
        node_type: NodeType::Action,
        name: "Click Login".to_string(),
        timestamp: 1_713_067_200.0,
        end_timestamp: Some(1_713_067_200.12),
        duration: Some(0.12),
        parent_id: Some("n-0".to_string()),
        status: NodeStatus::Success,
        metadata: sample_node_metadata(),
        error: None,
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn tree_match_location_point_form_skips_w_h() {
    // Point-form (no width/height) must skip optional fields on the wire.
    let m = TreeMatchLocation {
        x: 5,
        y: 10,
        w: None,
        h: None,
    };
    let json1 = serde_json::to_string(&m).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert!(v.get("w").is_none());
    assert!(v.get("h").is_none());
    let back: TreeMatchLocation = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn top_match_roundtrips() {
    let m = sample_top_match();
    let json1 = serde_json::to_string(&m).expect("serialize");
    let back: TopMatch = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn runtime_data_type_action_roundtrips() {
    // RuntimeData is a union-of-optionals; the TYPE-action shape exercises a
    // different subset of fields from FIND.
    let r = RuntimeData {
        typed_text: Some("hello world".to_string()),
        character_count: Some(11),
        ..Default::default()
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["typed_text"], "hello world");
    assert_eq!(v["character_count"], 11);
    assert!(
        v.get("found").is_none(),
        "unset optional fields must be skipped"
    );
    let back: RuntimeData = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn runtime_data_go_to_state_roundtrips() {
    let r = RuntimeData {
        source_states: Some(vec!["home".to_string()]),
        target_states: Some(vec!["login".to_string()]),
        targets_reached: Some(vec!["login".to_string()]),
        transitions_executed: Some(vec!["home->login".to_string()]),
        already_at_target: Some(false),
        ..Default::default()
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: RuntimeData = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn state_context_roundtrips() {
    let s = StateContext {
        active_before: vec!["home".to_string(), "header".to_string()],
        active_after: vec!["login".to_string(), "header".to_string()],
        changed: true,
        activated: vec!["login".to_string()],
        deactivated: vec!["home".to_string()],
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let back: StateContext = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn timing_info_in_flight_omits_end_and_duration() {
    let t = TimingInfo {
        start_time: "2026-04-14T03:00:00.000Z".to_string(),
        end_time: None,
        duration_ms: None,
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert!(v.get("end_time").is_none());
    assert!(v.get("duration_ms").is_none());
    let back: TimingInfo = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn outcome_failed_roundtrips() {
    let o = Outcome {
        success: false,
        error: Some("element not found".to_string()),
        retry_count: 3,
    };
    let json1 = serde_json::to_string(&o).expect("serialize");
    let back: Outcome = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn node_metadata_roundtrips() {
    let m = sample_node_metadata();
    let json1 = serde_json::to_string(&m).expect("serialize");
    let back: NodeMetadata = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn tree_node_roundtrips() {
    let n = sample_tree_node();
    let json1 = serde_json::to_string(&n).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["node_type"], "action");
    assert_eq!(v["status"], "success");
    let back: TreeNode = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn path_element_roundtrips() {
    let p = PathElement {
        id: "n-0".to_string(),
        name: "Root workflow".to_string(),
        node_type: NodeType::Workflow,
    };
    let json1 = serde_json::to_string(&p).expect("serialize");
    let back: PathElement = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn tree_event_default_type_field() {
    // When deserializing an event payload missing `type`, the default
    // `"tree_event"` must fill in.
    let json_no_type = r#"{
        "event_type":"action_started",
        "node":{"id":"n-1","node_type":"action","name":"Click","timestamp":1.0,"status":"running"},
        "timestamp":1.0
    }"#;
    let e: TreeEvent = serde_json::from_str(json_no_type).expect("deserialize");
    assert_eq!(e.r#type, "tree_event");

    let e2 = TreeEvent {
        r#type: "tree_event".to_string(),
        event_type: TreeEventType::ActionCompleted,
        node: sample_tree_node(),
        path: vec![PathElement {
            id: "n-0".to_string(),
            name: "Root".to_string(),
            node_type: NodeType::Workflow,
        }],
        timestamp: 1_713_067_200.0,
        sequence: 17,
    };
    let json1 = serde_json::to_string(&e2).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["type"], "tree_event");
    assert_eq!(v["event_type"], "action_completed");
    let back: TreeEvent = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn display_node_is_expanded_default_true() {
    let json_no_exp = r#"{
        "id":"n-1",
        "node_type":"action",
        "name":"Click",
        "timestamp":1.0,
        "status":"success"
    }"#;
    let d: DisplayNode = serde_json::from_str(json_no_exp).expect("deserialize");
    assert!(d.is_expanded, "is_expanded default must be true");
    assert_eq!(d.level, 0);

    let d2 = DisplayNode {
        id: "n-1".to_string(),
        node_type: NodeType::Action,
        name: "Click".to_string(),
        timestamp: 1_713_067_200.0,
        end_timestamp: Some(1_713_067_200.12),
        duration: Some(0.12),
        status: NodeStatus::Success,
        metadata: NodeMetadata::default(),
        error: None,
        children: vec![DisplayNode {
            id: "n-2".to_string(),
            node_type: NodeType::Action,
            name: "Child".to_string(),
            timestamp: 1_713_067_200.05,
            end_timestamp: None,
            duration: None,
            status: NodeStatus::Running,
            metadata: NodeMetadata::default(),
            error: None,
            children: vec![],
            is_expanded: false,
            level: 1,
        }],
        is_expanded: true,
        level: 0,
    };
    let json1 = serde_json::to_string(&d2).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["children"][0]["is_expanded"], false);
    assert_eq!(v["children"][0]["level"], 1);
    let back: DisplayNode = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn tree_event_create_roundtrips() {
    let c = TreeEventCreate {
        event_type: TreeEventType::ActionStarted,
        node: sample_tree_node(),
        path: vec![PathElement {
            id: "n-0".to_string(),
            name: "Root".to_string(),
            node_type: NodeType::Workflow,
        }],
        timestamp: 1_713_067_200.0,
        sequence: 42,
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let back: TreeEventCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn tree_event_response_roundtrips() {
    let r = TreeEventResponse {
        id: "te-001".to_string(),
        run_id: "run-789".to_string(),
        event_type: TreeEventType::ActionCompleted,
        node_id: "n-1".to_string(),
        node_type: NodeType::Action,
        node_name: "Click Login".to_string(),
        parent_node_id: Some("n-0".to_string()),
        path: vec![PathElement {
            id: "n-0".to_string(),
            name: "Root".to_string(),
            node_type: NodeType::Workflow,
        }],
        sequence: 1,
        event_timestamp: 1_713_067_200.12,
        status: NodeStatus::Success,
        error_message: None,
        metadata: Some(sample_node_metadata()),
        created_at: "2026-04-14T03:00:00.200Z".to_string(),
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: TreeEventResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn tree_event_list_response_roundtrips() {
    let r = TreeEventListResponse {
        events: vec![],
        total: 0,
        limit: 50,
        offset: 0,
        has_more: false,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: TreeEventListResponse = serde_json::from_str(&json1).expect("deserialize");
    assert_eq!(json1, serde_json::to_string(&back).expect("re-serialize"));
}

#[test]
fn execution_tree_response_roundtrips() {
    let mut name_map = HashMap::new();
    name_map.insert("state-login".to_string(), "Login page".to_string());
    name_map.insert("state-home".to_string(), "Home page".to_string());
    let r = ExecutionTreeResponse {
        run_id: "run-789".to_string(),
        root_nodes: vec![DisplayNode {
            id: "n-0".to_string(),
            node_type: NodeType::Workflow,
            name: "Root workflow".to_string(),
            timestamp: 1_713_067_200.0,
            end_timestamp: Some(1_713_067_215.0),
            duration: Some(15.0),
            status: NodeStatus::Success,
            metadata: NodeMetadata::default(),
            error: None,
            children: vec![],
            is_expanded: true,
            level: 0,
        }],
        total_events: 42,
        workflow_name: Some("Login smoke test".to_string()),
        status: NodeStatus::Success,
        duration_ms: Some(15_000.0),
        initial_state_ids: vec!["state-home".to_string()],
        state_name_map: name_map,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: ExecutionTreeResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

// ============================================================================
// ── findings ─────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::findings::{
    FindingActionType, FindingBatchCreate, FindingCategory, FindingCodeContext, FindingCreate,
    FindingDetail, FindingListResponse, FindingSeverity, FindingStatus, FindingSummary,
    FindingUpdate, FindingUserInput,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn finding_category_all_variants_roundtrip() {
    for (variant, expected) in [
        (FindingCategory::CodeBug, "code_bug"),
        (FindingCategory::Security, "security"),
        (FindingCategory::Performance, "performance"),
        (FindingCategory::Todo, "todo"),
        (FindingCategory::Enhancement, "enhancement"),
        (FindingCategory::ConfigIssue, "config_issue"),
        (FindingCategory::TestIssue, "test_issue"),
        (FindingCategory::Documentation, "documentation"),
        (FindingCategory::RuntimeIssue, "runtime_issue"),
        (FindingCategory::AlreadyFixed, "already_fixed"),
        (FindingCategory::ExpectedBehavior, "expected_behavior"),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, format!("\"{}\"", expected));
        let back: FindingCategory = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, variant);
    }
}

#[test]
fn finding_severity_all_variants_roundtrip() {
    for (variant, expected) in [
        (FindingSeverity::Critical, "critical"),
        (FindingSeverity::High, "high"),
        (FindingSeverity::Medium, "medium"),
        (FindingSeverity::Low, "low"),
        (FindingSeverity::Info, "info"),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, format!("\"{}\"", expected));
        let back: FindingSeverity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, variant);
    }
}

#[test]
fn finding_status_all_variants_roundtrip() {
    for (variant, expected) in [
        (FindingStatus::Detected, "detected"),
        (FindingStatus::InProgress, "in_progress"),
        (FindingStatus::NeedsInput, "needs_input"),
        (FindingStatus::Resolved, "resolved"),
        (FindingStatus::WontFix, "wont_fix"),
        (FindingStatus::Deferred, "deferred"),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, format!("\"{}\"", expected));
        let back: FindingStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, variant);
    }
}

#[test]
fn finding_action_type_all_variants_roundtrip() {
    for (variant, expected) in [
        (FindingActionType::AutoFix, "auto_fix"),
        (FindingActionType::NeedsUserInput, "needs_user_input"),
        (FindingActionType::Informational, "informational"),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, format!("\"{}\"", expected));
        let back: FindingActionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, variant);
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn finding_code_context_fully_populated_roundtrips() {
    let c = FindingCodeContext {
        file: Some("src/lib.rs".to_string()),
        line: Some(42),
        column: Some(8),
        snippet: Some("let x = y + z;".to_string()),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["file"], "src/lib.rs");
    assert_eq!(v["line"], 42);
    assert_eq!(v["column"], 8);
    assert_eq!(v["snippet"], "let x = y + z;");
    let back: FindingCodeContext = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_code_context_empty_skips_all_fields() {
    let c = FindingCodeContext::default();
    let json = serde_json::to_string(&c).expect("serialize");
    // All four fields are Optional with skip_serializing_if = "Option::is_none".
    assert_eq!(json, "{}");
}

#[test]
fn finding_user_input_fully_populated_roundtrips() {
    let u = FindingUserInput {
        question: "Which approach should we take?".to_string(),
        input_type: "choice".to_string(),
        options: Some(vec!["A".to_string(), "B".to_string()]),
    };
    let json1 = serde_json::to_string(&u).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["question"], "Which approach should we take?");
    assert_eq!(v["input_type"], "choice");
    assert_eq!(v["options"][0], "A");
    let back: FindingUserInput = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_user_input_default_input_type_applied() {
    // input_type has a Python-side default of "text". Omitting the field must
    // deserialize to that default via serde(default = ...).
    let json = r#"{"question":"Free-form question?"}"#;
    let u: FindingUserInput = serde_json::from_str(json).expect("deserialize");
    assert_eq!(u.input_type, "text");
    assert_eq!(u.question, "Free-form question?");
    assert!(u.options.is_none());
}

#[test]
fn finding_create_fully_populated_roundtrips() {
    let c = FindingCreate {
        task_run_id: "run-abc".to_string(),
        session_num: 3,
        category: FindingCategory::CodeBug,
        severity: FindingSeverity::High,
        title: "Null pointer in parser".to_string(),
        description: "Dereferences `ptr` without checking for null.".to_string(),
        code_context: Some(FindingCodeContext {
            file: Some("src/parser.rs".to_string()),
            line: Some(120),
            column: Some(17),
            snippet: Some("let v = *ptr;".to_string()),
        }),
        signature_hash: Some("abc123".to_string()),
        action_type: FindingActionType::AutoFix,
        user_input: None,
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["task_run_id"], "run-abc");
    assert_eq!(v["category"], "code_bug");
    assert_eq!(v["severity"], "high");
    assert_eq!(v["action_type"], "auto_fix");
    assert_eq!(v["code_context"]["line"], 120);
    assert!(
        v.get("user_input").is_none(),
        "None Option must be skipped on the wire"
    );
    let back: FindingCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_create_with_user_input_roundtrips() {
    let c = FindingCreate {
        task_run_id: "run-xyz".to_string(),
        session_num: 1,
        category: FindingCategory::ConfigIssue,
        severity: FindingSeverity::Medium,
        title: "Ambiguous config value".to_string(),
        description: "Two plausible settings — user must pick.".to_string(),
        code_context: None,
        signature_hash: None,
        action_type: FindingActionType::NeedsUserInput,
        user_input: Some(FindingUserInput {
            question: "Strict or lenient?".to_string(),
            input_type: "choice".to_string(),
            options: Some(vec!["strict".to_string(), "lenient".to_string()]),
        }),
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["user_input"]["question"], "Strict or lenient?");
    assert_eq!(v["user_input"]["options"][1], "lenient");
    let back: FindingCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_batch_create_roundtrips() {
    let b = FindingBatchCreate {
        findings: vec![
            FindingCreate {
                task_run_id: "run-1".to_string(),
                session_num: 1,
                category: FindingCategory::Todo,
                severity: FindingSeverity::Low,
                title: "TODO: factor out".to_string(),
                description: "Inline helper should move to utils.".to_string(),
                code_context: None,
                signature_hash: None,
                action_type: FindingActionType::Informational,
                user_input: None,
            },
            FindingCreate {
                task_run_id: "run-1".to_string(),
                session_num: 1,
                category: FindingCategory::Security,
                severity: FindingSeverity::Critical,
                title: "Secret in repo".to_string(),
                description: "API key committed.".to_string(),
                code_context: None,
                signature_hash: Some("sec-dedupe".to_string()),
                action_type: FindingActionType::AutoFix,
                user_input: None,
            },
        ],
    };
    let json1 = serde_json::to_string(&b).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["findings"].as_array().unwrap().len(), 2);
    assert_eq!(v["findings"][0]["category"], "todo");
    assert_eq!(v["findings"][1]["severity"], "critical");
    let back: FindingBatchCreate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_update_fully_populated_roundtrips() {
    let u = FindingUpdate {
        status: Some(FindingStatus::Resolved),
        resolution: Some("Fixed in commit abc123.".to_string()),
        resolved_in_session: Some(5),
        user_response: Some("Looks good.".to_string()),
    };
    let json1 = serde_json::to_string(&u).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["status"], "resolved");
    assert_eq!(v["resolved_in_session"], 5);
    let back: FindingUpdate = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_update_all_none_serializes_empty() {
    let u = FindingUpdate::default();
    let json = serde_json::to_string(&u).expect("serialize");
    assert_eq!(json, "{}");
    let back: FindingUpdate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, u);
}

#[test]
fn finding_detail_fully_populated_roundtrips() {
    let d = FindingDetail {
        id: "11111111-2222-3333-4444-555555555555".to_string(),
        task_run_id: "run-abc".to_string(),
        session_num: 2,
        category: FindingCategory::Performance,
        severity: FindingSeverity::Medium,
        status: FindingStatus::InProgress,
        title: "Quadratic loop".to_string(),
        description: "Inner loop scans the whole list.".to_string(),
        resolution: Some("Switched to HashSet.".to_string()),
        code_context: Some(FindingCodeContext {
            file: Some("src/scan.rs".to_string()),
            line: Some(88),
            column: None,
            snippet: None,
        }),
        signature_hash: Some("sig-abc".to_string()),
        action_type: FindingActionType::AutoFix,
        user_input: None,
        user_response: None,
        detected_at: "2026-04-14T12:00:00Z".to_string(),
        resolved_at: Some("2026-04-14T13:00:00Z".to_string()),
        resolved_in_session: Some(3),
    };
    let json1 = serde_json::to_string(&d).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["id"], "11111111-2222-3333-4444-555555555555");
    assert_eq!(v["status"], "in_progress");
    assert_eq!(v["detected_at"], "2026-04-14T12:00:00Z");
    assert_eq!(v["resolved_at"], "2026-04-14T13:00:00Z");
    assert_eq!(v["code_context"]["line"], 88);
    let back: FindingDetail = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_list_response_empty_roundtrips() {
    let r = FindingListResponse {
        findings: vec![],
        total: 0,
        limit: 50,
        offset: 0,
        has_more: false,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["total"], 0);
    assert_eq!(v["limit"], 50);
    assert_eq!(v["has_more"], false);
    let back: FindingListResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_list_response_populated_roundtrips() {
    let detail = FindingDetail {
        id: "id-1".to_string(),
        task_run_id: "run-1".to_string(),
        session_num: 1,
        category: FindingCategory::Enhancement,
        severity: FindingSeverity::Info,
        status: FindingStatus::Detected,
        title: "T".to_string(),
        description: "D".to_string(),
        resolution: None,
        code_context: None,
        signature_hash: None,
        action_type: FindingActionType::Informational,
        user_input: None,
        user_response: None,
        detected_at: "2026-04-14T12:00:00Z".to_string(),
        resolved_at: None,
        resolved_in_session: None,
    };
    let r = FindingListResponse {
        findings: vec![detail],
        total: 1,
        limit: 25,
        offset: 0,
        has_more: false,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let back: FindingListResponse = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_summary_fully_populated_roundtrips() {
    let mut by_cat = HashMap::new();
    by_cat.insert("security".to_string(), 2);
    by_cat.insert("performance".to_string(), 1);
    let mut by_sev = HashMap::new();
    by_sev.insert("critical".to_string(), 1);
    by_sev.insert("medium".to_string(), 2);
    let mut by_status = HashMap::new();
    by_status.insert("detected".to_string(), 1);
    by_status.insert("resolved".to_string(), 2);

    let s = FindingSummary {
        task_run_id: "run-xyz".to_string(),
        total: 3,
        by_category: by_cat,
        by_severity: by_sev,
        by_status,
        needs_input_count: 0,
        resolved_count: 2,
        outstanding_count: 1,
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["task_run_id"], "run-xyz");
    assert_eq!(v["total"], 3);
    assert_eq!(v["by_category"]["security"], 2);
    assert_eq!(v["by_severity"]["critical"], 1);
    assert_eq!(v["by_status"]["resolved"], 2);
    assert_eq!(v["resolved_count"], 2);
    let back: FindingSummary = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn finding_summary_empty_maps_skipped() {
    let s = FindingSummary {
        task_run_id: "run-empty".to_string(),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // Empty HashMaps are skipped via skip_serializing_if = "HashMap::is_empty".
    assert!(v.get("by_category").is_none());
    assert!(v.get("by_severity").is_none());
    assert!(v.get("by_status").is_none());
    assert_eq!(v["total"], 0);
    assert_eq!(v["needs_input_count"], 0);
    assert_eq!(v["resolved_count"], 0);
    assert_eq!(v["outstanding_count"], 0);
}

// ============================================================================
// process_management ───────────────────────────────────────────────────────
// ============================================================================

#[test]
fn parser_type_roundtrips_snake_case() {
    // Generic is the default so an empty wire form round-trips to Generic.
    let cases = [
        (ParserType::Python, "\"python\""),
        (ParserType::JavaScript, "\"javascript\""),
        (ParserType::Rust, "\"rust\""),
        (ParserType::Generic, "\"generic\""),
    ];
    for (pt, expected) in cases {
        let json = serde_json::to_string(&pt).unwrap();
        assert_eq!(json, expected, "ParserType snake_case mismatch");
        let back: ParserType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pt);
    }
}

#[test]
fn parser_type_javascript_legacy_alias_deserializes() {
    // Older settings files may contain `"java_script"` from the original
    // rename_all = "snake_case" — the alias keeps them parseable.
    let back: ParserType = serde_json::from_str("\"java_script\"").unwrap();
    assert_eq!(back, ParserType::JavaScript);
    // Canonical form stays `"javascript"`.
    let json = serde_json::to_string(&back).unwrap();
    assert_eq!(json, "\"javascript\"");
}

#[test]
fn process_state_snake_case() {
    let cases = [
        (ProcessState::Stopped, "\"stopped\""),
        (ProcessState::Starting, "\"starting\""),
        (ProcessState::Building, "\"building\""),
        (ProcessState::Running, "\"running\""),
        (ProcessState::Healthy, "\"healthy\""),
        (ProcessState::Stopping, "\"stopping\""),
        (ProcessState::Failed, "\"failed\""),
    ];
    for (state, expected) in cases {
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, expected, "ProcessState snake_case mismatch");
        let back: ProcessState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
    }
}

#[test]
fn output_stream_snake_case() {
    let cases = [
        (OutputStream::Stdout, "\"stdout\""),
        (OutputStream::Stderr, "\"stderr\""),
    ];
    for (stream, expected) in cases {
        let json = serde_json::to_string(&stream).unwrap();
        assert_eq!(json, expected, "OutputStream snake_case mismatch");
        let back: OutputStream = serde_json::from_str(&json).unwrap();
        assert_eq!(back, stream);
    }
}

#[test]
fn output_line_roundtrips() {
    let line = OutputLine {
        timestamp: "2026-04-16T12:34:56Z".to_string(),
        stream: OutputStream::Stdout,
        line: "hello world".to_string(),
    };
    let json1 = serde_json::to_string(&line).unwrap();
    let back: OutputLine = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn process_config_fully_populated_roundtrips() {
    let mut env = HashMap::new();
    env.insert("RUST_LOG".to_string(), "info".to_string());
    let cfg = ProcessConfig {
        id: "proc-1".to_string(),
        name: "FastAPI Backend".to_string(),
        command: "poetry".to_string(),
        args: vec!["run".to_string(), "uvicorn".to_string(), "main:app".to_string()],
        cwd: "/repo/backend".to_string(),
        env,
        health_port: Some(8000),
        parser: ParserType::Python,
        auto_start: true,
        category: "backend".to_string(),
        buffer_size: 2000,
        enabled: true,
        ignore_patterns: vec!["DEBUG: .*".to_string()],
        start_group: 1,
        dev_only: false,
        rebuild_enabled: true,
        build_command: Some("poetry".to_string()),
        build_args: vec!["install".to_string()],
    };
    let json1 = serde_json::to_string(&cfg).unwrap();
    let back: ProcessConfig = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn process_config_minimal_applies_serde_defaults() {
    // Minimal wire form: only the non-defaulted fields. The rest must
    // hydrate from the #[serde(default = "...")] helpers so older configs
    // keep working.
    let json = r#"{
        "id": "proc-min",
        "name": "Minimal",
        "command": "echo",
        "cwd": "/tmp"
    }"#;
    let cfg: ProcessConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.id, "proc-min");
    assert!(cfg.args.is_empty());
    assert!(cfg.env.is_empty());
    assert_eq!(cfg.health_port, None);
    assert_eq!(cfg.parser, ParserType::Generic);
    assert!(!cfg.auto_start);
    assert_eq!(cfg.category, "general");
    assert_eq!(cfg.buffer_size, 2000);
    assert!(cfg.enabled);
    assert!(cfg.ignore_patterns.is_empty());
    assert_eq!(cfg.start_group, 0);
    assert!(!cfg.dev_only);
    assert!(cfg.rebuild_enabled);
    assert_eq!(cfg.build_command, None);
    assert!(cfg.build_args.is_empty());
}

#[test]
fn process_status_roundtrips() {
    let status = ProcessStatus {
        id: "proc-1".to_string(),
        name: "FastAPI Backend".to_string(),
        state: ProcessState::Healthy,
        pid: Some(12345),
        uptime_secs: Some(3600),
        port_healthy: Some(true),
        restart_count: 2,
        error_count: 0,
        category: "backend".to_string(),
        has_build_command: true,
    };
    let json1 = serde_json::to_string(&status).unwrap();
    let back: ProcessStatus = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    // Confirm state wire form.
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["state"], "healthy");
}

#[test]
fn process_status_not_running_roundtrips() {
    let status = ProcessStatus {
        id: "proc-2".to_string(),
        name: "Idle".to_string(),
        state: ProcessState::Stopped,
        pid: None,
        uptime_secs: None,
        port_healthy: None,
        restart_count: 0,
        error_count: 0,
        category: "general".to_string(),
        has_build_command: false,
    };
    let json1 = serde_json::to_string(&status).unwrap();
    let back: ProcessStatus = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
}

// ============================================================================
// ── ticket_system ────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::ticket_system::{
    Ticket, TicketComment, TicketProviderConfig, TicketSource, TicketState,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn ticket_source_snake_case() {
    let cases = [
        (TicketSource::GitHub, "\"github\""),
        (TicketSource::Linear, "\"linear\""),
        (TicketSource::Jira, "\"jira\""),
    ];
    for (src, expected) in cases {
        let json = serde_json::to_string(&src).unwrap();
        assert_eq!(json, expected, "TicketSource snake_case mismatch");
        let back: TicketSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, src);
    }
}

#[test]
fn ticket_state_snake_case() {
    let cases = [
        (TicketState::Open, "\"open\""),
        (TicketState::InProgress, "\"in_progress\""),
        (TicketState::Done, "\"done\""),
        (TicketState::Closed, "\"closed\""),
    ];
    for (state, expected) in cases {
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, expected, "TicketState snake_case mismatch");
        let back: TicketState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
    }
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn ticket_fully_populated_roundtrips() {
    let t = Ticket {
        external_id: "42".to_string(),
        source: TicketSource::GitHub,
        title: "Null deref in parser".to_string(),
        body: "See stack trace.".to_string(),
        labels: vec!["bug".to_string(), "automate".to_string()],
        assignee: Some("alice".to_string()),
        url: "https://github.com/qontinui/runner/issues/42".to_string(),
        state: TicketState::Open,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T01:00:00Z".to_string(),
    };
    let json1 = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["external_id"], "42");
    assert_eq!(v["source"], "github");
    assert_eq!(v["state"], "open");
    assert_eq!(v["labels"][1], "automate");
    assert_eq!(v["assignee"], "alice");
    let back: Ticket = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn ticket_minimal_skips_optional_fields() {
    // No labels, no assignee — labels uses skip_serializing_if = Vec::is_empty,
    // assignee uses skip_serializing_if = Option::is_none.
    let t = Ticket {
        external_id: "t-1".to_string(),
        source: TicketSource::Linear,
        title: "x".to_string(),
        body: "y".to_string(),
        labels: vec![],
        assignee: None,
        url: "https://linear.app/q/issue/ENG-1".to_string(),
        state: TicketState::InProgress,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&t).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("labels").is_none(), "empty labels must be skipped");
    assert!(
        v.get("assignee").is_none(),
        "None assignee must be skipped"
    );
    let back: Ticket = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
}

#[test]
fn ticket_comment_roundtrips() {
    let c = TicketComment {
        id: "c-100".to_string(),
        author: "bob".to_string(),
        body: "LGTM — merging.".to_string(),
        created_at: "2026-04-14T12:00:00Z".to_string(),
    };
    let json1 = serde_json::to_string(&c).unwrap();
    let back: TicketComment = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["id"], "c-100");
    assert_eq!(v["author"], "bob");
}

#[test]
fn ticket_provider_config_fully_populated_roundtrips() {
    let cfg = TicketProviderConfig {
        source: TicketSource::GitHub,
        api_token: "ghp_secret_token".to_string(),
        target: "qontinui/runner".to_string(),
        actionable_labels: vec!["automate".to_string(), "qontinui".to_string()],
        workflow_id: "wf-abc123".to_string(),
        poll_interval_seconds: 120,
        update_on_completion: true,
    };
    let json1 = serde_json::to_string(&cfg).unwrap();
    let v: Value = serde_json::from_str(&json1).unwrap();
    // The api_token is intentionally on the wire — DB persistence relies
    // on it. UI-facing consumers must redact it.
    assert_eq!(v["api_token"], "ghp_secret_token");
    assert_eq!(v["source"], "github");
    assert_eq!(v["target"], "qontinui/runner");
    assert_eq!(v["poll_interval_seconds"], 120);
    assert_eq!(v["update_on_completion"], true);
    let back: TicketProviderConfig = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn ticket_provider_config_minimal_applies_serde_defaults() {
    // Minimum required fields; poll_interval_seconds and update_on_completion
    // must hydrate from their serde defaults, and actionable_labels from its
    // `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
    let json = r#"{
        "source": "linear",
        "api_token": "lin_abc",
        "target": "ENG",
        "workflow_id": "wf-linear"
    }"#;
    let cfg: TicketProviderConfig = serde_json::from_str(json).unwrap();
    assert!(matches!(cfg.source, TicketSource::Linear));
    assert_eq!(cfg.api_token, "lin_abc");
    assert_eq!(cfg.target, "ENG");
    assert!(cfg.actionable_labels.is_empty());
    assert_eq!(cfg.workflow_id, "wf-linear");
    assert_eq!(cfg.poll_interval_seconds, 60);
    assert!(cfg.update_on_completion);
}

#[test]
fn ticket_provider_config_empty_labels_skipped() {
    let cfg = TicketProviderConfig {
        source: TicketSource::Jira,
        api_token: "jira_tok".to_string(),
        target: "PROJ".to_string(),
        actionable_labels: vec![],
        workflow_id: "wf-1".to_string(),
        poll_interval_seconds: 60,
        update_on_completion: true,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(
        v.get("actionable_labels").is_none(),
        "empty actionable_labels must be skipped"
    );
}

// ============================================================================
// orchestration_config ──────────────────────────────────────────────────────
// ============================================================================

#[test]
fn exit_strategy_roundtrips_tagged_snake_case() {
    let cases = [
        (ExitStrategy::Reflection, r#"{"type":"reflection"}"#),
        (
            ExitStrategy::WorkflowVerification,
            r#"{"type":"workflow_verification"}"#,
        ),
        (
            ExitStrategy::FixedIterations,
            r#"{"type":"fixed_iterations"}"#,
        ),
        (
            ExitStrategy::DiagnosticEvaluation,
            r#"{"type":"diagnostic_evaluation"}"#,
        ),
    ];
    for (e, expected) in cases {
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(json, expected, "ExitStrategy tag mismatch");
        let back: ExitStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);
    }
    // Default is Reflection.
    assert_eq!(ExitStrategy::default(), ExitStrategy::Reflection);
}

#[test]
fn between_iterations_roundtrips_with_payload() {
    let cases = [
        (
            BetweenIterations::None,
            r#"{"type":"none"}"#,
        ),
        (
            BetweenIterations::WaitHealthy,
            r#"{"type":"wait_healthy"}"#,
        ),
        (
            BetweenIterations::RestartRunner { rebuild: true },
            r#"{"type":"restart_runner","rebuild":true}"#,
        ),
        (
            BetweenIterations::RestartOnSignal { rebuild: false },
            r#"{"type":"restart_on_signal","rebuild":false}"#,
        ),
    ];
    for (b, expected) in cases {
        let json = serde_json::to_string(&b).unwrap();
        assert_eq!(json, expected, "BetweenIterations mismatch");
        let back: BetweenIterations = serde_json::from_str(&json).unwrap();
        assert_eq!(back, b);
    }
    assert_eq!(BetweenIterations::default(), BetweenIterations::None);
}

#[test]
fn loop_phase_roundtrips_snake_case() {
    let json = serde_json::to_string(&LoopPhase::ImplementingFixes).unwrap();
    assert_eq!(json, "\"implementing_fixes\"");
    let back: LoopPhase = serde_json::from_str(&json).unwrap();
    assert_eq!(back, LoopPhase::ImplementingFixes);
    assert_eq!(LoopPhase::default(), LoopPhase::Idle);
}

#[test]
fn root_cause_category_roundtrips_snake_case() {
    let cases = [
        (RootCauseCategory::BadUiRendering, "\"bad_ui_rendering\""),
        (
            RootCauseCategory::BadUiBridgeEvaluation,
            "\"bad_ui_bridge_evaluation\"",
        ),
        (
            RootCauseCategory::BadVerificationSteps,
            "\"bad_verification_steps\"",
        ),
        (
            RootCauseCategory::BadGenerationPrompt,
            "\"bad_generation_prompt\"",
        ),
        (
            RootCauseCategory::BadStateMachineLogic,
            "\"bad_state_machine_logic\"",
        ),
        (
            RootCauseCategory::InfrastructureIssue,
            "\"infrastructure_issue\"",
        ),
        (RootCauseCategory::Unknown, "\"unknown\""),
    ];
    for (rc, expected) in cases {
        let json = serde_json::to_string(&rc).unwrap();
        assert_eq!(json, expected);
        let back: RootCauseCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(back, rc);
    }
}

#[test]
fn stall_detector_config_has_expected_defaults() {
    let cfg = StallDetectorConfig::default();
    assert_eq!(cfg.max_repeated_actions, 5);
    assert_eq!(cfg.max_total_steps, 100);
    assert_eq!(cfg.stall_timeout_secs, 300);
    assert_eq!(cfg.oscillation_window, 10);
}

#[test]
fn summarization_config_has_expected_defaults() {
    let cfg = SummarizationConfig::default();
    assert!(cfg.enabled);
    assert!((cfg.token_threshold_pct - 0.75).abs() < f32::EPSILON);
    assert_eq!(cfg.max_tokens_budget, 80000);
    assert_eq!(cfg.preserve_last_n_iterations, 2);
    assert_eq!(cfg.summary_max_tokens, 2000);
}

#[test]
fn decomposer_config_has_expected_defaults() {
    let cfg = DecomposerConfig::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.min_subtasks, 3);
    assert_eq!(cfg.max_subtasks, 7);
    assert_eq!(cfg.model_override, None);
}

#[test]
fn orchestration_loop_config_minimal_roundtrips() {
    // Minimal config — only workflow_id required, everything else defaults.
    let json = r#"{
        "workflow_id": "wf-42"
    }"#;
    let cfg: OrchestrationLoopConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.workflow_id, "wf-42");
    assert_eq!(cfg.supervisor_port, 9875, "default supervisor_port");
    assert!(cfg.wait_for_fixer, "default wait_for_fixer");
    assert!(!cfg.retry_on_failure);
    assert_eq!(cfg.max_iterations, None);
    assert!(cfg.pipeline.is_none());
    assert_eq!(cfg.exit_strategy, ExitStrategy::Reflection);
    assert_eq!(cfg.between_iterations, BetweenIterations::None);
}

#[test]
fn orchestration_loop_config_optional_fields_skipped_when_none() {
    let cfg = OrchestrationLoopConfig {
        target_runner_port: None,
        target_runner_id: None,
        supervisor_port: 9875,
        workflow_id: "wf-x".to_string(),
        max_iterations: None,
        exit_strategy: ExitStrategy::default(),
        between_iterations: BetweenIterations::default(),
        retry_on_failure: false,
        wait_for_fixer: true,
        pipeline: None,
        stall_detection: None,
        summarization: None,
        decomposition: None,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("target_runner_port").is_none());
    assert!(v.get("target_runner_id").is_none());
    assert!(v.get("max_iterations").is_none());
    assert!(v.get("pipeline").is_none());
    assert!(v.get("stall_detection").is_none());
    assert!(v.get("summarization").is_none());
    assert!(v.get("decomposition").is_none());
}

#[test]
fn ol_config_roundtrips() {
    let cfg = OlConfig {
        id: "abc".to_string(),
        name: "Nightly".to_string(),
        description: Some("A preset".to_string()),
        is_favorite: true,
        config_json: json!({"workflow_id": "wf-42"}),
        created_at: "2026-04-16T00:00:00Z".to_string(),
        updated_at: "2026-04-16T00:00:00Z".to_string(),
    };
    let json1 = serde_json::to_string(&cfg).unwrap();
    let back: OlConfig = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn create_ol_config_request_omits_optional_description() {
    let req = CreateOlConfigRequest {
        name: "n".to_string(),
        description: None,
        config_json: json!({}),
    };
    let json = serde_json::to_string(&req).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("description").is_none(), "description must be skipped");
}

#[test]
fn update_ol_config_request_all_none_serializes_empty() {
    let req = UpdateOlConfigRequest {
        name: None,
        description: None,
        is_favorite: None,
        config_json: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn diagnose_phase_config_defaults_applied() {
    let json = "{}";
    let cfg: DiagnosePhaseConfig = serde_json::from_str(json).unwrap();
    assert!(cfg.assertions.is_empty());
    assert!(cfg.capture_snapshot, "capture_snapshot defaults to true");
    assert_eq!(cfg.snapshot_max_chars, 8000);
    assert_eq!(cfg.model_override, None);
}

#[test]
fn iteration_result_optional_fields_skipped() {
    let ir = IterationResult {
        iteration: 1,
        started_at: "2026-04-16T00:00:00Z".to_string(),
        completed_at: "2026-04-16T00:01:00Z".to_string(),
        task_run_id: "tr-1".to_string(),
        reflection_task_run_id: None,
        fix_count: None,
        exit_check: ExitCheckResult {
            should_exit: false,
            reason: "continue".to_string(),
        },
        generated_workflow_id: None,
        fixes_implemented: None,
        rebuild_triggered: None,
        stall_detected: None,
        context_summarized: None,
        diagnostic_result: None,
    };
    let json = serde_json::to_string(&ir).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("reflection_task_run_id").is_none());
    assert!(v.get("diagnostic_result").is_none());
    assert!(v.get("rebuild_triggered").is_none());
}

#[test]
fn multi_loop_config_roundtrips() {
    let cfg = MultiLoopConfig {
        loops: vec![MultiLoopEntry {
            loop_id: "l1".to_string(),
            label: Some("Pages 1-10".to_string()),
            config: serde_json::from_str(r#"{"workflow_id":"wf-a"}"#).unwrap(),
        }],
        stop_all_on_error: true,
    };
    let json1 = serde_json::to_string(&cfg).unwrap();
    let back: MultiLoopConfig = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    assert_eq!(back.loops.len(), 1);
    assert_eq!(back.loops[0].loop_id, "l1");
}

// ============================================================================
// ── discovery ────────────────────────────────────────────────────────────────
// ============================================================================

use qontinui_types::discovery::{
    DiscoveredState, DiscoveredStateImage, DiscoveredTransition, DiscoveryBoundingBox,
    DiscoverySourceType, DiscoveryTransitionTrigger, StateDiscoveryResult,
    StateDiscoveryResultCreate, StateDiscoveryResultListResponse,
    StateDiscoveryResultSummary, StateDiscoveryResultUpdate, StateMachineExport,
    StateMachineImport, TransitionTriggerType,
};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[test]
fn discovery_source_type_all_variants_roundtrip() {
    for (variant, expected) in [
        (DiscoverySourceType::Playwright, "playwright"),
        (DiscoverySourceType::UiBridge, "ui_bridge"),
        (DiscoverySourceType::Recording, "recording"),
        (DiscoverySourceType::Vision, "vision"),
        (DiscoverySourceType::Manual, "manual"),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, format!("\"{}\"", expected));
        let back: DiscoverySourceType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, variant);
    }
}

#[test]
fn transition_trigger_type_all_variants_roundtrip() {
    for (variant, expected) in [
        (TransitionTriggerType::Click, "click"),
        (TransitionTriggerType::Type, "type"),
        (TransitionTriggerType::Scroll, "scroll"),
        (TransitionTriggerType::Hover, "hover"),
        (TransitionTriggerType::Custom, "custom"),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, format!("\"{}\"", expected));
        let back: TransitionTriggerType =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, variant);
    }
}

#[test]
fn transition_trigger_type_default_is_click() {
    // Mirrors Python `default=TransitionTriggerType.CLICK` on
    // `DiscoveryTransitionTrigger.type`.
    assert_eq!(TransitionTriggerType::default(), TransitionTriggerType::Click);
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn discovery_bounding_box_fully_populated_roundtrips() {
    let b = DiscoveryBoundingBox {
        x: 10.5,
        y: 20.0,
        width: 100.25,
        height: 50.0,
    };
    let json1 = serde_json::to_string(&b).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["x"], 10.5);
    assert_eq!(v["y"], 20.0);
    assert_eq!(v["width"], 100.25);
    assert_eq!(v["height"], 50.0);
    let back: DiscoveryBoundingBox = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn discovery_transition_trigger_fully_populated_roundtrips() {
    let t = DiscoveryTransitionTrigger {
        r#type: TransitionTriggerType::Type,
        image_id: Some("img-1".to_string()),
        element_id: Some("el-42".to_string()),
        selector: Some("#login-button".to_string()),
        value: Some("hello".to_string()),
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["type"], "type");
    assert_eq!(v["imageId"], "img-1");
    assert_eq!(v["elementId"], "el-42");
    assert_eq!(v["selector"], "#login-button");
    assert_eq!(v["value"], "hello");
    let back: DiscoveryTransitionTrigger =
        serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn discovery_transition_trigger_defaults_and_skip_none() {
    let t = DiscoveryTransitionTrigger::default();
    let json = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    // type has #[serde(default)] (no skip) so it should be present as "click".
    assert_eq!(v["type"], "click");
    // None Options must be skipped.
    assert!(v.get("imageId").is_none());
    assert!(v.get("elementId").is_none());
    assert!(v.get("selector").is_none());
    assert!(v.get("value").is_none());
}

#[test]
fn discovered_state_image_fully_populated_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("tag".to_string(), Value::String("primary".to_string()));
    let img = DiscoveredStateImage {
        id: "img-1".to_string(),
        screenshot_id: Some("scr-1".to_string()),
        screenshot_url: Some("https://ex.com/s.png".to_string()),
        bbox: DiscoveryBoundingBox {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        pixel_hash: Some("deadbeef".to_string()),
        state_id: Some("st-1".to_string()),
        element_type: Some("button".to_string()),
        label: Some("Login".to_string()),
        confidence: 0.9,
        metadata: Some(meta),
    };
    let json1 = serde_json::to_string(&img).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["id"], "img-1");
    assert_eq!(v["screenshotId"], "scr-1");
    assert_eq!(v["screenshotUrl"], "https://ex.com/s.png");
    assert_eq!(v["bbox"]["width"], 100.0);
    assert_eq!(v["pixelHash"], "deadbeef");
    assert_eq!(v["stateId"], "st-1");
    assert_eq!(v["elementType"], "button");
    assert_eq!(v["label"], "Login");
    assert_eq!(v["confidence"], 0.9);
    assert_eq!(v["metadata"]["tag"], "primary");
    let back: DiscoveredStateImage = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn discovered_state_image_confidence_defaults_to_one() {
    // Omitting confidence on the wire should hydrate to 1.0 (Python default).
    let json = r#"{
        "id":"img-x",
        "bbox":{"x":0.0,"y":0.0,"width":1.0,"height":1.0}
    }"#;
    let img: DiscoveredStateImage = serde_json::from_str(json).expect("deserialize");
    assert_eq!(img.confidence, 1.0);
    assert!(img.metadata.is_none());
    assert!(img.screenshot_id.is_none());
}

#[test]
fn discovered_state_fully_populated_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("source".to_string(), Value::String("playwright".to_string()));
    let s = DiscoveredState {
        id: "st-1".to_string(),
        name: "Login page".to_string(),
        image_ids: vec!["img-1".to_string(), "img-2".to_string()],
        render_ids: vec!["r-1".to_string()],
        element_ids: vec!["el-1".to_string(), "el-2".to_string()],
        confidence: 0.85,
        description: Some("The login screen".to_string()),
        metadata: Some(meta),
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["id"], "st-1");
    assert_eq!(v["name"], "Login page");
    assert_eq!(v["imageIds"].as_array().unwrap().len(), 2);
    assert_eq!(v["renderIds"][0], "r-1");
    assert_eq!(v["elementIds"][1], "el-2");
    assert_eq!(v["confidence"], 0.85);
    assert_eq!(v["description"], "The login screen");
    assert_eq!(v["metadata"]["source"], "playwright");
    let back: DiscoveredState = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn discovered_state_empty_lists_are_skipped() {
    let s = DiscoveredState {
        id: "st-empty".to_string(),
        name: "Empty".to_string(),
        image_ids: vec![],
        render_ids: vec![],
        element_ids: vec![],
        confidence: 1.0,
        description: None,
        metadata: None,
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    assert!(v.get("imageIds").is_none());
    assert!(v.get("renderIds").is_none());
    assert!(v.get("elementIds").is_none());
    assert!(v.get("description").is_none());
    assert!(v.get("metadata").is_none());
}

#[test]
fn discovered_transition_fully_populated_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("score".to_string(), Value::from(0.42));
    let t = DiscoveredTransition {
        id: "tr-1".to_string(),
        from_state_id: "st-1".to_string(),
        to_state_id: "st-2".to_string(),
        trigger: Some(DiscoveryTransitionTrigger {
            r#type: TransitionTriggerType::Click,
            image_id: Some("img-1".to_string()),
            element_id: None,
            selector: Some("button#submit".to_string()),
            value: None,
        }),
        confidence: 0.95,
        metadata: Some(meta),
    };
    let json1 = serde_json::to_string(&t).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["id"], "tr-1");
    assert_eq!(v["fromStateId"], "st-1");
    assert_eq!(v["toStateId"], "st-2");
    assert_eq!(v["trigger"]["type"], "click");
    assert_eq!(v["trigger"]["imageId"], "img-1");
    assert_eq!(v["trigger"]["selector"], "button#submit");
    assert_eq!(v["confidence"], 0.95);
    let back: DiscoveredTransition = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_discovery_result_fully_populated_roundtrips() {
    let mut disco_meta = HashMap::new();
    disco_meta.insert("engine".to_string(), Value::String("v2".to_string()));
    let mut e2r: HashMap<String, Vec<String>> = HashMap::new();
    e2r.insert(
        "el-1".to_string(),
        vec!["r-1".to_string(), "r-2".to_string()],
    );

    let img = DiscoveredStateImage {
        id: "img-1".to_string(),
        screenshot_id: None,
        screenshot_url: None,
        bbox: DiscoveryBoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
        pixel_hash: None,
        state_id: Some("st-1".to_string()),
        element_type: None,
        label: None,
        confidence: 1.0,
        metadata: None,
    };
    let state = DiscoveredState {
        id: "st-1".to_string(),
        name: "Home".to_string(),
        image_ids: vec!["img-1".to_string()],
        render_ids: vec![],
        element_ids: vec!["el-1".to_string()],
        confidence: 1.0,
        description: None,
        metadata: None,
    };
    let tr = DiscoveredTransition {
        id: "tr-1".to_string(),
        from_state_id: "st-1".to_string(),
        to_state_id: "st-2".to_string(),
        trigger: None,
        confidence: 0.8,
        metadata: None,
    };

    let result = StateDiscoveryResult {
        id: "res-1".to_string(),
        project_id: "proj-xyz".to_string(),
        name: "Nightly discovery".to_string(),
        description: Some("Playwright run over /app".to_string()),
        source_type: DiscoverySourceType::Playwright,
        source_session_id: Some("sess-99".to_string()),
        discovery_strategy: Some("auto".to_string()),
        images: vec![img],
        states: vec![state],
        transitions: vec![tr],
        element_to_renders: e2r,
        image_count: 1,
        state_count: 1,
        transition_count: 1,
        render_count: 5,
        unique_element_count: 3,
        confidence: 0.9,
        discovery_metadata: disco_meta,
        created_at: "2026-04-16T00:00:00Z".to_string(),
        updated_at: "2026-04-16T01:00:00Z".to_string(),
    };
    let json1 = serde_json::to_string(&result).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["id"], "res-1");
    assert_eq!(v["projectId"], "proj-xyz");
    assert_eq!(v["sourceType"], "playwright");
    assert_eq!(v["sourceSessionId"], "sess-99");
    assert_eq!(v["discoveryStrategy"], "auto");
    assert_eq!(v["elementToRenders"]["el-1"][1], "r-2");
    assert_eq!(v["imageCount"], 1);
    assert_eq!(v["stateCount"], 1);
    assert_eq!(v["transitionCount"], 1);
    assert_eq!(v["renderCount"], 5);
    assert_eq!(v["uniqueElementCount"], 3);
    assert_eq!(v["discoveryMetadata"]["engine"], "v2");
    assert_eq!(v["createdAt"], "2026-04-16T00:00:00Z");
    assert_eq!(v["updatedAt"], "2026-04-16T01:00:00Z");
    let back: StateDiscoveryResult = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_discovery_result_summary_roundtrips() {
    let s = StateDiscoveryResultSummary {
        id: "res-1".to_string(),
        project_id: "proj-1".to_string(),
        name: "Summary".to_string(),
        description: Some("brief".to_string()),
        source_type: DiscoverySourceType::UiBridge,
        discovery_strategy: Some("fingerprint".to_string()),
        image_count: 10,
        state_count: 3,
        transition_count: 4,
        confidence: 0.77,
        created_at: "2026-04-16T00:00:00Z".to_string(),
    };
    let json1 = serde_json::to_string(&s).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["sourceType"], "ui_bridge");
    assert_eq!(v["discoveryStrategy"], "fingerprint");
    assert_eq!(v["imageCount"], 10);
    assert_eq!(v["stateCount"], 3);
    assert_eq!(v["transitionCount"], 4);
    assert_eq!(v["createdAt"], "2026-04-16T00:00:00Z");
    let back: StateDiscoveryResultSummary =
        serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_discovery_result_list_response_roundtrips() {
    let r = StateDiscoveryResultListResponse {
        items: vec![StateDiscoveryResultSummary {
            id: "res-1".to_string(),
            project_id: "p-1".to_string(),
            name: "N".to_string(),
            description: None,
            source_type: DiscoverySourceType::Manual,
            discovery_strategy: None,
            image_count: 0,
            state_count: 0,
            transition_count: 0,
            confidence: 0.0,
            created_at: "2026-04-16T00:00:00Z".to_string(),
        }],
        total: 1,
    };
    let json1 = serde_json::to_string(&r).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["total"], 1);
    assert_eq!(v["items"][0]["sourceType"], "manual");
    let back: StateDiscoveryResultListResponse =
        serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_discovery_result_create_fully_populated_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("note".to_string(), Value::String("seed".to_string()));
    let mut e2r: HashMap<String, Vec<String>> = HashMap::new();
    e2r.insert("el-x".to_string(), vec!["r-1".to_string()]);
    let c = StateDiscoveryResultCreate {
        name: "Create".to_string(),
        description: Some("desc".to_string()),
        source_type: DiscoverySourceType::Recording,
        source_session_id: Some("sess-1".to_string()),
        discovery_strategy: Some("legacy".to_string()),
        images: vec![],
        states: vec![],
        transitions: vec![],
        element_to_renders: e2r,
        confidence: 0.5,
        discovery_metadata: meta,
    };
    let json1 = serde_json::to_string(&c).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["sourceType"], "recording");
    assert_eq!(v["sourceSessionId"], "sess-1");
    assert_eq!(v["discoveryStrategy"], "legacy");
    assert_eq!(v["elementToRenders"]["el-x"][0], "r-1");
    assert_eq!(v["discoveryMetadata"]["note"], "seed");
    // Empty arrays must be skipped.
    assert!(v.get("images").is_none());
    assert!(v.get("states").is_none());
    assert!(v.get("transitions").is_none());
    let back: StateDiscoveryResultCreate =
        serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_discovery_result_update_fully_populated_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert("v".to_string(), Value::from(2));
    let u = StateDiscoveryResultUpdate {
        name: Some("new name".to_string()),
        description: Some("new desc".to_string()),
        images: Some(vec![]),
        states: Some(vec![]),
        transitions: Some(vec![]),
        discovery_metadata: Some(meta),
    };
    let json1 = serde_json::to_string(&u).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["name"], "new name");
    assert_eq!(v["description"], "new desc");
    // `images` is Option<Vec<…>> → Some(vec![]) serializes as []
    assert!(v["images"].is_array());
    assert_eq!(v["images"].as_array().unwrap().len(), 0);
    assert_eq!(v["discoveryMetadata"]["v"], 2);
    let back: StateDiscoveryResultUpdate =
        serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_discovery_result_update_all_none_serializes_empty() {
    let u = StateDiscoveryResultUpdate::default();
    let json = serde_json::to_string(&u).expect("serialize");
    assert_eq!(json, "{}");
}

#[test]
fn state_machine_export_fully_populated_roundtrips() {
    let mut meta = HashMap::new();
    meta.insert(
        "originalId".to_string(),
        Value::String("res-1".to_string()),
    );
    let mut e2r: HashMap<String, Vec<String>> = HashMap::new();
    e2r.insert("el-a".to_string(), vec!["r-a".to_string()]);
    let e = StateMachineExport {
        version: "1.0.0".to_string(),
        name: "Export".to_string(),
        description: Some("portable".to_string()),
        source_type: "playwright".to_string(),
        images: vec![],
        states: vec![],
        transitions: vec![],
        element_to_renders: e2r,
        metadata: meta,
    };
    let json1 = serde_json::to_string(&e).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["version"], "1.0.0");
    assert_eq!(v["name"], "Export");
    assert_eq!(v["sourceType"], "playwright");
    assert_eq!(v["elementToRenders"]["el-a"][0], "r-a");
    assert_eq!(v["metadata"]["originalId"], "res-1");
    let back: StateMachineExport = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_machine_export_version_defaults_when_omitted() {
    // Python default for `version` is "1.0.0". Omitting it on the wire must
    // hydrate via the #[serde(default = "…")] helper.
    let json = r#"{
        "name": "E",
        "sourceType": "manual"
    }"#;
    let e: StateMachineExport = serde_json::from_str(json).expect("deserialize");
    assert_eq!(e.version, "1.0.0");
    assert_eq!(e.source_type, "manual");
    assert!(e.description.is_none());
    assert!(e.images.is_empty());
    assert!(e.metadata.is_empty());
}

#[test]
fn state_machine_import_fully_populated_roundtrips() {
    let export = StateMachineExport {
        version: "1.0.0".to_string(),
        name: "E".to_string(),
        description: None,
        source_type: "manual".to_string(),
        images: vec![],
        states: vec![],
        transitions: vec![],
        element_to_renders: HashMap::new(),
        metadata: HashMap::new(),
    };
    let i = StateMachineImport {
        state_machine: export,
        name: Some("override".to_string()),
    };
    let json1 = serde_json::to_string(&i).expect("serialize");
    let v: Value = serde_json::from_str(&json1).expect("parse");
    assert_eq!(v["stateMachine"]["name"], "E");
    assert_eq!(v["stateMachine"]["sourceType"], "manual");
    assert_eq!(v["name"], "override");
    let back: StateMachineImport = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_json_values_equal(&json1, &json2);
}

#[test]
fn state_machine_import_name_optional() {
    let export = StateMachineExport {
        version: "1.0.0".to_string(),
        name: "E".to_string(),
        description: None,
        source_type: "manual".to_string(),
        images: vec![],
        states: vec![],
        transitions: vec![],
        element_to_renders: HashMap::new(),
        metadata: HashMap::new(),
    };
    let i = StateMachineImport {
        state_machine: export,
        name: None,
    };
    let json = serde_json::to_string(&i).expect("serialize");
    let v: Value = serde_json::from_str(&json).expect("parse");
    assert!(v.get("name").is_none(), "None override name must be skipped");
}

// ============================================================================
// ── verification ───
// Wire contract: planning-agent outputs consumed by the verification loop +
// persisted to DB + returned from the task-run API.
// ============================================================================

use qontinui_types::verification::{
    Confidence, CriterionOverride, CriterionType, DomainAssignment, DomainVerificationResult,
    ExtendIterationsRequest, Finding as VerificationFinding, IterationVerificationResults,
    OverrideCollection, StageTransition, SuccessCriterion, TaskCompletionResult,
    VerificationAgentContext, VerificationMethod as VerifMethod,
    VerificationPlan as VerifPlan, VerificationResult as VerifResult,
    WorkerCoordinationMessage, WorkerDomain, WorkerInstance, WorkerSignal, WorkerStatus,
};

#[test]
fn criterion_type_roundtrips_snake_case() {
    let v = CriterionType::AiEvaluated;
    let json = serde_json::to_string(&v).unwrap();
    assert_eq!(json, r#""ai_evaluated""#);
    let back: CriterionType = serde_json::from_str(&json).unwrap();
    assert_eq!(back, CriterionType::AiEvaluated);

    let v2 = CriterionType::Deterministic;
    let json2 = serde_json::to_string(&v2).unwrap();
    assert_eq!(json2, r#""deterministic""#);
}

#[test]
fn verification_method_roundtrips_snake_case() {
    let cases = [
        (VerifMethod::BuildSuccess, "build_success"),
        (VerifMethod::UnitTest, "unit_test"),
        (VerifMethod::IntegrationTest, "integration_test"),
        (VerifMethod::Playwright, "playwright"),
        (VerifMethod::LogPattern, "log_pattern"),
        (VerifMethod::GuiAutomation, "gui_automation"),
        (VerifMethod::TypeCheck, "type_check"),
        (VerifMethod::LintCheck, "lint_check"),
        (VerifMethod::CustomCommand, "custom_command"),
    ];
    for (v, s) in cases {
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, format!("\"{}\"", s));
        let back: VerifMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn success_criterion_minimal_roundtrips() {
    // `type` is the wire-renamed discriminator. `required` / `is_critical`
    // default to true and should hydrate from serde defaults when omitted.
    let json = r#"{
        "id": "c1",
        "description": "Type check passes",
        "type": "deterministic"
    }"#;
    let c: SuccessCriterion = serde_json::from_str(json).unwrap();
    assert_eq!(c.id, "c1");
    assert_eq!(c.criterion_type, CriterionType::Deterministic);
    assert!(c.required);
    assert!(c.is_critical);
    assert!(c.verification_method.is_none());
    assert!(c.weight.is_none());
    assert!(c.domain.is_none());

    let json2 = serde_json::to_string(&c).unwrap();
    let back: SuccessCriterion = serde_json::from_str(&json2).unwrap();
    assert_eq!(json2, serde_json::to_string(&back).unwrap());
}

#[test]
fn success_criterion_full_roundtrips() {
    let c = SuccessCriterion {
        id: "c2".to_string(),
        description: "Playwright script passes".to_string(),
        criterion_type: CriterionType::Deterministic,
        verification_method: Some(VerifMethod::Playwright),
        verification_config: Some(serde_json::json!({"script": "tests/login.spec.ts"})),
        evaluation_prompt: None,
        required: true,
        is_critical: false,
        weight: Some(0.5),
        domain: Some("frontend".to_string()),
    };
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    // Confirm the wire-side rename: `criterion_type` → `type`.
    assert_eq!(v["type"], "deterministic");
    assert_eq!(v["verification_method"], "playwright");
    assert_eq!(v["weight"], 0.5);
    assert_eq!(v["domain"], "frontend");
    let back: SuccessCriterion = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn verification_plan_minimal_roundtrips() {
    // `execution_steps` / `worker_domains` absent; `suggested_worker_count`
    // and `version` default to 1.
    let json = r#"{
        "goal_summary": "Make tests pass",
        "success_criteria": []
    }"#;
    let p: VerifPlan = serde_json::from_str(json).unwrap();
    assert_eq!(p.goal_summary, "Make tests pass");
    assert!(p.success_criteria.is_empty());
    assert!(p.execution_steps.is_empty());
    assert_eq!(p.suggested_worker_count, 1);
    assert_eq!(p.version, 1);
    assert!(p.worker_domains.is_none());

    let json2 = serde_json::to_string(&p).unwrap();
    let back: VerifPlan = serde_json::from_str(&json2).unwrap();
    assert_eq!(json2, serde_json::to_string(&back).unwrap());
}

#[test]
fn verification_plan_full_roundtrips() {
    let p = VerifPlan {
        goal_summary: "Add login flow".to_string(),
        success_criteria: vec![SuccessCriterion {
            id: "c1".to_string(),
            description: "Login succeeds".to_string(),
            criterion_type: CriterionType::AiEvaluated,
            verification_method: None,
            verification_config: None,
            evaluation_prompt: Some("Check screenshot shows dashboard".to_string()),
            required: true,
            is_critical: true,
            weight: None,
            domain: Some("frontend".to_string()),
        }],
        execution_steps: vec![serde_json::json!({"type": "gui_automation", "action": "click"})],
        suggested_worker_count: 2,
        worker_domains: Some(vec![WorkerDomain {
            worker_id: "w1".to_string(),
            specialization: Some("frontend".to_string()),
            file_patterns: vec!["src/**/*.tsx".to_string()],
            system_prompt_additions: None,
        }]),
        version: 3,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: VerifPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn domain_assignment_roundtrips() {
    let d = DomainAssignment {
        domain_id: "frontend".to_string(),
        name: "Frontend".to_string(),
        description: "UI layer".to_string(),
        file_patterns: vec!["src/ui/**/*.ts".to_string()],
        keywords: vec!["react".to_string(), "tsx".to_string()],
        assigned_workers: vec!["w1".to_string()],
        domain_criteria: vec!["c1".to_string()],
        system_prompt_context: Some("Frontend specialist".to_string()),
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: DomainAssignment = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn worker_status_roundtrips_snake_case() {
    let cases = [
        (WorkerStatus::Idle, "idle"),
        (WorkerStatus::Active, "active"),
        (WorkerStatus::AwaitingVerification, "awaiting_verification"),
        (WorkerStatus::Paused, "paused"),
        (WorkerStatus::Completed, "completed"),
        (WorkerStatus::Error, "error"),
    ];
    for (v, s) in cases {
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, format!("\"{}\"", s));
        let back: WorkerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn worker_instance_minimal_roundtrips() {
    let w = WorkerInstance {
        worker_id: "w1".to_string(),
        name: "Worker One".to_string(),
        status: WorkerStatus::Idle,
        domain: None,
        iteration: 0,
        max_iterations: 10,
        last_signal: None,
        findings: vec![],
        touched_files: vec![],
        started_at: None,
        completed_at: None,
        error_message: None,
    };
    let json = serde_json::to_string(&w).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    // Empty Vecs / None options are skipped.
    assert!(v.get("findings").is_none());
    assert!(v.get("touched_files").is_none());
    assert!(v.get("started_at").is_none());
    let back: WorkerInstance = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn worker_coordination_message_files_modified_roundtrips() {
    // Externally tagged as { "type": ..., "data": { ... } }.
    let m = WorkerCoordinationMessage::FilesModified {
        worker_id: "w1".to_string(),
        files: vec!["src/a.rs".to_string()],
    };
    let json = serde_json::to_string(&m).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "files_modified");
    assert_eq!(v["data"]["worker_id"], "w1");
    assert_eq!(v["data"]["files"][0], "src/a.rs");
    let back: WorkerCoordinationMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn worker_coordination_message_all_variants_roundtrip() {
    let finding = VerificationFinding {
        id: "f1".to_string(),
        finding_type: "bug".to_string(),
        description: "crash".to_string(),
        evidence: None,
        confidence: Confidence::High,
        related_files: vec![],
    };
    let variants = vec![
        WorkerCoordinationMessage::SharedFinding {
            worker_id: "w1".to_string(),
            finding,
        },
        WorkerCoordinationMessage::Blocked {
            worker_id: "w1".to_string(),
            waiting_for: "w2".to_string(),
            reason: "schema update".to_string(),
        },
        WorkerCoordinationMessage::ReadyForVerification {
            worker_id: "w1".to_string(),
            domain: Some("frontend".to_string()),
        },
        WorkerCoordinationMessage::SyncPoint {
            worker_ids: vec!["w1".to_string(), "w2".to_string()],
            reason: "merge".to_string(),
        },
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: WorkerCoordinationMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&back).unwrap());
    }
}

#[test]
fn worker_signal_roundtrips_all_variants() {
    // Tagged with `signal` (not `type`) to match pre-extraction wire.
    let wc = WorkerSignal::WorkComplete {
        reason: Some("done".to_string()),
    };
    let json = serde_json::to_string(&wc).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["signal"], "work_complete");
    assert_eq!(v["data"]["reason"], "done");
    let back: WorkerSignal = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());

    let rp = WorkerSignal::NeedReplan {
        reason: "plan wrong".to_string(),
    };
    let json2 = serde_json::to_string(&rp).unwrap();
    let v2: Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(v2["signal"], "need_replan");
    let back2: WorkerSignal = serde_json::from_str(&json2).unwrap();
    assert_eq!(json2, serde_json::to_string(&back2).unwrap());

    let cont = WorkerSignal::Continue;
    let json3 = serde_json::to_string(&cont).unwrap();
    let v3: Value = serde_json::from_str(&json3).unwrap();
    assert_eq!(v3["signal"], "continue");

    let f = WorkerSignal::Finding(VerificationFinding {
        id: "f1".to_string(),
        finding_type: "bug".to_string(),
        description: "crash".to_string(),
        evidence: None,
        confidence: Confidence::Medium,
        related_files: vec![],
    });
    let json4 = serde_json::to_string(&f).unwrap();
    let v4: Value = serde_json::from_str(&json4).unwrap();
    assert_eq!(v4["signal"], "finding");
    let back4: WorkerSignal = serde_json::from_str(&json4).unwrap();
    assert_eq!(json4, serde_json::to_string(&back4).unwrap());
}

#[test]
fn finding_minimal_roundtrips() {
    let f = VerificationFinding {
        id: "f1".to_string(),
        finding_type: "observation".to_string(),
        description: "Slow DB query".to_string(),
        evidence: None,
        confidence: Confidence::Low,
        related_files: vec![],
    };
    let json = serde_json::to_string(&f).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("evidence").is_none());
    assert!(v.get("related_files").is_none());
    let back: VerificationFinding = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn confidence_roundtrips_lowercase() {
    for (c, s) in [
        (Confidence::High, "high"),
        (Confidence::Medium, "medium"),
        (Confidence::Low, "low"),
    ] {
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, format!("\"{}\"", s));
        let back: Confidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }
}

#[test]
fn verification_result_roundtrips() {
    let r = VerifResult {
        criterion_id: "c1".to_string(),
        passed: false,
        criterion_type: CriterionType::Deterministic,
        confidence: Some(Confidence::High),
        observations: vec!["Type error in main.rs:42".to_string()],
        issues: vec!["E0308 mismatched types".to_string()],
        suggestions: vec!["Convert u32 to i32".to_string()],
        raw_output: Some("error[E0308]".to_string()),
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: VerifResult = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn iteration_verification_results_roundtrips() {
    let r = IterationVerificationResults {
        iteration: 3,
        deterministic_results: vec![],
        ai_results: vec![],
        deterministic_passed: true,
        ai_passed: true,
        all_passed: true,
        failure_summary: None,
        applied_overrides: vec![],
        overridden_criteria: vec![],
    };
    let json = serde_json::to_string(&r).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    // Empty collection fields should be skipped on the wire.
    assert!(v.get("applied_overrides").is_none());
    assert!(v.get("overridden_criteria").is_none());
    assert!(v.get("failure_summary").is_none());
    let back: IterationVerificationResults = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn criterion_override_roundtrips() {
    let o = CriterionOverride {
        criterion_id: "c1".to_string(),
        item: "FatClass".to_string(),
        justification: "Pre-existing; out of scope".to_string(),
        iteration: 2,
        worker_id: Some("w1".to_string()),
        recorded_at: "2026-04-16T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&o).unwrap();
    let back: CriterionOverride = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn override_collection_default_roundtrips() {
    let c = OverrideCollection::default();
    let json = serde_json::to_string(&c).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    // Empty overrides skipped on the wire.
    assert!(v.get("overrides").is_none());
    let back: OverrideCollection = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn task_completion_result_success_roundtrips() {
    // Internally tagged by `status`.
    let r = TaskCompletionResult::Success {
        iterations: 5,
        findings: vec![],
        verification_results: IterationVerificationResults {
            iteration: 5,
            deterministic_results: vec![],
            ai_results: vec![],
            deterministic_passed: true,
            ai_passed: true,
            all_passed: true,
            failure_summary: None,
            applied_overrides: vec![],
            overridden_criteria: vec![],
        },
    };
    let json = serde_json::to_string(&r).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "success");
    assert_eq!(v["iterations"], 5);
    let back: TaskCompletionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn task_completion_result_all_variants_roundtrip() {
    let variants = vec![
        TaskCompletionResult::Failed {
            reason: "max iterations".to_string(),
            iterations: 10,
            last_results: None,
            findings: vec![],
        },
        TaskCompletionResult::Stopped {
            at_iteration: 3,
            findings: vec![],
            can_resume: true,
        },
        TaskCompletionResult::Paused {
            at_iteration: 10,
            max_iterations: 10,
            last_results: None,
            findings: vec![],
        },
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: TaskCompletionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&back).unwrap());
    }
}

#[test]
fn stage_transition_roundtrips() {
    let s = StageTransition {
        from: "planning".to_string(),
        to: "execution".to_string(),
        timestamp: "2026-04-16T12:00:00Z".to_string(),
        iteration: 1,
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: StageTransition = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn verification_agent_context_roundtrips() {
    let c = VerificationAgentContext {
        screenshot_base64: "iVBORw0KGgo=".to_string(),
        evaluation_prompt: "Does the dashboard show?".to_string(),
        goal_context: "Add login flow".to_string(),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: VerificationAgentContext = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn extend_iterations_request_roundtrips() {
    let r = ExtendIterationsRequest {
        additional_iterations: 5,
        guidance: Some("focus on tests".to_string()),
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: ExtendIterationsRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());

    let r2 = ExtendIterationsRequest {
        additional_iterations: 2,
        guidance: None,
    };
    let json2 = serde_json::to_string(&r2).unwrap();
    let v2: Value = serde_json::from_str(&json2).unwrap();
    assert!(v2.get("guidance").is_none(), "None guidance must be skipped");
}

#[test]
fn domain_verification_result_roundtrips() {
    let d = DomainVerificationResult {
        domain_id: "frontend".to_string(),
        worker_ids: vec!["w1".to_string()],
        results: vec![],
        all_passed: true,
        failure_summary: None,
    };
    let json = serde_json::to_string(&d).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("failure_summary").is_none());
    let back: DomainVerificationResult = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

// ============================================================================
// terminal ──────────────────────────────────────────────────────────────────
// ============================================================================

#[test]
fn terminal_info_fully_populated_roundtrips() {
    let info = TerminalInfo {
        id: "term-1".to_string(),
        title: "Terminal 1".to_string(),
        pid: Some(4242),
        cols: 120,
        rows: 30,
        working_dir: "/repo".to_string(),
        is_alive: true,
        exit_code: None,
        created_at: 1_713_270_000_000,
        total_bytes_produced: 1024,
        page_id: "page-a".to_string(),
    };
    let json1 = serde_json::to_string(&info).unwrap();
    let back: TerminalInfo = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    // Unset Option<T> fields must be skipped on the wire.
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert!(v.get("exit_code").is_none(), "None exit_code must be skipped");
}

#[test]
fn terminal_info_exited_roundtrips() {
    let info = TerminalInfo {
        id: "term-2".to_string(),
        title: "Done".to_string(),
        pid: None,
        cols: 80,
        rows: 24,
        working_dir: "/tmp".to_string(),
        is_alive: false,
        exit_code: Some(0),
        created_at: 1_713_270_000_000,
        total_bytes_produced: 0,
        page_id: "default".to_string(),
    };
    let json1 = serde_json::to_string(&info).unwrap();
    let back: TerminalInfo = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert!(v.get("pid").is_none(), "None pid must be skipped");
    assert_eq!(v["exit_code"], 0);
}

#[test]
fn terminal_info_page_id_default_hydrates() {
    // Older wire forms without `page_id` must hydrate to "default".
    let json = r#"{
        "id": "term-old",
        "title": "Legacy",
        "cols": 80,
        "rows": 24,
        "working_dir": "/",
        "is_alive": true,
        "created_at": 0,
        "total_bytes_produced": 0
    }"#;
    let info: TerminalInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.page_id, "default");
    assert_eq!(info.pid, None);
    assert_eq!(info.exit_code, None);
}

#[test]
fn terminal_output_event_roundtrips() {
    let ev = TerminalOutputEvent {
        terminal_id: "term-1".to_string(),
        data: "SGVsbG8sIHdvcmxkIQ==".to_string(),
    };
    let json1 = serde_json::to_string(&ev).unwrap();
    let back: TerminalOutputEvent = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["terminal_id"], "term-1");
    assert_eq!(v["data"], "SGVsbG8sIHdvcmxkIQ==");
}

#[test]
fn terminal_exit_event_with_code_roundtrips() {
    let ev = TerminalExitEvent {
        terminal_id: "term-1".to_string(),
        exit_code: Some(137),
    };
    let json1 = serde_json::to_string(&ev).unwrap();
    let back: TerminalExitEvent = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json2);
    let v: Value = serde_json::from_str(&json1).unwrap();
    assert_eq!(v["exit_code"], 137);
}

#[test]
fn terminal_exit_event_without_code_skips_field() {
    let ev = TerminalExitEvent {
        terminal_id: "term-lost".to_string(),
        exit_code: None,
    };
    let json = serde_json::to_string(&ev).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("exit_code").is_none(), "None exit_code must be skipped");
    let back: TerminalExitEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}


// ── mcp_config ───────────────────────────────────────────────────────────────

#[test]
fn mcp_transport_lowercase() {
    use qontinui_types::mcp_config::McpTransport;
    assert_eq!(serde_json::to_string(&McpTransport::Stdio).unwrap(), "\"stdio\"");
    assert_eq!(serde_json::to_string(&McpTransport::Http).unwrap(), "\"http\"");
    let back: McpTransport = serde_json::from_str("\"stdio\"").unwrap();
    assert_eq!(back, McpTransport::Stdio);
}

#[test]
fn stdio_config_roundtrips() {
    use qontinui_types::mcp_config::StdioConfig;
    use std::collections::HashMap;
    let mut env = HashMap::new();
    env.insert("API_TOKEN".to_string(), "sk-test".to_string());
    let cfg = StdioConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic/some-server".to_string()],
        cwd: Some("D:/projects/x".to_string()),
        env,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["command"], "npx");
    let back: StdioConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&cfg).unwrap()
    );
}

#[test]
fn stdio_config_minimal_elides_empty() {
    use qontinui_types::mcp_config::StdioConfig;
    use std::collections::HashMap;
    let cfg = StdioConfig {
        command: "python".to_string(),
        args: vec![],
        cwd: None,
        env: HashMap::new(),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("args").is_none(), "empty args elided");
    assert!(v.get("cwd").is_none(), "None cwd elided");
    assert!(v.get("env").is_none(), "empty env elided");
}

#[test]
fn http_config_roundtrips() {
    use qontinui_types::mcp_config::HttpConfig;
    use std::collections::HashMap;
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer xyz".to_string());
    let cfg = HttpConfig {
        url: "http://localhost:8080/mcp".to_string(),
        headers,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: HttpConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&cfg).unwrap()
    );
}

#[test]
fn mcp_server_config_stdio_roundtrips() {
    use qontinui_types::mcp_config::{McpServerConfig, McpTransport, StdioConfig};
    use std::collections::HashMap;
    let cfg = McpServerConfig {
        id: "srv-1".to_string(),
        name: "My Server".to_string(),
        description: Some("Local stdio MCP server".to_string()),
        transport: McpTransport::Stdio,
        stdio_config: Some(StdioConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            cwd: None,
            env: HashMap::new(),
        }),
        http_config: None,
        enabled: true,
        auto_start: false,
        timeout_seconds: 30,
        cached_tools: None,
        tools_cached_at: None,
        created_at: "2026-04-16T18:00:00Z".to_string(),
        updated_at: "2026-04-16T18:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let v1: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v1["transport"], "stdio");
    assert!(v1.get("http_config").is_none(), "None http_config elided");
    let back: McpServerConfig = serde_json::from_str(&json).unwrap();
    let v2 = serde_json::to_value(&back).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn mcp_tool_input_schema_renames_type() {
    use qontinui_types::mcp_config::McpToolInputSchema;
    let s = McpToolInputSchema {
        schema_type: "object".to_string(),
        description: Some("Args".to_string()),
        properties: Some(serde_json::json!({"x": {"type": "string"}})),
        required: Some(vec!["x".to_string()]),
    };
    let json = serde_json::to_string(&s).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    // `schema_type` serializes as `"type"` per `#[serde(rename = "type")]`.
    assert_eq!(v["type"], "object");
    assert!(v.get("schema_type").is_none());
    let back: McpToolInputSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&s).unwrap()
    );
}

#[test]
fn mcp_tool_info_renames_input_schema() {
    use qontinui_types::mcp_config::{McpToolInfo, McpToolInputSchema};
    let info = McpToolInfo {
        name: "read_file".to_string(),
        description: Some("Read a file".to_string()),
        input_schema: McpToolInputSchema {
            schema_type: "object".to_string(),
            description: None,
            properties: None,
            required: None,
        },
    };
    let json = serde_json::to_string(&info).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    // `input_schema` serializes as camelCase `"inputSchema"` per MCP spec.
    assert!(
        v.get("inputSchema").is_some(),
        "inputSchema must be camelCase"
    );
    assert!(v.get("input_schema").is_none());
}

#[test]
fn mcp_server_status_roundtrips() {
    use qontinui_types::mcp_config::{McpServerStatus, McpToolInfo, McpToolInputSchema};
    let status = McpServerStatus {
        server_id: "srv-1".to_string(),
        connected: true,
        error: None,
        tools: Some(vec![McpToolInfo {
            name: "ping".to_string(),
            description: None,
            input_schema: McpToolInputSchema {
                schema_type: "object".to_string(),
                description: None,
                properties: None,
                required: None,
            },
        }]),
        last_connect_attempt: Some("2026-04-16T18:00:00Z".to_string()),
        last_connected: Some("2026-04-16T18:00:00Z".to_string()),
    };
    let json = serde_json::to_string(&status).unwrap();
    let back: McpServerStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&status).unwrap()
    );
}

#[test]
fn mcp_tool_call_result_roundtrips() {
    use qontinui_types::mcp_config::McpToolCallResult;
    let ok = McpToolCallResult {
        success: true,
        content: Some(serde_json::json!({"result": 42})),
        error: None,
        response_type: "json".to_string(),
        duration_ms: 125,
    };
    let json = serde_json::to_string(&ok).unwrap();
    let back: McpToolCallResult = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&ok).unwrap()
    );
}

#[test]
fn mcp_call_record_roundtrips() {
    use qontinui_types::mcp_config::McpCallRecord;
    let record = McpCallRecord {
        id: "call-1".to_string(),
        task_run_id: "tr-1".to_string(),
        step_id: "step-a".to_string(),
        step_name: Some("Fetch data".to_string()),
        server_id: "srv-1".to_string(),
        server_name: Some("My Server".to_string()),
        tool_name: "query".to_string(),
        arguments: Some("{\"q\":\"x\"}".to_string()),
        resolved_arguments: Some("{\"q\":\"x\"}".to_string()),
        response: Some("{\"ok\":true}".to_string()),
        response_type: "json".to_string(),
        duration_ms: 200,
        extractions: None,
        assertions: None,
        success: true,
        error_message: None,
        created_at: "2026-04-16T18:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: McpCallRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&record).unwrap()
    );
}

// ── ai_workflows ───────────────────────────────────────────────────────────

#[test]
fn execution_step_roundtrips() {
    use qontinui_types::ai_workflows::ExecutionStep;
    let step = ExecutionStep {
        id: "step-1".to_string(),
        step_type: "playwright".to_string(),
        name: "Run login test".to_string(),
        take_screenshot: true,
        screenshot_delay: Some(1.5),
        playwright_script_id: Some("script-abc".to_string()),
        playwright_script_content: Some("page.goto('/')".to_string()),
        playwright_target_url: Some("http://localhost:3000".to_string()),
        prompt_id: None,
        prompt_content: None,
        action_type: None,
        target_image_id: None,
        target_image_name: None,
        screenshot_monitor: Some(serde_json::json!(1)),
    };
    let json = serde_json::to_string(&step).unwrap();
    let back: ExecutionStep = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&step).unwrap()
    );
}

#[test]
fn ai_workflow_roundtrips() {
    use qontinui_types::ai_workflows::{AiWorkflow, ExecutionStep};
    let wf = AiWorkflow {
        id: "wf-1".to_string(),
        name: "Smoke test".to_string(),
        description: "End-to-end smoke".to_string(),
        steps: vec![ExecutionStep {
            id: "s1".to_string(),
            step_type: "prompt".to_string(),
            name: "Ask AI".to_string(),
            prompt_content: Some("fix it".to_string()),
            ..Default::default()
        }],
        goal: "Verify login flow".to_string(),
        max_iterations: Some(5),
        capture_input_validation: false,
        category: "Testing".to_string(),
        tags: vec!["smoke".to_string(), "login".to_string()],
        context_ids: vec!["ctx-1".to_string()],
        disabled_context_ids: vec![],
        auto_include_contexts: true,
        created_at: "2026-04-16T12:00:00Z".to_string(),
        modified_at: "2026-04-16T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&wf).unwrap();
    let back: AiWorkflow = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&wf).unwrap()
    );
}

#[test]
fn ai_workflow_defaults_roundtrip() {
    // Minimal JSON — only required fields — should deserialize with defaults.
    let minimal = r#"{
        "id": "wf-min",
        "name": "Minimal",
        "created_at": "2026-04-16T00:00:00Z",
        "modified_at": "2026-04-16T00:00:00Z"
    }"#;
    let wf: qontinui_types::ai_workflows::AiWorkflow =
        serde_json::from_str(minimal).unwrap();
    assert_eq!(wf.id, "wf-min");
    assert!(wf.steps.is_empty());
    assert!(wf.auto_include_contexts); // default true
    assert_eq!(wf.max_iterations, None);
    // Re-serialize and round-trip.
    let json2 = serde_json::to_string(&wf).unwrap();
    let back: qontinui_types::ai_workflows::AiWorkflow =
        serde_json::from_str(&json2).unwrap();
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&wf).unwrap()
    );
}

// ── ui_bridge ───────────────────────────────────────────────────────────────

#[test]
fn element_rect_roundtrips() {
    use qontinui_types::ui_bridge::ElementRect;
    let rect = ElementRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
        left: 10.0,
    };
    let json = serde_json::to_string(&rect).unwrap();
    let back: ElementRect = serde_json::from_str(&json).unwrap();
    assert_eq!(rect, back);
}

#[test]
fn element_state_roundtrips() {
    use qontinui_types::ui_bridge::{ElementRect, ElementState};
    let state = ElementState {
        visible: true,
        enabled: true,
        focused: false,
        rect: ElementRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 40.0,
            top: 0.0,
            right: 200.0,
            bottom: 40.0,
            left: 0.0,
        },
        value: Some("hello".to_string()),
        checked: None,
        selected_options: None,
        text_content: Some("Click me".to_string()),
    };
    let json = serde_json::to_string(&state).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("checked").is_none(), "None checked must be skipped");
    let back: ElementState = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn element_identifier_roundtrips() {
    use qontinui_types::ui_bridge::ElementIdentifier;
    let id = ElementIdentifier {
        ui_id: Some("btn-submit".to_string()),
        test_id: Some("submit-button".to_string()),
        awas_id: None,
        html_id: Some("submitBtn".to_string()),
        xpath: "/html/body/form/button[1]".to_string(),
        selector: "form > button.submit".to_string(),
    };
    let json = serde_json::to_string(&id).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["uiId"], "btn-submit", "camelCase rename");
    assert!(v.get("awasId").is_none(), "None awasId must be skipped");
    let back: ElementIdentifier = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn ui_bridge_element_roundtrips() {
    use qontinui_types::ui_bridge::*;
    let elem = UIBridgeElement {
        id: "elem-1".to_string(),
        element_type: "button".to_string(),
        label: Some("Submit".to_string()),
        actions: vec!["click".to_string()],
        custom_actions: None,
        identifier: ElementIdentifier {
            ui_id: None,
            test_id: None,
            awas_id: None,
            html_id: None,
            xpath: "/button[1]".to_string(),
            selector: "button".to_string(),
        },
        state: ElementState {
            visible: true,
            enabled: true,
            focused: false,
            rect: ElementRect {
                x: 0.0, y: 0.0, width: 80.0, height: 30.0,
                top: 0.0, right: 80.0, bottom: 30.0, left: 0.0,
            },
            value: None,
            checked: None,
            selected_options: None,
            text_content: None,
        },
        registered_at: 1713200000000,
        mounted: true,
    };
    let json = serde_json::to_string(&elem).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["type"], "button", "element_type renames to type");
    let back: UIBridgeElement = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn discovery_request_default_roundtrips() {
    use qontinui_types::ui_bridge::DiscoveryRequest;
    let req = DiscoveryRequest::default();
    let json = serde_json::to_string(&req).unwrap();
    assert_eq!(json, "{}", "all-None DiscoveryRequest serializes to empty object");
    let back: DiscoveryRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn action_response_roundtrips() {
    use qontinui_types::ui_bridge::ActionResponse;
    let resp = ActionResponse {
        success: true,
        element_state: None,
        result: Some(serde_json::json!({"clicked": true})),
        error: None,
        stack: None,
        duration_ms: 42,
        timestamp: 1713200000000,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("elementState").is_none(), "None element_state skipped");
    assert!(v.get("error").is_none(), "None error skipped");
    let back: ActionResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn wait_options_roundtrips() {
    use qontinui_types::ui_bridge::WaitOptions;
    let opts = WaitOptions {
        visible: Some(true),
        enabled: None,
        focused: None,
        timeout: Some(5000),
        interval: None,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("enabled").is_none(), "None enabled skipped");
    let back: WaitOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, back);
}

#[test]
fn ui_bridge_snapshot_roundtrips() {
    use qontinui_types::ui_bridge::UIBridgeSnapshot;
    let snap = UIBridgeSnapshot {
        timestamp: 1713200000000,
        elements: vec![],
        components: vec![],
        workflows: vec![],
    };
    let json = serde_json::to_string(&snap).unwrap();
    // Empty vecs should be skipped
    assert_eq!(json, r#"{"timestamp":1713200000000}"#);
    let back: UIBridgeSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&back).unwrap());
}

#[test]
fn workflow_info_roundtrips() {
    use qontinui_types::ui_bridge::WorkflowInfo;
    let wf = WorkflowInfo {
        id: "wf-1".to_string(),
        name: "Login Flow".to_string(),
        description: Some("Tests the login page".to_string()),
        step_count: 5,
    };
    let json = serde_json::to_string(&wf).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["stepCount"], 5, "camelCase rename");
    let back: WorkflowInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(wf, back);
}
