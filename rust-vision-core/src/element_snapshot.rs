//! Source-agnostic structural snapshot of a UI page.
//!
//! [`ElementSnapshot`] is the input the analyzers + assertion DSL consume.
//! It's deliberately decoupled from any specific snapshot source (the
//! runner's `discover`, native a11y trees, mobile UIBridge, etc) — every
//! one of those callers projects into this shape before invoking
//! `vision/analyze` or `vision/assert`.
//!
//! Pixel-space units throughout. Geometry uses [`Region`] (from `frame.rs`);
//! visual properties (color, font size, etc) are optional because not every
//! snapshot source can populate them — analyzers/assertions that depend on
//! a missing field surface a soft "skipped: missing X" finding instead of
//! erroring.

use serde::{Deserialize, Serialize};

use crate::frame::Region;

/// Snapshot of one rendered page worth of elements, ready to feed
/// [`crate::analyzers`] or [`crate::assertions`]. Caller is responsible
/// for capturing this in lockstep with the [`crate::Frame`] that will be
/// used in the same `analyze`/`assert` call — analyzers do not re-fetch
/// from the runner, they trust the snapshot supplied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementSnapshot {
    pub elements: Vec<Element>,
    /// Identity of the capture these elements came from, as minted by the
    /// PRODUCER. Current grammar — five segments, prefix `ubs2`:
    ///
    /// ```text
    /// ubs2_{count36}_{mountEvidence36}_{content16hex}_{generation16hex}
    /// ```
    ///
    /// `count` is `elements.length` and `mountEvidence` is how many of those
    /// elements actually carried a `registeredAt`, both in base 36. The two
    /// hex halves are independent FNV-1a-64 folds over the elements in array
    /// order, unchanged from signature spec v1 — the grammar bumped from
    /// `ubs1`'s four segments, the fold did not, so the same capture folds to
    /// the same two values under either prefix.
    ///
    /// `content` folds each element's `id`, `category`, `state.textContent`
    /// and `state.ariaPressed`. `generation` folds each element's **`id` and
    /// its `registeredAt`** — the `id` feeds BOTH folds. Anyone reading only
    /// this comment to build another producer must take the `id` in
    /// `generation` with it: a `generation` folded over timestamps alone
    /// mints a different token for the same capture, which is exactly the
    /// silent drift the one normative spec exists to prevent.
    ///
    /// `mountEvidence` exists so the generation half can be read honestly.
    /// At `mountEvidence: 0` the generation fold saw no timestamp at all, so
    /// two ids agreeing on it is an **absence of observation**, not an
    /// observation of "no remount" — a serializer that omits `registeredAt`
    /// contributes no bytes rather than spurious ones, and without this
    /// segment that silence is indistinguishable from a stable mount.
    ///
    /// No producer has shipped yet. The runner's module-private
    /// `SnapshotSignature` is the building block the spec promotes, and the
    /// UI Bridge SDK mints the id on a paired branch; until one of those
    /// lands, every real capture reaching this crate is unattributed — a
    /// first-class state here (see below), not a defect.
    ///
    /// This crate is a CARRIER of the id and never a producer: it does not
    /// recompute the fold, and by the same argument does not reimplement the
    /// grammar either — a third parser is the same drift risk as a third
    /// producer. The value is an opaque token here, neither parsed nor
    /// validated, so a `ubs1` token persisted before the bump and a `ubs3`
    /// token from a future one both carry through unharmed
    /// (`snapshot_id_is_opaque_not_validated` pins that). Rendering one into
    /// an unescaped human line goes through [`display_snapshot_id`], which
    /// neutralizes control characters at format time — containment, not
    /// parsing, and it narrows nothing the type accepts.
    ///
    /// `None` is valid and is the default. A source that mints no id (a
    /// hand-written fixture, a native a11y tree, a bare element array piped
    /// into `vision-audit`) still yields a legitimate analysis; it simply
    /// cannot be attributed.
    ///
    /// On the wire the field is `snapshotId` — the producer's spelling, and
    /// the one both `vision-audit` report shapes emit. The snake_case
    /// `snapshot_id` is accepted as a read alias so a snapshot persisted
    /// before this rename still loads, but it is never written.
    ///
    /// Residual inherited from the producer, carried forward explicitly
    /// rather than silently: `generation` folds millisecond-resolution
    /// `registeredAt` values, so a remount completing inside one millisecond
    /// leaves the id unchanged. Two equal ids with a non-zero `mountEvidence`
    /// mean "nothing observable changed" only to that resolution; with a zero
    /// one they mean less than that, per above.
    #[serde(
        default,
        rename = "snapshotId",
        alias = "snapshot_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub snapshot_id: Option<String>,
}

