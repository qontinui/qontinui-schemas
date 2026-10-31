# Changelog

## [2.5.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.4.1...ts-v2.5.0) (2026-10-31)


### Features

* generated bindings for the spec_model types ([7bab6d4](https://github.com/qontinui/qontinui-schemas/commit/7bab6d418bb3eee3b96fa41fd342338fe09c48f6))
* **scheduler:** RemoteAgent carries effort, ultracode and effort_profile ([55eedce](https://github.com/qontinui/qontinui-schemas/commit/55eedceb1a60d0f348866c221338e7a162f98bc4))
* **types:** journey graph queries over the edge ledger, pinned to multistate ([429d17f](https://github.com/qontinui/qontinui-schemas/commit/429d17f64f28c5aafcdc089f48d04ee70a9670fd))
* **vision-core:** interaction timeline, timeline assertions, ledger projection and edge timing ([3d6c9bf](https://github.com/qontinui/qontinui-schemas/commit/3d6c9bfb338bb240fba1b021f59ce2b0a557c07d))


### Bug Fixes

* **codegen:** drop runner[#1990](https://github.com/qontinui/qontinui-schemas/issues/1990)'s expired bindings no runner main exports ([dc6c2d0](https://github.com/qontinui/qontinui-schemas/commit/dc6c2d01adecea867159c680069d696b9a7378ce))

## [2.4.1](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.4.0...ts-v2.4.1) (2026-10-22)


### Bug Fixes

* **ci-exec:** bump zip to 9 for RUSTSEC-2026-0367..0369 ([3ac862b](https://github.com/qontinui/qontinui-schemas/commit/3ac862bcb2ed38860445a4befbd30185096d6ef0))
* restore run-outcomes capture and ts 2.4.0 reverted by 3ac862bc ([0d2fd65](https://github.com/qontinui/qontinui-schemas/commit/0d2fd65ed1f2b7e71b7276901b0266270c167aa9))

## [2.4.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.3.1...ts-v2.4.0) (2026-10-19)


### Features

* add ScheduledTaskType::Script (command, working_directory, timeout_seconds) ([20c02a8](https://github.com/qontinui/qontinui-schemas/commit/20c02a87d257a34fbff761c8f8f82f3a2c9fa44a))
* **ts:** ImageSlotSpec bindings, exported from the package entry ([c415a43](https://github.com/qontinui/qontinui-schemas/commit/c415a432156823d0b31c5b976e4ef9014576aae9))


### Bug Fixes

* **vision-core:** absent requires considered &gt; 0; bindings documented as types, not validators ([37f3530](https://github.com/qontinui/qontinui-schemas/commit/37f3530606a895adfa463295405b50c270aaf9e2))

## [2.3.1](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.3.0...ts-v2.3.1) (2026-10-13)


### Bug Fixes

* address pre-PR review of the glossary v2 corrections ([e797a9b](https://github.com/qontinui/qontinui-schemas/commit/e797a9b443121d3580b7d6129b4ed961378639b7))
* correct seven glossary terms that disagree with the services (glossary v2) ([2253e55](https://github.com/qontinui/qontinui-schemas/commit/2253e55cc8d40a90508484dc3ff65d3231616b68))

## [2.3.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.2.0...ts-v2.3.0) (2026-10-13)


### Features

* **types:** generated TS/Python bindings for the cli_session types ([86710c8](https://github.com/qontinui/qontinui-schemas/commit/86710c896b0834516ae072959e92013530022b0d))

## [2.2.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.1.0...ts-v2.2.0) (2026-10-06)


### Features

* **refusal:** generic HTTP-class refusal codes every producer can share ([3db4fba](https://github.com/qontinui/qontinui-schemas/commit/3db4fba446afa07774cd04991ce93cd29e5e0ae5))


### Bug Fixes

* **refusal:** carry the retry_later target doc into the generated bindings ([c1be97a](https://github.com/qontinui/qontinui-schemas/commit/c1be97a259900ff8bb56bad8840bb98e7c1d00c2))

## [2.1.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v2.0.0...ts-v2.1.0) (2026-10-02)


### Features

* add require_probe scheduler condition (ProbeCondition) ([9374ddc](https://github.com/qontinui/qontinui-schemas/commit/9374ddc1ea11cd9995b09bbaa18739469dc99f21))
* generated TS and Python bindings for GlossaryTerm and Refusal ([3e4b7e2](https://github.com/qontinui/qontinui-schemas/commit/3e4b7e29a67d1746e714afa77620f9c4496c81a1))
* **ts:** export RunnerInstance and RunnerInstanceRole from the runner entry ([947e6d1](https://github.com/qontinui/qontinui-schemas/commit/947e6d13a2ecb075e5c89cb76de410cac350803a))
* **ts:** publish the glossary table and refusal types to TypeScript ([c0af6ca](https://github.com/qontinui/qontinui-schemas/commit/c0af6ca8ebad89295a6060db1c4f5da86d6d52c4))


### Bug Fixes

* regenerate bindings for unrecognised_kind and unrecognised_source ([db5facc](https://github.com/qontinui/qontinui-schemas/commit/db5facc2ff06299084f1cf85bc619b5ffe959364))
* regenerate Refusal bindings for the lenient envelope ([201d6ea](https://github.com/qontinui/qontinui-schemas/commit/201d6ea8bf0539729168c28524be310dfed9e8a7))
* **ts:** quote glossary table keys so the generator does not lean on the id invariant ([6564c0f](https://github.com/qontinui/qontinui-schemas/commit/6564c0f7f4850016205b822777acf8097904ab86))

## [2.0.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v1.1.0...ts-v2.0.0) (2026-09-30)


### ⚠ BREAKING CHANGES

* UiBridgeAction's #[default] moves from Navigate to Snapshot. An action-less ui_bridge step is displayed as `snapshot` by both step editors and run as one by the runner's handler; only the typed view said `navigate`. Code that relied on UiBridgeAction::default() (or on UiBridgeStep::default().action) being Navigate now gets Snapshot.
* FullRunnerStep and UiBridgeAction are public, non-#[non_exhaustive] enums and gain variants, so an exhaustive match on either no longer compiles.

### Features

* FullRunnerStep covers all 20 registered handlers and parses every live step shape ([8502266](https://github.com/qontinui/qontinui-schemas/commit/8502266f3894f5c0c2c7d0485f751ea1835c6ab6))
* UiBridgeAction defaults to snapshot and gains component_action ([f733c88](https://github.com/qontinui/qontinui-schemas/commit/f733c88910dd559f9618e54098964d9c3c5430bb))

## [1.1.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v1.0.0...ts-v1.1.0) (2026-09-23)


### Features

* **schemas:** Runner.instances — per-instance wire rows ([a3711db](https://github.com/qontinui/qontinui-schemas/commit/a3711db3741adaaf89ea1db68c38c22368fce1b8))

## [1.0.0](https://github.com/qontinui/qontinui-schemas/compare/ts-v0.14.0...ts-v1.0.0) (2026-09-13)


### ⚠ BREAKING CHANGES

* **ui-bridge:** the published schema for `UIBridgeElement.customActions` changes from `string[] | null` to `ElementActionInfo[] | null`. Backward compatibility is deliberately not preserved — no shim, no union type.

### Features

* **ui-bridge:** element custom actions become objects that can carry `effect` ([cbcbe2f](https://github.com/qontinui/qontinui-schemas/commit/cbcbe2fa3452942edb1ecc87ab37e15b3aeb87c7))

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
