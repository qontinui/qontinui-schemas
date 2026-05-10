# Changelog

## [0.2.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.1.3...rust-v0.2.0) (2026-05-10)


### Features

* **codegen:** cross-file TypeScript generation via schema preprocessing ([db27f3f](https://github.com/qontinui/qontinui-schemas/commit/db27f3f08f562930f1ab1b8172c1714382b8b70a))
* **events:** concretize loose Tauri payloads to typed structs ([101a557](https://github.com/qontinui/qontinui-schemas/commit/101a557896ee3e31e3bb40df823f614cf9584bea))
* **events:** typed UI Bridge request/response envelopes ([2aca16e](https://github.com/qontinui/qontinui-schemas/commit/2aca16e190b9359dc4650dc23c4dc2ae19382742))
* **rust:** UIBridgeSnapshot carries modalStack/toasts/undoRedo + route ([5d1b1ce](https://github.com/qontinui/qontinui-schemas/commit/5d1b1ce4e99fd797ab5720566c5d46b0f0326faa))
* **schemas:** add canonical Runner type ([ed315d3](https://github.com/qontinui/qontinui-schemas/commit/ed315d3ac071aa6c2901a14d679728999de9f71a))
* **schemas:** add FullRunnerStep typed union covering all 16 runner step variants ([d2a083c](https://github.com/qontinui/qontinui-schemas/commit/d2a083cdfff6b6376eeeaeccc5ad543332048482))
* **schemas:** add HTN config fields to UnifiedWorkflow ([c5b35be](https://github.com/qontinui/qontinui-schemas/commit/c5b35be5978cecbb8bb91bd7a7327dbbe4b48af1))
* **schemas:** add scheduler reliability fields + RemoteAgent task type ([0079343](https://github.com/qontinui/qontinui-schemas/commit/00793432877c3b68cbf2728f7a469a58e348ed0d))
* **schemas:** auto_commit_subagents field on UnifiedWorkflow ([ea5d4a5](https://github.com/qontinui/qontinui-schemas/commit/ea5d4a5538c55923483dfffdefdcbd5b96c6663a))
* **schemas:** Rust source-of-truth for task-run, execution, state-machine ([db33145](https://github.com/qontinui/qontinui-schemas/commit/db331459354633661992f8d26ffe04d915fbaaf1))
* surface element bbox + visibility on UIBridgeElement ([5094dcd](https://github.com/qontinui/qontinui-schemas/commit/5094dcdc8281b5be377a2dd22444e75c449b9d45))
* **types:** add discovery, orchestration_config, verification modules ([717015a](https://github.com/qontinui/qontinui-schemas/commit/717015a36d2e7198fbe69e744befa3da2e3fb1c9))
* **types:** add findings, process_management, ticket_system modules; expand execution ([59d97c3](https://github.com/qontinui/qontinui-schemas/commit/59d97c3a450064b715c3339c1a83467825f8b992))
* **types:** add mcp_config and terminal modules ([b28a541](https://github.com/qontinui/qontinui-schemas/commit/b28a541e7bc9b9952be3aec42f937559ca328731))
* **types:** add ui_bridge, ai_workflows modules; expand rag ([93e7562](https://github.com/qontinui/qontinui-schemas/commit/93e7562a45c4da957f5f10ae3998d244ec6b0fc6))
* **types:** extract AppEvent + FlowEvent to app_events module ([10faa25](https://github.com/qontinui/qontinui-schemas/commit/10faa25e52c788828244d9e8404cf9305fd73616))
* **types:** extract WorkerOutput family — schema_export now mirror-free ([dc49658](https://github.com/qontinui/qontinui-schemas/commit/dc49658bd27e9183256b8795939ed6723c161aa9))
* **types:** host the SDK_FEATURES inventory as a shared module ([d5dd04c](https://github.com/qontinui/qontinui-schemas/commit/d5dd04c38f56740bcac6d6ea1f5813cc503e56b5))
* **types:** port accessibility + config from Pydantic to Rust ([7ff7ade](https://github.com/qontinui/qontinui-schemas/commit/7ff7ade905eb1f09aa79a66c824f8002a2e64240))
* **types:** port geometry + tree_events from Pydantic to Rust ([ab1eab5](https://github.com/qontinui/qontinui-schemas/commit/ab1eab51df5527502bb94a985d5117db1d735fbf))
* **types:** port targets + rag to Rust, delete Pydantic TS pipeline ([b68131c](https://github.com/qontinui/qontinui-schemas/commit/b68131c3b4b81cd25d1088fcb74481a4128583de))
* **types:** rename RunnerKind serde tag from "kind" to "type" ([45d0ced](https://github.com/qontinui/qontinui-schemas/commit/45d0ced287d96ae1c9c92f774c4f38032ffefb7f))
* **types:** runner-supervisor modularity types (wire::placement, RunnerKind, webview2_data_dir, rust-runner-client) ([#6](https://github.com/qontinui/qontinui-schemas/issues/6)) ([5788fcf](https://github.com/qontinui/qontinui-schemas/commit/5788fcfd424508dd7b2c1ca05f257d12858a835a))
* **types:** universal camelCase wire normalization with backward-compat aliases ([854d4b7](https://github.com/qontinui/qontinui-schemas/commit/854d4b74064d58fa71044256a261f1951d11f038))


### Bug Fixes

* **ci:** pin datamodel-codegen 0.57.0 + regen artifacts (post-PR-[#24](https://github.com/qontinui/qontinui-schemas/issues/24)) ([#25](https://github.com/qontinui/qontinui-schemas/issues/25)) ([b961022](https://github.com/qontinui/qontinui-schemas/commit/b961022c52d763e74ab1aedd0a2aff30e12b541c))
* **ci:** repair schema-drift sibling-checkout layout for standalone repo ([#24](https://github.com/qontinui/qontinui-schemas/issues/24)) ([98888a3](https://github.com/qontinui/qontinui-schemas/commit/98888a310cb3b0a733750da680bfc8e424f7b92b))
* **schemas:** align TaskRunFinding/Session wire name with backend ([6246c7a](https://github.com/qontinui/qontinui-schemas/commit/6246c7a2711915ad5f48e837f3c68d10624c0595))
* **schemas:** make TransitionInfo.to_state required-nullable ([3c30b44](https://github.com/qontinui/qontinui-schemas/commit/3c30b4444dbc0e0c0bcc83cd048d013a8c0fc697))
* **types:** add deny_unknown_fields to eliminate TS index signatures ([380e4ab](https://github.com/qontinui/qontinui-schemas/commit/380e4aba2c5aca381d920929fba23e775d17ad96))
* **types:** add Manual variant to FindingActionType + rebuild stale dist ([33640c6](https://github.com/qontinui/qontinui-schemas/commit/33640c6acac4a37b31f045eff8021b685617fd99))
