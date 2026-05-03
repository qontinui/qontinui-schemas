//! Spawn-placement wire types shared between qontinui-runner (producer) and
//! qontinui-supervisor (consumer).
//!
//! The runner exposes three HTTP endpoints under `/spawn-placement/*`:
//!
//! * `GET /spawn-placement/preview?slot=N&overflow=wrap` — placement for a
//!   configured runner instance slot.
//! * `GET /spawn-placement/temp?index=N&overflow=wrap` — placement for the
//!   `index`-th temp-runner placement (round-robin).
//! * `GET /spawn-placement/temps` — list temp-runner placements (runner-only,
//!   used by the runner's own settings UI).
//!
//! Both single-slot endpoints return [`SpawnPlacementResponse`] wrapped in the
//! runner's standard `ApiResponse<T>` envelope. The envelope itself stays a
//! runner-side HTTP convention; the supervisor unwraps it locally.
//!
//! ## Strictness
//!
//! All types here use:
//!
//! * `#[non_exhaustive]` on responses so consumers must construct via field
//!   init shorthand (preventing the "added a field, forgot to update the
//!   producer" bug).
//! * `#[serde(deny_unknown_fields)]` on requests so the runner rejects
//!   typo'd query params at parse time instead of silently ignoring them.

use serde::{Deserialize, Serialize};

/// Response payload for `GET /spawn-placement/preview` and
/// `GET /spawn-placement/temp`. The runner produces this; the supervisor
/// parses it (via the `ApiResponse<T>` envelope) and turns it into
/// `QONTINUI_WINDOW_*` env vars on the spawned runner's command.
///
/// `#[non_exhaustive]` prevents external struct-literal construction so a
/// new field added here only requires updating producers via
/// [`SpawnPlacementResponse::new`] / setter mutators rather than every
/// call site (which is the bug we hit with `decorations`). The runner
/// constructs via [`new`] then sets optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpawnPlacementResponse {
    /// Absolute virtual-desktop X in physical pixels.
    pub global_x: i32,
    /// Absolute virtual-desktop Y in physical pixels.
    pub global_y: i32,
    /// Window width in physical pixels.
    pub width: u32,
    /// Window height in physical pixels.
    pub height: u32,
    /// Resolved monitor label (e.g. `"primary"`, `"left"`, OS device name).
    pub monitor_label: String,
    /// Effective slot index. For preview this is the configured slot; for
    /// temp lookup this is the resolved index after `% len` round-robin.
    pub slot_index: usize,
    /// Either the configured instance name, `"primary"`, or
    /// `"temp[<index>]"` for temp placements.
    pub slot_label: String,
    /// Discriminator for which placement source resolved this response:
    /// `"configured"` for `runner_instances` slots, `"temp"` for temp-runner
    /// placements.
    pub source: String,
    /// Per-placement window decorations toggle. `None` means "use the
    /// runner's default" (chrome on). Forwarded by the supervisor as
    /// `QONTINUI_WINDOW_DECORATIONS=0|1` so a borderless placement lands
    /// flush with the configured rect (no OS window border inset).
    ///
    /// Omitted from the wire when `None` (`#[serde(skip_serializing_if)]`)
    /// to preserve the existing wire shape from before this field was
    /// extracted to a shared crate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorations: Option<bool>,
}

impl SpawnPlacementResponse {
    /// Construct a response with all required fields. `decorations` defaults
    /// to `None` (use the runner's default chrome); call
    /// [`with_decorations`](Self::with_decorations) to override.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global_x: i32,
        global_y: i32,
        width: u32,
        height: u32,
        monitor_label: String,
        slot_index: usize,
        slot_label: String,
        source: String,
    ) -> Self {
        Self {
            global_x,
            global_y,
            width,
            height,
            monitor_label,
            slot_index,
            slot_label,
            source,
            decorations: None,
        }
    }

    /// Override the per-placement decorations toggle. Pass `None` to keep
    /// "use the runner's default", `Some(true)` for chrome on, `Some(false)`
    /// for borderless.
    #[must_use]
    pub fn with_decorations(mut self, decorations: Option<bool>) -> Self {
        self.decorations = decorations;
        self
    }
}

/// Query for `GET /spawn-placement/preview`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnPlacementPreviewQuery {
    /// Slot index. 0 = primary, 1.. = configured `runner_instances` in
    /// saved order.
    pub slot: usize,
    /// Behavior when `slot` is past the end of the list. `"wrap"` rotates
    /// `slot % count` over slots that have placements; default = 404.
    #[serde(default)]
    pub overflow: Option<String>,
}

/// Query for `GET /spawn-placement/temp`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TempPlacementLookupQuery {
    /// 0-based index into the temp placement list. Round-robin'd via
    /// `index % len` when `overflow=wrap`.
    pub index: usize,
    /// Behavior when `index >= len`. `"wrap"` (default if missing) rotates;
    /// `"default"` (or anything else) returns 404.
    #[serde(default)]
    pub overflow: Option<String>,
}

/// Response payload for `GET /spawn-placement/temps` (the runner's own
/// list endpoint — no supervisor consumer today).
///
/// Generic over the per-placement type so the runner can use its on-disk
/// `settings::SpawnPlacement` type directly while still living in the
/// shared wire module. External callers (none today) can default to
/// [`serde_json::Value`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TempPlacementsListResponse<P = serde_json::Value> {
    pub placements: Vec<P>,
    pub count: usize,
}

impl<P> TempPlacementsListResponse<P> {
    /// Construct a response from a placement list. `count` is filled from
    /// `placements.len()` so the two fields can never disagree.
    pub fn new(placements: Vec<P>) -> Self {
        let count = placements.len();
        Self { placements, count }
    }
}
