//! Text over images, measured on real pixels (plan
//! 2026-10-13-image-generation-and-visual-judgment, Phase 16).
//!
//! A text element laid over an image has no single background colour: the
//! snapshot's `bg_color` is the resolved-up-the-tree opaque ancestor, which a
//! hero image is not, and sampling the frame under the text reads the glyphs
//! themselves. So the measurement takes TWO frames of one snapshot:
//!
//! - frame A, the page as it renders;
//! - frame B, the same page with the text hidden (the SDK's dev-only
//!   `control/text-visibility` makes glyphs transparent, removes
//!   `text-shadow`, and restores both).
//!
//! # The two frames must be one layout
//!
//! [`pair_frames`] refuses a pair of different sizes or scale factors, and
//! [`registered`] refuses one whose IMAGE content differs away from the text.
//! The image's box, outside every text box over it (each widened by
//! [`REGISTRATION_MARGIN_CSS`]), is cut into tiles of
//! [`REGISTRATION_TILE_CSS`] CSS px. A tile DIFFERS when more than
//! [`TILE_DIFFERS_SHARE`] of its pixels do. Misregistration — a one-pixel
//! shift, a zoom, a capture of another page — spreads across the image, so
//! the frames are refused when more than [`MAX_DIFFERING_TILE_SHARE`] of the
//! textured tiles differ, or of the flat ones (a flat tile hides a shift but
//! not a different page). Text over the image that the snapshot does not
//! carry (a photo credit, a tagline) forms a compact cluster of a few tiles
//! and is tolerated. A differing pixel that moved AWAY from the colour of
//! every text over the image is a removed `text-shadow` glow, not
//! misregistration, and is not counted. The flat arm needs at least
//! [`MIN_FLAT_TILES`] flat tiles.
//!
//! That global check cannot see a layer displaced under PART of the image (a
//! parallax layer moving a few px under the text). So each text element is
//! also checked LOCALLY ([`Registration::local`]): a differing, textured
//! tile in the ring around its box is DISPLACED image unless its difference
//! reads as alpha-blended text — every differing pixel moved toward one
//! colour — which untracked text that only frame A carries always does and
//! moved texture, at any offset, does not. Any displaced tile right beside
//! the text refuses on its own, and the refusal names its position.
//! PRODUCERS SHOULD PAUSE ANIMATIONS (spinners, carousels, chevrons, CSS
//! transitions) for both captures: anything that moves between them near the
//! text reads as misregistration, and the text is not measured.
//!
//! # Where the glyphs are
//!
//! The two frames differ exactly where glyphs paint. The GLYPH MASK of a
//! text element is the set of pixels inside its box where frames A and B
//! differ by more than [`GLYPH_DIFF_MIN`] AND the pixel moved TOWARD THE
//! GLYPH COLOUR (closer to it in A than in B): a glyph and its alpha-blended
//! edge always do, while a dark `text-shadow` halo over a light image moved
//! away and is left out — before the density checks, so a glow neither
//! fills the mask nor counts toward its ceiling. Frame B is sampled under the
//! mask's pixels over the image: what the glyphs actually sit on. Sampling
//! the whole box instead would read unrelated parts of the image — a
//! block-level heading's box spans the hero's full width while its glyphs
//! cover the left third.
//!
//! A mask is usable when it holds at least [`MIN_GLYPH_CSS_PX2`] CSS px² of
//! glyphs (scaled by the frame's scale factor squared) and covers at most
//! [`MAX_GLYPH_SHARE`] of the box. There is no floor on the SHARE: a
//! two-letter label in a full-width block box covers a fraction of a percent
//! of it. An unusable mask is UNKNOWN: an empty or sparse one means the text
//! was not hidden in frame B or that it is invisible against the image, and
//! the two cannot be told apart. Frame B that STILL PAINTS the glyphs (a
//! non-deterministic raster of an unhidden twin) is Unknown too: when more
//! than [`STILL_PAINTED_SHARE`] of the mask's pixels have a glyph-coloured
//! frame-B pixel within one CSS px, the glyph is still there.
//!
//! # Glyphs the mask cannot see
//!
//! Where the image under a glyph is within [`GLYPH_DIFF_MIN`] of the glyph
//! colour (blown-out sky, snow, a white studio backdrop under white text;
//! crushed shadows under black), A and B differ by less than the mask's
//! threshold, and those are exactly the invisible glyph pixels. A glyph that
//! covers a pixel only partly moves it less: at the sampling floor's coverage
//! `m` ([`MIN_GLYPH_COVERAGE_SHARE`] below), the mask misses it on any
//! background within `GLYPH_DIFF_MIN / m` of the glyph colour. Those pixels
//! are found from frame B alone: the BLIND pixels of the overlap, within
//! `GLYPH_DIFF_MIN / m` of the glyph colour. A blind pixel
//! that differs at all and moved toward the glyph colour is a FAINT glyph
//! pixel and is sampled like any other — when the registration check found
//! the capture free of sub-threshold noise ([`FAINT_NOISE_MAX`]); otherwise
//! it is treated as undetectable. With a noise-free capture only a blind
//! pixel EXACTLY the glyph colour can hide a glyph (on any other background
//! the glyph's core differs, and was seen), so only those are undetectable;
//! with a noisy one, every blind pixel is. EVERY undetectable pixel counts
//! as ink: no visible band bounds the density of a hidden word, which may be
//! heavier than any visible one (a density taken from the visible line passed
//! a heavy glyph on a tight patch, the `heavy_m` / `heavy_w` fixtures). The
//! cost — a few true passes read as Unknown — was measured during review on a
//! private corpus of real photos, which this repository does not carry; see
//! PR qontinui/qontinui-schemas#257.
//! [`TextOverImageContrast::percentile_ratio_with_hidden`] is the
//! percentile with those pixels added at their own contrasts. A check passes
//! only on that value, fails on the measured one, and is Unknown in between:
//! the hidden ink may or may not be there.
//!
//! A mask pixel is SAMPLED only where the glyph covers it at least
//! [`MIN_GLYPH_COVERAGE_SHARE`] of the best-covered pixel of its mask, and at
//! least [`MIN_GLYPH_COVERAGE_ABS`] (α = (A−B)·(c−B)/|c−B|² under alpha
//! blending): a low-coverage edge clears the difference threshold more
//! readily over a background far from the glyph colour, which weighted the
//! sample toward high contrast. Relative, so a true hairline face whose ink
//! never reaches full coverage is still measured. Faint pixels are sampled
//! whatever their coverage.
//!
//! # Evidence of the hide, and the limits without it
//!
//! Whether frame B hid a given text element is otherwise INFERRED from the
//! pixels. A producer that can say what it hid passes the ids as
//! [`crate::analyzers::AnalyzeInput::text_hidden_ids`] (CLI
//! `--text-hidden-ids`): a text element not in the list is then Unknown
//! ([`not_reported_hidden`]), and one in it skips the still-painted inference.
//! Without the list, two known limits remain:
//!
//! - glyph-coloured background CONFINED under the text with a dark ring
//!   around it reads "still painted" (Unknown) instead of a failure: the
//!   image's own rate is read from the ring, which shows none;
//! - an UNHIDDEN re-rasterised twin over a near-white hero reads as a
//!   failure: the ring is glyph-coloured, so the chance rate explains the
//!   neighbours and the inference cannot fire.
//!
//! When the box shows glyphs and fewer than the floor of them reach the
//! image, the text is not over the image (a label whose line box overhangs a
//! thumbnail), and is not measured — unless the image under the box is blind
//! enough to hide the floor's worth (a box straddling the image's edge whose
//! glyphs over it sit on their own colour): that is Unknown.
//!
//! Only LEAF text elements are measured ([`TextIndex`]): a wrapper whose
//! innerText includes a child's is measured through the child, against the
//! child's colour. Text a wrapper paints directly beside its children is not
//! measured, and a wrapper over an image is disclosed as such
//! ([`PagePairs::wrappers`]).
//!
//! # Against which colour
//!
//! The ratio is WCAG 2.x contrast of the glyph colour against each sampled
//! frame-B pixel. The glyph colour is the element's declared `fg_color` when
//! the snapshot carries one: it is the colour WCAG's definition is stated in,
//! it is exact, and it is free of the anti-aliasing that blends every edge
//! pixel of a glyph in frame A into the background. Only where `fg_color` is
//! absent is the colour MEASURED from frame A, as the per-channel median of
//! the raw difference's core (the quarter of differing pixels that differ
//! most); that reading is an estimate, and its confidence is the share of
//! core pixels within [`GLYPH_COLOUR_TOLERANCE`] of the median. A heavy
//! `text-shadow` on a light image differs from frame B by more than the
//! glyph does and can take the core, so a measured colour under such a
//! shadow reads the shadow's colour: declare `fg_color`. The declared colour
//! cannot see a translucent ancestor (`opacity`) or a blend mode; that is the
//! price of preferring it.
//!
//! # What is reported
//!
//! The minimum ratio and the `percentile`-th percentile (nearest rank;
//! [`DEFAULT_PERCENTILE`] = 5) over the sampled pixels, and the percentile
//! with the estimated hidden ink. The minimum is one pixel's word — a single
//! bright speck under a stroke sets it — so a check gates on the percentile.
//!
//! APCA is not computed: the APCA-W3 reference implementation is not under a
//! permissive open-source licence, and the plan adds it only after a licence
//! read admits it.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::{unreadable, unrendered, Ancestry, LocateError};
use crate::element_snapshot::{intersection, region_contains, Element, ElementSnapshot, Rgb};
use crate::frame::{Frame, Region};
use crate::observation::UnknownCode;

