# Changelog

## [1.0.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.14.0...ts-v1.0.0) (2026-09-04)


### ⚠ BREAKING CHANGES

* extend UIBridgeElement with accessibility fields; replace IrState.required_elements with assertions

### Features

* add acceptance_criteria field to UnifiedWorkflow and AcceptanceCriteria canvas component ([01046f9](https://github.com/qontinui/qontinui-schemas/commit/01046f9bdb42b2e6935be64bfbdd1c44e4036a43))
* add AgentTextUnit canonical types for the agent text corpus ([c61fc22](https://github.com/qontinui/qontinui-schemas/commit/c61fc2220f76fd0b09567c68099772a366fe0442))
* add canvas panel types and update workflow types ([69b5a3d](https://github.com/qontinui/qontinui-schemas/commit/69b5a3d9bf1b33b7562f1ca934ffa46975418f5d))
* add chat types, reflection_mode field, and relax output_data type ([53a9ddb](https://github.com/qontinui/qontinui-schemas/commit/53a9ddb1cb733e3b9b8c6dfc2e17099985041c1e))
* add constraint_overrides field to UnifiedWorkflow schema ([16efafc](https://github.com/qontinui/qontinui-schemas/commit/16efafcf66cf335ad6581b1bd96933df156b7b6d))
* add discovery and template capture schema modules ([371b1d6](https://github.com/qontinui/qontinui-schemas/commit/371b1d67f73a0f6ee956d377fa68d90ddc62d2a9))
* add elementThumbnails field to StateNodeData schema ([14b9e71](https://github.com/qontinui/qontinui-schemas/commit/14b9e71b6fea08f40b74962d658a3aab249bdf7e))
* add flow_control_json and phase_timeouts_json to UnifiedWorkflow schema ([2cef5b6](https://github.com/qontinui/qontinui-schemas/commit/2cef5b60881e42f698c288d56a84d56ebff5db60))
* add hooks and workflow generation template types ([b6da956](https://github.com/qontinui/qontinui-schemas/commit/b6da956fb980036e997400726bfdcea3c46e6182))
* add is_favorite field to UnifiedWorkflow schema ([438bf76](https://github.com/qontinui/qontinui-schemas/commit/438bf76cfe5bbdd71a7d4869e04b085ba3c97814))
* add known issues types, canvas panel types, and rose accent color ([5748354](https://github.com/qontinui/qontinui-schemas/commit/5748354ce0ebeb7fecfaf7b49487e3835410e53b))
* add memory shared types for runner-mediated federation ([#58](https://github.com/qontinui/qontinui-schemas/issues/58)) ([cc7b5b3](https://github.com/qontinui/qontinui-schemas/commit/cc7b5b36cb553ec769f0a46bd6782159bd48a99a))
* add MissionBrief canvas component type and data schema ([efa6a30](https://github.com/qontinui/qontinui-schemas/commit/efa6a3091f6eee61e9367e34c31f9e1609daa125))
* add model overrides, stage conditions, skill composition types ([119090a](https://github.com/qontinui/qontinui-schemas/commit/119090a0ad6dd4e296d348912979677b97289cc9))
* add multi-agent, worktree, and architecture fields to UnifiedWorkflow ([b85292f](https://github.com/qontinui/qontinui-schemas/commit/b85292fac7e41af867087e2f08dac9a76f01cc70))
* add multi-tenant app registry types (spec-multi-app Stream A) ([bc9d56d](https://github.com/qontinui/qontinui-schemas/commit/bc9d56d05fd5d940fc3584bf5132d8a084caff99))
* add native_accessibility to StepTypeName across all schema layers ([28c8568](https://github.com/qontinui/qontinui-schemas/commit/28c8568a0552ac913bacb3734b88ceb8cb9828a6))
* add npm publish workflow for @qontinui/shared-types ([#4](https://github.com/qontinui/qontinui-schemas/issues/4)) ([ae69278](https://github.com/qontinui/qontinui-schemas/commit/ae692781a15963771bc1cc19d08e794395886ad1))
* add projects module — SavedProject registry and ProjectSnapshot view ([6ba7a07](https://github.com/qontinui/qontinui-schemas/commit/6ba7a077bc1c2121cfc7efe90a6798c0809ee3fb))
* add quality analysis, dependency graph, and cost annotation types ([740e5ea](https://github.com/qontinui/qontinui-schemas/commit/740e5eae8233d0a28b485b7e765f08c5e2efb9d0))
* add state machine type definitions ([c38ec91](https://github.com/qontinui/qontinui-schemas/commit/c38ec91c741b2ec4441fd41a06673b42520937f2))
* add structured action plan types for UI Bridge ([a43cb59](https://github.com/qontinui/qontinui-schemas/commit/a43cb59a251d4b2a4b1996ac48e73dd90d362185))
* add test specifications schema module ([cf638a0](https://github.com/qontinui/qontinui-schemas/commit/cf638a0f915cbffddc33875f0439ef975a1eb2ed))
* add TypeScript implementation for schemas ([4be80fb](https://github.com/qontinui/qontinui-schemas/commit/4be80fb3f3faf050af95a403c1850e6999ce13d4))
* add ui_bridge_snapshot_target to UiBridgeStep schema ([1496bef](https://github.com/qontinui/qontinui-schemas/commit/1496befcf156e4a2385e214af699ad0419a72525))
* add workflow_id field to scheduler task type ([25ed6ff](https://github.com/qontinui/qontinui-schemas/commit/25ed6ff97c0051f316486742a8f01ec8b0da1dbe))
* AgentTextUnitDefault — the embedded-default layer the corpus never had ([de80cca](https://github.com/qontinui/qontinui-schemas/commit/de80ccab1ebcf29ea6f71c572ba3c2b345a399f8))
* **codegen:** cross-file TypeScript generation via schema preprocessing ([db27f3f](https://github.com/qontinui/qontinui-schemas/commit/db27f3f08f562930f1ab1b8172c1714382b8b70a))
* declare paramSchema, path, actionInvocationPath and effect on the canonical types ([bcfb344](https://github.com/qontinui/qontinui-schemas/commit/bcfb34490e36350c07093d33f2031706568c0e47))
* **events:** canonical UI Bridge HTTP envelopes (error + health) ([#68](https://github.com/qontinui/qontinui-schemas/issues/68)) ([235295c](https://github.com/qontinui/qontinui-schemas/commit/235295caf9780f09de71d1be294c491ebfe95609))
* **events:** concretize loose Tauri payloads to typed structs ([101a557](https://github.com/qontinui/qontinui-schemas/commit/101a557896ee3e31e3bb40df823f614cf9584bea))
* **events:** typed UI Bridge request/response envelopes ([2aca16e](https://github.com/qontinui/qontinui-schemas/commit/2aca16e190b9359dc4650dc23c4dc2ae19382742))
* expand scheduler types with task variants and execution records ([efdb92f](https://github.com/qontinui/qontinui-schemas/commit/efdb92f20f3f130e635dd75ff526efcd46e6b423))
* extend UIBridgeElement with accessibility fields; replace IrState.required_elements with assertions ([861644a](https://github.com/qontinui/qontinui-schemas/commit/861644a843a3eac657d5a4478a8fa9b541b9c371))
* **functional-spec:** freeze functional-spec-contract v0 (spec + rubric + profile) ([#79](https://github.com/qontinui/qontinui-schemas/issues/79)) ([6b86061](https://github.com/qontinui/qontinui-schemas/commit/6b860617e543a35a4ea06613c4251e0ceac48bd0))
* **ir:** add IrApiCheck type for Spec CI API-contract assertions ([#66](https://github.com/qontinui/qontinui-schemas/issues/66)) ([598a6ae](https://github.com/qontinui/qontinui-schemas/commit/598a6ae86ed864a8fe71766cd06d393a64e034e8))
* **ir:** add IRGroup + IRDocument.synthesizedGroups for synthesis output ([#10](https://github.com/qontinui/qontinui-schemas/issues/10)) ([414353a](https://github.com/qontinui/qontinui-schemas/commit/414353af49be04fa3764ac18d6411917fece7a70))
* **ir:** add IrProvenance.status + ProposalStatus enum for flywheel ([ee1e75f](https://github.com/qontinui/qontinui-schemas/commit/ee1e75fd7dd42d287dbb03027b94208d37a6882b))
* **orchestration:** conductor LoopPhase variants (Approach-D Phase 3) ([#83](https://github.com/qontinui/qontinui-schemas/issues/83)) ([a68b1c3](https://github.com/qontinui/qontinui-schemas/commit/a68b1c369eee71093949ec93344b19acd0e2edf0))
* **process-state:** add ExternallyOwned variant + regenerate bindings ([#65](https://github.com/qontinui/qontinui-schemas/issues/65)) ([89058f6](https://github.com/qontinui/qontinui-schemas/commit/89058f673fb0adfda8716c0b4ed9198805a11f88))
* restructure TypeScript package with modular schema exports ([54033bd](https://github.com/qontinui/qontinui-schemas/commit/54033bd6dcc1e4fd00d56c5ae4644ba484f2ba80))
* **schemas:** add canonical Runner type ([ed315d3](https://github.com/qontinui/qontinui-schemas/commit/ed315d3ac071aa6c2901a14d679728999de9f71a))
* **schemas:** add ElementState.disabled + ariaDisabled ([2c4fd12](https://github.com/qontinui/qontinui-schemas/commit/2c4fd129c4ec81a43146a117a829fe29f081c1da))
* **schemas:** add FullRunnerStep typed union covering all 16 runner step variants ([d2a083c](https://github.com/qontinui/qontinui-schemas/commit/d2a083cdfff6b6376eeeaeccc5ad543332048482))
* **schemas:** add HTN config fields to UnifiedWorkflow ([c5b35be](https://github.com/qontinui/qontinui-schemas/commit/c5b35be5978cecbb8bb91bd7a7327dbbe4b48af1))
* **schemas:** add scheduler reliability fields + RemoteAgent task type ([0079343](https://github.com/qontinui/qontinui-schemas/commit/00793432877c3b68cbf2728f7a469a58e348ed0d))
* **schemas:** Classification thresholds for spec evaluation ([8973f27](https://github.com/qontinui/qontinui-schemas/commit/8973f27edf6a93a4897a1e650f7ff9533fe19e3d))
* **schemas:** forward-compat device-namespace aliases for unified-devices rollout ([#54](https://github.com/qontinui/qontinui-schemas/issues/54)) ([07726bf](https://github.com/qontinui/qontinui-schemas/commit/07726bf625a02b8542eec4c9bcf451944d84c5d6))
* **schemas:** Rust source-of-truth for task-run, execution, state-machine ([db33145](https://github.com/qontinui/qontinui-schemas/commit/db331459354633661992f8d26ffe04d915fbaaf1))
* **spec-api-events:** add SpecApiEvent enum for Plan 06 broadcasts ([#43](https://github.com/qontinui/qontinui-schemas/issues/43)) ([df1baeb](https://github.com/qontinui/qontinui-schemas/commit/df1baebe1289afec4c9ab7ffc6cf6f8ae7ba9332))
* **spec-check:** add snapshot_sha256 field to SpecCheckResult ([#60](https://github.com/qontinui/qontinui-schemas/issues/60)) ([95af1aa](https://github.com/qontinui/qontinui-schemas/commit/95af1aa8b3d0090fadb46f2f160dc81d8e738644))
* **spec-check:** add SpecCheckSummary.recommendation_reason ([#45](https://github.com/qontinui/qontinui-schemas/issues/45)) ([0b70bd4](https://github.com/qontinui/qontinui-schemas/commit/0b70bd47242e40ceefc96dd13f76aa4bb9420a6c))
* surface element bbox + visibility on UIBridgeElement ([5094dcd](https://github.com/qontinui/qontinui-schemas/commit/5094dcdc8281b5be377a2dd22444e75c449b9d45))
* **ts:** commit missing generated bindings for relay + status events ([05d34b7](https://github.com/qontinui/qontinui-schemas/commit/05d34b7951c1e661f320b00ceb3080e852e6de3e))
* **ts:** export SpecValidation + ProposalStatus from main entry; bump to 0.3.0 ([1f43a54](https://github.com/qontinui/qontinui-schemas/commit/1f43a544ae3c20ef445437a089ed565ecff81797))
* **ts:** export Tauri event payload bindings via @qontinui/shared-types/tauri-events ([1b3cef9](https://github.com/qontinui/qontinui-schemas/commit/1b3cef9ec5b82bc70cbd329925a0ca265dd2ebe4))
* **types:** add discovery, orchestration_config, verification modules ([717015a](https://github.com/qontinui/qontinui-schemas/commit/717015a36d2e7198fbe69e744befa3da2e3fb1c9))
* **types:** add findings, process_management, ticket_system modules; expand execution ([59d97c3](https://github.com/qontinui/qontinui-schemas/commit/59d97c3a450064b715c3339c1a83467825f8b992))
* **types:** add ir, spec_check, text_norm modules (Plan 01 foundation) ([c7790cc](https://github.com/qontinui/qontinui-schemas/commit/c7790cc58007b53796374b6bed288711027f592f))
* **types:** add mcp_config and terminal modules ([b28a541](https://github.com/qontinui/qontinui-schemas/commit/b28a541e7bc9b9952be3aec42f937559ca328731))
* **types:** add ui_bridge, ai_workflows modules; expand rag ([93e7562](https://github.com/qontinui/qontinui-schemas/commit/93e7562a45c4da957f5f10ae3998d244ec6b0fc6))
* **types:** port accessibility + config from Pydantic to Rust ([7ff7ade](https://github.com/qontinui/qontinui-schemas/commit/7ff7ade905eb1f09aa79a66c824f8002a2e64240))
* **types:** port geometry + tree_events from Pydantic to Rust ([ab1eab5](https://github.com/qontinui/qontinui-schemas/commit/ab1eab51df5527502bb94a985d5117db1d735fbf))
* **types:** port targets + rag to Rust, delete Pydantic TS pipeline ([b68131c](https://github.com/qontinui/qontinui-schemas/commit/b68131c3b4b81cd25d1088fcb74481a4128583de))
* **types:** universal camelCase wire normalization with backward-compat aliases ([854d4b7](https://github.com/qontinui/qontinui-schemas/commit/854d4b74064d58fa71044256a261f1951d11f038))
* **ui_bridge:** opaque effect_verification on ActionResponse (D3 Phase 3a) ([#70](https://github.com/qontinui/qontinui-schemas/issues/70)) ([11936fb](https://github.com/qontinui/qontinui-schemas/commit/11936fb9b310aa45f6513d14bc5c256f97e08e48))
* **ui-bridge-ir:** add IR module + projection + tsup entry ([432e4bd](https://github.com/qontinui/qontinui-schemas/commit/432e4bdcff2d0425178d73600e8841a4090cffa6))
* **ui-bridge-ir:** add IRPairingPolicy.acceptDrift schema ([#5](https://github.com/qontinui/qontinui-schemas/issues/5)) ([54e50b9](https://github.com/qontinui/qontinui-schemas/commit/54e50b99b2a6e5c75dd23364b5eb6de97f98e427))
* **ui-bridge-ir:** inverse projection + tagName/accessibleName/precondition ([#2](https://github.com/qontinui/qontinui-schemas/issues/2)) ([5070a74](https://github.com/qontinui/qontinui-schemas/commit/5070a7446b5cff5570b3f04a39bc1c8b60da35b6))


### Bug Fixes

* add missing dist chunks and sync UnifiedWorkflow types across TS files ([92ee3bd](https://github.com/qontinui/qontinui-schemas/commit/92ee3bd02de9bf9a0f76572533aa967ef63222b5))
* **apps:** serialize AppError fields as camelCase ([#61](https://github.com/qontinui/qontinui-schemas/issues/61)) ([6c0cf31](https://github.com/qontinui/qontinui-schemas/commit/6c0cf311ef2ac1dc2c05081f0bc5a07fce18429a))
* **codegen:** make TS codegen formatting deterministic to unjam schema-drift CI ([bc2d903](https://github.com/qontinui/qontinui-schemas/commit/bc2d90372c446861c8ced46879234e79e063813b))
* **codegen:** set Confidence enum title to SpecCheckConfidence ([#44](https://github.com/qontinui/qontinui-schemas/issues/44)) ([79238b2](https://github.com/qontinui/qontinui-schemas/commit/79238b2433e3771e9576c18be3aadcafd2e76065))
* restore is_favorite field to UnifiedWorkflow TypeScript type ([0b15dcd](https://github.com/qontinui/qontinui-schemas/commit/0b15dcdac622a698cdd7cdf35d57edb21c8b8f07))
* restore the checked-in TypeScript bindings ([e8b9102](https://github.com/qontinui/qontinui-schemas/commit/e8b9102015a2962ac738beb88c6e1da17e8682c6))
* **schemas:** align TaskRunFinding/Session wire name with backend ([6246c7a](https://github.com/qontinui/qontinui-schemas/commit/6246c7a2711915ad5f48e837f3c68d10624c0595))
* **schemas:** auto-fix ruff lint ([7f18279](https://github.com/qontinui/qontinui-schemas/commit/7f18279e52f83ae033d5f5563c6e0dde9fc2569d))
* **schemas:** make TransitionInfo.to_state required-nullable ([3c30b44](https://github.com/qontinui/qontinui-schemas/commit/3c30b4444dbc0e0c0bcc83cd048d013a8c0fc697))
* **ts:** declare $defs types and gate generated .d.ts with skipLibCheck off ([fb3389b](https://github.com/qontinui/qontinui-schemas/commit/fb3389bfcfd1a88ab13d36f6c122dfb15cadc3db))
* **ts:** fail codegen on forked type declarations; gate the publish path ([0b0df85](https://github.com/qontinui/qontinui-schemas/commit/0b0df856d1f73fa596473933c8e936dda112f8d6))
* **ts:** publish ./tauri-events subpath in package.json ([a910b79](https://github.com/qontinui/qontinui-schemas/commit/a910b7931b7a21d8fef8970ea1ba23f9868a1992))
* **types:** add deny_unknown_fields to eliminate TS index signatures ([380e4ab](https://github.com/qontinui/qontinui-schemas/commit/380e4aba2c5aca381d920929fba23e775d17ad96))
* **types:** add Manual variant to FindingActionType + rebuild stale dist ([33640c6](https://github.com/qontinui/qontinui-schemas/commit/33640c6acac4a37b31f045eff8021b685617fd99))
* **ui-bridge-ir:** drop placeholder assertion when state has zero requiredElements ([#3](https://github.com/qontinui/qontinui-schemas/issues/3)) ([ae0b7f1](https://github.com/qontinui/qontinui-schemas/commit/ae0b7f15313bad841f4d576675f479a585f86ddb))

## [0.14.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.13.0...ts-v0.14.0) (2026-09-02)


### Features

* AgentTextUnitDefault — the embedded-default layer the corpus never had ([de80cca](https://github.com/qontinui/qontinui-schemas/commit/de80ccab1ebcf29ea6f71c572ba3c2b345a399f8))

## [0.13.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.12.1...ts-v0.13.0) (2026-08-31)


### Features

* add AgentTextUnit canonical types for the agent text corpus ([c61fc22](https://github.com/qontinui/qontinui-schemas/commit/c61fc2220f76fd0b09567c68099772a366fe0442))

## [0.12.1](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.12.0...ts-v0.12.1) (2026-08-28)


### Bug Fixes

* **ts:** declare $defs types and gate generated .d.ts with skipLibCheck off ([fb3389b](https://github.com/qontinui/qontinui-schemas/commit/fb3389bfcfd1a88ab13d36f6c122dfb15cadc3db))
* **ts:** fail codegen on forked type declarations; gate the publish path ([0b0df85](https://github.com/qontinui/qontinui-schemas/commit/0b0df856d1f73fa596473933c8e936dda112f8d6))

## [0.12.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.11.0...ts-v0.12.0) (2026-08-22)


### Features

* declare paramSchema, path, actionInvocationPath and effect on the canonical types ([bcfb344](https://github.com/qontinui/qontinui-schemas/commit/bcfb34490e36350c07093d33f2031706568c0e47))

## [0.11.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.10.1...ts-v0.11.0) (2026-08-06)


### Features

* **schemas:** add ElementState.disabled + ariaDisabled ([2c4fd12](https://github.com/qontinui/qontinui-schemas/commit/2c4fd129c4ec81a43146a117a829fe29f081c1da))

## [0.10.1](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.10.0...ts-v0.10.1) (2026-07-30)


### Bug Fixes

* restore the checked-in TypeScript bindings ([e8b9102](https://github.com/qontinui/qontinui-schemas/commit/e8b9102015a2962ac738beb88c6e1da17e8682c6))

## [0.10.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.9.0...ts-v0.10.0) (2026-07-29)


### Features

* add projects module — SavedProject registry and ProjectSnapshot view ([6ba7a07](https://github.com/qontinui/qontinui-schemas/commit/6ba7a077bc1c2121cfc7efe90a6798c0809ee3fb))

## [0.9.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.8.1...ts-v0.9.0) (2026-07-02)


### Features

* **schemas:** Classification thresholds for spec evaluation ([8973f27](https://github.com/qontinui/qontinui-schemas/commit/8973f27edf6a93a4897a1e650f7ff9533fe19e3d))

## [0.8.1](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.8.0...ts-v0.8.1) (2026-06-29)


### Bug Fixes

* **codegen:** make TS codegen formatting deterministic to unjam schema-drift CI ([bc2d903](https://github.com/qontinui/qontinui-schemas/commit/bc2d90372c446861c8ced46879234e79e063813b))

## [0.8.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.7.0...ts-v0.8.0) (2026-06-15)


### Features

* **functional-spec:** freeze functional-spec-contract v0 (spec + rubric + profile) ([#79](https://github.com/qontinui/qontinui-schemas/issues/79)) ([6b86061](https://github.com/qontinui/qontinui-schemas/commit/6b860617e543a35a4ea06613c4251e0ceac48bd0))
* **orchestration:** conductor LoopPhase variants (Approach-D Phase 3) ([#83](https://github.com/qontinui/qontinui-schemas/issues/83)) ([a68b1c3](https://github.com/qontinui/qontinui-schemas/commit/a68b1c369eee71093949ec93344b19acd0e2edf0))

## [0.7.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.6.0...ts-v0.7.0) (2026-06-07)


### Features

* **events:** canonical UI Bridge HTTP envelopes (error + health) ([#68](https://github.com/qontinui/qontinui-schemas/issues/68)) ([235295c](https://github.com/qontinui/qontinui-schemas/commit/235295caf9780f09de71d1be294c491ebfe95609))
* **ir:** add IrApiCheck type for Spec CI API-contract assertions ([#66](https://github.com/qontinui/qontinui-schemas/issues/66)) ([598a6ae](https://github.com/qontinui/qontinui-schemas/commit/598a6ae86ed864a8fe71766cd06d393a64e034e8))
* **ui_bridge:** opaque effect_verification on ActionResponse (D3 Phase 3a) ([#70](https://github.com/qontinui/qontinui-schemas/issues/70)) ([11936fb](https://github.com/qontinui/qontinui-schemas/commit/11936fb9b310aa45f6513d14bc5c256f97e08e48))

## [0.6.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.5.0...ts-v0.6.0) (2026-05-25)


### Features

* **process-state:** add ExternallyOwned variant + regenerate bindings ([#65](https://github.com/qontinui/qontinui-schemas/issues/65)) ([89058f6](https://github.com/qontinui/qontinui-schemas/commit/89058f673fb0adfda8716c0b4ed9198805a11f88))
* **spec-check:** add snapshot_sha256 field to SpecCheckResult ([#60](https://github.com/qontinui/qontinui-schemas/issues/60)) ([95af1aa](https://github.com/qontinui/qontinui-schemas/commit/95af1aa8b3d0090fadb46f2f160dc81d8e738644))


### Bug Fixes

* **apps:** serialize AppError fields as camelCase ([#61](https://github.com/qontinui/qontinui-schemas/issues/61)) ([6c0cf31](https://github.com/qontinui/qontinui-schemas/commit/6c0cf311ef2ac1dc2c05081f0bc5a07fce18429a))

## [0.5.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.4.0...ts-v0.5.0) (2026-05-23)


### Features

* add memory shared types for runner-mediated federation ([#58](https://github.com/qontinui/qontinui-schemas/issues/58)) ([cc7b5b3](https://github.com/qontinui/qontinui-schemas/commit/cc7b5b36cb553ec769f0a46bd6782159bd48a99a))
* add multi-tenant app registry types (spec-multi-app Stream A) ([bc9d56d](https://github.com/qontinui/qontinui-schemas/commit/bc9d56d05fd5d940fc3584bf5132d8a084caff99))
