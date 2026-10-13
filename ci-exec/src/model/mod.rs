//! Model access for CI steps (plan
//! `2026-10-13-ci-node-runs-model-dependent-eval-steps`).
//!
//! A step never holds a model credential. A repo declares what its steps may
//! spend in `[model]` ([`declaration`]); the steps that need a model set
//! `model = true`; and the executor hands only those steps a dispatch-bound
//! capability for a per-dispatch loopback broker, which calls the model
//! through an invoker the HOST provides.
//!
//! | Module | What it owns |
//! |---|---|
//! | [`declaration`] | the `[model]` table, the closed [`declaration::MODEL_REGISTRY`] and the budget ceilings |

pub mod declaration;