/// The percentile reported (and gated on, unless the caller says otherwise).
pub const DEFAULT_PERCENTILE: f64 = 5.0;

/// Smallest RGB (Euclidean) distance between frames A and B that marks a
/// glyph pixel. The same 8 the image analyzer's `edge_contrast` uses to tell
/// stroke pixels from their surround: above capture noise, below the
/// difference any visible glyph paints. Glyph pixels on a background within
/// this distance of the glyph colour are the BLIND ones (module docs).
pub const GLYPH_DIFF_MIN: f64 = 8.0;

/// Fewest glyph pixels a usable mask holds, in CSS px² (a dot of a 6 px
/// font's `i`); the buffer floor is this times the scale factor squared.
pub const MIN_GLYPH_CSS_PX2: f64 = 6.0;

/// Largest share of the text box a usable mask covers. Glyphs of even a
/// heavy face in a tight box cover well under this; a larger difference
/// toward the glyph colour is not glyphs.
pub const MAX_GLYPH_SHARE: f64 = 0.70;

/// How far, in CSS px, each text box is widened before the registration
/// check, for anti-aliasing.
pub const REGISTRATION_MARGIN_CSS: f64 = 4.0;

/// The registration check's tile side, in CSS px.
pub const REGISTRATION_TILE_CSS: f64 = 16.0;

/// Share of a tile's pixels that must differ for the tile to differ.
pub const TILE_DIFFERS_SHARE: f64 = 0.05;

/// Largest share of textured (or of flat) tiles that may differ for the two
/// frames to count as registered.
pub const MAX_DIFFERING_TILE_SHARE: f64 = 0.5;

/// Luma range (max − min) at or above which a tile is TEXTURED: a shift or a
/// zoom shows there, while a flat tile reads the same either way.
pub const TEXTURE_LUMA_RANGE: u8 = 16;

/// Largest share of the image's pixels away from the text that may differ
/// below [`GLYPH_DIFF_MIN`] (capture noise) for FAINT glyph pixels to be
/// trusted. A lossless capture of a static page reads 0.
pub const FAINT_NOISE_MAX: f64 = 0.001;

/// Share of glyph-mask pixels with a glyph-coloured frame-B pixel within
/// one CSS px — one that ALSO differs in frame A, so a glyph-coloured speckle
/// of the image itself does not count — above which frame B still paints the
/// glyphs (an unhidden twin whose raster moved a sub-pixel). Measured 0% on
/// every hidden fixture, 1.2% with half the glyphs over near-white, and 45.6%
/// on a quarter-pixel re-raster. Read as an EXCESS over the rate the image's
/// own glyph-coloured pixels away from the glyphs explain (a 253 / 225 mottle
/// explains nearly all of it), relative to the headroom left.
pub const STILL_PAINTED_SHARE: f64 = 0.2;

/// Fewest flat tiles before the flat arm of the global registration check
/// can refuse: one flat tile is no evidence of a different capture.
pub const MIN_FLAT_TILES: usize = 4;

/// Width, in tiles, of the ring around each text box that the LOCAL
/// registration check reads ([`Registration::local`]).
pub const LOCAL_RING_TILES: i64 = 2;

/// Largest displacement, in CSS px, the local check's shift test tries on a
/// text-like tile (a parallax or a sticky panel moves further than a
/// sub-pixel re-layout).
pub const LOCAL_MAX_SHIFT_CSS: f64 = 12.0;

/// Share of a ring tile's differing pixels that must have moved toward one
/// colour for the tile to read as untracked text rather than moved image.
pub const LOCAL_TEXT_LIKE_SHARE: f64 = 0.9;

/// Share of the ring's tiles that must show displaced content for the local
/// check to refuse.
pub const LOCAL_DISPLACED_TILE_SHARE: f64 = 0.1;

/// Smallest estimated glyph coverage ([`coverage`]) of a sampled mask pixel,
/// as a share of the highest coverage in the mask over the image: a hairline
/// face whose ink peaks at 45% is sampled from about 22%. Faint pixels (on a
/// background within [`GLYPH_DIFF_MIN`] of the glyph colour) are sampled
/// whatever their coverage.
pub const MIN_GLYPH_COVERAGE_SHARE: f64 = 0.5;

/// The absolute floor under [`MIN_GLYPH_COVERAGE_SHARE`], so a mask of
/// nothing but faint edges or noise never qualifies on its own scale.
pub const MIN_GLYPH_COVERAGE_ABS: f64 = 0.15;

/// RGB distance within which a core glyph pixel agrees with the measured
/// glyph colour.
pub const GLYPH_COLOUR_TOLERANCE: f64 = 24.0;

/// Largest scale factor (device pixel ratio) a frame pair is measured at.
/// The work grows with its square and more (the local check's shift search
/// tries `(2·⌈12·s⌉+1)²` offsets), and no display ships one above 4; a larger
/// value is a mistyped input, refused as Unknown rather than clamped.
pub const MAX_SCALE_FACTOR: f64 = 8.0;

/// A frame's scale factor, refused unless it is finite, positive and at most
/// [`MAX_SCALE_FACTOR`].
pub fn usable_scale(s: f64) -> Result<f64, LocateError> {
    if s.is_finite() && s > 0.0 && s <= MAX_SCALE_FACTOR {
        Ok(s)
    } else {
        Err((
            UnknownCode::InputMissing,
            format!(
                "the frame's scale factor {s} is outside 0 < s <= {MAX_SCALE_FACTOR}: no display \
                 captures at that ratio, and the glyph search would not finish, so no contrast \
                 was measured"
            ),
        ))
    }
}

/// Where the glyph colour came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlyphColourSource {
    /// The snapshot's `fg_color`.
    Declared,
    /// Measured from frame A under the glyph mask (no `fg_color`).
    Measured,
}

