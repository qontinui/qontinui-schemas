//! Media facts for one snapshot element — what an image, picture, SVG, video,
//! canvas, `[role=img]` element or CSS background image is, as the snapshot
//! producer measured it. Carried on [`crate::Element::media`].
//!
//! # Where the facts come from
//!
//! The UI Bridge SDK measures them per media row as ONE `Observation` envelope
//! (`mediaMetadata`: `{status, value, provenance}`, plan
//! 2026-10-13-image-generation-and-visual-judgment Phase 5). This crate never
//! sees the envelope: every converter that projects a UI Bridge payload into an
//! [`crate::ElementSnapshot`] UNWRAPS it —
//!
//! - `status: "measured"` → its `value` supplies the fields below;
//! - a field the envelope lists in `provenance.coverage.unmeasured`, or an
//!   envelope whose status is `unknown` or `absent` → the field is OMITTED;
//! - an older SDK's un-enveloped `mediaMetadata` → the fields that mean the
//!   same thing are projected, the ones that do not (its DPR-blind
//!   `oversizeRatio`, its cache-blind `transferSize`) are omitted.
//!
//! An envelope object passed through unwrapped does not degrade gracefully: it
//! fails serde on the scalar field it lands on, and that rejects the WHOLE
//! snapshot (`an_envelope_passed_through_rejects_the_snapshot` pins this).
//!
//! # Absence
//!
//! As everywhere in [`crate::Element`], an absent field means "the producer
//! did not measure it", never zero, `false` or "none". An analyzer or
//! assertion that needs an absent field answers a typed Unknown. The ONE place
//! a measured "none" is representable is [`MediaAlt::text`], because "this
//! image has no text alternative" is itself the defect an accessibility check
//! must be able to report — see [`ElementMedia::alt`].
//!
//! # Redaction
//!
//! A media element inside a `data-bridge-redact` boundary (the SDK's
//! `state.redaction.content`) has its URL, alt text and slot id replaced by one
//! sentinel string before they ever leave the page. A converter therefore omits
//! [`ElementMedia::src_digest`], [`ElementMedia::alt`] and
//! [`ElementMedia::slot_id`] for such an element: every redacted URL would
//! digest to the same value and read as one duplicated image, and a sentinel
//! is not anybody's alt text.

use serde::{Deserialize, Serialize};

use crate::cache::{hex_encode, sha256_of};

/// What kind of media element a snapshot element is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    /// An `<img>` (and a native image component).
    Img,
    /// A `<picture>`. One row covers the picture and the `<img>` it paints
    /// through: the image facts are the `<img>`'s.
    Picture,
    /// An inline `<svg>`.
    Svg,
    /// A `<video>`.
    Video,
    /// A `<canvas>`.
    Canvas,
    /// A non-media element carrying `role="img"`.
    RoleImg,
    /// An element painting a CSS `background-image: url(…)`. Presentation by
    /// construction: it has no alt channel.
    CssBackground,
}

/// A width × height pair. The units are stated on each field that carries one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MediaSize {
    pub w: f32,
    pub h: f32,
}

impl MediaSize {
    pub const fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }
}

/// The computed CSS `object-fit` — how the image is laid out inside the
/// element's content box. A native image's resize mode is projected onto the
/// keyword that paints the same way (RN `stretch` → `fill`, `center` →
/// `scale-down`); a mode with no equivalent (RN `repeat`, which tiles) is
/// omitted rather than approximated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectFit {
    /// Stretched to the box on each axis independently — can distort.
    Fill,
    /// Scaled uniformly to fit entirely inside the box.
    Contain,
    /// Scaled uniformly to cover the box; the overflow is cropped.
    Cover,
    /// Not scaled at all (CSS `none`).
    None,
    /// `none` or `contain`, whichever is smaller — never scaled up.
    ScaleDown,
}

/// The effective `loading` attribute (an absent attribute is the spec default,
/// `eager`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaLoading {
    Lazy,
    Eager,
}

