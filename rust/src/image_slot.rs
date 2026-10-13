//! Image-slot specs: the declared contract of one image placement.
//!
//! Introduced by plan `2026-10-13-image-generation-and-visual-judgment`
//! (Phase 14, design decision PD3). An image placement in a product UI is a
//! *slot*. Its spec says what the image is for — its job, its audience, the
//! contexts it renders in, its geometry, the style it follows, what it must
//! and must not depict, its text-in-image and alt-text policy, and where the
//! thresholds of its deterministic checks come from. That turns "is this image
//! good?" into "does this image do the job its slot declares?", which every
//! later stage answers against this one record:
//!
//! - the placeholder a UI renders before an asset exists (qontinui-web's
//!   `<SlotImage>` reserves [`ImageSlotAspect::preferred`] and marks the element
//!   `data-ui-bridge-image-slot="<slot>"`);
//! - the asset compile step (qontinui-studio), which generates or selects an
//!   asset per slot and records the binding in [`ImageSlotSpec::provenance`];
//! - the judges, whose yes/no rubric questions come from [`ImageSlotSpec::job`],
//!   [`ImageSlotContent`], [`ImageSlotTextInImage`] and [`ImageSlotFocal`];
//! - the deterministic image assertions, which receive slot-derived values
//!   (the focal region, the expected copy, threshold overrides) as parameters.
//!   vision-core never reads a slot spec itself.
//!
//! Specs are authored as YAML, one file per slot. Rust is the source of truth;
//! the TypeScript and Python (pydantic) bindings are generated from the JSON
//! Schema `schemars` emits here.
//!
//! # The contract is closed
//!
//! Every struct and every tagged union is `deny_unknown_fields`: a misspelt key
//! (`must_survive_crops`) is refused, never silently dropped, and the schema
//! says `additionalProperties: false` so the generated pydantic model refuses
//! it too. Union arms that carry no data are EMPTY STRUCT variants
//! (`Decorative {}`), not unit variants, because serde ignores extra keys on an
//! internally tagged unit variant and would let `{kind: decorative, text: …}`
//! through while the schema refused it.
//!
//! # Two layers of validation
//!
//! 1. **Structural**, enforced by deserialization in every language: required
//!    fields, closed enums, closed objects, value types.
//! 2. **Value rules.** Most are carried in the schema (`pattern`,
//!    `minimum`/`maximum`, `minItems`), so the generated pydantic model enforces
//!    them as well. [`ImageSlotSpec::validate`] checks all of them in Rust,
//!    plus the cross-field rules JSON Schema cannot express — the
//!    [`ImageSlotProblem`] variants for which
//!    [`ImageSlotProblem::is_schema_rule`] is false. A validator in another
//!    language must mirror those by hand.
//!
//! A spec is valid iff it deserializes AND `validate` returns `Ok`.
//!
//! A validator generated from the schema matches these refusals only when it
//! validates STRICTLY and reads the YAML as YAML 1.2. Pydantic's default (lax)
//! mode coerces scalars that serde refuses, such as the string `"yes"` for a
//! boolean; validate the YAML's JSON form with
//! `model_validate_json(…, strict=True)` instead. A YAML 1.1 loader (PyYAML)
//! turns an unquoted `yes`, `no`, `on` or `off` into a boolean, and `4:3` into
//! 243, before any validator sees it.
//!
//! # Provenance is a record, not a claim about the current spec
//!
//! [`ImageSlotProvenance`] records the binding the pipeline made for the spec
//! whose [`ImageSlotSpec::content_sha256`] was
//! [`ImageSlotProvenance::spec_sha256`]. When the spec has changed since, the
//! binding is STALE, not invalid, so `validate` checks a binding only for its
//! own consistency and never against the current authored fields. The
//! pipeline's obligations toward the spec it binds are documented on each
//! field.
//!
//! # Wire-format notes
//!
//! - Field names are `snake_case`, the spelling the plan and the authored YAML
//!   use (`text_in_image`, `must_not`, `must_survive_crop`).
//! - Tagged unions are internally tagged with `"kind"`; enum values are
//!   `snake_case` (the crate convention, as in [`crate::cli_session`]).
//! - Aspect ratios are `"W/H"` strings (`"4/3"`), never `"4:3"`: a YAML 1.1
//!   loader (PyYAML, which the Python consumers use) reads an unquoted `4:3` as
//!   the base-60 integer 243. `W/H` is a plain string in every YAML version,
//!   and it is also CSS `aspect-ratio` syntax.
//! - Optional fields are omitted when absent (the crate convention). Lists are
//!   always serialized, even when empty, so [`ImageSlotSpec::content_sha256`]
//!   has one canonical input in every language.
//! - Every digest is lowercase hex sha256, the form the image pipeline uses
//!   for asset store keys, recipe hashes and manifest file hashes.
//! - Every type name carries the `ImageSlot` prefix. The bindings generator
//!   imports a nested `$defs` entry that shares a registered top-level name
//!   (`Region`, `Finding`, `AssertionResult`, `Context`, …) from that type's own
//!   file instead of declaring it, so a short name would silently become a
//!   different type.
//! - No type is referenced both from inside a tagged-union arm and from
//!   anywhere else. The TypeScript generator declares such a type twice, the
//!   second copy under an invented name, and its fork guard refuses the result.
//!   That is one reason [`ImageSlotSafeArea`] is not [`ImageSlotRegion`].

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::canonical_hash::{canonical_hash, CanonicalHashError};

// ============================================================================
// Value rules shared by the schema and `validate`
// ============================================================================

/// A slot id, an exemplar-set id or a manifest id: lowercase letters and digits
/// in segments joined by `.`, `-` or `_`.
pub const SLUG_PATTERN: &str = r"^[a-z0-9]+(?:[._-][a-z0-9]+)*$";
/// Longest slug, in characters.
pub const SLUG_MAX_LEN: usize = 128;
/// A reference to an `audience_profile` prompt document. The name part is
/// coord's own rule for a hand-authored prompt document name.
pub const AUDIENCE_REF_PATTERN: &str = r"^audience_profile/[a-z0-9]+(?:-[a-z0-9]+)*$";
/// A reference to a declared imagery treatment: a `domain_spec` or a
/// `decision_record`, optionally narrowed to one section (`#imagery`).
pub const IMAGERY_REF_PATTERN: &str =
    r"^(?:domain_spec|decision_record)/[a-z0-9]+(?:-[a-z0-9]+)*(?:#[a-z0-9]+(?:-[a-z0-9]+)*)?$";
/// An aspect ratio, `W/H`, with W and H whole numbers from 1 to 9999.
pub const ASPECT_RATIO_PATTERN: &str = r"^[1-9][0-9]{0,3}/[1-9][0-9]{0,3}$";
/// A BCP 47 language tag (`en`, `de-CH`, `zh-Hant-TW`).
pub const LOCALE_PATTERN: &str = r"^[a-z]{2,3}(?:-[A-Za-z0-9]{1,8})*$";
/// Text with at least one non-whitespace character. Anchored at both ends so
/// it reads the same under search, match and full-match semantics.
pub const NON_BLANK_PATTERN: &str = r"^\s*\S[\s\S]*$";
/// A requirement id: one token with no whitespace.
pub const REQUIREMENT_REF_PATTERN: &str = r"^\S+$";
/// Longest requirement id, in characters.
pub const REQUIREMENT_REF_MAX_LEN: usize = 256;
/// A sha256 digest as lowercase hex.
pub const SHA256_HEX_PATTERN: &str = r"^[0-9a-f]{64}$";
/// Widest breakpoint, in CSS pixels.
pub const BREAKPOINT_MAX_PX: u32 = 16_384;
/// Smallest device pixel ratio a context may declare.
pub const DPR_MIN: f64 = 0.5;
/// Largest device pixel ratio a context may declare.
pub const DPR_MAX: f64 = 8.0;

/// Slack for float sums such as `x + width`, so `0.7 + 0.3` reads as 1.
const REGION_EPSILON: f64 = 1e-9;

// ============================================================================
// The spec
// ============================================================================