/// The contrast of one text element over one image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextOverImageContrast {
    /// The text element's id.
    pub element: String,
    /// The image element's id.
    pub image: String,
    /// The lowest WCAG ratio over the sampled pixels.
    pub min_ratio: f64,
    /// The percentile reported in [`Self::percentile_ratio`] (0..=100).
    pub percentile: f64,
    /// The `percentile`-th percentile WCAG ratio (nearest rank) over the
    /// sampled glyph pixels. A FAILURE is decided on this.
    pub percentile_ratio: f64,
    /// The same percentile with [`Self::hidden_px_estimate`] samples added
    /// at the blind pixels' contrasts: the reading if the glyphs the mask
    /// cannot see are there. A PASS is decided on this; it equals
    /// [`Self::percentile_ratio`] when nothing is hidden.
    pub percentile_ratio_with_hidden: f64,
    /// The colour the ratios are computed against.
    pub glyph_colour: Rgb,
    pub glyph_colour_source: GlyphColourSource,
    /// How many frame-B pixels were sampled (glyph pixels over the image),
    /// faint ones included.
    pub sampled_px: u64,
    /// Of [`Self::sampled_px`], the FAINT glyph pixels: on a BLIND
    /// background (within `GLYPH_DIFF_MIN` over the sampling floor's coverage
    /// of the glyph colour; module docs), found by any difference at all.
    pub faint_px: u64,
    /// The estimated glyph pixels on such a background that no difference
    /// shows (module docs).
    pub hidden_px_estimate: u64,
    /// The examined buffer region: the text box clipped to the image box,
    /// mapped onto the frame and clamped to it.
    pub region: Region,
    /// 0..=1: the share of the examined box inside the frame, times the
    /// measured glyph colour's agreement when it was measured.
    pub confidence: f64,
}

/// Whether `b` can serve as `a`'s text-hidden twin: the same buffer size and
/// the same scale factor. A pair that differs in either is not two captures
/// of one layout, and no pixel of one can be read at the other's position.
/// That the CONTENT lines up is [`registered`]'s question.
pub fn pair_frames(a: &Frame, b: &Frame) -> Result<(), LocateError> {
    if (a.width, a.height) != (b.width, b.height) {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the text-hidden frame is {}x{} px and the frame is {}x{} px: two captures of \
                 one layout share their size, so no glyph pixel can be paired across them",
                b.width, b.height, a.width, a.height
            ),
        ));
    }
    let (sa, sb) = (a.source.scale_factor, b.source.scale_factor);
    if !(sa.is_finite() && sb.is_finite() && (sa - sb).abs() <= 1e-9 * sa.abs().max(1.0)) {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the text-hidden frame was captured at scale factor {sb} and the frame at {sa}: \
                 the same CSS box lands on different pixels, so no glyph pixel can be paired"
            ),
        ));
    }
    usable_scale(sa)?;
    Ok(())
}

/// One text element over one image, and the CSS-px region where they overlap.
#[derive(Debug, Clone, Copy)]
pub struct TextOverImage<'a> {
    pub text: &'a Element,
    pub image: &'a Element,
    pub css: Region,
}

/// Whether `t` is a text element that paints: non-blank text, not itself
/// media, not [`unrendered`].
fn paints_text(snapshot: &ElementSnapshot, t: &Element) -> bool {
    t.media.is_none()
        && t.text.as_deref().is_some_and(|s| !s.trim().is_empty())
        && unrendered(snapshot, t).is_none()
}

/// Which painted text elements are LEAVES: no other painted text element
/// descends from them (`parent_id` chain or `children_ids`, as
/// [`super::drop_nested`] reads it). Built once per snapshot.
#[derive(Debug)]
pub struct TextIndex<'a> {
    leaves: HashSet<&'a str>,
}

impl<'a> TextIndex<'a> {
    pub fn of(snapshot: &'a ElementSnapshot) -> Self {
        let ancestry = Ancestry::of(snapshot);
        let texts: Vec<&'a str> = snapshot
            .elements
            .iter()
            .filter(|t| paints_text(snapshot, t))
            .map(|t| t.id.as_str())
            .collect();
        let ancestors: Vec<HashSet<&'a str>> =
            texts.iter().map(|id| ancestry.ancestors(id)).collect();
        // `w` wraps `c` when `w` is among `c`'s ancestors and not the other
        // way round (a cycle in the producer's chain is not a nesting).
        let wrappers: HashSet<&'a str> = texts
            .iter()
            .enumerate()
            .filter(|(wi, w)| {
                texts.iter().enumerate().any(|(ci, _)| {
                    ci != *wi && ancestors[ci].contains(*w) && !ancestors[*wi].contains(texts[ci])
                })
            })
            .map(|(_, w)| *w)
            .collect();
        Self {
            leaves: texts
                .into_iter()
                .filter(|t| !wrappers.contains(t))
                .collect(),
        }
    }

    /// Whether `id` is a painted leaf text element.
    pub fn is_leaf(&self, id: &str) -> bool {
        self.leaves.contains(id)
    }
}

/// The text over one image ([`texts_over`]).
#[derive(Debug, Default)]
pub struct Over<'a> {
    /// The leaf text elements over it, each with the overlap region.
    pub pairs: Vec<TextOverImage<'a>>,
    /// Leaf text elements over it that cannot themselves be attributed to
    /// the frame ([`unreadable`]).
    pub unreadable_texts: usize,
    /// Wrapper text elements over it: measured through their leaf children,
    /// so any text they paint beside those children is not measured.
    pub wrappers: usize,
}

/// The LEAF text elements laid over `image` ([`TextIndex`]). A WRAPPER box
/// that contains the image whole is a container (a section, a card, a
/// caption wrapper), not an overlay, and is skipped. A LEAF box that contains
/// it is not skipped on its box: an inset-0 overlay whose text node is its
/// own has exactly the image's box, and its glyphs lie over the image. The
/// glyph test decides ([`measure`]'s `None` when they do not reach it).
///
/// The image itself is gated by the caller on [`unrendered`] only: an
/// OCCLUDED image (the usual `<img>` hero under an inset-0 scrim or text
/// layer) is measured, because nothing here reads the image's crop — frame B
/// shows whatever the glyphs sit on.
pub fn texts_over<'a>(
    snapshot: &'a ElementSnapshot,
    image: &'a Element,
    index: &TextIndex<'_>,
) -> Over<'a> {
    let mut over = Over::default();
    let Some(img) = image.bbox else {
        return over;
    };
    for t in &snapshot.elements {
        if t.id == image.id || !paints_text(snapshot, t) {
            continue;
        }
        let Some(tb) = t.bbox else { continue };
        let Some(css) = intersection(tb, img) else {
            continue;
        };
        if !index.is_leaf(&t.id) {
            if !region_contains(tb, img) {
                over.wrappers += 1;
            }
            continue;
        }
        if unreadable(snapshot, t).is_some() {
            over.unreadable_texts += 1;
            continue;
        }
        over.pairs.push(TextOverImage {
            text: t,
            image,
            css,
        });
    }
    over
}

/// Every text-over-image pair on the page, and what was left out.
#[derive(Debug, Default)]
pub struct PagePairs<'a> {
    pub pairs: Vec<TextOverImage<'a>>,
    /// Unrendered images (they paint nothing) with text over them.
    pub skipped_images: usize,
    /// Text elements over an image that cannot themselves be attributed to
    /// the frame.
    pub skipped_texts: usize,
    /// Wrapper text elements over an image, measured through their children
    /// only (see [`Over::wrappers`]).
    pub wrappers: usize,
}

impl PagePairs<'_> {
    /// The disclosure a page-wide detail carries.
    pub fn note(&self) -> String {
        let mut note = String::new();
        if self.skipped_images > 0 {
            note.push_str(&format!(
                " ({} unrendered image(s) with text over them skipped)",
                self.skipped_images
            ));
        }
        if self.skipped_texts > 0 {
            note.push_str(&format!(
                " ({} text element(s) over an image skipped: reported not visible)",
                self.skipped_texts
            ));
        }
        if self.wrappers > 0 {
            note.push_str(&format!(
                " ({} wrapper text element(s) over an image measured through their children \
                 only: text they paint beside them is not measured)",
                self.wrappers
            ));
        }
        note
    }

    /// The distinct images the pairs lie over, in snapshot order.
    pub fn images(&self) -> Vec<&Element> {
        let mut seen = HashSet::new();
        self.pairs
            .iter()
            .filter(|p| seen.insert(p.image.id.as_str()))
            .map(|p| p.image)
            .collect()
    }
}

