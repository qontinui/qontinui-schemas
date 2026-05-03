//! Typed HTTP client for the qontinui-runner `/spawn-placement` endpoints.
//!
//! Used by `qontinui-supervisor` (and any future fleet-management UI) to
//! avoid duplicating URL building / response parsing.
//!
//! The client wraps the runner's two single-slot lookup endpoints:
//!
//! * `GET /spawn-placement/preview?slot=N&overflow=wrap` — placement for a
//!   configured runner-instance slot.
//! * `GET /spawn-placement/temp?index=N&overflow=wrap` — placement for the
//!   `index`-th temp-runner placement (round-robin via `% len`).
//!
//! Both endpoints return [`SpawnPlacementResponse`](qontinui_types::wire::placement::SpawnPlacementResponse)
//! wrapped in the runner's `ApiResponse<T>` envelope (`{success, data, error}`).
//! The client unwraps the envelope locally and returns the bare payload, so
//! callers don't have to care about that runner-side HTTP convention.
//!
//! Layered on `qontinui-types::wire::placement` so the wire format stays in
//! one place and the runner / supervisor / any future consumer can never
//! drift on the shape.

pub mod placement;

pub use placement::{Overflow, SpawnPlacementClient, SpawnPlacementClientError};
