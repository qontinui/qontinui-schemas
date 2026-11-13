// SPDX-License-Identifier: MIT
//! [`ComponentCapture`] — the structural capture of one rendered component,
//! and [`ComponentBaseline`], a capture stored under a name.
//!
//! This is the exact wire shape of the UI Bridge SDK routes
//! `GET /control/component/:id/capture` and `GET /control/element/:id/capture`
//! (plan `2026-11-13-ui-bridge-matches-isolated-render-assertion`). The same
//! SDK produces it on both sides of the comparison: once against an isolated
//! render (a Storybook story), stored as a [`ComponentBaseline`], and once in
//! situ on the real page. [`crate::Assertion::MatchesIsolatedRender`]
//! compares the two.
//!
//! Wire rules: camelCase keys, optional fields omitted when absent and
//! defaulted when missing, and unknown fields tolerated (no
//! `deny_unknown_fields`) so the SDK can grow the capture without breaking
//! this reader.
//!
//! **Geometry.** [`CaptureRoot::bbox`], [`CaptureRoot::parent_content_box`]
//! and every [`ClipAncestor::rect`] are in CSS px, PAGE coordinates. Every
//! [`ComponentCaptureNode::bbox`] is RELATIVE to the root bbox's origin, which
//! is what lets two captures taken at different viewports or page offsets be
//! compared.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The capture schema version this crate reads.
pub const COMPONENT_CAPTURE_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    COMPONENT_CAPTURE_SCHEMA_VERSION
}

/// A rectangle in CSS px. Page coordinates or root-relative, depending on the
/// field that holds it (see the module doc).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CaptureRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl CaptureRect {
    /// All four components are finite numbers.
    pub fn is_finite(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// This rect shifted by `origin`'s top-left corner — a root-relative node
    /// rect converted to page coordinates.
    pub fn offset_by(&self, origin: &CaptureRect) -> CaptureRect {
        CaptureRect {
            x: self.x + origin.x,
            y: self.y + origin.y,
            width: self.width,
            height: self.height,
        }
    }
}

/// What the capture was taken of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureTargetKind {
    /// A registered `useUIComponent` component; its root is the nearest
    /// common DOM ancestor of its mounted `elementIds`.
    Component,
    /// A registered element; its root is that element.
    Element,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTarget {
    pub kind: CaptureTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureViewport {
    pub width: f64,
    pub height: f64,
}

/// The captured root's geometry, in page coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRoot {
    /// The root element's border box. `None` (or non-finite) is a capture
    /// with no geometry, which the assertion answers `unknown{input_missing}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<CaptureRect>,
    /// The root's parent element's content box; `None` when the root has no
    /// parent element. The `size` check's fill ratio is measured against it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_content_box: Option<CaptureRect>,
}

/// A node whose DOM parent differs from its React parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePortal {
    /// A CSS-ish descriptor of the DOM parent it mounted into, e.g.
    /// `body>div#portal-root`.
    pub mount_target: String,
    /// The tree path of its React parent, when that parent is inside the
    /// captured tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub react_parent_path: Option<String>,
}

/// One node of the captured tree. `nodes` is depth-first pre-order and
/// `nodes[0]` is the root itself.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCaptureNode {
    /// Child-index path from the root over element children, `/`-joined;
    /// `""` for the root. Two captures' nodes are paired by this path.
    #[serde(default)]
    pub path: String,
    /// Lowercase tag name.
    #[serde(default)]
    pub tag: String,
    /// Explicit `role` attribute, else a best-effort implicit role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The UI Bridge element or component id registered on this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_id: Option<String>,
    /// The nearest owning React component's name, read from the fiber.
    /// `None` is UNREADABLE (minified production name, or redacted by
    /// `extractReactState`), never "no component".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
    /// Border box RELATIVE to the root bbox's origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<CaptureRect>,
    /// Whitelisted computed styles, camelCase CSS property names to string
    /// values. Every key is optional.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub styles: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portal: Option<CapturePortal>,
}

/// An ancestor OUTSIDE the root whose `overflowX` or `overflowY` is not
/// `visible`, i.e. one that can clip the component.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipAncestor {
    #[serde(default)]
    pub descriptor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_x: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_y: Option<String>,
    /// The ancestor's clip rect, in PAGE coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<CaptureRect>,
}

impl ClipAncestor {
    /// Whether this ancestor clips along the horizontal axis. An absent
    /// overflow value is taken as clipping: the SDK lists only ancestors that
    /// clip on at least one axis.
    pub fn clips_x(&self) -> bool {
        self.overflow_x.as_deref().is_none_or(|v| v != "visible")
    }

    /// Whether this ancestor clips along the vertical axis.
    pub fn clips_y(&self) -> bool {
        self.overflow_y.as_deref().is_none_or(|v| v != "visible")
    }
}

/// The structural capture of one rendered component (see the module doc).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCapture {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<CaptureTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    /// `location.pathname` at capture time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// The source sha the app exposes (`window.__QONTINUI_SOURCE_SHA__` or
    /// `<meta name="qontinui-source-sha">`), when it exposes one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<CaptureViewport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dpr: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<CaptureRoot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<ComponentCaptureNode>,
    /// Nearest-first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clip_ancestors: Vec<ClipAncestor>,
    /// The component's `state()` when it is a registered component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<serde_json::Value>,
    /// Scrubbed props (values redacted the way `extractReactState` does).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    /// Capture-side notes about checks the SDK could not perform.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_checks: Vec<String>,
}

impl ComponentCapture {
    /// The root bbox when the capture has usable geometry: present and
    /// finite.
    pub fn root_bbox(&self) -> Option<CaptureRect> {
        self.root
            .and_then(|r| r.bbox)
            .filter(CaptureRect::is_finite)
    }

    /// The root's parent content box, when present and finite.
    pub fn parent_content_box(&self) -> Option<CaptureRect> {
        self.root
            .and_then(|r| r.parent_content_box)
            .filter(CaptureRect::is_finite)
    }
}

/// A [`ComponentCapture`] of an isolated render, stored under `name`
/// (conventionally `<story-id>@<sha>`) with its provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentBaseline {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_id: Option<String>,
    /// The source sha the baseline was captured at. When absent, the
    /// capture's own `sourceSha` stands in (see [`Self::effective_source_sha`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<DateTime<Utc>>,
    pub capture: ComponentCapture,
}

impl ComponentBaseline {
    /// The baseline's source sha: its own, else its capture's.
    pub fn effective_source_sha(&self) -> Option<&str> {
        self.source_sha
            .as_deref()
            .or(self.capture.source_sha.as_deref())
    }
}