/// [`texts_over`] every rendered media element of the snapshot.
pub fn page_pairs(snapshot: &ElementSnapshot) -> PagePairs<'_> {
    let index = TextIndex::of(snapshot);
    let mut page = PagePairs::default();
    for image in snapshot.elements.iter().filter(|e| e.media.is_some()) {
        let over = texts_over(snapshot, image, &index);
        if unrendered(snapshot, image).is_some() {
            if !over.pairs.is_empty() || over.unreadable_texts > 0 {
                page.skipped_images += 1;
            }
            continue;
        }
        page.skipped_texts += over.unreadable_texts;
        page.wrappers += over.wrappers;
        page.pairs.extend(over.pairs);
    }
    page
}

// ---------------------------------------------------------------------------
// Pixels
// ---------------------------------------------------------------------------

fn rgb_at(frame: &Frame, x: u32, y: u32) -> Rgb {
    let p = frame.buffer.get_pixel(x, y).0;
    Rgb::new(p[0], p[1], p[2])
}

fn dist2(a: Rgb, b: Rgb) -> f64 {
    let (dr, dg, db) = (
        a.r as f64 - b.r as f64,
        a.g as f64 - b.g as f64,
        a.b as f64 - b.b as f64,
    );
    dr * dr + dg * dg + db * db
}

/// The glyph's estimated coverage of a pixel, α = (A−B)·(c−B) / |c−B|²: the
/// share of the way from B to the glyph colour `c` that A moved, under alpha
/// blending. `0.0` where B is the glyph colour itself.
fn coverage(pa: Rgb, pb: Rgb, c: Rgb) -> f64 {
    let d = |p: u8, q: u8| p as f64 - q as f64;
    let (ar, ag, ab) = (d(pa.r, pb.r), d(pa.g, pb.g), d(pa.b, pb.b));
    let (cr, cg, cb) = (d(c.r, pb.r), d(c.g, pb.g), d(c.b, pb.b));
    let n = cr * cr + cg * cg + cb * cb;
    if n == 0.0 {
        0.0
    } else {
        (ar * cr + ag * cg + ab * cb) / n
    }
}

/// Whether a pixel moved toward `colour` between frame B and frame A: a
/// glyph (or its alpha-blended edge) painted over B, not a halo.
fn toward(pa: Rgb, pb: Rgb, colour: Rgb) -> bool {
    dist2(pa, colour) < dist2(pb, colour)
}

fn luma(c: Rgb) -> u8 {
    ((c.r as u32 * 299 + c.g as u32 * 587 + c.b as u32 * 114) / 1000) as u8
}

/// The sRGB channel linearisation, once per value: the same formula as
/// [`Rgb::relative_luminance`], so [`contrast`] equals
/// [`crate::analyzers::color::wcag_contrast`] exactly.
fn linear(c: u8) -> f64 {
    static LUT: OnceLock<[f64; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0; 256];
        for (i, v) in t.iter_mut().enumerate() {
            let s = i as f64 / 255.0;
            *v = if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            };
        }
        t
    })[c as usize]
}

fn luminance(c: Rgb) -> f64 {
    0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
}

/// WCAG contrast between two relative luminances.
fn contrast(la: f64, lb: f64) -> f64 {
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// The nearest-rank `percentile` (0..=100) of `sorted` (ascending, non-empty).
/// The rank is `percentile * n / 100`, multiplied first: `p / 100 * n` reads
/// 7 / 100 * 100 as 7.000000000000001 and takes the eighth value.
pub fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    let n = sorted.len();
    let rank = (percentile * n as f64 / 100.0).ceil() as usize;
    sorted[rank.clamp(1, n) - 1]
}

/// The buffer-px floor of a usable glyph mask at scale factor `s`.
pub fn min_glyph_px(s: f64) -> usize {
    ((MIN_GLYPH_CSS_PX2 * s * s).ceil() as usize).max(1)
}

/// Whether a glyph mask of `glyphs` toward-glyph pixels over a box of `area`
/// pixels reads as glyphs at scale factor `s` (see the module docs).
pub fn mask_usable(glyphs: usize, area: usize, s: f64) -> bool {
    glyphs >= min_glyph_px(s) && glyphs as f64 <= MAX_GLYPH_SHARE * area.max(1) as f64
}

/// A CSS-px box on the frame: mapped through the scale factor and clamped,
/// as `(x, y, w, h)` buffer indices plus the share of it inside the frame.
fn on_frame(frame: &Frame, css: Region) -> Option<((u32, u32, u32, u32), f64)> {
    let mapped = frame.css_region_to_pixels(css)?;
    let px = mapped
        .clamp_to_frame(frame.width, frame.height)
        .filter(|(_, _, w, h)| *w > 0 && *h > 0)?;
    let in_frame = (px.2 as f64 * px.3 as f64 / (mapped.w as f64 * mapped.h as f64)).min(1.0);
    Some((px, in_frame))
}

/// One pass over `px`: the pixels where frames `a` and `b` differ by more
/// than [`GLYPH_DIFF_MIN`] and that moved toward `colour` (`None`: every
/// differing pixel), as `(x, y, squared distance)`, and how many differed
/// at all past the threshold in any direction. `None` as soon as more than
/// `cap` pixels are kept: the caller has no use for a mask that dense.
#[allow(clippy::type_complexity)]
fn diff_mask(
    a: &Frame,
    b: &Frame,
    (x0, y0, w, h): (u32, u32, u32, u32),
    colour: Option<Rgb>,
    cap: usize,
) -> Option<(Vec<(u32, u32, f64)>, usize)> {
    let min2 = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let mut mask = Vec::new();
    let mut raw = 0usize;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
            let d = dist2(pa, pb);
            if d <= min2 {
                continue;
            }
            raw += 1;
            if colour.is_none_or(|c| toward(pa, pb, c)) {
                if mask.len() == cap {
                    return None;
                }
                mask.push((x, y, d));
            }
        }
    }
    Some((mask, raw))
}

/// What [`registered`] found about the capture over one image, and what the
/// per-text LOCAL check ([`Registration::local`]) needs: the image's buffer
/// rect, its hole bitmap (the widened text boxes, computed once) and the
/// text colours a removed glow moved away from.
#[derive(Debug, Clone, PartialEq)]
pub struct Registration {
    /// Share of the image's pixels away from the text that differ below
    /// [`GLYPH_DIFF_MIN`]: the capture's noise.
    pub noise_share: f64,
    rect: (u32, u32, u32, u32),
    holes: Vec<bool>,
    colours: Vec<Rgb>,
    tile: u32,
}

impl Registration {
    /// Whether a sub-threshold difference can be read as a FAINT glyph.
    pub fn faint_trusted(&self) -> bool {
        self.noise_share <= FAINT_NOISE_MAX
    }

    /// Whether buffer pixel `(x, y)` lies in a hole (a widened text box).
    /// Pixels outside the image rect are not the image's: treated as holes.
    fn hole(&self, x: u32, y: u32) -> bool {
        let (ix, iy, iw, ih) = self.rect;
        if x < ix || y < iy || x >= ix + iw || y >= iy + ih {
            return true;
        }
        self.holes[((y - iy) * iw + (x - ix)) as usize]
    }

    /// The ring [`LOCAL_RING_TILES`] tiles wide around `text_box`'s widened
    /// box, clipped to the image rect, as half-open buffer bounds.
    fn ring(&self, a: &Frame, text_box: Region) -> Option<(i64, i64, i64, i64)> {
        let s = a.source.scale_factor;
        let r = a.css_region_to_pixels(text_box)?;
        let margin =
            (REGISTRATION_MARGIN_CSS * s).ceil() as i64 + LOCAL_RING_TILES * self.tile as i64;
        let (ix, iy, iw, ih) = self.rect;
        let x0 = (r.x as i64 - margin).max(ix as i64);
        let y0 = (r.y as i64 - margin).max(iy as i64);
        let x1 = (r.right() + margin).min((ix + iw) as i64);
        let y1 = (r.bottom() + margin).min((iy + ih) as i64);
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }

    /// The share of the IMAGE's frame-B pixels in the ring around
    /// `text_box` (outside every widened text box) within
    /// [`GLYPH_DIFF_MIN`] of `colour`: how often the image itself is
    /// glyph-coloured near this text. Read outside the text so an unhidden
    /// twin's own glyph cores cannot inflate it. `0.0` when the ring holds no
    /// image pixel.
    pub fn glyph_coloured_share_around(
        &self,
        a: &Frame,
        b: &Frame,
        text_box: Region,
        colour: Rgb,
    ) -> f64 {
        let Some((x0, y0, x1, y1)) = self.ring(a, text_box) else {
            return 0.0;
        };
        let near = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
        let (mut n, mut k) = (0usize, 0usize);
        for y in y0..y1 {
            for x in x0..x1 {
                let (x, y) = (x as u32, y as u32);
                if self.hole(x, y) {
                    continue;
                }
                n += 1;
                k += (dist2(rgb_at(b, x, y), colour) <= near) as usize;
            }
        }
        k as f64 / n.max(1) as f64
    }

    /// The LOCAL registration check for one text element: the tiles in a
    /// ring [`LOCAL_RING_TILES`] tiles wide around its widened box, inside
    /// the image, must not show content DISPLACED between the frames.
    ///
    /// A differing, textured ring tile is displaced unless its difference
    /// reads as TEXT that only frame A carries (an untracked tagline, a
    /// credit). Text is alpha-blended over the image, so every one of its
    /// differing pixels moved toward ONE colour, its own: at least
    /// [`LOCAL_TEXT_LIKE_SHARE`] of the tile's differing pixels must lie
    /// closer in A than in B to the colour of the tile's most-changed
    /// quarter. Moved texture — a parallax layer at any offset, a rotation, a
    /// rescale — goes both ways (a median 52–71% per tile under 1–6 px parallax
    /// on a textured layer; untracked 12–18 px text, any colour, 100%).
    /// A text-like tile — few differing pixels can look text-like by chance —
    /// is still displaced when one shift of frame B by up to
    /// [`LOCAL_MAX_SHIFT_CSS`] CSS px lines up more than half of its
    /// differing pixels: moved image does, text only frame A carries does not. Refused when at
    /// least [`LOCAL_DISPLACED_TILE_SHARE`] of the ring's tiles (and at least
    /// one) are displaced, or when ANY tile of the ring's first row — within
    /// one tile of the widened text box — is: a panel moving just under the
    /// text shows only there. The global check cannot see this: a layer under
    /// part of the hero is under half its tiles.
    pub fn local(&self, a: &Frame, b: &Frame, text_box: Region) -> Result<(), LocateError> {
        let s = a.source.scale_factor;
        let ts = self.tile as i64;
        let Some((x0, y0, x1, y1)) = self.ring(a, text_box) else {
            return Ok(());
        };
        // The first row of tiles around the hole: within one tile of the
        // widened text box. A layer moving just under the text shows there
        // first, and one displaced tile there is enough.
        let first_row = a.css_region_to_pixels(text_box).map(|r| {
            let m = (REGISTRATION_MARGIN_CSS * s).ceil() as i64 + ts;
            (
                r.x as i64 - m,
                r.y as i64 - m,
                r.right() + m,
                r.bottom() + m,
            )
        });
        let mut adjacent_displaced = 0usize;
        // The first displaced first-row tile, named in the refusal.
        let mut first_at: Option<(i64, i64)> = None;
        let reach = (LOCAL_MAX_SHIFT_CSS * s).ceil().max(1.0) as i64;
        let min2 = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
        let (fw, fh) = (a.width as i64, a.height as i64);
        let (mut tiles, mut displaced) = (0usize, 0usize);
        let mut ty = y0;
        while ty < y1 {
            let mut tx = x0;
            while tx < x1 {
                let mut px: Vec<(u32, u32)> = Vec::new();
                let mut diffs: Vec<(Rgb, Rgb, f64)> = Vec::new();
                let (mut lo, mut hi) = (u8::MAX, u8::MIN);
                for y in ty..(ty + ts).min(y1) {
                    for x in tx..(tx + ts).min(x1) {
                        let (x, y) = (x as u32, y as u32);
                        if self.hole(x, y) {
                            continue;
                        }
                        px.push((x, y));
                        let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
                        let l = luma(pb);
                        lo = lo.min(l);
                        hi = hi.max(l);
                        let d = dist2(pa, pb);
                        if d > min2 && !self.halo(pa, pb) {
                            diffs.push((pa, pb, d));
                        }
                    }
                }
                let adjacent = first_row.is_some_and(|(fx0, fy0, fx1, fy1)| {
                    tx < fx1 && tx + ts > fx0 && ty < fy1 && ty + ts > fy0
                });
                tx += ts;
                if px.len() as u64 * 4 < ts as u64 * ts as u64 {
                    continue;
                }
                tiles += 1;
                if diffs.len() as f64 <= TILE_DIFFERS_SHARE * px.len() as f64
                    || hi.saturating_sub(lo) < TEXTURE_LUMA_RANGE
                {
                    continue;
                }
                if !text_like(&mut diffs) {
                    displaced += 1;
                    if adjacent {
                        adjacent_displaced += 1;
                        first_at.get_or_insert((tx - ts, ty));
                    }
                    continue;
                }
                // The shift test reads only the DIFFERING pixels: whether one
                // displacement of frame B lines up most of them. Static image
                // elsewhere in the tile does not vote.
                let moved: Vec<(u32, u32)> = px
                    .iter()
                    .copied()
                    .filter(|(x, y)| dist2(rgb_at(a, *x, *y), rgb_at(b, *x, *y)) > min2)
                    .collect();
                let base = moved.len();
                // Displaced when SOME shift leaves fewer than half of them
                // unmatched. A shift's count stops as soon as it reaches half,
                // and the search at the first shift that lines up: the same
                // answer as the minimum over every shift, without reading
                // every pixel at every offset.
                let lines_up = (-reach..=reach)
                    .flat_map(|dy| (-reach..=reach).map(move |dx| (dx, dy)))
                    .filter(|d| *d != (0, 0))
                    .any(|(dx, dy)| {
                        let mut misses = 0usize;
                        for (x, y) in &moved {
                            let (nx, ny) = (*x as i64 + dx, *y as i64 + dy);
                            let miss = nx < 0
                                || ny < 0
                                || nx >= fw
                                || ny >= fh
                                || dist2(rgb_at(a, *x, *y), rgb_at(b, nx as u32, ny as u32)) > min2;
                            if miss {
                                misses += 1;
                                if misses * 2 >= base {
                                    return false;
                                }
                            }
                        }
                        misses * 2 < base
                    });
                if lines_up {
                    displaced += 1;
                    if adjacent {
                        adjacent_displaced += 1;
                        first_at.get_or_insert((tx - ts, ty));
                    }
                }
            }
            ty += ts;
        }
        if adjacent_displaced > 0
            || (displaced > 0 && displaced as f64 >= LOCAL_DISPLACED_TILE_SHARE * tiles as f64)
        {
            let next_to = first_at
                .map(|(x, y)| {
                    format!(
                        "; {adjacent_displaced} of them right beside the text, the first at \
                         buffer px ({x}, {y}), CSS px ({:.0}, {:.0}) — an animation there \
                         (a spinner, a chevron) moves the same way; pause animations for \
                         both captures",
                        x as f64 / s,
                        y as f64 / s
                    )
                })
                .unwrap_or_default();
            return Err((
                UnknownCode::InputMissing,
                format!(
                    "the text-hidden frame is not registered with the frame around this text: \
                     {displaced} of the {tiles} {REGISTRATION_TILE_CSS}-CSS-px tile(s) around it \
                     ({:.1}%) show the image displaced between the captures (a parallax layer, \
                     a scroll-linked transform), not text that only the frame carries{next_to}",
                    displaced as f64 * 100.0 / tiles.max(1) as f64
                ),
            ));
        }
        Ok(())
    }