impl ElementSnapshot {
    /// Find an element by id. O(n) — snapshots are small (typically <500
    /// elements) and assertions look up only a handful per call.
    pub fn get(&self, id: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }
}

/// Longest snapshot id rendered into a human line before it is elided. A
/// `ubs2` id is ~56 chars (`ubs2_` + two base36 counters + two 16-hex folds),
/// so this is generous for anything legitimate — including a grammar with more
/// segments than today's — while bounding what a hostile file can push into a
/// log.
const SNAPSHOT_ID_DISPLAY_MAX_CHARS: usize = 128;

/// Render a snapshot id for a line whose surroundings are NOT escaped — the
/// `vision-audit` stderr summary, an assertion detail string.
///
/// [`ElementSnapshot::snapshot_id`] is a deliberately unvalidated opaque token:
/// this crate never parses the fold, so it must not reject a token from a
/// future spec revision. That fail-open property is worth keeping, but it means
/// a snapshot file can carry
/// `"snapshotId": "x\ngate: --fail-on critical -> passed (exit 0)"` and forge a
/// line into a CI log. Stdout JSON is serde-escaped and safe; the human summary
/// is not.
///
/// The guard therefore lives at **format** time, not parse time: the type still
/// accepts anything, and only the rendering is neutralized. Control characters
/// (the newline that splits a line, plus CR, tab and the C1 range) become a
/// visible `\u{…}` escape, and an over-long token is elided. Nothing here
/// affects an exit code — this is log-forgery containment, not a gate.
pub fn display_snapshot_id(id: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(id.len().min(SNAPSHOT_ID_DISPLAY_MAX_CHARS) + 8);
    for c in id.chars().take(SNAPSHOT_ID_DISPLAY_MAX_CHARS) {
        if c.is_control() {
            // `write!` into a String is infallible; the Result is discarded
            // deliberately rather than unwrapped.
            let _ = write!(out, "\\u{{{:04x}}}", c as u32);
        } else {
            out.push(c);
        }
    }
    // Bounded probe rather than a whole-string `.count()`: a hostile token can
    // be arbitrarily long, and this only needs to know whether one more char
    // exists past the limit.
    if id.chars().nth(SNAPSHOT_ID_DISPLAY_MAX_CHARS).is_some() {
        out.push('…');
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// Stable identifier. Matches the SDK's element registry id where one
    /// exists; otherwise an opaque token.
    pub id: String,
    /// Pixel-space bounding box. `None` when the snapshot source could not
    /// measure geometry (e.g. a mobile `discover` snapshot that includes
    /// hidden/unmeasured elements). Geometry-based analyzers/assertions
    /// SKIP bbox-less elements (they surface no spurious findings); the
    /// element is still counted for non-geometry checks (interactivity,
    /// text presence, typography). Web snapshots always populate this, so
    /// `Option` is a strict superset — existing payloads round-trip
    /// byte-identically (`skip_serializing_if` omits `None`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Region>,
    /// Visible text content (innerText / accessibilityLabel equivalent).
    /// `None` when the element has no text or the snapshot source didn't
    /// populate it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// ARIA role / native-platform equivalent. e.g., `"button"`, `"link"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// True if the element accepts pointer/key input (button, link, input,
    /// etc). Drives the "interactive coverage" + WCAG-target-size analyses.
    #[serde(default)]
    pub interactable: bool,
    /// Foreground (text / icon) color as RGB. None when the snapshot
    /// source can't determine computed style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg_color: Option<Rgb>,
    /// Effective background color the element renders against — usually
    /// the resolved-up-the-tree opaque ancestor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<Rgb>,
    /// Computed font size in CSS pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size_px: Option<f32>,
    /// Font family declaration as the SDK saw it. May be a comma-list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Computed line-height in CSS pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height_px: Option<f32>,
    /// Parent id, when known. Lets `no_clipping` walk up to the clip
    /// ancestor without re-running the layout engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Children ids, when known.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_ids: Vec<String>,
    /// Resolved index into the document's painting sequence. HIGHER paints
    /// LATER, i.e. on top. This is **not** a raw `z-index`: raw z-index is
    /// only comparable between siblings within one stacking context, so
    /// comparing it across contexts is actively wrong rather than merely
    /// imprecise. The motivating case: a `z-50` dropdown nested inside a
    /// title bar that is itself a stacking context at `z-index:10` paints
    /// BELOW a `z-20` panel outside it, because the whole title-bar subtree
    /// paints as one atom at rank 10 — raw z-index reports `50 > 20` and
    /// gets the answer backwards.
    ///
    /// `None` when the snapshot source cannot resolve stacking (native a11y
    /// trees, mobile `discover`). Assertions that need it report an honest
    /// "cannot answer" rather than guessing — see
    /// [`crate::assertions::Assertion::ElementAbove`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paint_order: Option<u32>,
    /// Id of the nearest ancestor that establishes a stacking context
    /// (`position` + `z-index`, `transform`, `filter`, `backdrop-filter`,
    /// `opacity < 1`, `will-change`, …), when the source can resolve it.
    ///
    /// Carried for FAILURE DETAIL only — [`paint_order`](Self::paint_order)
    /// alone decides an ordering verdict. Naming the context is what turns
    /// an absurd-looking "z-50 lost to z-20" into a diagnosis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stacking_context_id: Option<String>,
}

/// Linear-light RGB color (no alpha). Used for both `fg_color` and
/// `bg_color`; assertions/analyzers convert to the appropriate space
/// (Lab/sRGB) on demand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// W3C relative luminance, sRGB. Used for WCAG contrast.
    pub fn relative_luminance(self) -> f64 {
        fn channel(c: u8) -> f64 {
            let s = c as f64 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }
}

/// Whether a value lies in a bounded region. Distinct from
/// [`Region::fits_in`] which checks against frame dimensions — this is
/// "does element B fully nest inside element A".
pub fn region_contains(outer: Region, inner: Region) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

/// Whether two regions intersect at all (any pixel overlap).
pub fn regions_overlap(a: Region, b: Region) -> bool {
    (a.x as i64) < b.right()
        && (b.x as i64) < a.right()
        && (a.y as i64) < b.bottom()
        && (b.y as i64) < a.bottom()
}

/// Compute the intersection of two regions, if any.
pub fn intersection(a: Region, b: Region) -> Option<Region> {
    let x = a.x.max(b.x) as i64;
    let y = a.y.max(b.y) as i64;
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());
    if x < right && y < bottom {
        Some(Region {
            // `x`/`y` are the max of two `i32`s, so they fit back in `i32`;
            // `right - x` / `bottom - y` are positive and bounded by the
            // smaller of the two extents, so they fit in `u32`.
            x: x as i32,
            y: y as i32,
            w: (right - x) as u32,
            h: (bottom - y) as u32,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: i32, y: i32, w: u32, h: u32) -> Region {
        Region { x, y, w, h }
    }

    #[test]
    fn overlap_detected() {
        assert!(regions_overlap(region(0, 0, 10, 10), region(5, 5, 10, 10)));
    }

    #[test]
    fn touching_edges_not_overlap() {
        assert!(!regions_overlap(
            region(0, 0, 10, 10),
            region(10, 0, 10, 10)
        ));
    }

    #[test]
    fn nested_overlap() {
        assert!(regions_overlap(
            region(0, 0, 100, 100),
            region(40, 40, 10, 10)
        ));
    }

    #[test]
    fn region_contains_full_nest() {
        assert!(region_contains(
            region(0, 0, 100, 100),
            region(10, 10, 20, 20)
        ));
    }

    #[test]
    fn region_does_not_contain_partial() {
        assert!(!region_contains(
            region(0, 0, 50, 50),
            region(40, 40, 20, 20)
        ));
    }

    #[test]
    fn intersection_computed() {
        let i = intersection(region(0, 0, 10, 10), region(5, 5, 10, 10)).unwrap();
        assert_eq!(i, region(5, 5, 5, 5));
    }

    #[test]
    fn intersection_disjoint() {
        assert!(intersection(region(0, 0, 5, 5), region(10, 10, 5, 5)).is_none());
    }

    // ---------------------------------------------------------------
    // Signed origin. `getBoundingClientRect()` returns negative x/y for any
    // element scrolled or positioned off the top/left of the viewport. Before
    // the origin was widened to i32, producers had to clamp those to 0, which
    // reported every off-screen element as flush against the viewport edge.
    // These tests pin the TRUE coordinate surviving the wire round-trip and
    // the geometry helpers behaving correctly across the origin.
    // ---------------------------------------------------------------

    #[test]
    fn negative_origin_survives_json_round_trip() {
        // Exactly what a `left: -9999px` a11y-hidden node measures as.
        let json = r#"{"id":"offscreen","bbox":{"x":-9999,"y":-48,"w":120,"h":32}}"#;
        let el: Element = serde_json::from_str(json).expect("negative origin must deserialize");
        let bbox = el.bbox.expect("bbox present");
        assert_eq!(
            bbox.x, -9999,
            "true negative x must survive, not clamp to 0"
        );
        assert_eq!(bbox.y, -48, "true negative y must survive, not clamp to 0");
        assert_eq!(bbox.w, 120);
        assert_eq!(bbox.h, 32);

        // ...and re-serializes verbatim, so a snapshot can be persisted and
        // re-read without drifting toward the origin.
        let re = serde_json::to_string(&bbox).unwrap();
        let back: Region = serde_json::from_str(&re).unwrap();
        assert_eq!(back, bbox);
    }

    #[test]
    fn negative_extent_is_unrepresentable() {
        // `w`/`h` stay unsigned on purpose: a negative extent is meaningless,
        // so the type rejects it rather than a consumer having to defend.
        let json = r#"{"x":0,"y":0,"w":-5,"h":10}"#;
        assert!(serde_json::from_str::<Region>(json).is_err());
    }

    #[test]
    fn geometry_helpers_work_across_the_origin() {
        // A sticky header scrolled half off the top: y = -20, h = 40.
        let partly_offscreen = region(-20, -20, 40, 40);
        let viewport_corner = region(0, 0, 100, 100);
        assert!(regions_overlap(partly_offscreen, viewport_corner));
        let i = intersection(partly_offscreen, viewport_corner).unwrap();
        assert_eq!(i, region(0, 0, 20, 20));

        // Wholly off-screen: no overlap with the viewport at all. Under the
        // old clamped encoding this collapsed to (0,0) and produced a
        // spurious overlap against every top-left element.
        let wholly_offscreen = region(-9999, -9999, 120, 32);
        assert!(!regions_overlap(wholly_offscreen, viewport_corner));
        assert!(intersection(wholly_offscreen, viewport_corner).is_none());

        // Containment across the origin: an off-canvas drawer contains its
        // own child even though both sit at negative coordinates.
        assert!(region_contains(
            region(-200, -100, 300, 200),
            region(-150, -50, 50, 50)
        ));
        assert!(!region_contains(
            region(-200, -100, 300, 200),
            region(-250, -50, 50, 50)
        ));
    }

    #[test]
    fn fits_in_rejects_negative_origin() {
        assert!(region(0, 0, 10, 10).fits_in(100, 100));
        assert!(!region(-1, 0, 10, 10).fits_in(100, 100));
        assert!(!region(0, -1, 10, 10).fits_in(100, 100));
    }

    #[test]
    fn clamp_to_frame_narrows_signed_origin_to_buffer_indices() {
        // Straddling the origin -> the in-frame part only.
        assert_eq!(
            region(-20, -10, 40, 40).clamp_to_frame(100, 100),
            Some((0, 0, 20, 30))
        );
        // Wholly outside -> nothing to sample.
        assert_eq!(region(-9999, -9999, 120, 32).clamp_to_frame(100, 100), None);
        assert_eq!(region(200, 0, 10, 10).clamp_to_frame(100, 100), None);
        // Fully inside -> unchanged.
        assert_eq!(
            region(10, 10, 5, 5).clamp_to_frame(100, 100),
            Some((10, 10, 5, 5))
        );
    }

    // ---------------------------------------------------------------
    // Snapshot identity. The id is minted by the PRODUCER (SDK/runner) to
    // the normative `ubs2_…` signature grammar; this crate only CARRIES it,
    // so what is pinned here is that it survives the wire in both directions,
    // that its ABSENCE stays a first-class valid state, and that no grammar
    // — past, present or future — is validated on the way through.
    // ---------------------------------------------------------------

    #[test]
    fn snapshot_id_round_trips_through_serde() {
        let id = "ubs2_1_1_9f1c0a4b7e3d2610_00000191a4c3f2d8";
        let snap = ElementSnapshot {
            elements: vec![Element {
                id: "btn".to_string(),
                bbox: Some(region(0, 0, 40, 20)),
                text: None,
                role: None,
                interactable: true,
                fg_color: None,
                bg_color: None,
                font_size_px: None,
                font_family: None,
                line_height_px: None,
                parent_id: None,
                children_ids: vec![],
                paint_order: None,
                stacking_context_id: None,
            }],
            snapshot_id: Some(id.to_string()),
        };

        // The wire spelling is the PRODUCER's `snapshotId` — the same key the
        // `vision-audit` report shapes emit. Nothing in this stack renames on
        // the way out, so a JS caller reading `resp.snapshot.snapshotId` gets
        // the id rather than `undefined`.
        let wire = serde_json::to_string(&snap).unwrap();
        assert!(
            wire.contains(r#""snapshotId":"ubs2_1_1_9f1c0a4b7e3d2610_00000191a4c3f2d8""#),
            "id must reach the wire verbatim under the camelCase key, got {wire}"
        );
        assert!(
            !wire.contains("snapshot_id"),
            "snake_case is a read alias only, never written: {wire}"
        );

        let back: ElementSnapshot = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.snapshot_id.as_deref(), Some(id));
        assert_eq!(back.elements.len(), 1);
    }

    #[test]
    fn snapshot_id_is_opaque_not_validated() {
        // This crate is a CARRIER of the id, never a producer — and by the
        // same argument never a parser: a third implementation of the grammar
        // is the same drift risk as a third implementation of the fold. So
        // this is demonstrated across grammars rather than asserted in prose.
        for token in [
            // Today's grammar.
            "ubs2_2_2_9f1c0a4b7e3d2610_00000191a4c3f2d8",
            // The PREVIOUS one — four segments, no `mountEvidence`. A
            // snapshot persisted before the bump still analyzes; nothing here
            // rejects it, migrates it, or reads a missing segment as zero.
            "ubs1_2_9f1c0a4b7e3d2610_00000191a4c3f2d8",
            // A grammar that does not exist yet.
            "ubs3_whatever-comes-next",
            // Not a signature at all. Still carried, still unattributable to
            // anything this crate could check.
            "",
        ] {
            let json = serde_json::json!({ "elements": [], "snapshotId": token }).to_string();
            let snap: ElementSnapshot =
                serde_json::from_str(&json).expect("no grammar is rejected");
            assert_eq!(snap.snapshot_id.as_deref(), Some(token));

            // ...and it re-serializes byte-identically, so carrying a token
            // through this crate is lossless in both directions.
            let wire = serde_json::to_string(&snap).unwrap();
            let back: ElementSnapshot = serde_json::from_str(&wire).unwrap();
            assert_eq!(back.snapshot_id.as_deref(), Some(token));
        }
    }

    #[test]
    fn both_wire_spellings_deserialize() {
        // The canonical spelling, and the one this type emits.
        //
        // The fixture is a REAL id for the payload it sits on: FNV-1a-64 over
        // zero elements finishes at the offset basis `cbf29ce484222325`, and
        // both `count` and `mountEvidence` are 0, rendering as base36 `0`. An
        // all-zero fold is unproducible, so using one here would teach the
        // wrong shape even though the value is only ever an opaque token.
        let empty = "ubs2_0_0_cbf29ce484222325_cbf29ce484222325";
        let json = format!(r#"{{"elements":[],"snapshotId":"{empty}"}}"#);
        let snap: ElementSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap.snapshot_id.as_deref(), Some(empty));

        // ...and the snake_case alias, so a snapshot persisted by a build
        // from before the rename still loads.
        let legacy = format!(r#"{{"elements":[],"snapshot_id":"{empty}"}}"#);
        let snap: ElementSnapshot = serde_json::from_str(&legacy).unwrap();
        assert_eq!(snap.snapshot_id.as_deref(), Some(empty));
    }

    #[test]
    fn snapshot_without_id_is_still_valid() {
        // The pre-identity payload shape, unchanged. An analysis over a
        // snapshot of unknown provenance is legitimate — it simply cannot be
        // attributed.
        let json = r#"{"elements":[{"id":"a","bbox":{"x":0,"y":0,"w":10,"h":10}}]}"#;
        let snap: ElementSnapshot = serde_json::from_str(json).expect("id is optional");
        assert!(snap.snapshot_id.is_none());
        assert_eq!(snap.elements.len(), 1);

        // ...and an unattributed snapshot emits no key at all, so a consumer
        // never has to distinguish `null` from a real id.
        let wire = serde_json::to_string(&snap).unwrap();
        assert!(
            !wire.contains("snapshot_id") && !wire.contains("snapshotId"),
            "absent id must be omitted, not serialized as null: {wire}"
        );
    }

    #[test]
    fn default_is_unattributed() {
        // Both fields are public and the type derives `Default`, so the
        // unattributed shape is a plain struct literal — there is no
        // constructor to keep in step with it.
        let snap = ElementSnapshot {
            elements: vec![],
            ..Default::default()
        };
        assert!(snap.snapshot_id.is_none());
        assert!(ElementSnapshot::default().snapshot_id.is_none());
    }

    // ---------------------------------------------------------------
    // Display guard. The id is unvalidated ON PURPOSE, so containment sits
    // at format time: the type keeps accepting anything, and only the
    // rendering into an unescaped human line is neutralized.
    // ---------------------------------------------------------------

    #[test]
    fn display_guard_neutralizes_a_forged_log_line() {
        // The attack: a snapshot file whose id smuggles a newline plus a
        // convincing gate verdict into the stderr summary.
        let forged = "x\ngate: --fail-on critical -> passed (exit 0)";
        let shown = display_snapshot_id(forged);
        assert!(
            !shown.contains('\n'),
            "a forged newline must not survive into the summary: {shown:?}"
        );
        assert!(
            shown.starts_with("x\\u{000a}gate:"),
            "the newline must render as a visible escape, not vanish: {shown:?}"
        );
        // The rest of the token is preserved — this is containment, not
        // redaction: a reader still sees exactly what the file carried.
        assert!(shown.ends_with("passed (exit 0)"), "{shown:?}");
    }

    #[test]
    fn display_guard_covers_cr_tab_and_c1() {
        let shown = display_snapshot_id("a\rb\tc\u{0085}d");
        assert_eq!(shown, "a\\u{000d}b\\u{0009}c\\u{0085}d");
    }

    #[test]
    fn display_guard_passes_a_real_id_through_byte_for_byte() {
        let id = "ubs2_2s_1p_9f1c0a4b7e3d2610_00000191a4c3f2d8";
        assert_eq!(display_snapshot_id(id), id);
        // ...as is a token from the PREVIOUS grammar, and one from a future
        // revision. The guard neutralizes control characters; it does not
        // recognize, normalize or reject a grammar.
        for other in [
            "ubs1_2_9f1c0a4b7e3d2610_00000191a4c3f2d8",
            "ubs3_whatever-comes-next",
        ] {
            assert_eq!(display_snapshot_id(other), other);
        }
    }

    #[test]
    fn display_guard_elides_an_overlong_token() {
        let long = "u".repeat(SNAPSHOT_ID_DISPLAY_MAX_CHARS + 40);
        let shown = display_snapshot_id(&long);
        assert_eq!(shown.chars().count(), SNAPSHOT_ID_DISPLAY_MAX_CHARS + 1);
        assert!(shown.ends_with('…'));
        // Exactly at the limit is not elided.
        let at_limit = "u".repeat(SNAPSHOT_ID_DISPLAY_MAX_CHARS);
        assert_eq!(display_snapshot_id(&at_limit), at_limit);
    }

    #[test]
    fn luminance_black_zero() {
        assert!((Rgb::new(0, 0, 0).relative_luminance() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn luminance_white_one() {
        assert!((Rgb::new(255, 255, 255).relative_luminance() - 1.0).abs() < 1e-6);
    }
}
