# qontinui-runner-client

Typed Rust HTTP client for the [qontinui-runner](https://github.com/qontinui/qontinui-runner)'s
`/spawn-placement/{preview,temp}` endpoints. Layered on
[`qontinui-types::wire::placement`](https://docs.rs/qontinui-types) so the
wire format is single-sourced.

Used by `qontinui-supervisor` and any future fleet-management UI to avoid
duplicating URL building, the runner's `ApiResponse<T>` envelope unwrapping,
and the round-trip parsing.

## Usage

```rust,no_run
use qontinui_runner_client::{Overflow, SpawnPlacementClient};
use reqwest::Client;
use url::Url;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let base = Url::parse("http://localhost:9876")?;
let client = SpawnPlacementClient::new(base, Client::new());

// Look up the placement for a configured runner-instance slot.
let placement = client.preview(0, Overflow::Wrap).await?;
println!(
    "slot {} ({}) → ({}, {}) {}x{} on {}",
    placement.slot_index,
    placement.slot_label,
    placement.global_x,
    placement.global_y,
    placement.width,
    placement.height,
    placement.monitor_label,
);

// Or look up the i-th temp-runner placement (round-robin via index % len
// when overflow=wrap).
let temp = client.temp(3, Overflow::Wrap).await?;
# let _ = temp;
# Ok(())
# }
```

The client returns a bare
[`SpawnPlacementResponse`](https://docs.rs/qontinui-types/latest/qontinui_types/wire/placement/struct.SpawnPlacementResponse.html)
on success and a single
[`SpawnPlacementClientError`](https://docs.rs/qontinui-runner-client/latest/qontinui_runner_client/enum.SpawnPlacementClientError.html)
enum covering every failure mode (HTTP, URL parse, non-2xx status with body,
envelope-without-data, parse error). The error type is `#[non_exhaustive]`
so future variants are non-breaking.

## Feature flags

None currently. The crate is `default-features = false` on `reqwest` with
the `json` and `rustls` features enabled — TLS is handled via `rustls`,
not `native-tls`, to match the rest of the qontinui ecosystem.

## Compatibility

- **MSRV**: Rust 1.75 (matches the workspace's MSRV; documented but not yet
  enforced via `rust-version` in `Cargo.toml`).
- **Stability**: pre-1.0 (`0.x`). The public API is the items re-exported
  from `lib.rs` (`SpawnPlacementClient`, `Overflow`, `SpawnPlacementClientError`).
  Breaking changes will bump the minor version per Cargo's pre-1.0 SemVer
  rules; the CHANGELOG calls them out explicitly. Wire-format compatibility
  is the responsibility of `qontinui-types`.
- **Runner version**: built against the `/spawn-placement/{preview,temp}`
  surface the runner has stabilized as of `qontinui-types 0.1.x`. The
  `qontinui-types` dependency is constrained as `^0.1`, so a runner that
  speaks `qontinui-types 0.2.x` will not be compatible without a runner-client
  bump.

## License

Apache-2.0. See the workspace [`LICENSE`](https://github.com/qontinui/qontinui-schemas/blob/main/LICENSE).