/// The effective `decoding` attribute (an absent attribute is `auto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaDecoding {
    Sync,
    Async,
    Auto,
}

/// The effective `fetchpriority` attribute (an absent attribute is `auto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFetchPriority {
    High,
    Low,
    Auto,
}

/// Where the element's media is in its load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaLoadState {
    /// Requested and not yet complete.
    Pending,
    /// Complete, with an image (or, for inline SVG / canvas / `role=img`,
    /// painted with the document).
    Loaded,
    /// Complete with no image: a broken source, or a `<picture>` with no
    /// `<img>`.
    Error,
    /// Not yet complete because `loading="lazy"` deferred the request.
    Lazy,
}

/// The element's text alternative, read as ONE measurement: whether it is
/// presentation only, and the text it offers assistive technology.
///
/// The two halves are one record rather than two optional fields because only
/// together can they say "measured: this image has NO text alternative" — the
/// commonest image accessibility defect. With `alt` and `decorative` as
/// independent optionals, an absent `alt` would mean "not measured" (this
/// crate's absence rule), so a missing alt would be indistinguishable from an
/// unread one and a text-alternative check could never fail on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaAlt {
    /// Presentation only: an `alt` that is present and empty after stripping
    /// ASCII whitespace (HTML's definition: tab, LF, FF, CR, space — `alt=""`
    /// and `alt="  "` alike), `role="presentation"`, an `aria-hidden` SVG, or
    /// a CSS background. A decorative image needs no text, so its `text`
    /// carries no defect either way.
    pub decorative: bool,
    /// The text alternative: `alt` for an `<img>` / `<picture>`, the accessible
    /// label (`aria-label`, an SVG `<title>`, a canvas's fallback content) for
    /// the others, a native image's `accessibilityLabel` — stripped of ASCII
    /// whitespace, never empty (an alt that strips to nothing is decorative).
    ///
    /// **`None` is MEASURED here**: the producer read the element and it
    /// carries no text alternative at all. A producer that could not read the
    /// text alternative omits the whole [`ElementMedia::alt`] instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Lowercase hex SHA-256 of a source URL's UTF-8 bytes — the one definition
/// of [`ElementMedia::src_digest`] every producer shares.
///
/// The URL is hashed exactly as emitted, QUERY INCLUDED: every `next/image`
/// source is `/_next/image?url=…&w=…`, so stripping the query would make every
/// optimized image on a page one image.
///
/// A Rust `&str` cannot hold a lone UTF-16 surrogate. A JavaScript producer
/// can (a JSON `"\ud800"`), and every producer encodes one as U+FFFD before
/// hashing, as `TextEncoder` and Node's `Buffer` do.
pub fn src_digest_of(src: &str) -> String {
    hex_encode(&sha256_of(src.as_bytes()))
}

