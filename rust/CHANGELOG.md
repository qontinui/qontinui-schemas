# Changelog

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