/// One image placement's declared contract: what the image is for, where it
/// renders, and what it must and must not be.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotSpec {
    /// Stable slot id (`sessions.empty-state`): lowercase letters and digits in
    /// segments joined by `.`, `-` or `_`, at most 128 characters. The UI marks
    /// the rendered element `data-ui-bridge-image-slot="<slot>"`, and the asset
    /// store, the candidate records and the judges key on it.
    #[schemars(pattern(SLUG_PATTERN), length(max = SLUG_MAX_LEN))]
    pub slot: String,
    /// What the image is on its page.
    pub role: ImageSlotRole,
    /// The image's job, in plain sentences: what it must achieve for the person
    /// looking at it. Judges derive yes/no questions from it.
    #[schemars(pattern(NON_BLANK_PATTERN))]
    pub job: String,
    /// Who it is for: a reference to one of the tenant's `audience_profile`
    /// prompt documents (`audience_profile/external-operator`).
    #[schemars(pattern(AUDIENCE_REF_PATTERN))]
    pub audience: String,
    /// Every breakpoint × theme × device pixel ratio the image renders, is
    /// captured and is judged at.
    pub contexts: ImageSlotContexts,
    /// The aspect ratios the rendered box may take.
    pub aspect: ImageSlotAspect,
    /// The parts of the rendered image that page content is laid over, which
    /// must therefore stay calm enough for that content to read.
    pub safe_areas: ImageSlotSafeAreas,
    /// Where the subject sits in the image, and whether every crop must keep it.
    pub focal: ImageSlotFocal,
    /// The declared style the image follows.
    pub style: ImageSlotStyle,
    /// What the image must and must not depict.
    pub content: ImageSlotContent,
    /// Whether the image may carry text, and which.
    pub text_in_image: ImageSlotTextInImage,
    /// The image's text alternative.
    pub alt: ImageSlotAlt,
    /// Where the thresholds of the slot's deterministic image checks come from.
    pub thresholds: ImageSlotThresholds,
    /// The requirement this slot traces to, for requirement-level traceability:
    /// one token with no whitespace, at most 256 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        pattern(REQUIREMENT_REF_PATTERN),
        length(max = REQUIREMENT_REF_MAX_LEN)
    )]
    pub requirement_ref: Option<String>,
    /// The asset bound to the slot, written by the asset pipeline and never by
    /// the author. Absent until an asset is chosen; until then the UI renders
    /// the slot's placeholder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ImageSlotProvenance>,
}

/// What an image is on its page. Production rules and per-role taste models
/// group slots by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageSlotRole {
    /// The dominant image of a page or section.
    Hero,
    /// Shown while a view has nothing to list yet.
    EmptyState,
    /// Illustrates one feature or benefit beside its copy.
    Feature,
    /// Accompanies a first-run or onboarding step.
    Onboarding,
    /// A social preview card (Open Graph, Twitter), read by link crawlers
    /// rather than rendered on the page.
    OgCard,
    /// A background or texture behind other content.
    Background,
    /// Stands in for a person's missing avatar.
    AvatarFallback,
    /// A rich composed illustration or photograph that no other role describes.
    Scene,
    /// An identity mark. Identity assets are selected or supplied, never
    /// generated: the pipeline must not bind an `ai_generated` asset here.
    Logo,
    /// A capture of the running product. Captured from the product, never
    /// generated.
    Screenshot,
}

/// Where the image renders. Every combination is a context the image is
/// captured and judged in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotContexts {
    /// Viewport widths in CSS pixels, strictly ascending (`[360, 768, 1280]`).
    #[schemars(length(min = 1), inner(range(min = 1, max = BREAKPOINT_MAX_PX)))]
    pub breakpoints: Vec<u32>,
    /// Colour schemes, each at most once.
    #[schemars(length(min = 1))]
    pub themes: Vec<ImageSlotTheme>,
    /// Device pixel ratios from 0.5 to 8, strictly ascending (`[1, 2]`).
    #[schemars(length(min = 1), inner(range(min = DPR_MIN, max = DPR_MAX)))]
    pub dpr: Vec<f64>,
}

/// The colour scheme a context renders in (`prefers-color-scheme`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageSlotTheme {
    /// Dark colour scheme.
    Dark,
    /// Light colour scheme.
    Light,
}

/// The aspect ratios the image's rendered box may take.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotAspect {
    /// The ratio the box takes by default, as `W/H` with W and H whole numbers
    /// from 1 to 9999 (`4/3`, `1200/630`). The placeholder reserves it.
    #[schemars(pattern(ASPECT_RATIO_PATTERN))]
    pub preferred: String,
    /// Other ratios a layout may give the box, in the same form. None may equal
    /// `preferred` or another entry in value (`8/6` equals `4/3`). Empty means
    /// only `preferred`.
    #[serde(default)]
    #[schemars(inner(pattern(ASPECT_RATIO_PATTERN)))]
    pub allowed: Vec<String>,
}

/// A rectangle in the image's NATURAL (intrinsic) frame, as fractions (0 to 1)
/// of its width and height: `x` and `y` are the top-left corner, `width` and
/// `height` the size. It has a positive area and lies inside the frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotRegion {
    /// Left edge, as a fraction of the frame's width.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub x: f64,
    /// Top edge, as a fraction of the frame's height.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub y: f64,
    /// Width, as a fraction of the frame's width.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub width: f64,
    /// Height, as a fraction of the frame's height.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub height: f64,
}

/// A rectangle in the image's RENDERED box (the element's box, where page
/// content is positioned), as fractions (0 to 1) of its width and height: `x`
/// and `y` are the top-left corner, `width` and `height` the size. It has a
/// positive area and lies inside the box.
///
/// A distinct type from [`ImageSlotRegion`] because the two coordinate spaces
/// differ whenever object-fit crops or letterboxes the image.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotSafeArea {
    /// Left edge, as a fraction of the box's width.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub x: f64,
    /// Top edge, as a fraction of the box's height.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub y: f64,
    /// Width, as a fraction of the box's width.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub width: f64,
    /// Height, as a fraction of the box's height.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub height: f64,
}

/// The parts of the rendered image that page content (headings, copy, calls to
/// action) is laid over.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageSlotSafeAreas {
    /// Derived in each context, at capture time, from the UI Bridge geometry of
    /// the elements laid over the image.
    Derived {},
    /// Declared areas that hold in every context.
    Explicit {
        /// The areas, in the image's rendered box.
        #[schemars(length(min = 1))]
        regions: Vec<ImageSlotSafeArea>,
    },
}

/// Where the subject sits in the image.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotFocal {
    /// The subject's region in the image's natural frame. An image with no
    /// single subject, such as a texture, declares the whole frame with
    /// `must_survive_crop: false`.
    pub region: ImageSlotRegion,
    /// True when every rendered crop (object-fit and object-position in every
    /// context) must keep the whole focal region visible.
    pub must_survive_crop: bool,
}

/// The declared style the image follows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotStyle {
    /// The imagery treatment it follows: a section of one of the tenant's
    /// `domain_spec` prompt documents (`domain_spec/visual-system#imagery`), or
    /// a `decision_record` that settled a style fork.
    #[schemars(pattern(IMAGERY_REF_PATTERN))]
    pub imagery: String,
    /// The exemplar set (approved and rejected examples) the image is compared
    /// with, by id. Absent: none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(SLUG_PATTERN), length(max = SLUG_MAX_LEN))]
    pub exemplars: Option<String>,
    /// The house LoRA applied when generating, by its id in the image licence
    /// manifest. Absent: none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(SLUG_PATTERN), length(max = SLUG_MAX_LEN))]
    pub lora: Option<String>,
}

/// What the image must and must not depict. Each entry becomes a yes/no
/// question for the judges; an entry may not appear in both lists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotContent {
    /// Things the image must depict, one per entry. May be empty.
    #[schemars(inner(pattern(NON_BLANK_PATTERN)))]
    pub must: Vec<String>,
    /// Things the image must not depict, one per entry. May be empty.
    #[schemars(inner(pattern(NON_BLANK_PATTERN)))]
    pub must_not: Vec<String>,
}

/// Whether the image may carry text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageSlotTextInImage {
    /// No text in the image. Copy belongs in the page, where it can be
    /// translated, read by assistive technology and spell-checked.
    Forbidden {},
    /// The image must carry exactly this copy. OCR reads the image and reports
    /// its edit distance from `copy`.
    Expected {
        /// The text as it must read.
        #[schemars(pattern(NON_BLANK_PATTERN))]
        copy: String,
        /// Its language, as a BCP 47 tag (`en`, `de-CH`).
        #[schemars(pattern(LOCALE_PATTERN))]
        locale: String,
    },
    /// Incidental text is allowed and not checked.
    Allowed {},
}

/// The image's text alternative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageSlotAlt {
    /// Decorative: rendered with an empty `alt`, conveying nothing a reader
    /// needs.
    Decorative {},
    /// Informative, with a text alternative written for the bound asset, since
    /// only that image can be described. The pipeline records it in
    /// `provenance.alt_text`.
    Required {},
    /// Informative, with this fixed text alternative.
    Text {
        /// The text alternative.
        #[schemars(pattern(NON_BLANK_PATTERN))]
        text: String,
    },
}