/// Media facts of one snapshot element. Present on [`crate::Element::media`]
/// exactly when the element is a media element; every field except `kind` is
/// optional, and an absent field is UNMEASURED (see the module docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementMedia {
    /// What the element is. The one required field: a producer that cannot
    /// name the kind emits no `media` at all.
    pub kind: MediaKind,

    // ---- Size and resolution ---------------------------------------------
    /// The image RESOURCE's own pixel size — the width × height of the file
    /// the element displays (`videoWidth/Height` for a video, the backing
    /// store for a canvas).
    ///
    /// **Never the browser's density-corrected `naturalWidth`.** For a
    /// `srcset` candidate the browser divides the resource's size by the
    /// candidate's density: a 1280w source picked by a `w` descriptor at
    /// DPR 2 reads `naturalWidth` ≈ 640, and a `2x` candidate reads half its
    /// size. Read raw, every responsive image (every `next/image`) would look
    /// half as sharp as it is. A producer de-corrects it by the matched
    /// candidate's descriptor, and a producer that cannot establish the
    /// resource's own size OMITS this field — and
    /// [`Self::effective_resolution`] with it.
    ///
    /// Also absent for an image that has not loaded (unknown yet), for a
    /// broken image (there is no image), and for kinds with no intrinsic
    /// raster size (inline SVG, CSS backgrounds — CSS exposes none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natural_px: Option<MediaSize>,
    /// The element's DRAWN box: on the web, the `getBoundingClientRect()`
    /// size in CSS px — the border box with CSS transforms included, so a
    /// `transform: scale(2)` image reads twice its layout size; on native,
    /// the view's measured size in dp (`onLayout` / `measureInWindow`). Not
    /// the layout box, and before any device-pixel scaling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered_px: Option<MediaSize>,
    /// Source pixels per DEVICE pixel actually displayed: natural ÷ (displayed
    /// × device pixel ratio), where "displayed" is the image as
    /// [`Self::object_fit`] lays it out in the content box. 1.0 means the image
    /// carries exactly the pixels the screen shows, 2.0 twice as many
    /// (oversized), 0.5 half (upscaled, soft). Under `fill` the axes scale
    /// independently and the SMALLER density is reported.
    ///
    /// Never the DPR-blind natural ÷ rendered ratio (an older SDK's
    /// `oversizeRatio`): on a 2× screen that reads every correctly sized image
    /// as 2× oversized, so a converter omits this field rather than project it.
    /// Omitted, with [`Self::natural_px`], whenever the resource's own pixel
    /// size cannot be established — it is computed from that size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_resolution: Option<f32>,
    /// The computed `object-fit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_fit: Option<ObjectFit>,
    /// The computed `object-position`, verbatim (e.g. `"50% 50%"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_position: Option<String>,
    /// Whether the element's box is sized independently of the image's
    /// intrinsic size — both dimension attributes, a definite CSS width and
    /// height, or a CSS `aspect-ratio` with one definite dimension — so loading
    /// the image cannot shift the layout. Measured for the kinds whose box can
    /// wait on a network load (`img`, `picture`, `video`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dims_reserved: Option<bool>,

    // ---- Loading -----------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading: Option<MediaLoading>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoding: Option<MediaDecoding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetch_priority: Option<MediaFetchPriority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_state: Option<MediaLoadState>,
    /// Image format from the response's content type, else the URL
    /// (`"png"`, `"jpeg"`, `"webp"`, `"svg+xml"`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// The image's weight in bytes: the encoded body size of the resource the
    /// element displays (Resource Timing `encodedBodySize`, keyed on
    /// `currentSrc`), or a `data:` URL's payload size.
    ///
    /// Never `transferSize`, which reads 0 on every cache hit, so a weight
    /// check over it passes vacuously on a warm page. A cross-origin response
    /// without `Timing-Allow-Origin` reports 0 as well, which is "could not
    /// see" — the field is then absent, never `0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_bytes: Option<u64>,

    // ---- Text alternative --------------------------------------------------
    /// The text alternative, read as one measurement (see [`MediaAlt`]).
    /// Absent when the producer did not read it, and for an element inside a
    /// redaction boundary (its text is withheld).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<MediaAlt>,

    // ---- Paint timing ------------------------------------------------------
    /// Whether this element is the page's current Largest-Contentful-Paint
    /// element (a `<picture>` counts the `<img>` it paints through).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_lcp_candidate: Option<bool>,
    /// The summed score (`value`) of the layout shifts that moved this element,
    /// excluding shifts within 500 ms of user input, which CLS excludes. A
    /// shift that moved several elements counts its whole score toward each,
    /// so this is the element's exposure, not an exclusive share.
    /// `Some(0.0)` is measured: shifts were observable and none moved it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cls_contribution: Option<f32>,

    // ---- Image slots -------------------------------------------------------
    /// The image slot this element fills (`data-ui-bridge-image-slot`).
    /// Absent when the element carries no slot marker or the producer read
    /// none; no consumer needs to tell those apart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot_id: Option<String>,
    /// Whether the slot still renders its placeholder
    /// (`data-ui-bridge-placeholder="true"`). `Some(false)` is measured: the
    /// producer read the markers and the element carries none. It is carried
    /// here because the Phase 5 SDK already emits it and Phase 14's
    /// placeholder inventory needs it (plan
    /// 2026-10-13-image-generation-and-visual-judgment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<bool>,

    // ---- Identity and position ---------------------------------------------
    /// [`src_digest_of`] the URL the element displays (`currentSrc`, falling
    /// back to `src`). A digest rather than the URL so duplicates are
    /// detectable without a snapshot carrying a possibly signed URL. Absent
    /// for an element with no source (inline SVG, canvas) and for an element
    /// inside a redaction boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_digest: Option<String>,
    /// Whether the element's box intersects the viewport at capture time — the
    /// SDK's `state.inViewport`, independent of `visible`.
    /// [`crate::ElementSnapshot`] carries no viewport, so an off-screen
    /// question (is a below-the-fold image lazy?) cannot be decided without
    /// it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_viewport: Option<bool>,
}

