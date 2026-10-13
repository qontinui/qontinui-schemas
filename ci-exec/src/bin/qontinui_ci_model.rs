//! `qontinui-ci-model` — the model shim a `model = true` CI step runs as
//! `$QONTINUI_CI_MODEL_CMD`. It mirrors `claude -p` and talks to the
//! dispatch's loopback model broker; see [`qontinui_ci_exec::model::shim`].
//!
//! Exit codes: 0 ok; 1 a broker or provider error, or model access refused
//! for this dispatch; 2 a usage error.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = qontinui_ci_exec::model::shim::run(
        &args,
        &|key| std::env::var(key).ok(),
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
