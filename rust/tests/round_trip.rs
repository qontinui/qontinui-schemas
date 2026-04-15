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
