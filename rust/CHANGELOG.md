# Changelog

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
