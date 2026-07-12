# Changelog

## [1.0.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.9.0...rust-v1.0.0) (2026-07-12)


### ⚠ BREAKING CHANGES

* extend UIBridgeElement with accessibility fields; replace IrState.required_elements with assertions

### Features

* add federation report wire types for memory federation dashboard ([9b2ec2d](https://github.com/qontinui/qontinui-schemas/commit/9b2ec2db8d56eef6209b1a3ce3215d6f5a5fcbc8))
* add git_ops wire types for GitOp federation ([6e3e175](https://github.com/qontinui/qontinui-schemas/commit/6e3e175eb52fb966b0034601250bec27cd5aa925))
* add HelperTask/HelperAnswer schema types for helper task queue ([0ed6889](https://github.com/qontinui/qontinui-schemas/commit/0ed68896d425bc4eae5c6a13cca64b14be5daf1c))
* add memory shared types for runner-mediated federation ([#58](https://github.com/qontinui/qontinui-schemas/issues/58)) ([cc7b5b3](https://github.com/qontinui/qontinui-schemas/commit/cc7b5b36cb553ec769f0a46bd6782159bd48a99a))
* add multi-tenant app registry types (spec-multi-app Stream A) ([bc9d56d](https://github.com/qontinui/qontinui-schemas/commit/bc9d56d05fd5d940fc3584bf5132d8a084caff99))
* **canonical_hash:** promote canonical_hash&lt;T&gt; to qontinui-types ([f4bb0cf](https://github.com/qontinui/qontinui-schemas/commit/f4bb0cf0c53006242e4eff8939e20a98db760f69))
* **codegen:** cross-file TypeScript generation via schema preprocessing ([db27f3f](https://github.com/qontinui/qontinui-schemas/commit/db27f3f08f562930f1ab1b8172c1714382b8b70a))
* **completeness:** website→mobile vertical-slice rubric walk + anti-drift gate ([#81](https://github.com/qontinui/qontinui-schemas/issues/81)) ([1b873bd](https://github.com/qontinui/qontinui-schemas/commit/1b873bd109f34693cf83f71cb4a1d51ca1e0387c))
* **dev-states:** DevState + DevSignature registries for dev-event action snapshots ([#77](https://github.com/qontinui/qontinui-schemas/issues/77)) ([f6f9f24](https://github.com/qontinui/qontinui-schemas/commit/f6f9f24c8183e8684ce4a3c47cc7242ff5321f66))
* **diagnostics:** generated ui_bridge_diagnostics Rust mirror ([#55](https://github.com/qontinui/qontinui-schemas/issues/55)) ([9606e93](https://github.com/qontinui/qontinui-schemas/commit/9606e930c3e232bd9bfafe5310431bd4012c291b))
* **events:** canonical UI Bridge HTTP envelopes (error + health) ([#68](https://github.com/qontinui/qontinui-schemas/issues/68)) ([235295c](https://github.com/qontinui/qontinui-schemas/commit/235295caf9780f09de71d1be294c491ebfe95609))
* **events:** concretize loose Tauri payloads to typed structs ([101a557](https://github.com/qontinui/qontinui-schemas/commit/101a557896ee3e31e3bb40df823f614cf9584bea))
* **events:** typed UI Bridge request/response envelopes ([2aca16e](https://github.com/qontinui/qontinui-schemas/commit/2aca16e190b9359dc4650dc23c4dc2ae19382742))
* extend UIBridgeElement with accessibility fields; replace IrState.required_elements with assertions ([861644a](https://github.com/qontinui/qontinui-schemas/commit/861644a843a3eac657d5a4478a8fa9b541b9c371))
* **fleet-fresh:** P1a fields — add auto-fresh config to App schema ([aa38216](https://github.com/qontinui/qontinui-schemas/commit/aa382168e2d27be9a50b64f7b3ac153339f6a97e))
* **fleet-fresh:** RegisterAppRequest::new() constructor + fix test literal ([32edc97](https://github.com/qontinui/qontinui-schemas/commit/32edc97030a42d0b411c963615aca594a0b04345))
* **fleet-fresh:** validate_update_strategy + InvalidUpdateStrategy variant ([9482f81](https://github.com/qontinui/qontinui-schemas/commit/9482f8174a769f2cc31e15c27ee0d9ffd39dc80c))
* **functional-spec:** freeze functional-spec-contract v0 (spec + rubric + profile) ([#79](https://github.com/qontinui/qontinui-schemas/issues/79)) ([6b86061](https://github.com/qontinui/qontinui-schemas/commit/6b860617e543a35a4ea06613c4251e0ceac48bd0))
* **ir:** add IrApiCheck type for Spec CI API-contract assertions ([#66](https://github.com/qontinui/qontinui-schemas/issues/66)) ([598a6ae](https://github.com/qontinui/qontinui-schemas/commit/598a6ae86ed864a8fe71766cd06d393a64e034e8))
* **ir:** add IrProvenance.status + ProposalStatus enum for flywheel ([ee1e75f](https://github.com/qontinui/qontinui-schemas/commit/ee1e75fd7dd42d287dbb03027b94208d37a6882b))
* **orchestration:** conductor LoopPhase variants (Approach-D Phase 3) ([#83](https://github.com/qontinui/qontinui-schemas/issues/83)) ([a68b1c3](https://github.com/qontinui/qontinui-schemas/commit/a68b1c369eee71093949ec93344b19acd0e2edf0))
* **process-state:** add ExternallyOwned variant + regenerate bindings ([#65](https://github.com/qontinui/qontinui-schemas/issues/65)) ([89058f6](https://github.com/qontinui/qontinui-schemas/commit/89058f673fb0adfda8716c0b4ed9198805a11f88))
* **regen:** shared endpoint_for derivation + connect-runner state assertions ([#82](https://github.com/qontinui/qontinui-schemas/issues/82)) ([f8c91d3](https://github.com/qontinui/qontinui-schemas/commit/f8c91d3f0889afbe43f60e6e0c2ccde7c1097ddc))
* **rust-vision-core:** cache + Pipeline::hash + multi_run (Phase 3.1) ([#40](https://github.com/qontinui/qontinui-schemas/issues/40)) ([ec4b32e](https://github.com/qontinui/qontinui-schemas/commit/ec4b32e65a0f9b0658e248a9796b8068448167d1))
* **rust:** UIBridgeSnapshot carries modalStack/toasts/undoRedo + route ([5d1b1ce](https://github.com/qontinui/qontinui-schemas/commit/5d1b1ce4e99fd797ab5720566c5d46b0f0326faa))
* **schemas:** add canonical Runner type ([ed315d3](https://github.com/qontinui/qontinui-schemas/commit/ed315d3ac071aa6c2901a14d679728999de9f71a))
* **schemas:** Add classification thresholds for spec evaluation ([e72c478](https://github.com/qontinui/qontinui-schemas/commit/e72c47894ab2e4ca9d4a8c4e7fb2fc7291e8a2b0))
* **schemas:** add FullRunnerStep typed union covering all 16 runner step variants ([d2a083c](https://github.com/qontinui/qontinui-schemas/commit/d2a083cdfff6b6376eeeaeccc5ad543332048482))
* **schemas:** add HTN config fields to UnifiedWorkflow ([c5b35be](https://github.com/qontinui/qontinui-schemas/commit/c5b35be5978cecbb8bb91bd7a7327dbbe4b48af1))
* **schemas:** add scheduler reliability fields + RemoteAgent task type ([0079343](https://github.com/qontinui/qontinui-schemas/commit/00793432877c3b68cbf2728f7a469a58e348ed0d))
* **schemas:** auto_commit_subagents field on UnifiedWorkflow ([ea5d4a5](https://github.com/qontinui/qontinui-schemas/commit/ea5d4a5538c55923483dfffdefdcbd5b96c6663a))
* **schemas:** Classification thresholds for spec evaluation ([8973f27](https://github.com/qontinui/qontinui-schemas/commit/8973f27edf6a93a4897a1e650f7ff9533fe19e3d))
* **schemas:** derive Deserialize on AppListResponse for supervisor cron ([5b34033](https://github.com/qontinui/qontinui-schemas/commit/5b34033a402e7a05a4132ecf9c6f0b7b0db80313))
* **schemas:** forward-compat device-namespace aliases for unified-devices rollout ([#54](https://github.com/qontinui/qontinui-schemas/issues/54)) ([07726bf](https://github.com/qontinui/qontinui-schemas/commit/07726bf625a02b8542eec4c9bcf451944d84c5d6))
* **schemas:** Rust source-of-truth for task-run, execution, state-machine ([db33145](https://github.com/qontinui/qontinui-schemas/commit/db331459354633661992f8d26ffe04d915fbaaf1))
* **spec-api-events:** add SpecApiEvent enum for Plan 06 broadcasts ([#43](https://github.com/qontinui/qontinui-schemas/issues/43)) ([df1baeb](https://github.com/qontinui/qontinui-schemas/commit/df1baebe1289afec4c9ab7ffc6cf6f8ae7ba9332))
* **spec-check:** add snapshot_sha256 field to SpecCheckResult ([#60](https://github.com/qontinui/qontinui-schemas/issues/60)) ([95af1aa](https://github.com/qontinui/qontinui-schemas/commit/95af1aa8b3d0090fadb46f2f160dc81d8e738644))
* **spec-check:** add SpecCheckSummary.recommendation_reason ([#45](https://github.com/qontinui/qontinui-schemas/issues/45)) ([0b70bd4](https://github.com/qontinui/qontinui-schemas/commit/0b70bd47242e40ceefc96dd13f76aa4bb9420a6c))
* surface element bbox + visibility on UIBridgeElement ([5094dcd](https://github.com/qontinui/qontinui-schemas/commit/5094dcdc8281b5be377a2dd22444e75c449b9d45))
* **types:** add discovery, orchestration_config, verification modules ([717015a](https://github.com/qontinui/qontinui-schemas/commit/717015a36d2e7198fbe69e744befa3da2e3fb1c9))
* **types:** add findings, process_management, ticket_system modules; expand execution ([59d97c3](https://github.com/qontinui/qontinui-schemas/commit/59d97c3a450064b715c3339c1a83467825f8b992))
* **types:** add ir, spec_check, text_norm modules (Plan 01 foundation) ([c7790cc](https://github.com/qontinui/qontinui-schemas/commit/c7790cc58007b53796374b6bed288711027f592f))
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
* **ui_bridge:** opaque effect_verification on ActionResponse (D3 Phase 3a) ([#70](https://github.com/qontinui/qontinui-schemas/issues/70)) ([11936fb](https://github.com/qontinui/qontinui-schemas/commit/11936fb9b310aa45f6513d14bc5c256f97e08e48))


### Bug Fixes

* **apps:** serialize AppError fields as camelCase ([#61](https://github.com/qontinui/qontinui-schemas/issues/61)) ([6c0cf31](https://github.com/qontinui/qontinui-schemas/commit/6c0cf311ef2ac1dc2c05081f0bc5a07fce18429a))
* **ci:** pin datamodel-codegen 0.57.0 + regen artifacts (post-PR-[#24](https://github.com/qontinui/qontinui-schemas/issues/24)) ([#25](https://github.com/qontinui/qontinui-schemas/issues/25)) ([b961022](https://github.com/qontinui/qontinui-schemas/commit/b961022c52d763e74ab1aedd0a2aff30e12b541c))
* **ci:** repair schema-drift sibling-checkout layout for standalone repo ([#24](https://github.com/qontinui/qontinui-schemas/issues/24)) ([98888a3](https://github.com/qontinui/qontinui-schemas/commit/98888a310cb3b0a733750da680bfc8e424f7b92b))
* **clippy:** unbreak rust-ci clippy on main ([569f371](https://github.com/qontinui/qontinui-schemas/commit/569f3717db1672dffdfd8ec1fdee84103a69c3c7))
* **codegen:** set Confidence enum title to SpecCheckConfidence ([#44](https://github.com/qontinui/qontinui-schemas/issues/44)) ([79238b2](https://github.com/qontinui/qontinui-schemas/commit/79238b2433e3771e9576c18be3aadcafd2e76065))
* **regen:** connect-runner assertion criteria to camelCase (textContains) ([1e35258](https://github.com/qontinui/qontinui-schemas/commit/1e3525801091c2c0526bb56f909d04a2a247821d))
* **schemas:** align TaskRunFinding/Session wire name with backend ([6246c7a](https://github.com/qontinui/qontinui-schemas/commit/6246c7a2711915ad5f48e837f3c68d10624c0595))
* **schemas:** make TransitionInfo.to_state required-nullable ([3c30b44](https://github.com/qontinui/qontinui-schemas/commit/3c30b4444dbc0e0c0bcc83cd048d013a8c0fc697))
* **types:** add deny_unknown_fields to eliminate TS index signatures ([380e4ab](https://github.com/qontinui/qontinui-schemas/commit/380e4aba2c5aca381d920929fba23e775d17ad96))
* **types:** add Manual variant to FindingActionType + rebuild stale dist ([33640c6](https://github.com/qontinui/qontinui-schemas/commit/33640c6acac4a37b31f045eff8021b685617fd99))

## [0.9.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.8.0...rust-v0.9.0) (2026-07-06)


### Features

* **fleet-fresh:** P1a fields — add auto-fresh config to App schema ([aa38216](https://github.com/qontinui/qontinui-schemas/commit/aa382168e2d27be9a50b64f7b3ac153339f6a97e))
* **fleet-fresh:** RegisterAppRequest::new() constructor + fix test literal ([32edc97](https://github.com/qontinui/qontinui-schemas/commit/32edc97030a42d0b411c963615aca594a0b04345))
* **fleet-fresh:** validate_update_strategy + InvalidUpdateStrategy variant ([9482f81](https://github.com/qontinui/qontinui-schemas/commit/9482f8174a769f2cc31e15c27ee0d9ffd39dc80c))

## [0.8.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.7.0...rust-v0.8.0) (2026-07-02)


### Features

* **schemas:** Add classification thresholds for spec evaluation ([e72c478](https://github.com/qontinui/qontinui-schemas/commit/e72c47894ab2e4ca9d4a8c4e7fb2fc7291e8a2b0))
* **schemas:** Classification thresholds for spec evaluation ([8973f27](https://github.com/qontinui/qontinui-schemas/commit/8973f27edf6a93a4897a1e650f7ff9533fe19e3d))


### Bug Fixes

* **clippy:** unbreak rust-ci clippy on main ([569f371](https://github.com/qontinui/qontinui-schemas/commit/569f3717db1672dffdfd8ec1fdee84103a69c3c7))

## [0.7.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.6.0...rust-v0.7.0) (2026-06-29)


### Features

* add HelperTask/HelperAnswer schema types for helper task queue ([0ed6889](https://github.com/qontinui/qontinui-schemas/commit/0ed68896d425bc4eae5c6a13cca64b14be5daf1c))

## [0.6.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.5.0...rust-v0.6.0) (2026-06-15)


### Features

* **completeness:** website→mobile vertical-slice rubric walk + anti-drift gate ([#81](https://github.com/qontinui/qontinui-schemas/issues/81)) ([1b873bd](https://github.com/qontinui/qontinui-schemas/commit/1b873bd109f34693cf83f71cb4a1d51ca1e0387c))
* **functional-spec:** freeze functional-spec-contract v0 (spec + rubric + profile) ([#79](https://github.com/qontinui/qontinui-schemas/issues/79)) ([6b86061](https://github.com/qontinui/qontinui-schemas/commit/6b860617e543a35a4ea06613c4251e0ceac48bd0))
* **orchestration:** conductor LoopPhase variants (Approach-D Phase 3) ([#83](https://github.com/qontinui/qontinui-schemas/issues/83)) ([a68b1c3](https://github.com/qontinui/qontinui-schemas/commit/a68b1c369eee71093949ec93344b19acd0e2edf0))
* **regen:** shared endpoint_for derivation + connect-runner state assertions ([#82](https://github.com/qontinui/qontinui-schemas/issues/82)) ([f8c91d3](https://github.com/qontinui/qontinui-schemas/commit/f8c91d3f0889afbe43f60e6e0c2ccde7c1097ddc))


### Bug Fixes

* **regen:** connect-runner assertion criteria to camelCase (textContains) ([1e35258](https://github.com/qontinui/qontinui-schemas/commit/1e3525801091c2c0526bb56f909d04a2a247821d))

## [0.5.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.4.0...rust-v0.5.0) (2026-06-07)


### Features

* **dev-states:** DevState + DevSignature registries for dev-event action snapshots ([#77](https://github.com/qontinui/qontinui-schemas/issues/77)) ([f6f9f24](https://github.com/qontinui/qontinui-schemas/commit/f6f9f24c8183e8684ce4a3c47cc7242ff5321f66))
* **events:** canonical UI Bridge HTTP envelopes (error + health) ([#68](https://github.com/qontinui/qontinui-schemas/issues/68)) ([235295c](https://github.com/qontinui/qontinui-schemas/commit/235295caf9780f09de71d1be294c491ebfe95609))
* **ir:** add IrApiCheck type for Spec CI API-contract assertions ([#66](https://github.com/qontinui/qontinui-schemas/issues/66)) ([598a6ae](https://github.com/qontinui/qontinui-schemas/commit/598a6ae86ed864a8fe71766cd06d393a64e034e8))
* **ui_bridge:** opaque effect_verification on ActionResponse (D3 Phase 3a) ([#70](https://github.com/qontinui/qontinui-schemas/issues/70)) ([11936fb](https://github.com/qontinui/qontinui-schemas/commit/11936fb9b310aa45f6513d14bc5c256f97e08e48))

## [0.4.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.3.0...rust-v0.4.0) (2026-05-25)


### Features

* add federation report wire types for memory federation dashboard ([9b2ec2d](https://github.com/qontinui/qontinui-schemas/commit/9b2ec2db8d56eef6209b1a3ce3215d6f5a5fcbc8))
* add git_ops wire types for GitOp federation ([6e3e175](https://github.com/qontinui/qontinui-schemas/commit/6e3e175eb52fb966b0034601250bec27cd5aa925))
* **process-state:** add ExternallyOwned variant + regenerate bindings ([#65](https://github.com/qontinui/qontinui-schemas/issues/65)) ([89058f6](https://github.com/qontinui/qontinui-schemas/commit/89058f673fb0adfda8716c0b4ed9198805a11f88))
* **spec-check:** add snapshot_sha256 field to SpecCheckResult ([#60](https://github.com/qontinui/qontinui-schemas/issues/60)) ([95af1aa](https://github.com/qontinui/qontinui-schemas/commit/95af1aa8b3d0090fadb46f2f160dc81d8e738644))


### Bug Fixes

* **apps:** serialize AppError fields as camelCase ([#61](https://github.com/qontinui/qontinui-schemas/issues/61)) ([6c0cf31](https://github.com/qontinui/qontinui-schemas/commit/6c0cf311ef2ac1dc2c05081f0bc5a07fce18429a))

## [0.3.0](https://github.com/qontinui/qontinui-schemas/compare/rust-v0.2.0...rust-v0.3.0) (2026-05-23)


### Features

* add memory shared types for runner-mediated federation ([#58](https://github.com/qontinui/qontinui-schemas/issues/58)) ([cc7b5b3](https://github.com/qontinui/qontinui-schemas/commit/cc7b5b36cb553ec769f0a46bd6782159bd48a99a))
* add multi-tenant app registry types (spec-multi-app Stream A) ([bc9d56d](https://github.com/qontinui/qontinui-schemas/commit/bc9d56d05fd5d940fc3584bf5132d8a084caff99))
* **schemas:** derive Deserialize on AppListResponse for supervisor cron ([5b34033](https://github.com/qontinui/qontinui-schemas/commit/5b34033a402e7a05a4132ecf9c6f0b7b0db80313))