/// Where the thresholds of the slot's deterministic image checks come from. A
/// threshold that is neither overridden here nor declared in the tenant's visual
/// system is UNKNOWN: its check answers unknown, never a pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageSlotThresholds {
    /// Every threshold comes from the tenant's declared visual system.
    FromVisualSystem {},
    /// These values override the visual system for this slot. A threshold not
    /// named here still comes from it. At least one must be set.
    Overrides {
        /// Floor on effective resolution, natural ÷ (rendered × DPR): the
        /// `min_effective` of `image_resolution_sufficient`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0.0))]
        min_effective_resolution: Option<f64>,
        /// Ceiling on effective resolution, against oversized images: the
        /// `max_effective` of `image_not_oversized`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0.0))]
        max_effective_resolution: Option<f64>,
        /// Tolerated relative difference between the natural and rendered
        /// aspect ratios: the `max_aspect_delta` of `image_not_distorted`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0.0))]
        max_aspect_delta: Option<f64>,
        /// Encoded byte budget: the `max_bytes` of `image_weight_within`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
        max_bytes: Option<u64>,
        /// Largest CIEDE2000 distance from the declared palette that still
        /// counts as on-palette: the `max_delta_e` of `image_palette_within`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0.0))]
        max_palette_delta_e: Option<f64>,
        /// Least fraction (0 to 1) of the image's pixels that must be
        /// on-palette: the `min_coverage` of `image_palette_within`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0.0, max = 1.0))]
        min_palette_coverage: Option<f64>,
        /// Least WCAG contrast ratio (1 to 21) of text laid over the image: the
        /// `min_ratio` of `text_over_image_contrast`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1.0, max = 21.0))]
        min_text_over_image_contrast: Option<f64>,
        /// Which percentile (0 to 100) of glyph-pixel contrast is compared with
        /// that floor: the `percentile` of `text_over_image_contrast`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0.0, max = 100.0))]
        text_over_image_percentile: Option<f64>,
        /// Least WCAG contrast ratio (1 to 21) of a transparent image's edge
        /// and stroke pixels against what surrounds it: the `min_ratio` of
        /// `transparent_image_legible`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1.0, max = 21.0))]
        min_transparent_edge_contrast: Option<f64>,
    },
}

/// The asset bound to a slot, as the asset pipeline records it; never
/// authored. It describes the binding made for the spec whose content hash was
/// `spec_sha256`. When the spec's current content hash differs, the binding is
/// stale (the spec changed since), not invalid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSlotProvenance {
    /// sha256 of the bound asset's original bytes: its content address in the
    /// tenant's image store, which keys its generation record (prompt, seed,
    /// weights, licence) and its judging record.
    #[schemars(pattern(SHA256_HEX_PATTERN))]
    pub asset_sha256: String,
    /// The spec's content hash when the asset was bound: sha256 of the spec's
    /// RFC 8785 canonical JSON with `provenance` left out.
    #[schemars(pattern(SHA256_HEX_PATTERN))]
    pub spec_sha256: String,
    /// How the asset was obtained.
    pub source: ImageSlotAssetSource,
    /// True when a model generated or edited the pixels. Such an asset, and
    /// every derivative of it, carries the IPTC `trainedAlgorithmicMedia`
    /// marker. Always true for `generated` and `edited`.
    pub ai_generated: bool,
    /// Hash of the generation recipe. Required for `generated` and `edited`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(SHA256_HEX_PATTERN))]
    pub recipe_sha256: Option<String>,
    /// sha256 of the imagery treatment's text as read when the asset was
    /// produced. Required for `generated` and `edited`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(SHA256_HEX_PATTERN))]
    pub imagery_sha256: Option<String>,
    /// sha256 identifying the version of the exemplar set in force when the
    /// asset was produced; recorded whenever the slot names a set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(SHA256_HEX_PATTERN))]
    pub exemplars_sha256: Option<String>,
    /// sha256 of the LoRA weights applied; recorded whenever the slot names a
    /// LoRA and a model produced the asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(SHA256_HEX_PATTERN))]
    pub lora_sha256: Option<String>,
    /// The text alternative written for this asset; recorded whenever the
    /// slot's `alt` is `required`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(pattern(NON_BLANK_PATTERN))]
    pub alt_text: Option<String>,
}

/// How a bound asset was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageSlotAssetSource {
    /// A generative model produced it from a prompt and a recipe.
    Generated,
    /// A generative model edited a parent asset (an edit or inpaint recipe).
    Edited,
    /// Vectorised from a raster.
    Traced,
    /// Written as code: an SVG using `currentColor` and token variables, or a
    /// code-rendered card.
    Authored,
    /// Taken from an existing set or library, such as an icon set or an
    /// approved asset.
    Selected,
    /// Captured from the running product.
    Captured,
    /// Supplied by a person, such as the operator's own photograph or mark.
    Supplied,
}

impl ImageSlotAssetSource {
    /// True for the sources in which a model made the pixels.
    pub fn is_model_made(self) -> bool {
        matches!(self, Self::Generated | Self::Edited)
    }
}

// ============================================================================
// Content hash
// ============================================================================

impl ImageSlotSpec {
    /// The spec's content address: lowercase hex sha256 of its RFC 8785
    /// canonical JSON (see [`crate::canonical_hash`]) with `provenance` left
    /// out, so binding an asset never changes the hash and any authored edit
    /// does.
    ///
    /// The canonical input is the spec's JSON form: absent optional fields
    /// omitted, never `null`; lists always present, even empty. A pydantic
    /// model reproduces it with
    /// `model_dump(mode="json", exclude_none=True, exclude={"provenance"})`
    /// fed to an RFC 8785 canonicalizer.
    pub fn content_sha256(&self) -> Result<String, CanonicalHashError> {
        let authored = Self {
            provenance: None,
            ..self.clone()
        };
        let prefixed = canonical_hash(&authored)?;
        prefixed
            .strip_prefix("sha256-")
            .map(str::to_owned)
            .ok_or_else(|| {
                CanonicalHashError::Canonicalize(format!(
                    "canonical_hash returned {prefixed:?}, not a sha256- digest"
                ))
            })
    }
}

// ============================================================================
// Validation
// ============================================================================

/// One rule a spec breaks.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageSlotIssue {
    /// Where, in the spec's own spelling (`contexts.dpr[1]`, `focal.region`).
    pub path: String,
    /// What is wrong there.
    pub problem: ImageSlotProblem,
}

impl fmt::Display for ImageSlotIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.problem)
    }
}

/// What is wrong with one value of a spec.
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSlotProblem {
    /// The value does not match the field's pattern.
    Pattern(&'static str),
    /// The value is longer than the field's limit, in characters.
    TooLong(usize),
    /// A list that needs at least one entry is empty.
    Empty,
    /// A number outside the field's inclusive range.
    OutOfRange {
        /// Lower bound.
        min: f64,
        /// Upper bound, `f64::INFINITY` when the field has none.
        max: f64,
    },
    /// A list that must be strictly ascending is not.
    NotAscending,
    /// A list holds the same value twice (aspect ratios compare by value,
    /// content entries ignoring case and surrounding whitespace).
    Duplicate,
    /// The same entry is both required and forbidden content.
    Contradictory,
    /// A region with no area, or one reaching past its reference box.
    RegionOutsideBox,
    /// `thresholds` is `overrides` but names no threshold.
    NoOverride,
    /// `min_effective_resolution` is above `max_effective_resolution`.
    MinAboveMax,
    /// A model-made asset is not marked `ai_generated`.
    UnmarkedModelAsset,
    /// A model-made asset lacks the input digest that makes it re-renderable
    /// when that input changes.
    MissingInputDigest,
}

impl ImageSlotProblem {
    /// True when the rule is carried in the JSON Schema, so every validator
    /// generated from it enforces the rule too. False for the cross-field rules
    /// a non-Rust validator must mirror by hand.
    pub fn is_schema_rule(&self) -> bool {
        matches!(
            self,
            Self::Pattern(_) | Self::TooLong(_) | Self::Empty | Self::OutOfRange { .. }
        )
    }
}

impl fmt::Display for ImageSlotProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pattern(pattern) => write!(f, "does not match {pattern}"),
            Self::TooLong(max) => write!(f, "is longer than {max} characters"),
            Self::Empty => f.write_str("must have at least one entry"),
            Self::OutOfRange { min, max } if max.is_infinite() => {
                write!(f, "must be at least {min}")
            }
            Self::OutOfRange { min, max } => write!(f, "must be between {min} and {max}"),
            Self::NotAscending => f.write_str("must be strictly ascending"),
            Self::Duplicate => f.write_str("repeats an earlier entry"),
            Self::Contradictory => f.write_str("is listed in both `must` and `must_not`"),
            Self::RegionOutsideBox => {
                f.write_str("must have a positive area and lie inside its reference box")
            }
            Self::NoOverride => {
                f.write_str("overrides no threshold; use `kind: from_visual_system`")
            }
            Self::MinAboveMax => {
                f.write_str("min_effective_resolution is above max_effective_resolution")
            }
            Self::UnmarkedModelAsset => {
                f.write_str("a generated or edited asset must be marked ai_generated")
            }
            Self::MissingInputDigest => {
                f.write_str("a generated or edited asset must record this input digest")
            }
        }
    }
}

/// Why a spec was refused: every rule it breaks, not just the first.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageSlotSpecInvalid {
    /// The refused spec's `slot`, as written.
    pub slot: String,
    /// The broken rules, in field order.
    pub issues: Vec<ImageSlotIssue>,
}

