//! Arming tests for the `[model]` declaration, written against base-era API
//! only: they enter where every consumer enters (`manifest::parse_and_validate`,
//! which `qontinui-ci run` and the CI node call on a repo's `.qontinui/ci.toml`)
//! and name nothing this change added, so on the pre-`[model]` executor they are
//! RED (a `[model]` table was an unknown key; a `*_MODEL_CMD` env key was merely
//! "not allowlisted") rather than failing to compile.

use qontinui_ci_exec::manifest::parse_and_validate;

#[test]
fn arming_a_declared_model_table_parses() {
    let text = "version = 1\n\
        [model]\n\
        models = [\"claude-sonnet\"]\n\
        max_calls = 10\n\
        max_tokens = 1000\n\
        max_wall_secs = 60\n\
        [[steps]]\n\
        name = \"b2\"\n\
        command = [\"true\"]\n\
        model = true\n";
    parse_and_validate(text)
        .unwrap_or_else(|e| panic!("a declared [model] table was refused: {e}"));
}

#[test]
fn arming_a_model_command_env_points_at_the_model_declaration() {
    let text = "version = 1\n\
        [[steps]]\n\
        name = \"b2\"\n\
        command = [\"true\"]\n\
        env = { DOCS_EVAL_MODEL_CMD = \"claude -p\" }\n";
    let err = parse_and_validate(text).expect_err("a manifest must not choose the model program");
    assert!(
        err.contains("[model]"),
        "the refusal must point at [model]: {err}"
    );
}
