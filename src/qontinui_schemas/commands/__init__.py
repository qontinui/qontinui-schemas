"""Command schemas for the web -> runner WebSocket bridge.

These schemas define the request/reply envelopes for HTTP endpoints in
qontinui-web that dispatch work to a connected qontinui-runner over the
existing WS relay (``runner:commands:{runner_id}`` /
``runner:responses:{runner_id}`` Redis pub/sub).

Each submodule defines pydantic request/response/error models for a
specific surface area:

- ``state_machine``: state-machine discovery + UI bridge runtime calls.
- ``recording_pipeline``: recording-pipeline async-with-progress runs.

The pattern is hand-authored pydantic (NOT Rust-codegen), because the
runner-side Python dispatcher imports these directly when it handles
inbound commands. See the plan at
``plans/2026-05-17-web-runner-ws-bridge-plan-b.md`` for the disposition
rationale.
"""