impl fmt::Display for ImageSlotSpecInvalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "image slot spec {:?} breaks {} rule(s)",
            self.slot,
            self.issues.len()
        )?;
        for issue in &self.issues {
            write!(f, "; {issue}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ImageSlotSpecInvalid {}

impl ImageSlotSpec {
    /// Check every value rule — the schema's and the cross-field ones — and
    /// report all the spec breaks. A spec is valid iff it deserializes AND this
    /// returns `Ok`.
    pub fn validate(&self) -> Result<(), ImageSlotSpecInvalid> {
        let mut c = Checker::default();

        c.slug("slot", &self.slot);
        c.pattern("job", &self.job, NON_BLANK_PATTERN, is_non_blank);
        c.pattern(
            "audience",
            &self.audience,
            AUDIENCE_REF_PATTERN,
            is_audience_ref,
        );
        self.contexts.check(&mut c);
        self.aspect.check(&mut c);
        if let ImageSlotSafeAreas::Explicit { regions } = &self.safe_areas {
            if regions.is_empty() {
                c.push("safe_areas.regions", ImageSlotProblem::Empty);
            }
            for (i, region) in regions.iter().enumerate() {
                region.check(&mut c, &format!("safe_areas.regions[{i}]"));
            }
        }
        self.focal.region.check(&mut c, "focal.region");
        self.style.check(&mut c);
        self.content.check(&mut c);
        match &self.text_in_image {
            ImageSlotTextInImage::Expected { copy, locale } => {
                c.pattern("text_in_image.copy", copy, NON_BLANK_PATTERN, is_non_blank);
                c.pattern("text_in_image.locale", locale, LOCALE_PATTERN, is_locale);
            }
            ImageSlotTextInImage::Forbidden {} | ImageSlotTextInImage::Allowed {} => {}
        }
        if let ImageSlotAlt::Text { text } = &self.alt {
            c.pattern("alt.text", text, NON_BLANK_PATTERN, is_non_blank);
        }
        self.thresholds.check(&mut c);
        if let Some(requirement) = &self.requirement_ref {
            c.pattern(
                "requirement_ref",
                requirement,
                REQUIREMENT_REF_PATTERN,
                is_requirement_ref,
            );
            c.max_len("requirement_ref", requirement, REQUIREMENT_REF_MAX_LEN);
        }
        if let Some(provenance) = &self.provenance {
            provenance.check(&mut c);
        }

        if c.issues.is_empty() {
            Ok(())
        } else {
            Err(ImageSlotSpecInvalid {
                slot: self.slot.clone(),
                issues: c.issues,
            })
        }
    }
}

impl ImageSlotContexts {
    fn check(&self, c: &mut Checker) {
        c.non_empty("contexts.breakpoints", self.breakpoints.len());
        for (i, &px) in self.breakpoints.iter().enumerate() {
            c.range(
                &format!("contexts.breakpoints[{i}]"),
                f64::from(px),
                1.0,
                f64::from(BREAKPOINT_MAX_PX),
            );
        }
        if !self.breakpoints.windows(2).all(|w| w[0] < w[1]) {
            c.push("contexts.breakpoints", ImageSlotProblem::NotAscending);
        }

        c.non_empty("contexts.themes", self.themes.len());
        for (i, theme) in self.themes.iter().enumerate() {
            if self.themes[..i].contains(theme) {
                c.push(format!("contexts.themes[{i}]"), ImageSlotProblem::Duplicate);
            }
        }

        c.non_empty("contexts.dpr", self.dpr.len());
        for (i, &dpr) in self.dpr.iter().enumerate() {
            c.range(&format!("contexts.dpr[{i}]"), dpr, DPR_MIN, DPR_MAX);
        }
        if !self.dpr.windows(2).all(|w| w[0] < w[1]) {
            c.push("contexts.dpr", ImageSlotProblem::NotAscending);
        }
    }
}

impl ImageSlotAspect {
    fn check(&self, c: &mut Checker) {
        let preferred = parse_aspect_ratio(&self.preferred);
        if preferred.is_none() {
            c.push(
                "aspect.preferred",
                ImageSlotProblem::Pattern(ASPECT_RATIO_PATTERN),
            );
        }
        let mut seen: Vec<(u32, u32)> = preferred.into_iter().map(reduce).collect();
        for (i, ratio) in self.allowed.iter().enumerate() {
            let path = format!("aspect.allowed[{i}]");
            match parse_aspect_ratio(ratio) {
                None => c.push(path, ImageSlotProblem::Pattern(ASPECT_RATIO_PATTERN)),
                Some(parsed) => {
                    let value = reduce(parsed);
                    if seen.contains(&value) {
                        c.push(path, ImageSlotProblem::Duplicate);
                    } else {
                        seen.push(value);
                    }
                }
            }
        }
    }
}

impl ImageSlotRegion {
    fn check(&self, c: &mut Checker, path: &str) {
        check_rect(c, path, self.x, self.y, self.width, self.height);
    }
}

impl ImageSlotSafeArea {
    fn check(&self, c: &mut Checker, path: &str) {
        check_rect(c, path, self.x, self.y, self.width, self.height);
    }
}

/// The rule both rectangle types share: each value a fraction, a positive area,
/// and no edge past the reference box.
fn check_rect(c: &mut Checker, path: &str, x: f64, y: f64, width: f64, height: f64) {
    let mut in_range = true;
    for (name, value) in [("x", x), ("y", y), ("width", width), ("height", height)] {
        in_range &= c.range(&format!("{path}.{name}"), value, 0.0, 1.0);
    }
    if in_range
        && (width <= 0.0
            || height <= 0.0
            || x + width > 1.0 + REGION_EPSILON
            || y + height > 1.0 + REGION_EPSILON)
    {
        c.push(path, ImageSlotProblem::RegionOutsideBox);
    }
}

impl ImageSlotStyle {
    fn check(&self, c: &mut Checker) {
        c.pattern(
            "style.imagery",
            &self.imagery,
            IMAGERY_REF_PATTERN,
            is_imagery_ref,
        );
        if let Some(exemplars) = &self.exemplars {
            c.slug("style.exemplars", exemplars);
        }
        if let Some(lora) = &self.lora {
            c.slug("style.lora", lora);
        }
    }
}

impl ImageSlotContent {
    fn check(&self, c: &mut Checker) {
        for (list, entries) in [
            ("content.must", &self.must),
            ("content.must_not", &self.must_not),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                let path = format!("{list}[{i}]");
                c.pattern(&path, entry, NON_BLANK_PATTERN, is_non_blank);
                if entries[..i]
                    .iter()
                    .any(|earlier| content_key(earlier) == content_key(entry))
                {
                    c.push(path, ImageSlotProblem::Duplicate);
                }
            }
        }
        for (i, entry) in self.must_not.iter().enumerate() {
            if self
                .must
                .iter()
                .any(|required| content_key(required) == content_key(entry))
            {
                c.push(
                    format!("content.must_not[{i}]"),
                    ImageSlotProblem::Contradictory,
                );
            }
        }
    }
}

impl ImageSlotThresholds {
    fn check(&self, c: &mut Checker) {
        let ImageSlotThresholds::Overrides {
            min_effective_resolution,
            max_effective_resolution,
            max_aspect_delta,
            max_bytes,
            max_palette_delta_e,
            min_palette_coverage,
            min_text_over_image_contrast,
            text_over_image_percentile,
            min_transparent_edge_contrast,
        } = self
        else {
            return;
        };
        let bounded = [
            (
                "min_effective_resolution",
                *min_effective_resolution,
                0.0,
                f64::INFINITY,
            ),
            (
                "max_effective_resolution",
                *max_effective_resolution,
                0.0,
                f64::INFINITY,
            ),
            ("max_aspect_delta", *max_aspect_delta, 0.0, f64::INFINITY),
            (
                "max_palette_delta_e",
                *max_palette_delta_e,
                0.0,
                f64::INFINITY,
            ),
            ("min_palette_coverage", *min_palette_coverage, 0.0, 1.0),
            (
                "min_text_over_image_contrast",
                *min_text_over_image_contrast,
                1.0,
                21.0,
            ),
            (
                "text_over_image_percentile",
                *text_over_image_percentile,
                0.0,
                100.0,
            ),
            (
                "min_transparent_edge_contrast",
                *min_transparent_edge_contrast,
                1.0,
                21.0,
            ),
        ];
        for (name, value, min, max) in bounded {
            if let Some(value) = value {
                c.range(&format!("thresholds.{name}"), value, min, max);
            }
        }
        if let Some(bytes) = max_bytes {
            if *bytes == 0 {
                c.push(
                    "thresholds.max_bytes",
                    ImageSlotProblem::OutOfRange {
                        min: 1.0,
                        max: f64::INFINITY,
                    },
                );
            }
        }
        if bounded.iter().all(|(_, value, ..)| value.is_none()) && max_bytes.is_none() {
            c.push("thresholds", ImageSlotProblem::NoOverride);
        }
        if let (Some(min), Some(max)) = (min_effective_resolution, max_effective_resolution) {
            if min > max {
                c.push("thresholds", ImageSlotProblem::MinAboveMax);
            }
        }
    }
}

impl ImageSlotProvenance {
    fn check(&self, c: &mut Checker) {
        c.digest("provenance.asset_sha256", &self.asset_sha256);
        c.digest("provenance.spec_sha256", &self.spec_sha256);
        for (name, digest) in [
            ("recipe_sha256", &self.recipe_sha256),
            ("imagery_sha256", &self.imagery_sha256),
            ("exemplars_sha256", &self.exemplars_sha256),
            ("lora_sha256", &self.lora_sha256),
        ] {
            if let Some(digest) = digest {
                c.digest(&format!("provenance.{name}"), digest);
            }
        }
        if let Some(alt_text) = &self.alt_text {
            c.pattern(
                "provenance.alt_text",
                alt_text,
                NON_BLANK_PATTERN,
                is_non_blank,
            );
        }
        if self.source.is_model_made() {
            if !self.ai_generated {
                c.push(
                    "provenance.ai_generated",
                    ImageSlotProblem::UnmarkedModelAsset,
                );
            }
            if self.recipe_sha256.is_none() {
                c.push(
                    "provenance.recipe_sha256",
                    ImageSlotProblem::MissingInputDigest,
                );
            }
            if self.imagery_sha256.is_none() {
                c.push(
                    "provenance.imagery_sha256",
                    ImageSlotProblem::MissingInputDigest,
                );
            }
        }
    }
}

/// Collects issues while `validate` walks a spec.
#[derive(Default)]
struct Checker {
    issues: Vec<ImageSlotIssue>,
}

impl Checker {
    fn push(&mut self, path: impl Into<String>, problem: ImageSlotProblem) {
        self.issues.push(ImageSlotIssue {
            path: path.into(),
            problem,
        });
    }

    fn pattern(&mut self, path: &str, value: &str, pattern: &'static str, ok: fn(&str) -> bool) {
        if !ok(value) {
            self.push(path, ImageSlotProblem::Pattern(pattern));
        }
    }

    fn max_len(&mut self, path: &str, value: &str, max: usize) {
        if value.chars().count() > max {
            self.push(path, ImageSlotProblem::TooLong(max));
        }
    }

    fn slug(&mut self, path: &str, value: &str) {
        self.pattern(path, value, SLUG_PATTERN, is_slug);
        self.max_len(path, value, SLUG_MAX_LEN);
    }

    fn digest(&mut self, path: &str, value: &str) {
        self.pattern(path, value, SHA256_HEX_PATTERN, is_sha256_hex);
    }

    fn non_empty(&mut self, path: &str, len: usize) {
        if len == 0 {
            self.push(path, ImageSlotProblem::Empty);
        }
    }

    /// Records an out-of-range (or non-finite) value; returns whether it was in range.
    fn range(&mut self, path: &str, value: f64, min: f64, max: f64) -> bool {
        let ok = value.is_finite() && (min..=max).contains(&value);
        if !ok {
            self.push(path, ImageSlotProblem::OutOfRange { min, max });
        }
        ok
    }
}

// ============================================================================
// Pattern predicates — each mirrors its `*_PATTERN` exactly (pinned by tests)
// ============================================================================

/// Content entries compare ignoring case and surrounding whitespace.
fn content_key(entry: &str) -> String {
    entry.trim().to_lowercase()
}

fn is_lower_alnum(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

fn is_slug(s: &str) -> bool {
    s.split(['.', '-', '_']).all(is_lower_alnum)
}

fn is_doc_name(s: &str) -> bool {
    s.split('-').all(is_lower_alnum)
}

fn is_audience_ref(s: &str) -> bool {
    s.strip_prefix("audience_profile/").is_some_and(is_doc_name)
}

fn is_imagery_ref(s: &str) -> bool {
    let Some(rest) = s
        .strip_prefix("domain_spec/")
        .or_else(|| s.strip_prefix("decision_record/"))
    else {
        return false;
    };
    match rest.split_once('#') {
        Some((name, section)) => is_doc_name(name) && is_doc_name(section),
        None => is_doc_name(rest),
    }
}

fn is_locale(s: &str) -> bool {
    let mut subtags = s.split('-');
    let language = subtags.next().unwrap_or_default();
    (2..=3).contains(&language.len())
        && language.bytes().all(|b| b.is_ascii_lowercase())
        && subtags
            .all(|t| (1..=8).contains(&t.len()) && t.bytes().all(|b| b.is_ascii_alphanumeric()))
}

fn is_non_blank(s: &str) -> bool {
    !s.trim().is_empty()
}

fn is_requirement_ref(s: &str) -> bool {
    !s.is_empty() && !s.chars().any(char::is_whitespace)
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Parse a `W/H` aspect ratio (see [`ASPECT_RATIO_PATTERN`]) into its whole
/// numbers, as written (`"8/6"` gives `(8, 6)`). `None` for anything else,
/// including `"4:3"`.
pub fn parse_aspect_ratio(s: &str) -> Option<(u32, u32)> {
    fn term(t: &str) -> Option<u32> {
        let ok = (1..=4).contains(&t.len())
            && !t.starts_with('0')
            && t.bytes().all(|b| b.is_ascii_digit());
        if ok {
            t.parse().ok()
        } else {
            None
        }
    }
    let (w, h) = s.split_once('/')?;
    Some((term(w)?, term(h)?))
}

/// A ratio in lowest terms, so `8/6` and `4/3` compare equal.
fn reduce((w, h): (u32, u32)) -> (u32, u32) {
    let (mut a, mut b) = (w, h);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    (w / a, h / a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn region(x: f64, y: f64, width: f64, height: f64) -> ImageSlotRegion {
        ImageSlotRegion {
            x,
            y,
            width,
            height,
        }
    }

    fn area(x: f64, y: f64, width: f64, height: f64) -> ImageSlotSafeArea {
        ImageSlotSafeArea {
            x,
            y,
            width,
            height,
        }
    }

    /// The report's §4.1 example, in the contract's own spelling.
    fn empty_state() -> ImageSlotSpec {
        ImageSlotSpec {
            slot: "sessions.empty-state".into(),
            role: ImageSlotRole::EmptyState,
            job: "Reassure a first-time operator that nothing is broken.".into(),
            audience: "audience_profile/external-operator".into(),
            contexts: ImageSlotContexts {
                breakpoints: vec![360, 768, 1280, 1920],
                themes: vec![ImageSlotTheme::Dark],
                dpr: vec![1.0, 2.0],
            },
            aspect: ImageSlotAspect {
                preferred: "4/3".into(),
                allowed: vec!["1/1".into()],
            },
            safe_areas: ImageSlotSafeAreas::Derived {},
            focal: ImageSlotFocal {
                region: region(0.05, 0.2, 0.45, 0.6),
                must_survive_crop: true,
            },
            style: ImageSlotStyle {
                imagery: "domain_spec/visual-system#imagery".into(),
                exemplars: Some("console-empty-states".into()),
                lora: None,
            },
            content: ImageSlotContent {
                must: vec!["abstract terminal motif".into()],
                must_not: vec!["people".into(), "third-party logos".into()],
            },
            text_in_image: ImageSlotTextInImage::Forbidden {},
            alt: ImageSlotAlt::Decorative {},
            thresholds: ImageSlotThresholds::FromVisualSystem {},
            requirement_ref: None,
            provenance: None,
        }
    }

    fn generated_binding() -> ImageSlotProvenance {
        ImageSlotProvenance {
            asset_sha256: DIGEST_A.into(),
            spec_sha256: DIGEST_B.into(),
            source: ImageSlotAssetSource::Generated,
            ai_generated: true,
            recipe_sha256: Some(DIGEST_B.into()),
            imagery_sha256: Some(DIGEST_A.into()),
            exemplars_sha256: None,
            lora_sha256: None,
            alt_text: None,
        }
    }

    fn no_overrides() -> ImageSlotThresholds {
        ImageSlotThresholds::Overrides {
            min_effective_resolution: None,
            max_effective_resolution: None,
            max_aspect_delta: None,
            max_bytes: None,
            max_palette_delta_e: None,
            min_palette_coverage: None,
            min_text_over_image_contrast: None,
            text_over_image_percentile: None,
            min_transparent_edge_contrast: None,
        }
    }

    fn issues(spec: &ImageSlotSpec) -> Vec<(String, ImageSlotProblem)> {
        match spec.validate() {
            Ok(()) => Vec::new(),
            Err(invalid) => invalid
                .issues
                .into_iter()
                .map(|i| (i.path, i.problem))
                .collect(),
        }
    }

    // ── wire shape ───────────────────────────────────────────────────────

    #[test]
    fn spec_wire_shape_is_snake_case_with_kind_tagged_unions() {
        let spec = empty_state();
        let wire = serde_json::to_value(&spec).unwrap();
        assert_eq!(
            wire,
            json!({
                "slot": "sessions.empty-state",
                "role": "empty_state",
                "job": "Reassure a first-time operator that nothing is broken.",
                "audience": "audience_profile/external-operator",
                "contexts": {"breakpoints": [360, 768, 1280, 1920], "themes": ["dark"], "dpr": [1.0, 2.0]},
                "aspect": {"preferred": "4/3", "allowed": ["1/1"]},
                "safe_areas": {"kind": "derived"},
                "focal": {"region": {"x": 0.05, "y": 0.2, "width": 0.45, "height": 0.6}, "must_survive_crop": true},
                "style": {"imagery": "domain_spec/visual-system#imagery", "exemplars": "console-empty-states"},
                "content": {"must": ["abstract terminal motif"], "must_not": ["people", "third-party logos"]},
                "text_in_image": {"kind": "forbidden"},
                "alt": {"kind": "decorative"},
                "thresholds": {"kind": "from_visual_system"},
            })
        );
        let back: ImageSlotSpec = serde_json::from_value(wire).unwrap();
        assert_eq!(back, spec);
    }

    /// Parse `wire` as `T`, then serialize it back: it must come out unchanged.
    fn wire_round_trip<T>(wire: Value) -> T
    where
        T: Serialize + serde::de::DeserializeOwned,
    {
        let parsed: T = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(&parsed).unwrap(), wire);
        parsed
    }

    #[test]
    fn data_carrying_arms_put_their_fields_beside_the_kind() {
        let expected: ImageSlotTextInImage =
            wire_round_trip(json!({"kind": "expected", "copy": "Ship it", "locale": "en"}));
        assert_eq!(
            expected,
            ImageSlotTextInImage::Expected {
                copy: "Ship it".into(),
                locale: "en".into()
            }
        );
        let alt: ImageSlotAlt = wire_round_trip(json!({"kind": "text", "text": "A chart"}));
        assert_eq!(
            alt,
            ImageSlotAlt::Text {
                text: "A chart".into()
            }
        );
        let areas: ImageSlotSafeAreas = wire_round_trip(json!({
            "kind": "explicit",
            "regions": [{"x": 0.0, "y": 0.0, "width": 0.5, "height": 1.0}],
        }));
        assert_eq!(
            areas,
            ImageSlotSafeAreas::Explicit {
                regions: vec![area(0.0, 0.0, 0.5, 1.0)]
            }
        );
        let thresholds: ImageSlotThresholds =
            wire_round_trip(json!({"kind": "overrides", "max_bytes": 250000}));
        assert!(
            matches!(
                thresholds,
                ImageSlotThresholds::Overrides {
                    max_bytes: Some(250_000),
                    min_effective_resolution: None,
                    min_text_over_image_contrast: None,
                    ..
                }
            ),
            "{thresholds:?}"
        );
    }

    /// `T` refuses `value` naming `stray` as the unknown field, and accepts the
    /// same value without it — so the refusal is the stray key's alone.
    fn refuses_stray_key<T>(value: Value, stray: &str)
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let err = serde_json::from_value::<T>(value.clone()).unwrap_err();
        assert!(
            err.to_string()
                .contains(&format!("unknown field `{stray}`")),
            "{value}: {err}"
        );
        let mut control = value;
        control.as_object_mut().unwrap().remove(stray);
        serde_json::from_value::<T>(control).unwrap();
    }

    #[test]
    fn a_data_less_arm_refuses_a_stray_key_like_the_schema_does() {
        refuses_stray_key::<ImageSlotAlt>(json!({"kind": "decorative", "text": "Logo"}), "text");
        refuses_stray_key::<ImageSlotAlt>(json!({"kind": "required", "text": "Logo"}), "text");
        refuses_stray_key::<ImageSlotTextInImage>(
            json!({"kind": "forbidden", "copy": "x"}),
            "copy",
        );
        refuses_stray_key::<ImageSlotTextInImage>(json!({"kind": "allowed", "copy": "x"}), "copy");
        refuses_stray_key::<ImageSlotSafeAreas>(
            json!({"kind": "derived", "regions": []}),
            "regions",
        );
        refuses_stray_key::<ImageSlotThresholds>(
            json!({"kind": "from_visual_system", "max_bytes": 1}),
            "max_bytes",
        );
    }

    #[test]
    fn unknown_fields_bad_enums_and_missing_fields_are_refused() {
        let mut wire = serde_json::to_value(empty_state()).unwrap();
        wire["focal"]["must_survive_crops"] = json!(true);
        let err = serde_json::from_value::<ImageSlotSpec>(wire).unwrap_err();
        assert!(
            err.to_string()
                .contains("unknown field `must_survive_crops`"),
            "{err}"
        );

        let mut wire = serde_json::to_value(empty_state()).unwrap();
        wire["role"] = json!("banner");
        let err = serde_json::from_value::<ImageSlotSpec>(wire).unwrap_err();
        assert!(
            err.to_string().contains("unknown variant `banner`"),
            "{err}"
        );

        let mut wire = serde_json::to_value(empty_state()).unwrap();
        wire.as_object_mut().unwrap().remove("job");
        let err = serde_json::from_value::<ImageSlotSpec>(wire).unwrap_err();
        assert!(err.to_string().contains("missing field `job`"), "{err}");
    }

    // ── the schema the bindings are generated from ───────────────────────

    fn schema() -> Value {
        serde_json::to_value(schemars::schema_for!(ImageSlotSpec)).unwrap()
    }

    /// Every object schema in the tree, with its JSON-pointer-ish path.
    fn objects<'a>(node: &'a Value, path: String, out: &mut Vec<(String, &'a Value)>) {
        match node {
            Value::Object(map) => {
                if map.contains_key("properties") {
                    out.push((path.clone(), node));
                }
                for (k, v) in map {
                    objects(v, format!("{path}/{k}"), out);
                }
            }
            Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    objects(v, format!("{path}/{i}"), out);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn every_object_in_the_schema_is_closed() {
        let schema = schema();
        let mut found = Vec::new();
        objects(&schema, String::new(), &mut found);
        assert!(
            found.len() >= 15,
            "only {} object schemas found",
            found.len()
        );
        for (path, object) in found {
            assert_eq!(
                object["additionalProperties"],
                json!(false),
                "object at {path} is open: {object}"
            );
        }
    }

    #[test]
    fn schema_requires_every_authored_field_and_nothing_pipeline_written() {
        let schema = schema();
        assert_eq!(schema["title"], "ImageSlotSpec");
        let mut required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        required.sort_unstable();
        assert_eq!(
            required,
            [
                "alt",
                "aspect",
                "audience",
                "content",
                "contexts",
                "focal",
                "job",
                "role",
                "safe_areas",
                "slot",
                "style",
                "text_in_image",
                "thresholds",
            ]
        );
    }

    #[test]
    fn every_tagged_union_carries_a_kind_discriminator() {
        let schema = schema();
        let defs = schema["$defs"].as_object().unwrap();
        for (name, kinds) in [
            ("ImageSlotSafeAreas", vec!["derived", "explicit"]),
            (
                "ImageSlotTextInImage",
                vec!["forbidden", "expected", "allowed"],
            ),
            ("ImageSlotAlt", vec!["decorative", "required", "text"]),
            (
                "ImageSlotThresholds",
                vec!["from_visual_system", "overrides"],
            ),
        ] {
            let arms = defs[name]["oneOf"]
                .as_array()
                .unwrap_or_else(|| panic!("{name} is not a oneOf union"));
            let consts: Vec<&str> = arms
                .iter()
                .map(|arm| arm["properties"]["kind"]["const"].as_str().unwrap())
                .collect();
            assert_eq!(consts, kinds, "{name}");
        }
    }

    /// Every `$ref` target under `node`.
    fn refs(node: &Value, out: &mut Vec<String>) {
        match node {
            Value::Object(map) => {
                if let Some(Value::String(r)) = map.get("$ref") {
                    out.push(r.trim_start_matches("#/$defs/").to_owned());
                }
                map.values().for_each(|v| refs(v, out));
            }
            Value::Array(items) => items.iter().for_each(|v| refs(v, out)),
            _ => {}
        }
    }

    /// The TypeScript generator (json-schema-to-typescript, behind
    /// `scripts/compile_typescript.mjs`) parses a definition reached from inside
    /// a tagged-union arm separately from the same definition reached anywhere
    /// else, and declares the second copy under an invented name
    /// (`ImageSlotRegion1`), which that script's fork guard then refuses. So no
    /// definition may be referenced both inside a union arm and outside one.
    #[test]
    fn no_definition_is_shared_between_a_union_arm_and_anything_else() {
        let schema = schema();
        let mut in_arms = Vec::new();
        let mut elsewhere = Vec::new();
        refs(&schema["properties"], &mut elsewhere);
        for def in schema["$defs"].as_object().unwrap().values() {
            match def.get("oneOf") {
                Some(arms) => refs(arms, &mut in_arms),
                None => refs(def, &mut elsewhere),
            }
        }
        assert!(
            in_arms.contains(&"ImageSlotSafeArea".to_owned()),
            "the probe must see the one definition a union arm references: {in_arms:?}"
        );
        for name in &in_arms {
            assert!(
                !elsewhere.contains(name),
                "{name} is referenced inside a union arm and outside one"
            );
        }
    }

    #[test]
    fn every_type_name_in_the_schema_carries_the_image_slot_prefix() {
        let schema = schema();
        for name in schema["$defs"].as_object().unwrap().keys() {
            assert!(name.starts_with("ImageSlot"), "unprefixed type {name}");
        }
    }

    #[test]
    fn schema_carries_the_value_rules_validate_checks() {
        let s = schema();
        let d = &s["$defs"];
        let cases: [(&Value, &str, Value); 19] = [
            (&s["properties"]["slot"], "pattern", json!(SLUG_PATTERN)),
            (&s["properties"]["slot"], "maxLength", json!(SLUG_MAX_LEN)),
            (&s["properties"]["job"], "pattern", json!(NON_BLANK_PATTERN)),
            (
                &s["properties"]["audience"],
                "pattern",
                json!(AUDIENCE_REF_PATTERN),
            ),
            (
                &s["properties"]["requirement_ref"],
                "pattern",
                json!(REQUIREMENT_REF_PATTERN),
            ),
            (
                &s["properties"]["requirement_ref"],
                "maxLength",
                json!(REQUIREMENT_REF_MAX_LEN),
            ),
            (
                &d["ImageSlotContexts"]["properties"]["breakpoints"],
                "minItems",
                json!(1),
            ),
            (
                &d["ImageSlotContexts"]["properties"]["breakpoints"]["items"],
                "maximum",
                json!(BREAKPOINT_MAX_PX),
            ),
            (
                &d["ImageSlotContexts"]["properties"]["breakpoints"]["items"],
                "minimum",
                json!(1),
            ),
            (
                &d["ImageSlotContexts"]["properties"]["themes"],
                "minItems",
                json!(1),
            ),
            (
                &d["ImageSlotContexts"]["properties"]["dpr"]["items"],
                "minimum",
                json!(DPR_MIN),
            ),
            (
                &d["ImageSlotContexts"]["properties"]["dpr"]["items"],
                "maximum",
                json!(DPR_MAX),
            ),
            (
                &d["ImageSlotAspect"]["properties"]["preferred"],
                "pattern",
                json!(ASPECT_RATIO_PATTERN),
            ),
            (
                &d["ImageSlotAspect"]["properties"]["allowed"]["items"],
                "pattern",
                json!(ASPECT_RATIO_PATTERN),
            ),
            (
                &d["ImageSlotRegion"]["properties"]["width"],
                "maximum",
                json!(1.0),
            ),
            (
                &d["ImageSlotStyle"]["properties"]["imagery"],
                "pattern",
                json!(IMAGERY_REF_PATTERN),
            ),
            (
                &d["ImageSlotStyle"]["properties"]["lora"],
                "pattern",
                json!(SLUG_PATTERN),
            ),
            (
                &d["ImageSlotContent"]["properties"]["must_not"]["items"],
                "pattern",
                json!(NON_BLANK_PATTERN),
            ),
            (
                &d["ImageSlotProvenance"]["properties"]["asset_sha256"],
                "pattern",
                json!(SHA256_HEX_PATTERN),
            ),
        ];
        for (node, key, expected) in cases {
            assert_eq!(node[key], expected, "{key} on {node}");
        }
        assert_eq!(
            d["ImageSlotSafeArea"]["properties"]["height"]["maximum"],
            json!(1.0)
        );
        let expected = &d["ImageSlotTextInImage"]["oneOf"][1]["properties"];
        assert_eq!(expected["locale"]["pattern"], json!(LOCALE_PATTERN));
        let overrides = &d["ImageSlotThresholds"]["oneOf"][1]["properties"];
        assert_eq!(overrides["min_palette_coverage"]["maximum"], json!(1.0));
        assert_eq!(
            overrides["min_text_over_image_contrast"]["minimum"],
            json!(1.0)
        );
        assert_eq!(overrides["max_bytes"]["minimum"], json!(1));
    }

    // ── each pattern predicate is its regex ──────────────────────────────

    #[test]
    fn every_predicate_agrees_with_its_schema_pattern() {
        type Case = (&'static str, fn(&str) -> bool, &'static [&'static str]);
        let ratio_ok = |s: &str| parse_aspect_ratio(s).is_some();
        let cases: [Case; 8] = [
            (
                SLUG_PATTERN,
                is_slug,
                &[
                    "a",
                    "sessions.empty-state",
                    "a_b-c.d9",
                    "",
                    "A",
                    "a..b",
                    ".a",
                    "a-",
                    "a b",
                    "é",
                    "a/b",
                ],
            ),
            (
                AUDIENCE_REF_PATTERN,
                is_audience_ref,
                &[
                    "audience_profile/external-operator",
                    "audience_profile/x",
                    "audience_profile/",
                    "audience_profile/a_b",
                    "audience_profile/a--b",
                    "domain_spec/x",
                    "external-operator",
                    "audience_profile/a/b",
                ],
            ),
            (
                IMAGERY_REF_PATTERN,
                is_imagery_ref,
                &[
                    "domain_spec/visual-system#imagery",
                    "domain_spec/visual-system",
                    "decision_record/house-style#a-b",
                    "domain_spec/visual-system#",
                    "domain_spec/visual-system#a#b",
                    "audience_profile/x",
                    "domain_spec/Visual",
                    "domain_spec/#imagery",
                ],
            ),
            (
                ASPECT_RATIO_PATTERN,
                ratio_ok,
                &[
                    "4/3", "1200/630", "9999/1", "4:3", "0/3", "04/3", "10000/1", "4/3/2", "4/",
                    "/3", "-4/3", "4 /3",
                ],
            ),
            (
                LOCALE_PATTERN,
                is_locale,
                &[
                    "en",
                    "de-CH",
                    "zh-Hant-TW",
                    "en-x-a",
                    "EN",
                    "e",
                    "engl",
                    "en-",
                    "en_US",
                    "en-123456789",
                ],
            ),
            (
                NON_BLANK_PATTERN,
                is_non_blank,
                &["x", "  x  ", "\nline\n", "", " ", "\t\n", "\u{2003}"],
            ),
            (
                REQUIREMENT_REF_PATTERN,
                is_requirement_ref,
                &["REQ-12", "spec#R3", "", "a b", " x", "x\n"],
            ),
            (
                SHA256_HEX_PATTERN,
                is_sha256_hex,
                &[
                    DIGEST_A,
                    DIGEST_B,
                    "",
                    "ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
                    "a",
                ],
            ),
        ];
        for (pattern, predicate, samples) in cases {
            let regex = regex::Regex::new(pattern).unwrap();
            let mut accepted = 0;
            for &sample in samples {
                let expected = regex.is_match(sample);
                accepted += usize::from(expected);
                assert_eq!(predicate(sample), expected, "{pattern} on {sample:?}");
            }
            assert!(
                accepted > 0 && accepted < samples.len(),
                "{pattern}: the samples must include both a match and a refusal"
            );
        }
    }

    #[test]
    fn aspect_ratios_parse_as_written_and_compare_in_lowest_terms() {
        assert_eq!(parse_aspect_ratio("8/6"), Some((8, 6)));
        assert_eq!(reduce((8, 6)), (4, 3));
        assert_eq!(reduce((1200, 630)), (40, 21));
        assert_eq!(parse_aspect_ratio("4:3"), None);
    }

    // ── validate ─────────────────────────────────────────────────────────

    #[test]
    fn the_report_example_is_valid() {
        assert_eq!(empty_state().validate(), Ok(()));
    }

    #[test]
    fn validate_reports_every_broken_rule_at_its_path() {
        use ImageSlotProblem as P;
        type Mutation = fn(&mut ImageSlotSpec);
        // A function argument is a coercion site, so each closure becomes a
        // `Mutation` here (inside `vec![…]` the tuples would not coerce).
        fn case(mutate: Mutation, path: &'static str, problem: P) -> (Mutation, &'static str, P) {
            (mutate, path, problem)
        }
        let cases = vec![
            case(
                |s| s.slot = "Sessions".into(),
                "slot",
                P::Pattern(SLUG_PATTERN),
            ),
            case(
                |s| s.slot = "a".repeat(129),
                "slot",
                P::TooLong(SLUG_MAX_LEN),
            ),
            case(
                |s| s.job = " \n".into(),
                "job",
                P::Pattern(NON_BLANK_PATTERN),
            ),
            case(
                |s| s.audience = "external-operator".into(),
                "audience",
                P::Pattern(AUDIENCE_REF_PATTERN),
            ),
            case(
                |s| s.contexts.breakpoints.clear(),
                "contexts.breakpoints",
                P::Empty,
            ),
            case(
                |s| s.contexts.breakpoints = vec![0, 10],
                "contexts.breakpoints[0]",
                P::OutOfRange {
                    min: 1.0,
                    max: 16_384.0,
                },
            ),
            case(
                |s| s.contexts.breakpoints = vec![768, 360],
                "contexts.breakpoints",
                P::NotAscending,
            ),
            case(|s| s.contexts.themes.clear(), "contexts.themes", P::Empty),
            case(
                |s| s.contexts.themes = vec![ImageSlotTheme::Dark, ImageSlotTheme::Dark],
                "contexts.themes[1]",
                P::Duplicate,
            ),
            case(
                |s| s.contexts.dpr = vec![2.0, 1.0],
                "contexts.dpr",
                P::NotAscending,
            ),
            case(
                |s| s.contexts.dpr = vec![f64::NAN],
                "contexts.dpr[0]",
                P::OutOfRange {
                    min: DPR_MIN,
                    max: DPR_MAX,
                },
            ),
            case(
                |s| s.aspect.preferred = "4:3".into(),
                "aspect.preferred",
                P::Pattern(ASPECT_RATIO_PATTERN),
            ),
            case(
                |s| s.aspect.allowed = vec!["8/6".into()],
                "aspect.allowed[0]",
                P::Duplicate,
            ),
            case(
                |s| s.safe_areas = ImageSlotSafeAreas::Explicit { regions: vec![] },
                "safe_areas.regions",
                P::Empty,
            ),
            case(
                |s| {
                    s.safe_areas = ImageSlotSafeAreas::Explicit {
                        regions: vec![area(0.0, 0.0, 0.4, 1.0), area(0.6, 0.0, 0.5, 1.0)],
                    }
                },
                "safe_areas.regions[1]",
                P::RegionOutsideBox,
            ),
            case(
                |s| s.focal.region = region(0.7, 0.0, 0.5, 1.0),
                "focal.region",
                P::RegionOutsideBox,
            ),
            case(
                |s| s.focal.region = region(0.0, 0.0, 0.0, 1.0),
                "focal.region",
                P::RegionOutsideBox,
            ),
            case(
                |s| s.focal.region = region(-0.1, 0.0, 0.5, 1.0),
                "focal.region.x",
                P::OutOfRange { min: 0.0, max: 1.0 },
            ),
            case(
                |s| s.style.imagery = "visual-system".into(),
                "style.imagery",
                P::Pattern(IMAGERY_REF_PATTERN),
            ),
            case(
                |s| s.style.lora = Some("House LoRA".into()),
                "style.lora",
                P::Pattern(SLUG_PATTERN),
            ),
            case(
                |s| s.content.must = vec!["".into()],
                "content.must[0]",
                P::Pattern(NON_BLANK_PATTERN),
            ),
            case(
                |s| s.content.must_not = vec!["people".into(), " People ".into()],
                "content.must_not[1]",
                P::Duplicate,
            ),
            case(
                |s| s.content.must_not = vec!["Abstract terminal motif".into()],
                "content.must_not[0]",
                P::Contradictory,
            ),
            case(
                |s| {
                    s.text_in_image = ImageSlotTextInImage::Expected {
                        copy: "Ship it".into(),
                        locale: "English".into(),
                    }
                },
                "text_in_image.locale",
                P::Pattern(LOCALE_PATTERN),
            ),
            case(
                |s| s.alt = ImageSlotAlt::Text { text: "".into() },
                "alt.text",
                P::Pattern(NON_BLANK_PATTERN),
            ),
            case(
                |s| s.thresholds = no_overrides(),
                "thresholds",
                P::NoOverride,
            ),
            case(
                |s| {
                    s.thresholds = ImageSlotThresholds::Overrides {
                        min_effective_resolution: Some(2.0),
                        max_effective_resolution: Some(1.0),
                        max_aspect_delta: None,
                        max_bytes: None,
                        max_palette_delta_e: None,
                        min_palette_coverage: None,
                        min_text_over_image_contrast: None,
                        text_over_image_percentile: None,
                        min_transparent_edge_contrast: None,
                    }
                },
                "thresholds",
                P::MinAboveMax,
            ),
            case(
                |s| {
                    s.thresholds = ImageSlotThresholds::Overrides {
                        min_effective_resolution: None,
                        max_effective_resolution: None,
                        max_aspect_delta: None,
                        max_bytes: Some(0),
                        max_palette_delta_e: None,
                        min_palette_coverage: Some(1.5),
                        min_text_over_image_contrast: None,
                        text_over_image_percentile: None,
                        min_transparent_edge_contrast: None,
                    }
                },
                "thresholds.min_palette_coverage",
                P::OutOfRange { min: 0.0, max: 1.0 },
            ),
            case(
                |s| s.requirement_ref = Some("REQ 12".into()),
                "requirement_ref",
                P::Pattern(REQUIREMENT_REF_PATTERN),
            ),
            case(
                |s| {
                    s.provenance = Some(ImageSlotProvenance {
                        asset_sha256: "ABC".into(),
                        ..generated_binding()
                    })
                },
                "provenance.asset_sha256",
                P::Pattern(SHA256_HEX_PATTERN),
            ),
            case(
                |s| {
                    s.provenance = Some(ImageSlotProvenance {
                        ai_generated: false,
                        ..generated_binding()
                    })
                },
                "provenance.ai_generated",
                P::UnmarkedModelAsset,
            ),
            case(
                |s| {
                    s.provenance = Some(ImageSlotProvenance {
                        recipe_sha256: None,
                        ..generated_binding()
                    })
                },
                "provenance.recipe_sha256",
                P::MissingInputDigest,
            ),
            case(
                |s| {
                    s.provenance = Some(ImageSlotProvenance {
                        alt_text: Some("\t".into()),
                        ..generated_binding()
                    })
                },
                "provenance.alt_text",
                P::Pattern(NON_BLANK_PATTERN),
            ),
        ];
        for (mutate, path, problem) in cases {
            let mut spec = empty_state();
            mutate(&mut spec);
            let found = issues(&spec);
            assert!(
                found.contains(&(path.to_owned(), problem.clone())),
                "expected {path}: {problem:?}, got {found:?}"
            );
        }
    }

    #[test]
    fn a_binding_by_hand_is_valid_when_nothing_was_model_made() {
        let mut spec = empty_state();
        spec.role = ImageSlotRole::Logo;
        spec.provenance = Some(ImageSlotProvenance {
            asset_sha256: DIGEST_A.into(),
            spec_sha256: DIGEST_B.into(),
            source: ImageSlotAssetSource::Supplied,
            ai_generated: false,
            recipe_sha256: None,
            imagery_sha256: None,
            exemplars_sha256: None,
            lora_sha256: None,
            alt_text: None,
        });
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn schema_rules_and_cross_field_rules_are_told_apart() {
        use ImageSlotProblem as P;
        for schema_rule in [
            P::Pattern(SLUG_PATTERN),
            P::TooLong(1),
            P::Empty,
            P::OutOfRange { min: 0.0, max: 1.0 },
        ] {
            assert!(schema_rule.is_schema_rule(), "{schema_rule:?}");
        }
        for cross_field in [
            P::NotAscending,
            P::Duplicate,
            P::Contradictory,
            P::RegionOutsideBox,
            P::NoOverride,
            P::MinAboveMax,
            P::UnmarkedModelAsset,
            P::MissingInputDigest,
        ] {
            assert!(!cross_field.is_schema_rule(), "{cross_field:?}");
        }
    }

    #[test]
    fn the_refusal_names_the_slot_and_every_issue() {
        let mut spec = empty_state();
        spec.job = String::new();
        spec.contexts.dpr = vec![2.0, 1.0];
        let refusal = spec.validate().unwrap_err();
        assert_eq!(refusal.issues.len(), 2);
        assert_eq!(
            refusal.to_string(),
            format!(
                "image slot spec \"sessions.empty-state\" breaks 2 rule(s); \
                 job: does not match {NON_BLANK_PATTERN}; contexts.dpr: must be strictly ascending"
            )
        );
    }

    // ── content hash ─────────────────────────────────────────────────────

    #[test]
    fn content_hash_ignores_the_binding_and_follows_every_authored_edit() {
        let spec = empty_state();
        let hash = spec.content_sha256().unwrap();
        assert!(is_sha256_hex(&hash), "{hash}");

        let mut bound = spec.clone();
        bound.provenance = Some(generated_binding());
        assert_eq!(bound.content_sha256().unwrap(), hash);

        let mut edited = spec.clone();
        edited.content.must_not.push("text".into());
        assert_ne!(edited.content_sha256().unwrap(), hash);

        let mut edited = spec;
        edited.focal.must_survive_crop = false;
        assert_ne!(edited.content_sha256().unwrap(), hash);
    }

    #[test]
    fn content_hash_reads_absent_and_explicitly_empty_allowed_as_one_spec() {
        // `allowed` defaults to empty, so an author who omits it and one who
        // writes `allowed: []` declare the same spec and must hash alike.
        let mut wire = serde_json::to_value(empty_state()).unwrap();
        wire["aspect"] = json!({"preferred": "4/3"});
        let omitted: ImageSlotSpec = serde_json::from_value(wire.clone()).unwrap();
        wire["aspect"] = json!({"preferred": "4/3", "allowed": []});
        let explicit: ImageSlotSpec = serde_json::from_value(wire).unwrap();
        assert_eq!(
            omitted.content_sha256().unwrap(),
            explicit.content_sha256().unwrap()
        );
    }
}