impl ElementMedia {
    /// A media record that measured nothing but its kind. Fixtures and
    /// projections spread it: `ElementMedia { natural_px: …, ..ElementMedia::new(MediaKind::Img) }`.
    pub const fn new(kind: MediaKind) -> Self {
        Self {
            kind,
            natural_px: None,
            rendered_px: None,
            effective_resolution: None,
            object_fit: None,
            object_position: None,
            dims_reserved: None,
            loading: None,
            decoding: None,
            fetch_priority: None,
            load_state: None,
            format: None,
            transfer_bytes: None,
            alt: None,
            is_lcp_candidate: None,
            cls_contribution: None,
            slot_id: None,
            placeholder: None,
            src_digest: None,
            in_viewport: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::{Element, ElementSnapshot};

    /// THE shared `src_digest` vector. The same URL and digest are pinned in
    /// all three converter suites (qontinui-claude-config
    /// `scripts/test-uibridge-to-elementsnapshot.py`, qontinui-web
    /// `tests/e2e/style-gate/normalize.test.ts`, ui-bridge-native
    /// `src/core/vision-fields.test.ts`), so a producer that hashes anything
    /// else — the URL without its query, a normalized URL, UTF-16 — disagrees
    /// with this crate in a test, not in a duplicate-image verdict.
    const SHARED_SRC: &str =
        "http://localhost:3001/_next/image?url=%2Fimages%2Fhero.png&w=1920&q=75";
    const SHARED_DIGEST: &str = "a846bcf412cf708ec0a4ac6a972307cbcb250b75509765fd44c49caecbd01f94";

    /// The second shared vector: a JavaScript producer's lone surrogate
    /// (`.../img/\ud800.png`) is encoded as U+FFFD before hashing, so every
    /// suite digests it as this `&str`, which spells the U+FFFD form.
    const SHARED_SURROGATE_SRC: &str = "http://localhost:3001/img/\u{FFFD}.png";
    const SHARED_SURROGATE_DIGEST: &str =
        "6f45e1fb51856d9f4f25eea51558783cc2c060af2ba43ea46c8bdab5dbdf23a6";

    #[test]
    fn src_digest_matches_the_shared_vector() {
        assert_eq!(src_digest_of(SHARED_SRC), SHARED_DIGEST);
    }

    #[test]
    fn src_digest_matches_the_shared_surrogate_vector() {
        assert_eq!(src_digest_of(SHARED_SURROGATE_SRC), SHARED_SURROGATE_DIGEST);
    }

    #[test]
    fn src_digest_keeps_the_query() {
        // Stripped of its query the URL is every other `next/image` on the
        // page; the digest must differ, or `no_duplicate_images` reads them as
        // one image.
        let stripped = SHARED_SRC.split('?').next().unwrap();
        assert_ne!(src_digest_of(stripped), SHARED_DIGEST);
        assert_ne!(
            src_digest_of(&SHARED_SRC.replace("hero.png", "logo.png")),
            SHARED_DIGEST
        );
    }

    #[test]
    fn src_digest_is_lowercase_hex() {
        let d = src_digest_of(SHARED_SRC);
        assert_eq!(d.len(), 64);
        assert!(d
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }

    /// Every field populated, in the wire spelling a converter emits.
    fn full_media_json() -> serde_json::Value {
        serde_json::json!({
            "kind": "picture",
            "natural_px": {"w": 1600.0, "h": 900.0},
            "rendered_px": {"w": 400.0, "h": 225.0},
            "effective_resolution": 2.0,
            "object_fit": "scale-down",
            "object_position": "50% 50%",
            "dims_reserved": true,
            "loading": "lazy",
            "decoding": "async",
            "fetch_priority": "high",
            "load_state": "loaded",
            "format": "webp",
            "transfer_bytes": 48213,
            "alt": {"decorative": false, "text": "A hero image"},
            "is_lcp_candidate": true,
            "cls_contribution": 0.125,
            "slot_id": "hero",
            "placeholder": false,
            "src_digest": SHARED_DIGEST,
            "in_viewport": false
        })
    }

    #[test]
    fn every_field_parses_from_the_converter_wire_shape() {
        let m: ElementMedia =
            serde_json::from_value(full_media_json()).expect("full record parses");
        assert_eq!(m.kind, MediaKind::Picture);
        assert_eq!(m.natural_px, Some(MediaSize::new(1600.0, 900.0)));
        assert_eq!(m.rendered_px, Some(MediaSize::new(400.0, 225.0)));
        assert_eq!(m.effective_resolution, Some(2.0));
        assert_eq!(m.object_fit, Some(ObjectFit::ScaleDown));
        assert_eq!(m.object_position.as_deref(), Some("50% 50%"));
        assert_eq!(m.dims_reserved, Some(true));
        assert_eq!(m.loading, Some(MediaLoading::Lazy));
        assert_eq!(m.decoding, Some(MediaDecoding::Async));
        assert_eq!(m.fetch_priority, Some(MediaFetchPriority::High));
        assert_eq!(m.load_state, Some(MediaLoadState::Loaded));
        assert_eq!(m.format.as_deref(), Some("webp"));
        assert_eq!(m.transfer_bytes, Some(48213));
        assert_eq!(
            m.alt,
            Some(MediaAlt {
                decorative: false,
                text: Some("A hero image".into())
            })
        );
        assert_eq!(m.is_lcp_candidate, Some(true));
        assert_eq!(m.cls_contribution, Some(0.125));
        assert_eq!(m.slot_id.as_deref(), Some("hero"));
        assert_eq!(m.placeholder, Some(false));
        assert_eq!(m.src_digest.as_deref(), Some(SHARED_DIGEST));
        assert_eq!(m.in_viewport, Some(false));

        // ...and serializes back to the same wire shape, key for key.
        assert_eq!(serde_json::to_value(&m).unwrap(), full_media_json());
    }

    #[test]
    fn kind_and_keyword_spellings_are_the_wire_contract() {
        let kinds = [
            (MediaKind::Img, "img"),
            (MediaKind::Picture, "picture"),
            (MediaKind::Svg, "svg"),
            (MediaKind::Video, "video"),
            (MediaKind::Canvas, "canvas"),
            (MediaKind::RoleImg, "role_img"),
            (MediaKind::CssBackground, "css_background"),
        ];
        for (kind, wire) in kinds {
            assert_eq!(serde_json::to_value(kind).unwrap(), wire);
        }
        // object-fit keeps the CSS keywords, hyphen included.
        let fits = [
            (ObjectFit::Fill, "fill"),
            (ObjectFit::Contain, "contain"),
            (ObjectFit::Cover, "cover"),
            (ObjectFit::None, "none"),
            (ObjectFit::ScaleDown, "scale-down"),
        ];
        for (fit, wire) in fits {
            assert_eq!(serde_json::to_value(fit).unwrap(), wire);
        }
        let states = [
            (MediaLoadState::Pending, "pending"),
            (MediaLoadState::Loaded, "loaded"),
            (MediaLoadState::Error, "error"),
            (MediaLoadState::Lazy, "lazy"),
        ];
        for (state, wire) in states {
            assert_eq!(serde_json::to_value(state).unwrap(), wire);
        }
    }

    #[test]
    fn a_kind_only_record_omits_every_unmeasured_field() {
        let m = ElementMedia::new(MediaKind::Svg);
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({"kind": "svg"})
        );
        let back: ElementMedia = serde_json::from_str(r#"{"kind":"svg"}"#).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn a_record_without_a_kind_is_refused() {
        assert!(serde_json::from_str::<ElementMedia>(r#"{"natural_px":{"w":1,"h":1}}"#).is_err());
    }

    #[test]
    fn a_missing_text_alternative_is_measured_not_absent() {
        // The read happened and found nothing: `alt` present, `text` absent.
        let missing: ElementMedia =
            serde_json::from_str(r#"{"kind":"img","alt":{"decorative":false}}"#).unwrap();
        assert_eq!(
            missing.alt,
            Some(MediaAlt {
                decorative: false,
                text: None
            })
        );
        // The read did not happen: no `alt` at all. A different statement.
        let unread: ElementMedia = serde_json::from_str(r#"{"kind":"img"}"#).unwrap();
        assert_eq!(unread.alt, None);
        assert_ne!(missing.alt, unread.alt);
        // `decorative` is required inside the record: a producer cannot
        // claim to have read the text alternative without deciding it.
        assert!(
            serde_json::from_str::<ElementMedia>(r#"{"kind":"img","alt":{"text":"x"}}"#).is_err()
        );
    }

    #[test]
    fn an_envelope_passed_through_rejects_the_snapshot() {
        // Why every converter must unwrap the SDK's Observation envelope: an
        // envelope object on a scalar field fails serde, and serde takes the
        // whole snapshot down with it — not just this element.
        let json = serde_json::json!({
            "elements": [
                {"id": "ok", "bbox": {"x": 0, "y": 0, "w": 10, "h": 10}},
                {"id": "img", "media": {
                    "kind": "img",
                    "effective_resolution": {"status": "measured", "value": 2.0}
                }}
            ]
        });
        assert!(serde_json::from_value::<ElementSnapshot>(json).is_err());
    }

    #[test]
    fn unknown_media_keys_are_ignored_like_element_keys() {
        // `Element` has no `deny_unknown_fields`, and neither does its media:
        // a newer producer's extra fact must not reject an older consumer's
        // whole snapshot.
        let m: ElementMedia = serde_json::from_str(
            r#"{"kind":"video","aspect_ratio":"16 / 9","load_state":"pending"}"#,
        )
        .unwrap();
        assert_eq!(m.kind, MediaKind::Video);
        assert_eq!(m.load_state, Some(MediaLoadState::Pending));
    }

    #[test]
    fn element_media_rides_the_element_and_is_omitted_when_absent() {
        let with: Element = serde_json::from_value(serde_json::json!({
            "id": "hero",
            "bbox": {"x": 0, "y": 0, "w": 400, "h": 225},
            "media": full_media_json()
        }))
        .unwrap();
        let media = with.media.as_ref().expect("media rides the element");
        assert_eq!(media.kind, MediaKind::Picture);

        // A non-media element, and every snapshot persisted before the field
        // existed: no `media` key in, none out.
        let without: Element = serde_json::from_str(r#"{"id":"btn","interactable":true}"#).unwrap();
        assert!(without.media.is_none());
        let wire = serde_json::to_string(&without).unwrap();
        assert!(
            !wire.contains("media"),
            "absent media must be omitted: {wire}"
        );
    }
}