    /// A removed glow: the pixel moved AWAY from every text colour.
    fn halo(&self, pa: Rgb, pb: Rgb) -> bool {
        !self.colours.is_empty() && self.colours.iter().all(|c| !toward(pa, pb, *c))
    }
}

/// Whether a tile's differing pixels `(A, B, squared distance)` read as
/// alpha-blended text: at least [`LOCAL_TEXT_LIKE_SHARE`] of them moved
/// toward the colour of the most-changed quarter (per-channel median of A).
fn text_like(diffs: &mut [(Rgb, Rgb, f64)]) -> bool {
    if diffs.is_empty() {
        return true;
    }
    diffs.sort_by(|p, q| q.2.total_cmp(&p.2));
    let core = &diffs[..diffs.len().div_ceil(4)];
    let median = |f: fn(&Rgb) -> u8| {
        let mut v: Vec<u8> = core.iter().map(|(pa, _, _)| f(pa)).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    let c = Rgb::new(median(|c| c.r), median(|c| c.g), median(|c| c.b));
    let toward_c = diffs
        .iter()
        .filter(|(pa, pb, _)| toward(*pa, *pb, c))
        .count();
    toward_c as f64 >= LOCAL_TEXT_LIKE_SHARE * diffs.len() as f64
}

/// Whether frames `a` and `b` are REGISTERED over `image` (module docs: the
/// tile test). Every painted text box over the image counts as a hole,
/// wrappers and not-visible ones included, so no glyph is mistaken for
/// misregistration. The caller has already paired the frames
/// ([`pair_frames`]). The per-text local check is [`Registration::local`].
pub fn registered(
    a: &Frame,
    b: &Frame,
    snapshot: &ElementSnapshot,
    image: &Element,
) -> Result<Registration, LocateError> {
    let id = &image.id;
    let s = usable_scale(a.source.scale_factor)?;
    let img = image.bbox.ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!(
                "image '{id}' carries no bbox, so the two frames' registration over it is unknown"
            ),
        )
    })?;
    let (ix, iy, iw, ih) = on_frame(a, img).map(|p| p.0).ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!(
                "image '{id}' lies outside the frame, so the two frames cannot be registered \
                 over it"
            ),
        )
    })?;
    let margin = (REGISTRATION_MARGIN_CSS * s).ceil() as i64;
    let mut holes = vec![false; iw as usize * ih as usize];
    let mut colours: Vec<Rgb> = Vec::new();
    for t in &snapshot.elements {
        if t.id == image.id || !paints_text(snapshot, t) {
            continue;
        }
        let Some(tb) = t.bbox else { continue };
        // A box containing the whole image leaves no image to register if cut
        // as a hole. Its glyphs read here as untracked text instead: tolerated
        // when compact, refused (Unknown) when they spread — never a pass.
        if region_contains(tb, img) || intersection(tb, img).is_none() {
            continue;
        }
        if let Some(c) = t.fg_color {
            colours.push(c);
        }
        let Some(r) = a.css_region_to_pixels(tb) else {
            continue;
        };
        // The widened box, clipped to the image rect, marked once.
        let hx0 = (r.x as i64 - margin).max(ix as i64) - ix as i64;
        let hy0 = (r.y as i64 - margin).max(iy as i64) - iy as i64;
        let hx1 = (r.right() + margin).min((ix + iw) as i64) - ix as i64;
        let hy1 = (r.bottom() + margin).min((iy + ih) as i64) - iy as i64;
        for y in hy0.max(0)..hy1 {
            let row = y as usize * iw as usize;
            for x in hx0.max(0)..hx1 {
                holes[row + x as usize] = true;
            }
        }
    }
    let ts = ((REGISTRATION_TILE_CSS * s).ceil() as u32).max(1);
    let mut reg = Registration {
        noise_share: 0.0,
        rect: (ix, iy, iw, ih),
        holes,
        colours,
        tile: ts,
    };
    let min2 = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let (mut considered_all, mut noisy) = (0usize, 0usize);
    // (textured tiles, of them differing, flat tiles, of them differing)
    let (mut te, mut td, mut fe, mut fd) = (0usize, 0usize, 0usize, 0usize);
    let mut ty = iy;
    while ty < iy + ih {
        let th = ts.min(iy + ih - ty);
        let mut tx = ix;
        while tx < ix + iw {
            let tw = ts.min(ix + iw - tx);
            let (mut considered, mut differing) = (0usize, 0usize);
            let (mut lo, mut hi) = (u8::MAX, u8::MIN);
            for y in ty..ty + th {
                for x in tx..tx + tw {
                    if reg.hole(x, y) {
                        continue;
                    }
                    considered += 1;
                    let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
                    let l = luma(pb);
                    lo = lo.min(l);
                    hi = hi.max(l);
                    let d = dist2(pa, pb);
                    if d > min2 {
                        if !reg.halo(pa, pb) {
                            differing += 1;
                        }
                    } else if d > 0.0 {
                        noisy += 1;
                    }
                }
            }
            considered_all += considered;
            if considered as u64 * 4 >= ts as u64 * ts as u64 {
                let differs = differing as f64 > TILE_DIFFERS_SHARE * considered as f64;
                if hi.saturating_sub(lo) >= TEXTURE_LUMA_RANGE {
                    te += 1;
                    td += differs as usize;
                } else {
                    fe += 1;
                    fd += differs as usize;
                }
            }
            tx += tw;
        }
        ty += th;
    }
    if te + fe == 0 {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "no {REGISTRATION_TILE_CSS}-CSS-px tile of image '{id}' lies mostly outside its \
                 text, so the text-hidden frame's registration with the frame cannot be confirmed"
            ),
        ));
    }
    let share = |d: usize, n: usize| if n == 0 { 0.0 } else { d as f64 / n as f64 };
    let (textured, flat) = (share(td, te), share(fd, fe));
    // A single flat tile is no evidence of a different capture: the flat arm
    // needs MIN_FLAT_TILES of them.
    let flat_refuses = fe >= MIN_FLAT_TILES && flat > MAX_DIFFERING_TILE_SHARE;
    if textured > MAX_DIFFERING_TILE_SHARE || flat_refuses {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the text-hidden frame is not registered with the frame over image '{id}': \
                 {:.1}% of its textured and {:.1}% of its flat {REGISTRATION_TILE_CSS}-CSS-px \
                 tiles away from any text differ (at most {:.0}% may), so it is shifted, \
                 rescaled, mid-animation or another capture",
                textured * 100.0,
                flat * 100.0,
                MAX_DIFFERING_TILE_SHARE * 100.0
            ),
        ));
    }
    reg.noise_share = share(noisy, considered_all);
    Ok(reg)
}

