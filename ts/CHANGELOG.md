# Changelog

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