/// The glyph colour measured from frame A over the mask's core, and how far
/// the core agrees with it (0..=1). `mask` is `(x, y, squared A−B distance)`.
fn measured_glyph_colour(a: &Frame, mask: &[(u32, u32, f64)]) -> (Rgb, f64) {
    let mut by_diff: Vec<&(u32, u32, f64)> = mask.iter().collect();
    by_diff.sort_by(|p, q| q.2.total_cmp(&p.2));
    let core: Vec<Rgb> = by_diff[..mask.len().div_ceil(4)]
        .iter()
        .map(|(x, y, _)| rgb_at(a, *x, *y))
        .collect();
    let median = |f: fn(&Rgb) -> u8| {
        let mut v: Vec<u8> = core.iter().map(f).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    let colour = Rgb::new(median(|c| c.r), median(|c| c.g), median(|c| c.b));
    let tol2 = GLYPH_COLOUR_TOLERANCE * GLYPH_COLOUR_TOLERANCE;
    let agree =
        core.iter().filter(|c| dist2(**c, colour) <= tol2).count() as f64 / core.len() as f64;
    (colour, agree)
}

/// Measure one text element over one image on the frame pair `(a, b)`, which
/// the caller has already paired ([`pair_frames`]) and registered over the
/// image (`reg`, from [`registered`]). See the module docs.
///
/// `Ok(None)` when the text's box shows glyphs and fewer than the glyph floor
/// of them reach the image, and the image under the box is not blind enough
/// to hide that many: not text over it. `Err` (Unknown) when the box shows no
/// usable mask, when the differing pixels moved away from the glyph colour,
/// when frame B still paints the glyphs, when the scale factor is unusable
/// ([`usable_scale`]), or when the image under the box could hide the glyphs
/// the mask did not find there.
pub fn measure(
    a: &Frame,
    b: &Frame,
    pair: &TextOverImage<'_>,
    percentile: f64,
    reg: &Registration,
    hide_reported: bool,
) -> Result<Option<TextOverImageContrast>, LocateError> {
    let id = pair.text.id.as_str();
    let s = usable_scale(a.source.scale_factor)?;
    let unknown = |detail: String| (UnknownCode::InputMissing, detail);
    let over_frame = |what: &str| {
        unknown(format!(
            "{what} of text '{id}' over '{}' cannot be placed on the frame (it lies outside it, \
             or the scale factor {s} is unusable)",
            pair.image.id
        ))
    };
    let text_box = pair.text.bbox.ok_or_else(|| over_frame("the box"))?;
    let (over_px, in_frame) = on_frame(a, pair.css).ok_or_else(|| over_frame("the overlap"))?;
    let (box_px, _) = on_frame(a, text_box).ok_or_else(|| over_frame("the box"))?;
    let area = box_px.2 as usize * box_px.3 as usize;
    let floor = min_glyph_px(s);
    let cap = (MAX_GLYPH_SHARE * area as f64) as usize;
    let dense = || {
        unknown(format!(
            "frames A and B differ toward the glyph colour at more than {:.0}% of text '{id}''s \
             box, which is not glyphs, so no contrast was measured",
            MAX_GLYPH_SHARE * 100.0
        ))
    };

    // The glyph colour first: the mask keeps only pixels that moved toward it.
    let (glyph_colour, source, colour_conf) = match pair.text.fg_color {
        Some(fg) => (fg, GlyphColourSource::Declared, 1.0),
        None => {
            let (raw, _) = diff_mask(a, b, box_px, None, area).ok_or_else(dense)?;
            if raw.len() < floor {
                return Err(not_hidden(id, raw.len(), floor));
            }
            let (c, agree) = measured_glyph_colour(a, &raw);
            (c, GlyphColourSource::Measured, agree)
        }
    };
    let (mask, raw) = diff_mask(a, b, box_px, Some(glyph_colour), cap).ok_or_else(dense)?;
    if !mask_usable(mask.len(), area, s) {
        if raw >= floor {
            return Err(unknown(format!(
                "frames A and B differ at {raw} pixel(s) of text '{id}''s box, but only {} moved \
                 toward its glyph colour rgb({},{},{}) (under the {floor} needed): the declared \
                 colour is not what paints, or only a shadow differs",
                mask.len(),
                glyph_colour.r,
                glyph_colour.g,
                glyph_colour.b
            )));
        }
        return Err(not_hidden(id, mask.len(), floor));
    }

    // Frame B still painting the glyphs: an unhidden twin, re-rasterised.
    // Where text was hidden, B around a mask pixel is background; where it
    // was not, the glyph is still there, a sub-pixel away.
    let near = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let reach = s.ceil().max(1.0) as i64;
    let (fw, fh) = (b.width as i64, b.height as i64);
    let still = mask
        .iter()
        .filter(|(x, y, _)| {
            (-reach..=reach).any(|dy| {
                (-reach..=reach).any(|dx| {
                    let (nx, ny) = (*x as i64 + dx, *y as i64 + dy);
                    if nx < 0 || ny < 0 || nx >= fw || ny >= fh {
                        return false;
                    }
                    let (na, nb) = (
                        rgb_at(a, nx as u32, ny as u32),
                        rgb_at(b, nx as u32, ny as u32),
                    );
                    na != nb && dist2(nb, glyph_colour) <= near
                })
            })
        })
        .count();
    // The rate the IMAGE itself explains: how often a pixel of the image
    // AROUND this text (outside every widened text box) is glyph-coloured in
    // B (a mottle, a near-white sky). Read outside the text, so an unhidden
    // twin's own glyph cores cannot inflate it. A neighbourhood of n such
    // pixels holds one by chance with probability 1 - (1 - q)^n; only the
    // excess over that is a still-painted glyph.
    let q = reg.glyph_coloured_share_around(a, b, text_box, glyph_colour);
    let chance = 1.0 - (1.0 - q).powi(((2 * reach + 1) * (2 * reach + 1)) as i32);
    let observed = still as f64 / mask.len() as f64;
    // A producer that REPORTED hiding this element is evidence of the hide,
    // and the heuristic below — which exists only to infer it — is skipped.
    if !hide_reported && observed - chance > STILL_PAINTED_SHARE * (1.0 - chance) {
        return Err(unknown(format!(
            "frame B still paints text '{id}': {still} of its {} glyph-mask pixel(s) have a \
             glyph-coloured pixel beside them in frame B (the image alone explains {:.1}%), so \
             the text was not hidden (a re-rasterised twin), and no contrast was measured",
            mask.len(),
            chance * 100.0
        )));
    }

    let (ox0, oy0, ow, oh) = over_px;
    let over = |x: u32, y: u32| x >= ox0 && x < ox0 + ow && y >= oy0 && y < oy0 + oh;
    let is_mask: HashSet<(u32, u32)> = mask.iter().map(|(x, y, _)| (*x, *y)).collect();
    let trusted = reg.faint_trusted();
    let mask_over = mask.iter().filter(|(x, y, _)| over(*x, *y)).count();
    let peak = mask
        .iter()
        .filter(|(x, y, _)| over(*x, *y))
        .map(|(x, y, _)| coverage(rgb_at(a, *x, *y), rgb_at(b, *x, *y), glyph_colour))
        .fold(0.0_f64, f64::max);
    let min_coverage = (MIN_GLYPH_COVERAGE_SHARE * peak).max(MIN_GLYPH_COVERAGE_ABS);
    // BLIND: a glyph covering a pixel at `min_coverage` moves it by
    // `min_coverage` times the background's distance from the glyph colour,
    // which stays at or under GLYPH_DIFF_MIN (so the mask misses a pixel the
    // sampling would keep) wherever that distance is at most
    // GLYPH_DIFF_MIN / min_coverage.
    let blind = (GLYPH_DIFF_MIN / min_coverage).powi(2);
    if mask_over < floor {
        // Few mask pixels over the image is "not over it" only if the image
        // under the box could have SHOWN glyphs. Its BLIND pixels (module
        // docs) cannot: a FAINT one is a glyph pixel the mask missed, and an
        // undetectable one may hide one. Counted the way the sampling below
        // classifies them, stopping once the floor is reached.
        let (mut faint, mut undetectable) = (0usize, 0usize);
        'scan: for y in oy0..oy0 + oh {
            for x in ox0..ox0 + ow {
                if is_mask.contains(&(x, y)) {
                    continue;
                }
                let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
                if dist2(pb, glyph_colour) > blind {
                    continue;
                }
                if trusted && pa != pb && toward(pa, pb, glyph_colour) {
                    faint += 1;
                } else if !trusted || pb == glyph_colour {
                    undetectable += 1;
                }
                if mask_over + faint >= floor {
                    break 'scan;
                }
            }
        }
        if mask_over + faint < floor {
            if mask_over + faint + undetectable < floor {
                return Ok(None);
            }
            return Err(unknown(format!(
                "only {mask_over} glyph-mask pixel(s) of text '{id}' reach '{}' (under the {floor} \
                 needed), but {undetectable} pixel(s) of the image under its box are within \
                 {:.1} of its glyph colour rgb({},{},{}), where the mask cannot see a glyph: \
                 whether its glyphs reach the image is unknown, so no contrast was \
                 measured",
                pair.image.id,
                blind.sqrt(),
                glyph_colour.r,
                glyph_colour.g,
                glyph_colour.b
            )));
        }
        // Enough FAINT glyph pixels reach the image: it is text over it, and
        // is measured below like any other.
    }
    // The image displaced under THIS text (a parallax layer) is invisible to
    // the global check; the ring around the text is read here.
    reg.local(a, b, text_box)?;

    let lg = luminance(glyph_colour);
    let mut ratios: Vec<f64> = Vec::new();
    let mut hidden: Vec<f64> = Vec::new();
    let mut faint = 0usize;
    for y in oy0..oy0 + oh {
        for x in ox0..ox0 + ow {
            let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
            let is_blind = dist2(pb, glyph_colour) <= blind;
            let r = || contrast(lg, luminance(pb));
            if is_mask.contains(&(x, y)) {
                // Only pixels the glyph covers at least half as well as its
                // best-covered one: a low-coverage edge clears the difference
                // threshold more readily over a background far from the glyph
                // colour, which would weight the sample toward high contrast.
                if coverage(pa, pb, glyph_colour) >= min_coverage {
                    ratios.push(r());
                }
            } else if is_blind {
                if trusted && pa != pb && toward(pa, pb, glyph_colour) {
                    faint += 1;
                    ratios.push(r());
                } else if !trusted || pb == glyph_colour {
                    // With a noise-free capture a glyph on any background
                    // but the glyph colour itself differs, and was seen.
                    hidden.push(r());
                }
            }
        }
    }
    if ratios.len() < floor {
        return Err(unknown(format!(
            "fewer than {floor} pixel(s) of text '{id}' over '{}' are covered at least {:.0}% by \
             its glyphs ({} are), so no contrast was measured",
            pair.image.id,
            min_coverage * 100.0,
            ratios.len()
        )));
    }
    // Every undetectable pixel counts as ink: no visible band bounds the
    // density of a hidden word, which may be heavier than any visible one.
    // Over-counting is the safe direction: it can turn a true pass into an
    // Unknown, never a failure into a pass (the module docs, "Glyphs the mask
    // cannot see").
    let estimate = hidden.len();
    ratios.sort_by(f64::total_cmp);
    hidden.sort_by(f64::total_cmp);
    let mut with_hidden = ratios.clone();
    for i in 0..estimate {
        // Spread across the undetectable pixels' own contrasts (all of them
        // while `estimate` is their count).
        with_hidden.push(hidden[(i * hidden.len()) / estimate]);
    }
    with_hidden.sort_by(f64::total_cmp);
    Ok(Some(TextOverImageContrast {
        element: id.to_string(),
        image: pair.image.id.clone(),
        min_ratio: ratios[0],
        percentile,
        percentile_ratio: nearest_rank(&ratios, percentile),
        percentile_ratio_with_hidden: nearest_rank(&with_hidden, percentile),
        glyph_colour,
        glyph_colour_source: source,
        sampled_px: ratios.len() as u64,
        faint_px: faint as u64,
        hidden_px_estimate: estimate as u64,
        region: Region {
            x: ox0 as i32,
            y: oy0 as i32,
            w: ow,
            h: oh,
        },
        confidence: (in_frame * colour_conf).clamp(0.0, 1.0),
    }))
}

/// The Unknown for a text element the producer did not report hiding, when
/// it reported a list ([`crate::analyzers::AnalyzeInput::text_hidden_ids`]).
pub fn not_reported_hidden(id: &str) -> LocateError {
    (
        UnknownCode::InputMissing,
        format!(
            "text '{id}' is not among the elements the producer reports it hid in the \
             text-hidden frame, so frame B may still paint it, and no contrast was measured"
        ),
    )
}

/// The Unknown for a mask too sparse to be glyphs.
fn not_hidden(id: &str, n: usize, floor: usize) -> LocateError {
    (
        UnknownCode::InputMissing,
        format!(
            "frames A and B differ toward the glyph colour at {n} pixel(s) of text '{id}''s box, \
             fewer than the {floor} a glyph mask needs: either the text was not hidden in the \
             text-hidden frame or it is invisible against the image, and the two cannot be told \
             apart, so no contrast was measured"
        ),
    )
}

/// A caller-supplied percentile, refused unless it is a finite 0..=100.
pub fn valid_percentile(p: f64) -> Result<f64, LocateError> {
    if p.is_finite() && (0.0..=100.0).contains(&p) {
        Ok(p)
    } else {
        Err((
            UnknownCode::InputMissing,
            format!("percentile {p} is not a number in 0..=100, so nothing was measured"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::color::wcag_contrast;

    #[test]
    fn nearest_rank_reads_the_minimum_at_zero_and_the_maximum_at_100() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(nearest_rank(&v, 0.0), 1.0);
        assert_eq!(nearest_rank(&v, 5.0), 5.0);
        assert_eq!(nearest_rank(&v, 50.0), 50.0);
        assert_eq!(nearest_rank(&v, 100.0), 100.0);
        assert_eq!(nearest_rank(&[3.0], 5.0), 3.0);
    }

    /// `7 / 100 * 100` is 7.000000000000001 in f64, whose ceiling is the
    /// eighth rank; multiplying first gives exactly 7.
    #[test]
    fn nearest_rank_is_not_off_by_one_from_float_error() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        for p in 1..=100 {
            assert_eq!(nearest_rank(&v, p as f64), p as f64, "p{p}");
        }
    }

    #[test]
    fn a_mask_is_usable_between_its_absolute_floor_and_its_share_ceiling() {
        assert!(!mask_usable(0, 1000, 1.0));
        assert!(!mask_usable(5, 100, 1.0), "under MIN_GLYPH_CSS_PX2");
        assert!(
            mask_usable(100, 1280 * 28, 1.0),
            "no share floor: a short label in a wide box"
        );
        assert!(!mask_usable(800, 1000, 1.0), "over MAX_GLYPH_SHARE");
        assert_eq!(min_glyph_px(2.0), 24, "the floor scales with DPR²");
        assert!(!mask_usable(10, 1000, 2.0));
    }

    #[test]
    fn the_luminance_table_matches_wcag_contrast_exactly() {
        for &(a, b) in &[
            (Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)),
            (Rgb::new(250, 250, 250), Rgb::new(215, 210, 225)),
            (Rgb::new(10, 200, 30), Rgb::new(9, 10, 11)),
            (Rgb::new(128, 128, 128), Rgb::new(128, 128, 128)),
        ] {
            assert_eq!(contrast(luminance(a), luminance(b)), wcag_contrast(a, b));
        }
    }

    #[test]
    fn a_scale_factor_is_usable_only_in_0_to_the_cap() {
        for bad in [
            0.0,
            -1.0,
            MAX_SCALE_FACTOR + 0.01,
            5000.0,
            f64::NAN,
            f64::INFINITY,
        ] {
            assert!(usable_scale(bad).is_err(), "{bad}");
        }
        for ok in [0.5, 1.0, 2.0, MAX_SCALE_FACTOR] {
            assert_eq!(usable_scale(ok).unwrap(), ok);
        }
    }

    #[test]
    fn a_percentile_outside_0_to_100_is_refused() {
        for bad in [-1.0, 100.5, f64::NAN, f64::INFINITY] {
            assert!(valid_percentile(bad).is_err(), "{bad}");
        }
        assert_eq!(valid_percentile(0.0).unwrap(), 0.0);
        assert_eq!(valid_percentile(100.0).unwrap(), 100.0);
    }
}
