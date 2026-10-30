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
//! # The verdict is a bracket
//!
//! The pixels rarely pin one reading down. Each reading therefore carries
//! the BEST case the pixels admit ([`TextOverImageContrast::percentile_ratio`])
//! and the WORST case ([`TextOverImageContrast::percentile_ratio_worst_case`]).
//! A check FAILS only when even the best case is below its threshold, PASSES
//! only when even the worst case reaches it, and is UNKNOWN in between. Every
//! ambiguity below widens the bracket or refuses the reading, and none is
//! resolved toward a pass, except where a section names a KNOWN LIMIT: a
//! reading the pixels cannot tell from the one taken (a gradient's far end
//! hidden over its own colour, a faint glyph wholly beside bold strokes, a
//! glyph too faint to change any pixel, a text whose own glyphs show only
//! where a larger or enclosing text's may).
//!
//! # The two frames must be one layout
//!
//! [`pair_frames`] refuses a pair of different sizes or scale factors, and
//! [`registered`] refuses one whose IMAGE content differs away from the text.
//! The image's box, outside every box of a text element that paints glyphs of
//! its own over it ([`TextIndex`], each widened by
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
//! the text refuses on its own, and the refusal names its position. The ring
//! is read through every text box except those that CONTAIN the text: the
//! padding of an overlay card around its heading is exactly what the ring
//! must read, and the card's own glyphs there read as text, not as moved
//! image. PRODUCERS SHOULD PAUSE ANIMATIONS (spinners, carousels, chevrons,
//! CSS transitions) for both captures: anything that moves between them near
//! the text reads as misregistration, and the text is not measured.
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
//! A box smaller than [`MIN_MEASURABLE_BOX_CSS_PX2`] CSS px² — the mask's
//! floor within its share ceiling — can hold no usable mask and no legible
//! glyph (a `.sr-only` label is a 1×1 box whose text is clipped away), and
//! is not text over the image ([`Over::too_small`], disclosed).
//!
//! # Whose glyphs they are
//!
//! A box is not a glyph area. A wrapped inline `<span>`'s box spans every
//! line it touches, its parent's words included; a negative margin lays one
//! text's box over another's glyphs. So a differing pixel inside the box of
//! ANOTHER element that paints glyphs of its own ([`TextOverImage::others`])
//! is attributed by colour, never cut away by box:
//!
//! - the two glyph colours agree (within [`GLYPH_COLOUR_TOLERANCE`]): it may
//!   be either's ink, and counts for this text;
//! - they differ: it counts for this text when this text's colour explains
//!   it — frame A lies within [`GLYPH_COLOUR_TOLERANCE`] of the alpha blend
//!   of frame B toward it ([`residual`]), whether or not the other's colour
//!   explains it too — or, when neither does, when frame A lies closer to
//!   this text's blend than to the other's;
//! - the other's box shows too few differing pixels to have a colour: it
//!   paints no mask, and the pixel counts for this text.
//!
//! A pixel counted for this text inside another's box is SHARED when the
//! other's colour explains it too (the colours agree, or frame A lies on the
//! blend toward the other's): a colour carries no opacity, so
//! `<p class="text-white/40">` around an opaque white `<strong>` reports one
//! colour for two inks, and a wrapped `<span>`'s box holds its parent's
//! words. Nothing in the pixels says which of them are whose — not their
//! opacity either, since a span at 95% sits beside an opaque parent — so a
//! shared pixel is OPTIONAL: the bracket allows every choice of them
//! ("Definite and optional pixels", below;
//! [`TextOverImageContrast::shared_px`]). A pixel the other's colour does not
//! explain cannot be its ink, and is this text's.
//!
//! A pixel where nothing differs is never attributed away: hidden ink there
//! may be this text's, and is counted for it (below).
//!
//! A text with INNER texts ([`TextOverImage::inner`]) is Unknown when fewer
//! than the mask's floor of its glyph pixels (decorations aside) are ones no
//! inner text could have painted: outside every inner text's box, or there
//! in a colour that text's colour does not explain (as for a shared pixel,
//! above). Every other pixel its mask holds may be their ink, which says
//! nothing of its own fill (below), and its own words, which may render
//! another colour over a patch of it, were not seen. A text whose glyphs do
//! not reach the image is not over it, and is not measured. Its inner texts
//! are:
//!
//! - the texts that DESCEND from it in the DOM, as a `<p>`'s link does,
//!   whatever their boxes;
//! - every other text whose box is no larger than its own: a badge or a
//!   caption laid inside or over it, and the label of an absolutely
//!   positioned child that outgrows it. Two unrelated texts whose boxes
//!   have one area are each the other's inner text;
//! - but not an ANCESTOR whose box covers at least half of its own. Such a
//!   box is ASSUMED to hold its glyphs by layout (a padded inline link
//!   overhangs its paragraph's box by a few pixels), where they would show
//!   no pixel apart from its ink. An ancestor no larger that covers less is
//!   an inner text: a link whose padding makes its box over twice its
//!   line's height, alone in a paragraph of its colour whose box is no
//!   larger than its own, is Unknown.
//!
//! So ancestors with such boxes, and other texts with larger boxes (an
//! inset-0 overlay; the heading a badge sits on), are never refused on.
//! That ASSUMES the larger box is the one laid under the smaller: refusing
//! on it would refuse every inline link in a paragraph and every badge on a
//! heading. Where such a text's ink lies in this text's box and this text's
//! own glyphs show nowhere — an inline child broken over lines, whose box
//! spans its parent's words; a positioned child at least half over its
//! ancestor's box; a smaller text under a larger one's glyphs — it is read
//! on what its mask holds: a KNOWN LIMIT.
//!
//! # Glyphs the mask cannot see
//!
//! The INK is every colour the glyph may render over a whole pixel:
//!
//! - its glyph colour;
//! - what its STROKE INTERIORS render — mask pixels covered at least
//!   [`INK_COVERAGE`] whose every neighbour is too, which an anti-aliased
//!   edge never is: a rendered colour a few units off the declared one (a
//!   `filter`, a text-fill colour within tolerance);
//! - every colour ONE fill painting its glyphs may be. An ink renders
//!   nothing beyond itself, so per channel the fill lies at or past the
//!   furthest any glyph pixel moved from frame B. Where some pixel reaches
//!   the colour that pins it; where none does — strokes too thin to cover a
//!   pixel whole, ink a few units off the colour — the fill may be anything
//!   from those pixels to the colour, since such a stroke cannot tell an off
//!   colour over whole pixels from the colour over part of them. A few
//!   pixels painted in another colour pin that bound wrongly — a dotted
//!   underline in the declared colour, a child link rendering it while the
//!   text's own glyphs render a text-fill colour a few units off — so the
//!   bound the MAJORITY of the text's own glyph pixels set is kept too: per
//!   channel, the median of its connected strokes' reaches weighted by their
//!   pixels, on the side nearer the background, over the pixels no other
//!   text may have painted (over all of them when every one may be another
//!   text's). Where the median stroke covers no pixel whole, the ink widens
//!   down to what it renders even when other strokes reach the colour: thin
//!   text with one bold stem is bounded as thin text. A connected stroke
//!   [`LINE_ASPECT`] times wider than tall is a decoration (an underline in
//!   another colour): it bounds its own colour, and casts no vote;
//! - for a MEASURED colour, the ray from the background past it to the edge
//!   of the RGB cube: a measured colour is only what the best-covered pixels
//!   show, and a translucent ink further along the ray renders it too.
//!
//! The model is one fill per text, as CSS paints one: a fill that changes
//! colour across the glyphs (a gradient) is bounded by its furthest
//! reaching pixels and by its median stroke, and its visible glyphs are read
//! where they render (below), but a far end that reaches less far than the
//! median, hidden over a patch of its own colour, is not counted. A
//! minority of glyphs in a second fill (a `::first-line` in another colour
//! than the rest) is the same limit.
//!
//! Where the image under a glyph is near the ink (blown-out sky,
//! snow, a white studio backdrop under white text; crushed shadows under
//! black), A and B differ by less than the mask's threshold, and those are
//! exactly the invisible glyph pixels. A glyph covering a pixel `m` moves it
//! `m` times the background's distance from the ink, so the mask misses it
//! on any background within `GLYPH_DIFF_MIN / m` of the ink; `m` is the
//! larger of [`MIN_GLYPH_COVERAGE_SHARE`] of the text's robust peak coverage
//! and [`MIN_GLYPH_COVERAGE_ABS`]. Those pixels are found from frame B alone:
//! the BLIND pixels of the overlap. A blind pixel that differs at all and
//! moved toward the ink is a FAINT glyph pixel and is sampled, at no better
//! than the contrast it renders (its coverage cannot be read so near the
//! ink) — when the registration check found the capture free of
//! sub-threshold noise ([`FAINT_NOISE_MAX`]); otherwise it is treated as
//! undetectable. With a noise-free capture a glyph covering a pixel whole
//! changes it unless its background is a colour the glyph renders there —
//! inside the ink set — so only those blind pixels are undetectable; with a
//! noisy one, every blind pixel is. A glyph so faint that it rounds to its
//! background everywhere changes no pixel at all, and nothing here tells it
//! from no glyph. EVERY undetectable pixel may be ink, at any coverage, so
//! painting 1:1: no visible band bounds the density of a hidden word, which
//! may be heavier than any visible one (`heavy_m` / `heavy_w`). They are
//! OPTIONAL: the worst case takes them, the best case never needs to.
//!
//! A glyph can also be too FAINT for the mask on any background: a word at
//! 2% opacity moves its pixels by less than [`GLYPH_DIFF_MIN`]. On a
//! noise-free capture such a pixel — one that moved toward the ink at all,
//! further than [`SUB_THRESHOLD_CLEARANCE_CSS`] from every mask pixel (nearer,
//! it is a visible stroke's anti-aliasing fringe) — is a SUB-THRESHOLD glyph
//! pixel, sampled like a mask pixel at its own coverage and counted as faint.
//! On a capture with sub-threshold noise the noise moves pixels alike, so
//! such a pixel is NOISE-OR-FAINT: OPTIONAL, read at no better than it
//! renders ([`TextOverImageContrast::noise_px`]). Nothing subtracts the
//! noise's own rate or tests how it clusters: a faint word at the rounding
//! edge moves scattered pixels as noise does, and either test would drop it
//! and pass. The cost falls on ordinary text: noise moving as few as one
//! pixel in fifty of the box toward the ink can put the worst case on the
//! noise, and the text is Unknown. Capture losslessly, twice from one raster.
//!
//! And a glyph pixel can move past [`GLYPH_DIFF_MIN`] toward the ink but
//! AWAY from the glyph colour — ink a few units off the colour over a
//! background between the two, a gradient's far end — so the mask, which
//! keeps only pixels that moved toward the colour, leaves it out. Such an
//! OFF-COLOUR pixel is sampled too, in the worst case at no better than it
//! renders ([`TextOverImageContrast::off_colour_px`]).
//!
//! A faint or off-colour pixel within [`HALO_CSS`] of the mask lies BESIDE a
//! visible stroke, where a glow of the glyph's colour paints alike (a white
//! `text-shadow` over a near-white photo moves it a unit or two toward the
//! ink; "Local peaks", below). So does a faint one JOINED to the mask by
//! pixels that moved toward the ink, at any distance: a glow wider than
//! [`HALO_CSS`] ramps down through them. Further off and unjoined, it is a
//! glyph of its own; an off-colour pixel, which moved past the mask's
//! threshold and is a glyph's (a gradient's far end joins its near end),
//! only beyond [`HALO_CSS`].
//!
//! Strokes too thin to cover a pixel whole widen the ink down to what they
//! render (above), so a bright speck of that range under the box, between
//! the glyphs, is a pixel where ink may hide: thin light text over a
//! speckled photo reads Unknown rather than a pass.
//!
//! When the box shows glyphs and fewer than the floor of them reach the
//! image, the text is not over the image (a label whose line box overhangs a
//! thumbnail), and is not measured — unless the image under the box is blind
//! enough to hide the floor's worth (a box straddling the image's edge whose
//! glyphs over it sit on their own colour): that is Unknown. One exemption:
//! an image that is the text's own DESCENDANT and lies inside its box (a
//! button's inline icon) is laid out BESIDE the glyphs by the inline
//! formatting context, so only its blind pixels within
//! [`BLIND_ADJACENT_CSS`] of the text's visible glyphs count — an icon in
//! `currentColor` is not a hidden word. An image the text merely covers gets
//! no exemption: a word laid over a patch of its own colour is exactly what
//! this section exists to catch.
//!
//! # Local peaks
//!
//! A glyph pixel is SAMPLED where the glyph covers it at least
//! [`MIN_GLYPH_COVERAGE_SHARE`] of its LOCAL peak — the best coverage within
//! one CSS px (α = (A−B)·(c−B)/|c−B|² under alpha blending) — and is read at
//! that peak. An anti-aliased edge beside a better-covered stroke is left
//! out: a low-coverage edge clears the difference threshold more readily over
//! a background far from the glyph colour, which would weight the sample
//! toward high contrast. A faint glyph — a fade, a translucent word beside an
//! opaque one, a hairline face — is its own local peak, and is sampled at it.
//!
//! Beside a better covered stroke a GLOW of the glyph's own colour (a white
//! `text-shadow` under white text) and a faint glyph paint alike. Read as a
//! glyph, a glow's ramp over a light photo fails text that sits on a dark
//! highlight; read as nothing, a faint word beside a bold one passes. A
//! stroke's anti-aliased edge is ambiguous the same way, and so is a ring of
//! glow within one CSS px of it, which shares the stroke's local peak and
//! would otherwise be read as the stroke itself — enough of such a ring
//! outnumbers a faint word. So each sampled pixel falls in one EXTENT:
//!
//! - BESIDE a better covered stroke: its local peak falls under
//!   [`INK_COVERAGE`] of the best coverage within [`HALO_CSS`], or of the
//!   peak of the connected mask stroke it belongs to (a glow's plateau,
//!   however wide, joins the stroke it surrounds); or it is faint and joined
//!   to the mask by pixels that moved toward the ink, or faint or off the
//!   colour within [`HALO_CSS`] of the mask (above);
//! - else an EDGE: covered under [`INK_COVERAGE`] of its own local peak;
//! - else a stroke's CORE.
//!
//! A reading takes the cores; or the cores and the edges; or all three
//! ([`TextOverImageContrast::halo_px`] counts the edges and the pixels
//! beside) — never part of an extent. A glow or a ring paints the whole of
//! its extent, as anti-aliasing paints every edge: taken pixel by pixel, the
//! worst case picked the edges that happen to lie over the lightest texture,
//! a glyph no font draws, and read ordinary anti-aliased text over a photo
//! Unknown. The KNOWN LIMIT this leaves: a faint glyph that lies wholly
//! beside bold strokes, among a glow or their edges, is read with the whole
//! extent or without it, never on its own.
//!
//! # Definite and optional pixels
//!
//! Within the extents a reading takes, every pixel is DEFINITE — this text's
//! glyph — or OPTIONAL: one the pixels admit, on its own, both as its glyph
//! and as something else. Optional are the SHARED pixels (another text's
//! ink), on a noisy capture the NOISE-OR-FAINT ones, and hidden ink, which
//! has no best case. The BEST case is the highest `percentile`-th
//! percentile any extent and choice of optional pixels give, the WORST case
//! the lowest, over every choice that leaves the text at least the mask's
//! floor of glyph pixels (one with fewer is not measured at all). Both are
//! exact: within an extent, for each count of optional pixels taken, the
//! ones reading highest raise every rank of the best case at least as far
//! as any others would, and the ones reading lowest lower every rank of the
//! worst, so the extremes over the counts are the extremes over every
//! choice. Taking an extent or a pixel only where it raises a percentile — a
//! ring of glow reading better than the faint word it would outnumber — is
//! never the worst case.
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
//! Each text is measured against its OWN colour ([`TextIndex`]). A LEAF text
//! element is measured on its box. A wrapper whose innerText is just its
//! leaf children's (a link around a heading) is measured through them, and
//! disclosed ([`PagePairs::wrappers`]). A wrapper that paints text of its own
//! beside them (`<h1>Hello <span>world</span></h1>`) is measured too, and so
//! is a CSS-background element's own text over its own background; the
//! glyphs inside their children's boxes are attributed as above. Painted
//! text with no box blocks a pass ([`PagePairs::boxless_texts`]): whether it
//! lies over an image is unknown.
//!
//! # Against which colour, and how opaque
//!
//! The ratio is WCAG 2.x contrast of the glyph colour against each sampled
//! frame-B pixel. The glyph colour is the element's declared `fg_color` when
//! the snapshot carries one, and MEASURED from frame A otherwise, as the
//! per-channel median of the raw difference's core (the quarter of differing
//! pixels that differ most) outside the boxes of the other texts that paint a
//! colour of their own there (but not of one that contains this text's box,
//! which would leave nothing to read), whose confidence is the share of core
//! pixels within [`GLYPH_COLOUR_TOLERANCE`] of it.
//!
//! Either way the colour must EXPLAIN the glyphs: under alpha blending every
//! glyph pixel of frame A lies on the segment from frame B toward the glyph
//! colour. At least [`COLOUR_FIT_SHARE`] of the glyph CORE — the quarter of
//! the mask's stroke cores ("Local peaks") the glyph covers most — must lie
//! within [`GLYPH_COLOUR_TOLERANCE`] of it
//! ([`TextOverImageContrast::colour_fit`]),
//! or the reading is Unknown: a `-webkit-text-fill-color` other than
//! `color`, a blend mode, and a gradient whose best-covered quarter is
//! another colour all paint a colour the declaration does not name, and a
//! ratio against it would be a ratio against nothing on screen. A gradient
//! that keeps the declared colour where it is best covered and fades or
//! turns elsewhere passes the fit; in the worst case its faded glyphs are
//! read at their own coverage (below), and a glyph pixel the blend does not
//! explain at no better than the colour it renders.
//!
//! A MEASURED colour is what the best-covered pixels show. Where the glyph
//! has stroke interiors (above) it is what the glyph renders over a whole
//! pixel; with none, the glyph may be a hairline of a more extreme ink on the
//! ray past it ([`TextOverImageContrast::hairline_ink`]), and the BEST case
//! reads that ink too — as a declared colour's best case reads the declared
//! colour whatever its pixels' coverage. Interiors refute a hairline, not a
//! translucent ink further along the ray, which stays in the ink set.
//!
//! Alpha blending also cannot tell a TRANSLUCENT colour (an `opacity`, an
//! `rgba` colour) from a hairline face that never covers a whole pixel: both
//! put every glyph pixel the same fraction of the way from B to the colour.
//! Each sampled pixel's LOCAL peak coverage is therefore carried into the
//! worst case: there it paints `B + α·(c − B)` rather than `c`. An opaque
//! glyph that covers whole pixels reads α = 1, and the two cases coincide.
//! [`TextOverImageContrast::glyph_alpha`] reports the faintest glyphs'
//! (the `percentile`-th percentile of the stroke cores' α, where one was
//! read). A stroke core there is read by its own neighbourhood alone: never
//! an edge, a pixel within [`HALO_CSS`] of a better covered stroke, a pixel
//! under the mask's threshold joined to it by a glow's ramp, or one off the
//! blend, which is read at what it renders instead. The faint end of a
//! stroke joined to a bolder one (a fade across joined letters) is a core
//! there, and the faintest glyph; so is a same-colour glow's plateau joined
//! to the strokes past [`HALO_CSS`], which paints exactly as such a fade
//! does: on a glowing text, `glyph_alpha` may report the glow.
//!
//! # What is reported
//!
//! The minimum ratio and the `percentile`-th percentile (nearest rank;
//! [`DEFAULT_PERCENTILE`] = 5) over the sampled pixels — the best case — and
//! the same percentile in the worst case: every glyph pixel at its local
//! peak coverage or at no better than it renders, each over its extreme
//! extent and choice of the optional pixels. The minimum is one pixel's word
//! — a single bright speck under a stroke sets it — so a check gates on the
//! percentile.
//! Readings are held in `f64` to the comparison: a ratio a hair under the
//! threshold rounds up to it in `f32`.
//!
//! # Cost
//!
//! The work per text is linear in its box's pixels: the dilations and the
//! local and halo peaks are separable window passes whose cost does not grow
//! with their radius, each image's painters' colours are measured once
//! ([`Registration`]), and the local check's shift search reads at most
//! [`LOCAL_SHIFT_SAMPLE`] of a tile's differing pixels per offset. Pairing
//! texts with images and with the other painters over them compares boxes
//! (no pixels), pairs × painters, and the bracket over optional pixels is
//! a two-list order statistic per extent and count of them, logarithmic
//! each. Memory per text is linear in its box too: about 30 bytes per box
//! pixel for its grids (three `f32` coverage grids, the strokes' labels, the
//! bitmaps, the window passes' scratch), about 70 per glyph pixel for the
//! mask, the colour fit and the readings, and 8 per hidden pixel, of which a
//! box over a blind image may hold one per box pixel; the bracket moves the
//! readings, never copying them. Not every grid is live at once: a
//! full-viewport hero's own text at scale factor 4 (5760 × 3200 px, glyphs
//! on a twentieth of it) was measured, in a release build, to raise the
//! process's peak by about 390 MiB, and an image under it blind across 72%
//! of the box (13.3 million hidden pixels) by about 90 MiB more.
//!
//! APCA is not computed: the plan (PD9) adds it only after a licence read
//! admits it.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::{unreadable, unrendered, Ancestry, LocateError};
use crate::element_media::MediaKind;
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

/// The smallest text box, in CSS px², that can hold a usable mask: its
/// [`MIN_GLYPH_CSS_PX2`] glyph pixels within [`MAX_GLYPH_SHARE`] of the box.
/// A smaller box holds no legible glyph (a `.sr-only` label is a 1×1 box).
pub const MIN_MEASURABLE_BOX_CSS_PX2: f64 = MIN_GLYPH_CSS_PX2 / MAX_GLYPH_SHARE;

/// Largest share of the text box a usable mask covers. Glyphs of even a
/// heavy face in a tight box cover well under this; a larger difference
/// toward the glyph colour is not glyphs.
pub const MAX_GLYPH_SHARE: f64 = 0.70;

/// How far, in CSS px, each text box is widened before the registration
/// check and the attribution of glyph pixels, for anti-aliasing.
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
/// glyphs (an unhidden twin whose raster moved a sub-pixel). Chosen between
/// the shares measured on the committed fixtures during review: near zero on
/// the hidden twins, far above it on the re-rasterised ones
/// (`text_hidden_rerastered`). Read as an EXCESS over the rate the image's
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

/// How many of a text-like ring tile's differing pixels the shift test reads
/// ([`Registration::local`]), evenly strided: the search tries
/// `(2·⌈12·s⌉+1)²` offsets, so reading every differing pixel at each would
/// grow with the fourth power of the scale factor.
pub const LOCAL_SHIFT_SAMPLE: usize = 64;

/// Share of a ring tile's differing pixels that must have moved toward one
/// colour for the tile to read as untracked text rather than moved image.
pub const LOCAL_TEXT_LIKE_SHARE: f64 = 0.9;

/// Share of the ring's tiles that must show displaced content for the local
/// check to refuse.
pub const LOCAL_DISPLACED_TILE_SHARE: f64 = 0.1;

/// Smallest estimated glyph coverage ([`coverage`]) of a sampled mask pixel,
/// as a share of its LOCAL peak — the best coverage within one CSS px: an
/// anti-aliased edge beside a better-covered stroke is left out, and a faint
/// glyph (a fade, a translucent word, a hairline face) is its own local peak
/// and is sampled. Faint pixels (on a background within the blind radius of
/// the ink) are sampled whatever their coverage.
pub const MIN_GLYPH_COVERAGE_SHARE: f64 = 0.5;

/// The coverage the BLIND radius is sized for: a glyph covering a pixel at
/// least this much, on a background further than `GLYPH_DIFF_MIN / m` from
/// its ink, moves it past [`GLYPH_DIFF_MIN`] and is seen by the mask. The
/// radius uses the larger of this and [`MIN_GLYPH_COVERAGE_SHARE`] of the
/// text's robust peak coverage, so a mask of faint edges or noise does not
/// shrink it without bound.
pub const MIN_GLYPH_COVERAGE_ABS: f64 = 0.15;

/// RGB distance within which a core glyph pixel agrees with the measured
/// glyph colour, within which two glyph colours are one ink, and within
/// which a glyph pixel lies on the alpha blend toward its colour.
pub const GLYPH_COLOUR_TOLERANCE: f64 = 24.0;

/// Least share of the glyph core the glyph colour must explain under alpha
/// blending ([`TextOverImageContrast::colour_fit`]). The core is the
/// best-covered quarter, where a grayscale or LCD anti-aliasing fringe is
/// rarest; an opaque glyph of the declared colour explains all of it, and a
/// gradient or a text-fill colour other than `color` explains little.
pub const COLOUR_FIT_SHARE: f64 = 0.5;

/// Largest scale factor (device pixel ratio) a frame pair is measured at.
/// Every threshold here is in CSS px and scales with the ratio; the suite
/// measures the plan's fixtures at ratios 1, 2 and 3 and the local check's
/// cost at 4 (`tests/text_over_image.rs`). The work grows with the ratio's
/// square, so a larger value is refused as Unknown, outside the range the
/// measurement has been run at, rather than clamped.
pub const MAX_SCALE_FACTOR: f64 = 4.0;

/// How near, in CSS px, a BLIND pixel must lie to one of the text's own
/// mask pixels to count as evidence of hidden glyphs when the image is the
/// text's own descendant inside its box (a button's inline icon): the inline
/// formatting context lays text out beside such an image, not over it, so
/// only blind pixels touching the visible glyphs can be theirs.
pub const BLIND_ADJACENT_CSS: f64 = 2.0;

/// The coverage at or above which a mask pixel's colour counts as INK:
/// what the glyph renders where it covers the pixel nearly whole, which a
/// text-fill colour or a filter may put a few units off the declared colour.
/// Also the share that sorts a sampled pixel into its EXTENT (module docs,
/// "Local peaks"): it is BESIDE a better covered stroke when its local peak
/// falls under this share of its halo peak or of the peak of the stroke it
/// is joined to, and else an EDGE when its own coverage falls under this
/// share of its local peak.
pub const INK_COVERAGE: f64 = 0.9;

/// How many times wider than tall a connected stroke of the glyph mask is
/// read as a DECORATION (an underline, an overline, a line-through) rather
/// than the fill: its colour is its own and does not bound the fill's.
pub const LINE_ASPECT: i64 = 4;

/// How far, in CSS px, a sampled pixel's HALO peak is read, and how near
/// the mask a faint or off-colour pixel lies BESIDE it: there a glow of the
/// glyph's colour and a faint glyph paint alike. A faint pixel joined to the
/// mask by pixels that moved toward the ink (a glow's ramp) lies beside it
/// at any distance (module docs, "Local peaks").
pub const HALO_CSS: f64 = 4.0;

/// How far, in CSS px, a SUB-THRESHOLD glyph pixel — one that moved toward
/// the ink by less than [`GLYPH_DIFF_MIN`] — must lie from every mask pixel
/// to be read as a glyph of its own: nearer, it is the anti-aliasing fringe
/// of a visible stroke, whose coverage decays over a pixel or two. On a
/// noisy capture such a pixel is optional
/// ([`TextOverImageContrast::noise_px`]).
pub const SUB_THRESHOLD_CLEARANCE_CSS: f64 = 2.0;

/// A frame's scale factor, refused unless it is finite, positive and at most
/// [`MAX_SCALE_FACTOR`].
pub fn usable_scale(s: f64) -> Result<f64, LocateError> {
    if s.is_finite() && s > 0.0 && s <= MAX_SCALE_FACTOR {
        Ok(s)
    } else {
        Err((
            UnknownCode::InputMissing,
            format!(
                "the frame's scale factor {s} is outside 0 < s <= {MAX_SCALE_FACTOR}, the range \
                 the measurement has been run at, so no contrast was measured"
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
    /// The lowest best-case WCAG ratio of any sampled pixel, definite or
    /// optional: one pixel's word, which a check does not gate on.
    pub min_ratio: f64,
    /// The percentile reported in [`Self::percentile_ratio`] (0..=100).
    pub percentile: f64,
    /// The BEST case the pixels admit: the `percentile`-th percentile WCAG
    /// ratio (nearest rank) of [`Self::glyph_colour`], opaque (or of
    /// [`Self::hairline_ink`] where that reads higher), against frame B under
    /// the sampled glyph pixels, at the HIGHEST any extent and choice of the
    /// optional pixels give it (module docs, "Definite and optional
    /// pixels"). A FAILURE is decided on this.
    pub percentile_ratio: f64,
    /// The WORST case the pixels admit: the same percentile with every glyph
    /// pixel the alpha blend explains painting `B + α·(c − B)` at its LOCAL
    /// peak coverage `α` rather than `c`, every other glyph pixel (faint, off
    /// the blend, off the colour) painting no better than it renders, and
    /// [`Self::hidden_px_estimate`] pixels of hidden ink painting 1:1, at the
    /// LOWEST any extent — the cores, with the edges, with the pixels beside
    /// a stroke — and choice of the optional pixels — shared, hidden,
    /// noise-or-faint — give it. A PASS is decided on this; it equals
    /// [`Self::percentile_ratio`] when every pixel is a core's and none is
    /// optional, translucent, faint or off the colour, and no
    /// [`Self::hairline_ink`] may paint the glyphs.
    pub percentile_ratio_worst_case: f64,
    /// The colour the ratios are computed against.
    pub glyph_colour: Rgb,
    pub glyph_colour_source: GlyphColourSource,
    /// A MEASURED colour whose strokes hold no interior may be a hairline of
    /// a more extreme ink, at partial coverage: that ink, the far end of the
    /// ray from the background through the colour, which the best case reads
    /// as well (module docs, "Against which colour"). `None` for a declared
    /// colour and for strokes a hairline cannot paint.
    pub hairline_ink: Option<Rgb>,
    /// The colours the glyph's ink may render over a whole pixel, as the low
    /// and the high corner of a per-channel box (module docs, "Glyphs the
    /// mask cannot see"): [`Self::hidden_px_estimate`] counts pixels whose
    /// background lies too near it for a difference to show.
    pub ink_range: [Rgb; 2],
    /// How opaque the FAINTEST glyphs are (0..=1): the `percentile`-th
    /// percentile (nearest rank) of the coverage of [`Self::glyph_colour`]
    /// each stroke CORE pixel on the blend toward it — no edge, none within
    /// [`HALO_CSS`] of a better covered stroke, no sub-threshold pixel joined
    /// to the mask by a glow's ramp, none off the blend; the faint end of a
    /// stroke joined to a bolder one counts, and so does a glow's plateau
    /// joined past [`HALO_CSS`], which paints alike — was read at (its local
    /// peak). 1 for an opaque glyph that covers whole pixels; below 1 for a
    /// translucent colour, a fade, and a hairline face alike (module docs).
    /// `None` when no such pixel was read at a coverage: every one is faint on
    /// a background too near the ink for a coverage to be read, or off the
    /// colour.
    pub glyph_alpha: Option<f64>,
    /// The share of the glyph core within [`GLYPH_COLOUR_TOLERANCE`] of the
    /// alpha blend toward [`Self::glyph_colour`] (0..=1), at least
    /// [`COLOUR_FIT_SHARE`] for any reading.
    pub colour_fit: f64,
    /// How many frame-B pixels were sampled (glyph pixels over the image),
    /// definite and optional, faint ones included.
    pub sampled_px: u64,
    /// Of [`Self::sampled_px`], the FAINT glyph pixels the mask cannot see,
    /// away from its visible strokes, found on a noise-free capture by any
    /// difference toward the ink at all: on a BLIND background (near the
    /// ink), or SUB-THRESHOLD (a glyph too faint to move its pixel past
    /// `GLYPH_DIFF_MIN`; module docs). Those beside a stroke are
    /// [`Self::halo_px`].
    pub faint_px: u64,
    /// The estimated glyph pixels on such a background that no difference
    /// shows (module docs). Optional: they enter the worst case.
    pub hidden_px_estimate: u64,
    /// Of [`Self::sampled_px`], the pixels inside the box of another text
    /// that paints glyphs of its own there, in a colour that explains them
    /// too: either text's ink, so optional (module docs, "Whose glyphs they
    /// are").
    pub shared_px: u64,
    /// Of [`Self::sampled_px`], the glyph pixels away from the strokes' sides
    /// that paint something other than the alpha blend toward
    /// [`Self::glyph_colour`] — ink a few units off the colour, a gradient's
    /// far end — each read in the worst case at no better than it renders
    /// (module docs, "Against which colour").
    pub off_colour_px: u64,
    /// Of [`Self::sampled_px`], the pixels of a stroke's EDGE or BESIDE it
    /// (module docs, "Local peaks"): covered under [`INK_COVERAGE`] of the
    /// best coverage within one CSS px (an anti-aliased edge, a ring), or
    /// locally peaking under it of the best within [`HALO_CSS`] or of the
    /// peak of the stroke they join (a glow's plateau); faint or off the
    /// colour within [`HALO_CSS`] of the mask; or faint and joined to the
    /// mask by pixels that moved toward the ink (a glow's ramp). An
    /// anti-aliased edge, a glow of the glyph's colour and a faint glyph
    /// paint alike there, so a reading takes each extent of them whole or
    /// not at all.
    pub halo_px: u64,
    /// Of [`Self::sampled_px`], on a capture with sub-threshold noise (over
    /// [`FAINT_NOISE_MAX`]), the pixels away from the strokes that moved
    /// toward the ink by less than [`GLYPH_DIFF_MIN`]: the capture's noise
    /// and a glyph too faint for the mask paint alike there, so they are
    /// optional, read at what they render (module docs, "Glyphs the mask
    /// cannot see").
    pub noise_px: u64,
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
#[derive(Debug, Clone)]
pub struct TextOverImage<'a> {
    pub text: &'a Element,
    pub image: &'a Element,
    pub css: Region,
    /// Every OTHER element that paints glyphs of its own
    /// ([`TextIndex::painters`]) whose box meets this text's: inside their
    /// boxes, glyph pixels are attributed by colour (module docs, "Whose
    /// glyphs they are").
    pub others: Vec<&'a Element>,
    /// One per [`Self::others`], in order: whether its ink may be all this
    /// text's glyphs show, an INNER text (module docs, "Whose glyphs they
    /// are"). Every text that DESCENDS from it is, whatever their boxes; so
    /// is every other text whose box is no larger than its own (a badge or a
    /// caption laid over it; a positioned child's label), except an ancestor
    /// whose box covers at least half of its own. A pair built with fewer
    /// flags than [`Self::others`] reads each missing one as inner, the
    /// cautious answer: it can only make this text Unknown.
    pub inner: Vec<bool>,
    /// Whether the image is this text's own descendant (an inline icon), the
    /// one case where blind pixels away from the visible glyphs are not
    /// evidence of hidden ink (module docs).
    pub image_inline: bool,
}

/// Whether `t` is a text element that paints: non-blank text, not itself
/// media, not [`unrendered`].
fn paints_text(snapshot: &ElementSnapshot, t: &Element) -> bool {
    t.media.is_none() && has_text(t) && unrendered(snapshot, t).is_none()
}

fn has_text(t: &Element) -> bool {
    t.text.as_deref().is_some_and(|s| !s.trim().is_empty())
}

/// Whether `t` is a CSS-background element with text, not [`unrendered`]: a
/// hero `<div>` paints its own text over its own background image, so that
/// text is over the image even though no other element carries it.
fn paints_own_text_media(snapshot: &ElementSnapshot, t: &Element) -> bool {
    t.media
        .as_ref()
        .is_some_and(|m| m.kind == MediaKind::CssBackground)
        && has_text(t)
        && unrendered(snapshot, t).is_none()
}

/// `s` with every whitespace character removed: two innerTexts compared
/// regardless of how a producer joins lines and blocks.
fn squeezed(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Which elements paint glyphs of their OWN, built once per snapshot in
/// O(elements × depth):
///
/// - the LEAF text elements: painted text with no painted text element
///   descending from them (`parent_id` chain or `children_ids`, as
///   [`super::drop_nested`] reads it);
/// - the WRAPPERS (and CSS-background elements with text) that paint text of
///   their own beside their leaves: their text, whitespace removed, is not
///   their leaf descendants' texts concatenated in snapshot order
///   (`<h1>Hello <span>world</span></h1>`).
///
/// A wrapper whose text is all its leaves' paints no glyph of its own.
#[derive(Debug)]
pub struct TextIndex<'a> {
    ancestry: Ancestry<'a>,
    leaves: HashSet<&'a str>,
    own_text: HashSet<&'a str>,
    painters: Vec<&'a Element>,
    boxless: usize,
}

impl<'a> TextIndex<'a> {
    pub fn of(snapshot: &'a ElementSnapshot) -> Self {
        let ancestry = Ancestry::of(snapshot);
        let texts: Vec<&'a Element> = snapshot
            .elements
            .iter()
            .filter(|t| paints_text(snapshot, t))
            .collect();
        let media_texts: Vec<&'a Element> = snapshot
            .elements
            .iter()
            .filter(|t| paints_own_text_media(snapshot, t))
            .collect();
        let ancestors: HashMap<&'a str, HashSet<&'a str>> = texts
            .iter()
            .chain(&media_texts)
            .map(|t| (t.id.as_str(), ancestry.ancestors(t.id.as_str())))
            .collect();
        let text_ids: HashSet<&'a str> = texts.iter().map(|t| t.id.as_str()).collect();
        // `w` wraps `c` when `w` is among `c`'s ancestors and not the other
        // way round (a cycle in the producer's chain is not a nesting).
        let wraps = |w: &str, c: &str| {
            w != c
                && ancestors.get(c).is_some_and(|a| a.contains(w))
                && !ancestors.get(w).is_some_and(|a| a.contains(c))
        };
        let mut wrappers: HashSet<&'a str> = HashSet::new();
        for t in &texts {
            for &w in &ancestors[t.id.as_str()] {
                if text_ids.contains(w) && wraps(w, t.id.as_str()) {
                    wrappers.insert(w);
                }
            }
        }
        let leaves: HashSet<&'a str> = text_ids
            .iter()
            .copied()
            .filter(|id| !wrappers.contains(id))
            .collect();
        // Each candidate's leaf descendants' texts, in snapshot order.
        let candidates: HashSet<&'a str> = wrappers
            .iter()
            .copied()
            .chain(media_texts.iter().map(|t| t.id.as_str()))
            .collect();
        let mut theirs: HashMap<&'a str, String> = HashMap::new();
        for t in texts.iter().filter(|t| leaves.contains(t.id.as_str())) {
            for &w in &ancestors[t.id.as_str()] {
                if candidates.contains(w) && wraps(w, t.id.as_str()) {
                    theirs
                        .entry(w)
                        .or_default()
                        .push_str(&squeezed(t.text.as_deref().unwrap_or_default()));
                }
            }
        }
        let own_text: HashSet<&'a str> = snapshot
            .elements
            .iter()
            .filter(|e| candidates.contains(e.id.as_str()))
            .filter(|e| {
                squeezed(e.text.as_deref().unwrap_or_default())
                    != theirs.get(e.id.as_str()).map(String::as_str).unwrap_or("")
            })
            .map(|e| e.id.as_str())
            .collect();
        let painters = snapshot
            .elements
            .iter()
            .filter(|e| leaves.contains(e.id.as_str()) || own_text.contains(e.id.as_str()))
            .collect();
        let boxless = texts.iter().filter(|t| t.bbox.is_none()).count();
        Self {
            ancestry,
            leaves,
            own_text,
            painters,
            boxless,
        }
    }

    /// Whether `id` is a painted leaf text element.
    pub fn is_leaf(&self, id: &str) -> bool {
        self.leaves.contains(id)
    }

    /// Whether `id` is a wrapper (or a CSS-background element) that paints
    /// text of its own beside its leaf descendants'.
    pub fn paints_own_text(&self, id: &str) -> bool {
        self.own_text.contains(id)
    }

    /// Every element that paints glyphs of its own — the leaves and the
    /// own-text elements — in snapshot order.
    pub fn painters(&self) -> &[&'a Element] {
        &self.painters
    }

    /// How many painted text elements carry no box.
    pub fn boxless_texts(&self) -> usize {
        self.boxless
    }

    /// Whether `descendant` descends from `ancestor` (and not the other way
    /// round: a cycle in the producer's chain is not a nesting).
    pub fn descends(&self, descendant: &str, ancestor: &str) -> bool {
        descendant != ancestor
            && self.ancestry.ancestors(descendant).contains(ancestor)
            && !self.ancestry.ancestors(ancestor).contains(descendant)
    }
}

/// The text over one image ([`texts_over`]).
#[derive(Debug, Default)]
pub struct Over<'a> {
    /// The text elements over it that are measured, each with the overlap
    /// region.
    pub pairs: Vec<TextOverImage<'a>>,
    /// Text elements over it that cannot themselves be attributed to the
    /// frame ([`unreadable`]).
    pub unreadable_texts: usize,
    /// Wrapper text elements over it whose text is all their leaf
    /// children's: measured through those children.
    pub wrappers: usize,
    /// Text elements over it whose box is smaller than
    /// [`MIN_MEASURABLE_BOX_CSS_PX2`]: no legible glyph fits (a `.sr-only`
    /// label), so they are not text over the image.
    pub too_small: usize,
}

/// The text elements laid over `image` ([`TextIndex`]) that are measured:
///
/// - every LEAF text element whose box meets the image's. A leaf box that
///   contains the image is not skipped on its box: an inset-0 overlay whose
///   text node is its own has exactly the image's box. The glyph test decides
///   ([`measure`]'s `None` when its glyphs do not reach the image).
/// - every WRAPPER that paints text of its own
///   ([`TextIndex::paints_own_text`]). A wrapper whose text is all its
///   children's is measured through them and counted ([`Over::wrappers`])
///   unless its box contains the image whole (a section, a card: a
///   container, not an overlay).
/// - a CSS-background `image` with text of its own, over itself.
///
/// A box under [`MIN_MEASURABLE_BOX_CSS_PX2`] is counted
/// ([`Over::too_small`]) and left out. The image itself is gated by the
/// caller on [`unrendered`] only: an OCCLUDED image (the usual `<img>` hero
/// under an inset-0 scrim or text layer) is measured, because nothing here
/// reads the image's crop — frame B shows whatever the glyphs sit on. An
/// image with no box has nothing over it here; the caller counts it
/// ([`PagePairs::boxless_images`]).
pub fn texts_over<'a>(
    snapshot: &'a ElementSnapshot,
    image: &'a Element,
    index: &TextIndex<'a>,
) -> Over<'a> {
    let mut over = Over::default();
    let Some(img) = image.bbox else {
        return over;
    };
    for t in &snapshot.elements {
        let itself = t.id == image.id;
        if itself {
            if !paints_own_text_media(snapshot, t) {
                continue;
            }
        } else if !paints_text(snapshot, t) {
            continue;
        }
        let Some(tb) = t.bbox else { continue };
        let Some(css) = intersection(tb, img) else {
            continue;
        };
        if !index.is_leaf(&t.id) && !index.paints_own_text(&t.id) {
            if !itself && !region_contains(tb, img) {
                over.wrappers += 1;
            }
            continue;
        }
        if (tb.w as f64) * (tb.h as f64) < MIN_MEASURABLE_BOX_CSS_PX2 {
            over.too_small += 1;
            continue;
        }
        if unreadable(snapshot, t).is_some() {
            over.unreadable_texts += 1;
            continue;
        }
        let others: Vec<&Element> = index
            .painters()
            .iter()
            .copied()
            .filter(|o| o.id != t.id)
            .filter(|o| o.bbox.is_some_and(|ob| intersection(ob, tb).is_some()))
            .collect();
        let area = |r: Region| r.w as u64 * r.h as u64;
        let t_area = area(tb);
        let inner = others
            .iter()
            .map(|o| {
                index.descends(&o.id, &t.id)
                    || o.bbox.is_some_and(|ob| {
                        // The overlap lies in this box, so `t_area - overlap`
                        // cannot wrap; `2 * overlap` could, for a box near
                        // `u32::MAX` square.
                        let overlap = intersection(ob, tb).map_or(0, area);
                        area(ob) <= t_area
                            && !(index.descends(&t.id, &o.id) && overlap >= t_area - overlap)
                    })
            })
            .collect();
        over.pairs.push(TextOverImage {
            text: t,
            image,
            css,
            others,
            inner,
            image_inline: !itself && index.descends(&image.id, &t.id),
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
    /// Wrapper text elements over an image whose text is all their
    /// children's, measured through them (see [`Over::wrappers`]).
    pub wrappers: usize,
    /// Text elements over an image whose box holds no legible glyph (see
    /// [`Over::too_small`]).
    pub too_small: usize,
    /// Rendered images with no box: whether any text lies over them is
    /// unknown.
    pub boxless_images: usize,
    /// Painted text elements with no box: whether they lie over an image is
    /// unknown.
    pub boxless_texts: usize,
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
        if self.wrappers > 0 {
            note.push_str(&format!(
                " ({} wrapper text element(s) over an image measured through their children, \
                 whose text is all of theirs)",
                self.wrappers
            ));
        }
        if self.too_small > 0 {
            note.push_str(&format!(
                " ({} text element(s) over an image have a box under {:.1} CSS px², which \
                 holds no legible glyph (a visually hidden label), and were not measured)",
                self.too_small, MIN_MEASURABLE_BOX_CSS_PX2
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
pub fn page_pairs<'a>(snapshot: &'a ElementSnapshot, index: &TextIndex<'a>) -> PagePairs<'a> {
    let mut page = PagePairs {
        boxless_texts: index.boxless_texts(),
        ..Default::default()
    };
    for image in snapshot.elements.iter().filter(|e| e.media.is_some()) {
        let over = texts_over(snapshot, image, index);
        if unrendered(snapshot, image).is_some() {
            if !over.pairs.is_empty() || over.unreadable_texts > 0 {
                page.skipped_images += 1;
            }
            continue;
        }
        if image.bbox.is_none() {
            page.boxless_images += 1;
            continue;
        }
        page.skipped_texts += over.unreadable_texts;
        page.wrappers += over.wrappers;
        page.too_small += over.too_small;
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

/// `b` blended toward `c` by `alpha` (0..=1), per channel: what a glyph of
/// colour `c` at coverage (or opacity) `alpha` paints over `b`.
fn blend(b: Rgb, c: Rgb, alpha: f64) -> Rgb {
    let ch = |p: u8, q: u8| {
        (p as f64 + alpha * (q as f64 - p as f64))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgb::new(ch(b.r, c.r), ch(b.g, c.g), ch(b.b, c.b))
}

/// The squared distance of frame A's pixel from the alpha blend of frame B
/// toward `c` at its own coverage (clamped to 0..=1): how far a glyph of
/// colour `c` fails to explain it. Zero for every pixel a glyph of exactly
/// that colour paints, at any coverage or opacity.
fn residual(pa: Rgb, pb: Rgb, c: Rgb) -> f64 {
    let alpha = coverage(pa, pb, c).clamp(0.0, 1.0);
    let lerp = |p: u8, q: u8| p as f64 + alpha * (q as f64 - p as f64);
    let (er, eg, eb) = (
        pa.r as f64 - lerp(pb.r, c.r),
        pa.g as f64 - lerp(pb.g, c.g),
        pa.b as f64 - lerp(pb.b, c.b),
    );
    er * er + eg * eg + eb * eb
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

/// The nearest-rank `percentile` (0..=100) of `sorted` (ascending); `None`
/// when it is empty.
/// The rank is `percentile * n / 100`, multiplied first: `p / 100 * n` reads
/// 7 / 100 * 100 as 7.000000000000001 and takes the eighth value.
pub fn nearest_rank<T: Copy + Into<f64>>(sorted: &[T], percentile: f64) -> Option<f64> {
    let n = sorted.len();
    (n > 0).then(|| sorted[nearest_rank_index(n, percentile)].into())
}

/// The 0-based index [`nearest_rank`] reads in `n >= 1` sorted values.
fn nearest_rank_index(n: usize, percentile: f64) -> usize {
    let rank = (percentile * n as f64 / 100.0).ceil() as usize;
    rank.clamp(1, n) - 1
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

/// A CSS box mapped onto the buffer and widened by `margin` px, as half-open
/// `(x0, y0, x1, y1)` bounds that may extend past the frame.
fn widened(frame: &Frame, css: Region, margin: i64) -> Option<(i64, i64, i64, i64)> {
    let r = frame.css_region_to_pixels(css)?;
    Some((
        r.x as i64 - margin,
        r.y as i64 - margin,
        r.right() + margin,
        r.bottom() + margin,
    ))
}

fn inside((x0, y0, x1, y1): (i64, i64, i64, i64), x: i64, y: i64) -> bool {
    x >= x0 && x < x1 && y >= y0 && y < y1
}

/// A boolean bitmap over a half-open buffer rect, which may extend past the
/// frame. Reads outside the rect are `false`.
#[derive(Debug, Clone)]
struct Grid {
    x0: i64,
    y0: i64,
    w: i64,
    h: i64,
    bits: Vec<bool>,
}

impl Grid {
    fn new((x0, y0, x1, y1): (i64, i64, i64, i64)) -> Self {
        let (w, h) = ((x1 - x0).max(0), (y1 - y0).max(0));
        Self {
            x0,
            y0,
            w,
            h,
            bits: vec![false; (w * h) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (dx, dy) = (x - self.x0, y - self.y0);
        (dx >= 0 && dy >= 0 && dx < self.w && dy < self.h).then(|| (dy * self.w + dx) as usize)
    }

    fn set(&mut self, x: i64, y: i64) {
        if let Some(i) = self.index(x, y) {
            self.bits[i] = true;
        }
    }

    fn get(&self, x: i64, y: i64) -> bool {
        self.index(x, y).is_some_and(|i| self.bits[i])
    }

    /// Set every cell of the half-open rect, clipped to the grid.
    fn fill(&mut self, (x0, y0, x1, y1): (i64, i64, i64, i64)) {
        for y in y0.max(self.y0)..y1.min(self.y0 + self.h) {
            let row = ((y - self.y0) * self.w) as usize;
            for x in x0.max(self.x0)..x1.min(self.x0 + self.w) {
                self.bits[row + (x - self.x0) as usize] = true;
            }
        }
    }

    /// Every cell within Chebyshev distance `r` of a set cell, in two
    /// separable running-count passes: O(cells), whatever `r`.
    fn dilated(&self, r: i64) -> Self {
        if r <= 0 {
            return self.clone();
        }
        let (w, h, r) = (self.w as usize, self.h as usize, r as usize);
        let mut across = vec![false; w * h];
        let mut prefix = vec![0u32; w.max(h) + 1];
        for y in 0..h {
            for x in 0..w {
                prefix[x + 1] = prefix[x] + self.bits[y * w + x] as u32;
            }
            for x in 0..w {
                across[y * w + x] = prefix[(x + r + 1).min(w)] > prefix[x.saturating_sub(r)];
            }
        }
        let mut bits = vec![false; w * h];
        for x in 0..w {
            for y in 0..h {
                prefix[y + 1] = prefix[y] + across[y * w + x] as u32;
            }
            for y in 0..h {
                bits[y * w + x] = prefix[(y + r + 1).min(h)] > prefix[y.saturating_sub(r)];
            }
        }
        Self {
            x0: self.x0,
            y0: self.y0,
            w: self.w,
            h: self.h,
            bits,
        }
    }
}

/// One pass over `px`: the pixels where frames `a` and `b` differ by more
/// than [`GLYPH_DIFF_MIN`], that `keep` attributes to the text, and that
/// moved toward `colour` (`None`: every attributed differing pixel), as
/// `(x, y, squared distance)`; and how many attributed pixels differed past
/// the threshold in any direction. `None` as soon as more than `cap` pixels
/// are kept: the caller has no use for a mask that dense.
#[allow(clippy::type_complexity)]
fn diff_mask(
    a: &Frame,
    b: &Frame,
    (x0, y0, w, h): (u32, u32, u32, u32),
    colour: Option<Rgb>,
    cap: usize,
    keep: &dyn Fn(u32, u32, Rgb, Rgb) -> bool,
) -> Option<(Vec<(u32, u32, f64)>, usize)> {
    let min2 = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let mut mask = Vec::new();
    let mut raw = 0usize;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
            let d = dist2(pa, pb);
            if d <= min2 || !keep(x, y, pa, pb) {
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
/// rect, its hole bitmap (the widened boxes of the elements that paint
/// glyphs of their own over it, computed once), those boxes themselves, and
/// the text colours a removed glow moved away from.
#[derive(Debug, Clone, PartialEq)]
pub struct Registration {
    /// Share of the image's pixels away from the text that differ below
    /// [`GLYPH_DIFF_MIN`]: the capture's noise.
    pub noise_share: f64,
    rect: (u32, u32, u32, u32),
    /// The image's CSS box.
    image: Region,
    holes: Vec<bool>,
    /// Each hole's widened buffer bounds and the CSS box it widens.
    boxes: Vec<((i64, i64, i64, i64), Region)>,
    colours: Vec<Rgb>,
    /// The glyph colour of every element that paints glyphs of its own over
    /// the image, by id: declared, or measured once over its box (`None`
    /// when its box shows too few differing pixels to paint a mask).
    painted: HashMap<String, Option<Rgb>>,
    tile: u32,
    margin: i64,
}

impl Registration {
    /// The glyph colour `el` paints, from the per-image cache when it paints
    /// over this image, else measured now.
    fn colour_of(&self, a: &Frame, b: &Frame, el: &Element) -> Option<Rgb> {
        match self.painted.get(&el.id) {
            Some(c) => *c,
            None => painter_colour(a, b, el, min_glyph_px(a.source.scale_factor)),
        }
    }

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
        let (x0, y0, x1, y1) = widened(
            a,
            text_box,
            self.margin + LOCAL_RING_TILES * self.tile as i64,
        )?;
        let (ix, iy, iw, ih) = self.rect;
        let x0 = x0.max(ix as i64);
        let y0 = y0.max(iy as i64);
        let x1 = x1.min((ix + iw) as i64);
        let y1 = y1.min((iy + ih) as i64);
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }

    /// The holes the ring around `text_box` is read through: every hole
    /// except those whose box CONTAINS the text's (module docs: the padding
    /// of a card around its heading is what the ring must read), and the
    /// text's own widened box unless it contains the whole image — then, as
    /// in the global check, its glyphs read as untracked text.
    fn ring_holes(&self, a: &Frame, text_box: Region, ring: (i64, i64, i64, i64)) -> Grid {
        let mut g = Grid::new(ring);
        for (px, css) in &self.boxes {
            if !region_contains(*css, text_box) {
                g.fill(*px);
            }
        }
        if !region_contains(text_box, self.image) {
            if let Some(own) = widened(a, text_box, self.margin) {
                g.fill(own);
            }
        }
        g
    }

    /// The share of the IMAGE's frame-B pixels in the ring around
    /// `text_box` (outside its holes, [`Self::ring_holes`]) within
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
        let Some(ring) = self.ring(a, text_box) else {
            return 0.0;
        };
        let holes = self.ring_holes(a, text_box, ring);
        let (x0, y0, x1, y1) = ring;
        let near = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
        let (mut n, mut k) = (0usize, 0usize);
        for y in y0..y1 {
            for x in x0..x1 {
                if holes.get(x, y) {
                    continue;
                }
                n += 1;
                k += (dist2(rgb_at(b, x as u32, y as u32), colour) <= near) as usize;
            }
        }
        k as f64 / n.max(1) as f64
    }

    /// The LOCAL registration check for one text element: the tiles in a
    /// ring [`LOCAL_RING_TILES`] tiles wide around its widened box, inside
    /// the image and outside its holes ([`Self::ring_holes`]), must not show
    /// content DISPLACED between the frames.
    ///
    /// A differing, textured ring tile is displaced unless its difference
    /// reads as TEXT that only frame A carries (an untracked tagline, a
    /// credit, a containing card's own words). Text is alpha-blended over the
    /// image, so every one of its differing pixels moved toward ONE colour,
    /// its own: at least [`LOCAL_TEXT_LIKE_SHARE`] of the tile's differing
    /// pixels must lie closer in A than in B to the colour of the tile's
    /// most-changed quarter. Moved texture — a parallax layer at any offset,
    /// a rotation, a rescale — goes both ways, while untracked text moves one
    /// way: measured on the committed fixtures during review (the `parallax`
    /// twins against the `credit` and `tagline` cases).
    /// A text-like tile — few differing pixels can look text-like by chance —
    /// is still displaced when one shift of frame B by up to
    /// [`LOCAL_MAX_SHIFT_CSS`] CSS px lines up more than half of its
    /// differing pixels: moved image does, text only frame A carries does
    /// not. Refused when at least [`LOCAL_DISPLACED_TILE_SHARE`] of the
    /// ring's tiles (and at least one) are displaced, or when ANY tile of the
    /// ring's first row — within one tile of the widened text box — is: a
    /// panel moving just under the text shows only there. The global check
    /// cannot see this: a layer under part of the hero is under half its
    /// tiles. A ring with no tile mostly outside its holes (neighbouring text
    /// covering it) confirms nothing, and is refused too.
    pub fn local(&self, a: &Frame, b: &Frame, text_box: Region) -> Result<(), LocateError> {
        let s = a.source.scale_factor;
        let ts = self.tile as i64;
        let unconfirmed = || {
            (
                UnknownCode::InputMissing,
                format!(
                    "no {REGISTRATION_TILE_CSS}-CSS-px tile of the image around this text lies \
                     mostly outside every text box, so the text-hidden frame's registration with \
                     the frame around it cannot be confirmed"
                ),
            )
        };
        let Some(ring) = self.ring(a, text_box) else {
            return Err(unconfirmed());
        };
        let holes = self.ring_holes(a, text_box, ring);
        let (x0, y0, x1, y1) = ring;
        // The first row of tiles around the hole: within one tile of the
        // widened text box. A layer moving just under the text shows there
        // first, and one displaced tile there is enough.
        let first_row = widened(a, text_box, self.margin + ts);
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
                        if holes.get(x, y) {
                            continue;
                        }
                        let (x, y) = (x as u32, y as u32);
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
                let all_moved: Vec<(u32, u32)> = px
                    .iter()
                    .copied()
                    .filter(|(x, y)| dist2(rgb_at(a, *x, *y), rgb_at(b, *x, *y)) > min2)
                    .collect();
                // A fixed, evenly strided sample of them, so the search costs
                // the same per tile at every scale factor.
                let stride = all_moved.len().div_ceil(LOCAL_SHIFT_SAMPLE).max(1);
                let moved: Vec<(u32, u32)> = all_moved.into_iter().step_by(stride).collect();
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
        // Text boxes covering the whole ring leave nothing to compare: the
        // global check refuses the same situation over the whole image.
        if tiles == 0 {
            return Err(unconfirmed());
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
/// tile test). The box of every element that paints glyphs of its own over
/// the image ([`TextIndex::painters`]) is a hole, not-visible ones included,
/// so no glyph is mistaken for misregistration; a wrapper whose text is all
/// its children's paints nothing its children's holes do not already cover.
/// The caller has already paired the frames ([`pair_frames`]). The per-text
/// local check is [`Registration::local`].
pub fn registered(
    a: &Frame,
    b: &Frame,
    image: &Element,
    index: &TextIndex<'_>,
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
    let mut holes = Grid::new((ix as i64, iy as i64, (ix + iw) as i64, (iy + ih) as i64));
    let mut boxes = Vec::new();
    let mut colours: Vec<Rgb> = Vec::new();
    let mut painted = HashMap::new();
    let floor = min_glyph_px(s);
    for t in index.painters() {
        let Some(tb) = t.bbox else { continue };
        if intersection(tb, img).is_none() {
            continue;
        }
        let colour = painter_colour(a, b, t, floor);
        painted.insert(t.id.clone(), colour);
        // A box containing the whole image leaves no image to register if cut
        // as a hole. Its glyphs read here as untracked text instead: tolerated
        // when compact, refused (Unknown) when they spread — never a pass.
        if t.id == image.id || region_contains(tb, img) {
            continue;
        }
        if let Some(c) = colour {
            colours.push(c);
        }
        let Some(px) = widened(a, tb, margin) else {
            continue;
        };
        holes.fill(px);
        boxes.push((px, tb));
    }
    let ts = ((REGISTRATION_TILE_CSS * s).ceil() as u32).max(1);
    let mut reg = Registration {
        noise_share: 0.0,
        rect: (ix, iy, iw, ih),
        image: img,
        holes: holes.bits,
        boxes,
        colours,
        painted,
        tile: ts,
        margin,
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

/// The glyph colour an element paints: its declared `fg_color`, or the colour
/// measured over its box's raw difference ([`measured_glyph_colour`]); `None`
/// when the box shows fewer than `floor` differing pixels.
fn painter_colour(a: &Frame, b: &Frame, el: &Element, floor: usize) -> Option<Rgb> {
    el.fg_color.or_else(|| {
        let (px, _) = on_frame(a, el.bbox?)?;
        let area = px.2 as usize * px.3 as usize;
        let (raw, _) = diff_mask(a, b, px, None, area, &|_, _, _, _| true)?;
        (raw.len() >= floor).then(|| measured_glyph_colour(a, &raw).0)
    })
}

/// The per-channel median of frame B under the core of a raw difference
/// mask (the quarter of its pixels that differ most): what the measured
/// glyph colour sits on.
fn median_under_core(b: &Frame, mask: &[(u32, u32, f64)]) -> Rgb {
    let mut by_diff: Vec<&(u32, u32, f64)> = mask.iter().collect();
    by_diff.sort_by(|p, q| q.2.total_cmp(&p.2));
    let under: Vec<Rgb> = by_diff[..mask.len().div_ceil(4)]
        .iter()
        .map(|(x, y, _)| rgb_at(b, *x, *y))
        .collect();
    let median = |f: fn(&Rgb) -> u8| {
        let mut v: Vec<u8> = under.iter().map(f).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    Rgb::new(median(|c| c.r), median(|c| c.g), median(|c| c.b))
}

/// Another glyph-painting text whose box meets the measured one: its widened
/// buffer bounds, and the colour its glyphs paint (`None`: its box shows too
/// few differing pixels to paint a mask of its own).
struct Other {
    px: (i64, i64, i64, i64),
    colour: Option<Rgb>,
    /// Whether its box contains the measured text's (a card around its
    /// heading, a wrapper around its inline child).
    contains: bool,
    /// Whether its ink may be all the measured text's glyphs show
    /// ([`TextOverImage::inner`]).
    inner: bool,
}

/// Measure one text element over one image on the frame pair `(a, b)`, which
/// the caller has already paired ([`pair_frames`]) and registered over the
/// image (`reg`, from [`registered`]). See the module docs.
///
/// `Ok(None)` when the text's box shows glyphs and fewer than the glyph floor
/// of them reach the image, and the image under the box is not blind enough
/// to hide that many: not text over it. `Err` (Unknown) when the box shows no
/// usable mask, when the differing pixels moved away from the glyph colour,
/// when frame B still paints the glyphs, when the glyph colour does not
/// explain the glyphs, when the scale factor is unusable ([`usable_scale`]),
/// or when the image under the box could hide the glyphs the mask did not
/// find there.
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
    let margin = (REGISTRATION_MARGIN_CSS * s).ceil() as i64;
    let tol2 = GLYPH_COLOUR_TOLERANCE * GLYPH_COLOUR_TOLERANCE;
    let dense = || {
        unknown(format!(
            "frames A and B differ toward the glyph colour at more than {:.0}% of text '{id}''s \
             box, which is not glyphs, so no contrast was measured",
            MAX_GLYPH_SHARE * 100.0
        ))
    };

    // The other glyph-painting texts whose boxes meet this one, with the
    // colour each paints: declared, or measured over its own box.
    let others: Vec<Other> = pair
        .others
        .iter()
        .zip(pair.inner.iter().copied().chain(std::iter::repeat(true)))
        .filter_map(|(o, inner)| {
            let ob = o.bbox?;
            let colour = reg.colour_of(a, b, o);
            Some(Other {
                px: widened(a, ob, margin)?,
                colour,
                contains: region_contains(ob, text_box),
                inner,
            })
        })
        .collect();
    let in_another = |x: u32, y: u32| {
        others
            .iter()
            .any(|o| !o.contains && o.colour.is_some() && inside(o.px, x as i64, y as i64))
    };
    // Whether a differing pixel is THIS text's ink (module docs, "Whose
    // glyphs they are").
    let mine = |x: u32, y: u32, pa: Rgb, pb: Rgb, c: Rgb| {
        others.iter().all(|o| match o.colour {
            _ if !inside(o.px, x as i64, y as i64) => true,
            None => true,
            Some(oc) if dist2(oc, c) <= tol2 => true,
            Some(oc) => {
                let r = residual(pa, pb, c);
                r <= tol2 || r < residual(pa, pb, oc)
            }
        })
    };

    // The glyph colour first: the mask keeps only pixels that moved toward it.
    // A measured colour is read outside every other text's box that does not
    // contain this one's, so another colour's glyphs cannot take its core (a
    // containing box would leave nothing to read; the colour fit below
    // refuses a colour its glyphs do not paint).
    //
    // A MEASURED colour is only what the best-covered pixels show: a glyph
    // that never covers a whole pixel renders a more extreme ink, on the ray
    // from the background through that colour. The ray's far end (`beyond`)
    // bounds the ink.
    let (glyph_colour, source, colour_conf, beyond) = match pair.text.fg_color {
        Some(fg) => (fg, GlyphColourSource::Declared, 1.0, None),
        None => {
            let (raw, _) = diff_mask(a, b, box_px, None, area, &|x, y, _, _| !in_another(x, y))
                .ok_or_else(dense)?;
            if raw.len() < floor {
                return Err(not_hidden(id, raw.len(), floor));
            }
            let (c, agree) = measured_glyph_colour(a, &raw);
            let under = median_under_core(b, &raw);
            (c, GlyphColourSource::Measured, agree, ray_exit(under, c))
        }
    };
    let c = glyph_colour;
    let (mask, raw) = diff_mask(a, b, box_px, Some(c), cap, &|x, y, pa, pb| {
        mine(x, y, pa, pb, c)
    })
    .ok_or_else(dense)?;
    if !mask_usable(mask.len(), area, s) {
        if raw >= floor {
            return Err(unknown(format!(
                "frames A and B differ at {raw} pixel(s) of text '{id}''s box, but only {} moved \
                 toward its glyph colour rgb({},{},{}) (under the {floor} needed): the declared \
                 colour is not what paints, or only a shadow differs",
                mask.len(),
                c.r,
                c.g,
                c.b
            )));
        }
        return Err(not_hidden(id, mask.len(), floor));
    }
    let box_rect = (
        box_px.0 as i64,
        box_px.1 as i64,
        (box_px.0 + box_px.2) as i64,
        (box_px.1 + box_px.3) as i64,
    );
    let mut is_mask = Grid::new(box_rect);
    for (x, y, _) in &mask {
        is_mask.set(*x as i64, *y as i64);
    }

    // Frame B still painting the glyphs: an unhidden twin, re-rasterised.
    // Where text was hidden, B around a mask pixel is background; where it
    // was not, the glyph is still there, a sub-pixel away.
    let near = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let reach = s.ceil().max(1.0) as i64;
    let (fw, fh) = (b.width as i64, b.height as i64);
    let around = (
        (box_rect.0 - reach).max(0),
        (box_rect.1 - reach).max(0),
        (box_rect.2 + reach).min(fw),
        (box_rect.3 + reach).min(fh),
    );
    let mut glyph_in_b = Grid::new(around);
    for y in around.1..around.3 {
        for x in around.0..around.2 {
            let (na, nb) = (rgb_at(a, x as u32, y as u32), rgb_at(b, x as u32, y as u32));
            if na != nb && dist2(nb, c) <= near {
                glyph_in_b.set(x, y);
            }
        }
    }
    let glyph_in_b = glyph_in_b.dilated(reach);
    let still = mask
        .iter()
        .filter(|(x, y, _)| glyph_in_b.get(*x as i64, *y as i64))
        .count();
    // The rate the IMAGE itself explains: how often a pixel of the image
    // AROUND this text (outside its holes) is glyph-coloured in B (a mottle,
    // a near-white sky). Read outside the text, so an unhidden twin's own
    // glyph cores cannot inflate it. A neighbourhood of n such pixels holds
    // one by chance with probability 1 - (1 - q)^n; only the excess over
    // that is a still-painted glyph.
    let q = reg.glyph_coloured_share_around(a, b, text_box, c);
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

    // Each glyph pixel's coverage of the colour, on a grid over the box.
    let (bw, bh) = (box_px.2 as usize, box_px.3 as usize);
    let cell = |x: u32, y: u32| (y - box_px.1) as usize * bw + (x - box_px.0) as usize;
    let mut cov_grid = vec![0.0f32; bw * bh];
    let mut by_cov: Vec<f64> = Vec::with_capacity(mask.len());
    for (x, y, _) in &mask {
        let cv = coverage(rgb_at(a, *x, *y), rgb_at(b, *x, *y), c).clamp(0.0, 1.0);
        cov_grid[cell(*x, *y)] = cv as f32;
        by_cov.push(cv);
    }
    by_cov.sort_by(|p, q| q.total_cmp(p));
    // The glyph's robust PEAK coverage: the highest that a floor's worth of
    // its mask pixels reach (the mask holds at least `floor`). It sizes the
    // blind radius below, nothing else.
    let peak = by_cov[floor.min(by_cov.len()) - 1];
    let min_coverage = (MIN_GLYPH_COVERAGE_SHARE * peak).max(MIN_GLYPH_COVERAGE_ABS);
    // A mask pixel's coverage, `None` off the mask; compared as stored.
    let mask_cov = |x: i64, y: i64| {
        is_mask
            .get(x, y)
            .then(|| cov_grid[cell(x as u32, y as u32)])
    };
    let ink_cov = INK_COVERAGE as f32;
    // The stroke INTERIORS: mask pixels covered nearly whole whose every
    // neighbour is too. An anti-aliased edge has a less-covered pixel beside
    // it, so what an interior pixel renders is what the glyph paints at full
    // coverage, whatever the background.
    let interior: Vec<(u32, u32)> = mask
        .iter()
        .map(|(x, y, _)| (*x, *y))
        .filter(|&(x, y)| {
            let (x, y) = (x as i64, y as i64);
            (-1..=1)
                .all(|dy| (-1..=1).all(|dx| mask_cov(x + dx, y + dy).is_some_and(|v| v >= ink_cov)))
        })
        .collect();
    // A MEASURED colour is what the best-covered pixels show. Where the glyph
    // has stroke interiors it is what the glyph renders over a whole pixel;
    // with none the glyph may be a hairline of a more extreme colour on the
    // ray, and the best case reads that colour too.
    let whole = beyond.is_none() || interior.len() >= floor;
    // Inside the box of another glyph-painting text whose colour explains it
    // too, a pixel may be either's (module docs, "Whose glyphs they are").
    let shared = |x: u32, y: u32, pa: Rgb, pb: Rgb| {
        others.iter().any(|o| {
            o.colour.is_some_and(|oc| {
                inside(o.px, x as i64, y as i64)
                    && (dist2(oc, c) <= tol2 || residual(pa, pb, oc) <= tol2)
            })
        })
    };
    // The INK: every colour the glyph may render over a whole pixel (module
    // docs, "Glyphs the mask cannot see"). Its colour; what each stroke
    // interior renders; the bounds its glyph pixels set on ONE fill — an
    // ink renders nothing beyond itself, so per channel it lies at or past
    // the furthest any glyph pixel moved, and where no stroke covers a pixel
    // whole (too thin, or ink a few units off the colour) it may be anything
    // from there to the colour. A few pixels in another colour (a dotted
    // underline, a child link's own text-fill colour) would pin that bound,
    // so the bound where the MAJORITY of this text's own glyph pixels reach
    // is kept too. Each line-like stroke bounds its own colour (a decoration
    // in another colour is not the fill). And for a MEASURED colour, the ray
    // past it to the edge of the RGB cube, which a translucent ink further
    // along it renders as that colour too.
    let mut ink = Ink::at(c);
    for &(x, y) in &interior {
        ink.include(rgb_at(a, x, y));
    }
    let mut furthest = Reach::default();
    // Each stroke's reach over this text's OWN glyph pixels (not shared with
    // another text), with how many there are; every pixel when none is.
    let mut strokes: Vec<(Reach, usize)> = Vec::new();
    let mut all_strokes: Vec<(Reach, usize)> = Vec::new();
    // Whether a pixel may be the ink of an inner text: inside its box, in a
    // colour its colour explains (as `shared` reads it).
    let inner_ink = |x: u32, y: u32, pa: Rgb, pb: Rgb| {
        others.iter().any(|o| {
            o.inner
                && o.colour.is_some_and(|oc| {
                    inside(o.px, x as i64, y as i64)
                        && (dist2(oc, c) <= tol2 || residual(pa, pb, oc) <= tol2)
                })
        })
    };
    // How many of its stroke pixels no inner text may have painted.
    let mut apart = 0usize;
    // Each mask pixel's connected stroke (1-based; 0 = not yet reached), and
    // each stroke's peak coverage.
    let mut stroke_of = vec![0u32; bw * bh];
    let mut stroke_peak: Vec<f32> = vec![0.0];
    let mut stack: Vec<(i64, i64)> = Vec::new();
    for (x, y, _) in &mask {
        if stroke_of[cell(*x, *y)] != 0 {
            continue;
        }
        let label = stroke_peak.len() as u32;
        stroke_of[cell(*x, *y)] = label;
        let start = (*x as i64, *y as i64);
        stack.push(start);
        let (mut stroke, mut own) = (Reach::default(), Reach::default());
        let (mut n, mut n_own, mut n_apart) = (0usize, 0usize, 0usize);
        let mut top = 0.0f32;
        let (mut lo, mut hi) = (start, start);
        while let Some((x, y)) = stack.pop() {
            let (ux, uy) = (x as u32, y as u32);
            let (pa, pb) = (rgb_at(a, ux, uy), rgb_at(b, ux, uy));
            stroke.add(pa, pb);
            n += 1;
            top = top.max(cov_grid[cell(ux, uy)]);
            if !shared(ux, uy, pa, pb) {
                own.add(pa, pb);
                n_own += 1;
            }
            n_apart += !inner_ink(ux, uy, pa, pb) as usize;
            (lo, hi) = ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y)));
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if is_mask.get(nx, ny) && stroke_of[cell(nx as u32, ny as u32)] == 0 {
                        stroke_of[cell(nx as u32, ny as u32)] = label;
                        stack.push((nx, ny));
                    }
                }
            }
        }
        stroke_peak.push(top);
        // An underline, an overline, a line-through: far wider than tall.
        let (lw, lh) = (hi.0 - lo.0 + 1, hi.1 - lo.1 + 1);
        if lw >= LINE_ASPECT * lh {
            stroke.bound(c, &mut ink);
        } else {
            furthest.merge(&stroke);
            all_strokes.push((stroke, n));
            apart += n_apart;
            if n_own > 0 {
                strokes.push((own, n_own));
            }
        }
    }
    furthest.bound(c, &mut ink);
    let voters = if strokes.is_empty() {
        &all_strokes
    } else {
        &strokes
    };
    Reach::majority(voters).bound(c, &mut ink);
    if let Some(e) = beyond {
        ink.include(e);
    }
    // Moved toward the ink: nearer the ink set in A than in B, or toward the
    // colour (or the ray's far end) itself.
    let toward_ink = |pa: Rgb, pb: Rgb| {
        ink.dist2(pa) < ink.dist2(pb)
            || toward(pa, pb, c)
            || beyond.is_some_and(|e| toward(pa, pb, e))
    };
    // BLIND: a glyph covering a pixel at `min_coverage` moves it by
    // `min_coverage` times the background's distance from the ink, which
    // stays at or under GLYPH_DIFF_MIN (so the mask misses a pixel the
    // sampling would keep) wherever that distance is at most
    // GLYPH_DIFF_MIN / min_coverage. Ink rendered on a pixel inside the ink
    // set changes it by nothing at all: UNDETECTABLE.
    let blind = (GLYPH_DIFF_MIN / min_coverage).powi(2);
    let (ox0, oy0, ow, oh) = over_px;
    let over = |x: u32, y: u32| x >= ox0 && x < ox0 + ow && y >= oy0 && y < oy0 + oh;
    let trusted = reg.faint_trusted();
    let in_box = |x: u32, y: u32| {
        x >= box_px.0 && x < box_px.0 + box_px.2 && y >= box_px.1 && y < box_px.1 + box_px.3
    };
    // Glyph pixels the mask left out (module docs, "Glyphs the mask cannot
    // see"). OFF-COLOUR ones moved past GLYPH_DIFF_MIN toward the ink but
    // not toward the colour: ink a few units off it, over a background
    // between the two. SUB-THRESHOLD ones belong to a glyph too faint to
    // move its pixel past GLYPH_DIFF_MIN (a word at 2% opacity), read away
    // from the visible strokes, whose anti-aliasing fringe this would
    // otherwise sample: on a noise-free capture they are sampled like the
    // mask; on a noisy one the capture's noise paints alike, and they are
    // NOISE-OR-FAINT, optional.
    let mut off_colour = Grid::new(box_rect);
    let mut noise = Grid::new(box_rect);
    let clear = is_mask.dilated((SUB_THRESHOLD_CLEARANCE_CSS * s).ceil() as i64);
    for y in oy0..oy0 + oh {
        for x in ox0..ox0 + ow {
            if !in_box(x, y) || is_mask.get(x as i64, y as i64) {
                continue;
            }
            let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
            if pa == pb || !mine(x, y, pa, pb, c) {
                continue;
            }
            if dist2(pa, pb) > GLYPH_DIFF_MIN * GLYPH_DIFF_MIN {
                if ink.dist2(pa) < ink.dist2(pb) {
                    off_colour.set(x as i64, y as i64);
                }
            } else if !clear.get(x as i64, y as i64) && ink.dist2(pb) > blind && toward_ink(pa, pb)
            {
                if trusted {
                    cov_grid[cell(x, y)] = coverage(pa, pb, c).clamp(0.0, 1.0) as f32;
                } else {
                    noise.set(x as i64, y as i64);
                }
            }
        }
    }
    // Off the mask, the pixels JOINED to it by pixels that moved toward the
    // ink: a glow's ramp, at any distance from the strokes. A faint glyph
    // joined to a visible one paints alike there.
    // Seeded one mask pixel at a time, so the stack holds only the frontier.
    let mut linked = Grid::new(box_rect);
    let mut stack: Vec<(i64, i64)> = Vec::new();
    for (mx, my, _) in &mask {
        stack.push((*mx as i64, *my as i64));
        while let Some((x, y)) = stack.pop() {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < box_rect.0
                        || ny < box_rect.1
                        || nx >= box_rect.2
                        || ny >= box_rect.3
                        || is_mask.get(nx, ny)
                        || linked.get(nx, ny)
                    {
                        continue;
                    }
                    let (ux, uy) = (nx as u32, ny as u32);
                    let (pa, pb) = (rgb_at(a, ux, uy), rgb_at(b, ux, uy));
                    if pa != pb && toward_ink(pa, pb) && mine(ux, uy, pa, pb, c) {
                        linked.set(nx, ny);
                        stack.push((nx, ny));
                    }
                }
            }
        }
    }
    // Each pixel's LOCAL peak, the best coverage within one CSS px, and its
    // HALO peak, the best within HALO_CSS.
    let r1 = s.ceil().max(1.0) as usize;
    let halo_r = (HALO_CSS * s).ceil() as usize;
    let local = sliding_max(&cov_grid, bw, bh, r1);
    let wide = sliding_max(&cov_grid, bw, bh, halo_r);
    // A pixel covered at least half as well as its local peak is SAMPLED at
    // that peak: an anti-aliased edge beside a better-covered stroke is left
    // out, and a faint glyph is its own local peak.
    let share = MIN_GLYPH_COVERAGE_SHARE as f32;
    let sampled_at = |x: u32, y: u32| -> Option<f64> {
        let i = cell(x, y);
        let (cov, loc) = (cov_grid[i], local[i]);
        (cov > 0.0 && cov >= share * loc).then_some(loc as f64)
    };
    // The EXTENT a sampled pixel belongs to (module docs, "Local peaks"):
    // BESIDE a better covered stroke where its local peak falls under
    // INK_COVERAGE of its halo peak, or of the peak of the stroke it is
    // joined to (a glow's plateau), or where a glow's ramp joins it to the
    // mask; else an EDGE where it is covered under INK_COVERAGE of its local
    // peak (an anti-aliased edge, a ring of glow within one CSS px of a
    // stroke, which shares the stroke's local peak); else a stroke's CORE.
    // Also whether it is a stroke's core by its own neighbourhood alone, the
    // joined stroke's peak aside: what a fade across joined letters is read
    // at for `glyph_alpha`.
    let extent = |x: u32, y: u32, in_mask: bool| {
        let i = cell(x, y);
        let (cov, loc) = (cov_grid[i], local[i]);
        let near = loc < ink_cov * wide[i] || (!in_mask && linked.get(x as i64, y as i64));
        let joined = in_mask && loc < ink_cov * stroke_peak[stroke_of[i] as usize];
        let edge = cov < ink_cov * loc;
        let level = if near || joined {
            BESIDE
        } else if edge {
            EDGE
        } else {
            CORE
        };
        (level, !near && !edge)
    };
    let sub_threshold = |x: u32, y: u32| {
        in_box(x, y) && !is_mask.get(x as i64, y as i64) && sampled_at(x, y).is_some()
    };
    let mask_over = mask.iter().filter(|(x, y, _)| over(*x, *y)).count()
        + (oy0..oy0 + oh)
            .flat_map(|y| (ox0..ox0 + ow).map(move |x| (x, y)))
            .filter(|&(x, y)| sub_threshold(x, y) || off_colour.get(x as i64, y as i64))
            .count();
    if mask_over < floor {
        // Few mask pixels over the image is "not over it" only if the image
        // under the box could have SHOWN glyphs. Its BLIND pixels (module
        // docs) cannot: a FAINT one is a glyph pixel the mask missed, and an
        // undetectable one may hide one. Counted the way the sampling below
        // classifies them, stopping once the floor is reached.
        //
        // When the image is the text's own descendant inside its box (a
        // button's inline icon), the text is laid out beside it, not over
        // it: only blind pixels within BLIND_ADJACENT_CSS of the text's own
        // visible glyphs count, so an icon in `currentColor` is not hidden
        // ink. An image the text merely covers gets no such exemption.
        let inline = pair.image_inline
            && pair
                .image
                .bbox
                .is_some_and(|ib| region_contains(text_box, ib));
        let beside_glyph = inline.then(|| is_mask.dilated((BLIND_ADJACENT_CSS * s).ceil() as i64));
        // On a noisy capture, a pixel that moved toward the ink by less than
        // the mask sees may be a glyph too faint for it, or noise.
        let (mut faint, mut undetectable, mut noisy) = (0usize, 0usize, 0usize);
        'scan: for y in oy0..oy0 + oh {
            for x in ox0..ox0 + ow {
                if is_mask.get(x as i64, y as i64)
                    || beside_glyph
                        .as_ref()
                        .is_some_and(|g| !g.get(x as i64, y as i64))
                {
                    continue;
                }
                let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
                if noise.get(x as i64, y as i64) {
                    noisy += 1;
                } else if ink.dist2(pb) > blind {
                    continue;
                } else if trusted && pa != pb && toward_ink(pa, pb) {
                    faint += mine(x, y, pa, pb, c) as usize;
                } else if !trusted || ink.contains(pb) {
                    undetectable += 1;
                }
                if mask_over + faint >= floor {
                    break 'scan;
                }
            }
        }
        if mask_over + faint < floor {
            if mask_over + faint + undetectable + noisy < floor {
                return Ok(None);
            }
            let mut unseen = Vec::new();
            if undetectable > 0 {
                unseen.push(format!(
                    "{undetectable} pixel(s) of the image under its box are within {:.1} of its \
                     ink {}, where the mask cannot see a glyph",
                    blind.sqrt(),
                    ink.describe()
                ));
            }
            if noisy > 0 {
                unseen.push(format!(
                    "{noisy} moved toward its ink by less than the mask sees, where on a capture \
                     with sub-threshold noise a glyph too faint for the mask and the noise paint \
                     alike"
                ));
            }
            return Err(unknown(format!(
                "only {mask_over} glyph pixel(s) of text '{id}' reach '{}' (under the {floor} \
                 needed), but {}: whether its glyphs reach the image is unknown, so no contrast \
                 was measured",
                pair.image.id,
                unseen.join(", and ")
            )));
        }
        // Enough FAINT glyph pixels reach the image: it is text over it, and
        // is measured below like any other.
    }
    // A text fewer than a floor's worth of whose glyph pixels no inner text
    // could have painted has shown nothing of its own glyphs: what its mask
    // holds may all be their ink, which bounds no fill of its own.
    if apart < floor && others.iter().any(|o| o.inner && o.colour.is_some()) {
        return Err(unknown(format!(
            "only {apart} glyph pixel(s) of text '{id}' are ones none of its inner texts (those \
             descended from it, or whose boxes are no larger than its own) could have painted \
             (under the {floor} needed): its own glyphs were not seen apart from theirs, so no \
             contrast was measured"
        )));
    }
    // The image displaced under THIS text (a parallax layer) is invisible to
    // the global check; the ring around the text is read here.
    reg.local(a, b, text_box)?;

    // The best case reads each glyph pixel at the glyph colour, opaque (and a
    // possible hairline's ink); the worst at its local peak coverage of it,
    // which a translucent colour and a hairline face both explain, and a
    // pixel the blend does not explain at no better than what it renders
    // (module docs).
    let lg = luminance(c);
    let hairline_ink = beyond.filter(|_| !whole);
    let far = hairline_ink.map(luminance);
    let best = |pb: Rgb| {
        let lb = luminance(pb);
        let r = contrast(lg, lb);
        far.map_or(r, |lf| r.max(contrast(lf, lb)))
    };
    let rendered = |pa: Rgb, pb: Rgb| contrast(luminance(pa), luminance(pb));
    let off_blend = |pa: Rgb, pb: Rgb| residual(pa, pb, c) > tol2;
    let worst = |pa: Rgb, pb: Rgb, alpha: f64| {
        let w = contrast(luminance(blend(pb, c, alpha)), luminance(pb));
        let w = if off_blend(pa, pb) {
            w.min(rendered(pa, pb))
        } else {
            w
        };
        w.min(best(pb))
    };
    // Off the mask, a pixel within HALO_CSS of it lies beside a visible
    // stroke, where a glow and a faint glyph paint alike.
    let beside_mask = is_mask.dilated(halo_r as i64);
    let mut read = Readings::default();
    let (mut hidden, mut faint, mut off, mut halo, mut shared_px, mut noise_px) =
        (0usize, 0, 0, 0, 0, 0);
    // The sampled MASK pixels with their coverage, for the colour fit.
    let mut core: Vec<(Rgb, Rgb, f64)> = Vec::new();
    for y in oy0..oy0 + oh {
        for x in ox0..ox0 + ow {
            let (pa, pb) = (rgb_at(a, x, y), rgb_at(b, x, y));
            let in_mask = is_mask.get(x as i64, y as i64);
            if in_mask || sub_threshold(x, y) {
                let Some(loc) = sampled_at(x, y) else {
                    continue;
                };
                let (level, own_core) = extent(x, y, in_mask);
                let sh = shared(x, y, pa, pb);
                shared_px += sh as usize;
                // What a pixel off the blend renders says nothing of how much
                // of the colour covers it.
                if own_core && !off_blend(pa, pb) {
                    read.alphas.push(loc as f32);
                }
                if level > CORE {
                    halo += 1;
                } else {
                    off += off_blend(pa, pb) as usize;
                    faint += !in_mask as usize;
                    if in_mask {
                        core.push((pa, pb, cov_grid[cell(x, y)] as f64));
                    }
                }
                read.push(level, !sh, best(pb), worst(pa, pb, loc));
            } else if off_colour.get(x as i64, y as i64) || ink.dist2(pb) <= blind {
                let is_faint = !off_colour.get(x as i64, y as i64);
                if is_faint && !(trusted && pa != pb && toward_ink(pa, pb)) {
                    // With a noise-free capture a glyph whose ink differs from
                    // the background at all differs, and was seen. Hidden ink
                    // is never attributed away: it may be this text's. On an
                    // ink-coloured background it paints 1:1, and any coverage
                    // of it may hide there.
                    if !trusted || ink.contains(pb) {
                        hidden += 1;
                        read.optional(CORE, None, 1.0);
                    }
                    continue;
                }
                // Another text's faint ink is its own, not this one's.
                if is_faint && !mine(x, y, pa, pb, c) {
                    continue;
                }
                // An off-colour or faint glyph pixel: read at no better than
                // what it renders, and BESIDE a visible stroke when it lies
                // near one (a faint one also where a glow's ramp joins it to
                // one).
                let sh = shared(x, y, pa, pb);
                shared_px += sh as usize;
                let bv = best(pb);
                let level = if beside_mask.get(x as i64, y as i64)
                    || (is_faint && linked.get(x as i64, y as i64))
                {
                    halo += 1;
                    BESIDE
                } else {
                    if is_faint {
                        faint += 1;
                    } else {
                        off += 1;
                    }
                    CORE
                };
                read.push(level, !sh, bv, rendered(pa, pb).min(bv));
            } else if noise.get(x as i64, y as i64) {
                // Noise, or a glyph too faint for the mask: optional, read
                // at no better than what it renders.
                shared_px += shared(x, y, pa, pb) as usize;
                noise_px += 1;
                let bv = best(pb);
                read.optional(CORE, Some(bv), rendered(pa, pb).min(bv));
            }
        }
    }
    let sampled = read.sampled();
    if sampled < floor {
        return Err(unknown(format!(
            "fewer than {floor} glyph pixel(s) of text '{id}' over '{}' are sampled ({sampled} \
             are), so no contrast was measured",
            pair.image.id
        )));
    }
    // The colour must explain the glyph core under alpha blending, or the
    // ratios are against a colour nothing on screen paints.
    core.sort_by(|p, q| q.2.total_cmp(&p.2));
    let n_core = core.len().div_ceil(4);
    let fit = if n_core == 0 {
        0.0
    } else {
        core[..n_core]
            .iter()
            .filter(|(pa, pb, _)| residual(*pa, *pb, c) <= tol2)
            .count() as f64
            / n_core as f64
    };
    if fit < COLOUR_FIT_SHARE {
        return Err(unknown(format!(
            "the glyphs of text '{id}' over '{}' do not paint its {} colour rgb({},{},{}): only \
             {:.0}% of the {n_core} best-covered glyph pixel(s) lie on the blend of frame B toward \
             it (at least {:.0}% must), so the text paints another colour — a gradient, a \
             -webkit-text-fill-color, a blend mode — and no contrast was measured",
            pair.image.id,
            match source {
                GlyphColourSource::Declared => "declared",
                GlyphColourSource::Measured => "measured",
            },
            c.r,
            c.g,
            c.b,
            fit * 100.0,
            COLOUR_FIT_SHARE * 100.0
        )));
    }
    let mut alphas = std::mem::take(&mut read.alphas);
    alphas.sort_by(f32::total_cmp);
    let (best_p, worst_p, min_ratio) = read.bracket(floor, percentile);
    Ok(Some(TextOverImageContrast {
        element: id.to_string(),
        image: pair.image.id.clone(),
        min_ratio,
        percentile,
        percentile_ratio: best_p,
        percentile_ratio_worst_case: worst_p,
        glyph_colour: c,
        glyph_colour_source: source,
        hairline_ink,
        ink_range: [ink.lo, ink.hi].map(|v| Rgb::new(v[0], v[1], v[2])),
        glyph_alpha: nearest_rank(&alphas, percentile),
        colour_fit: fit,
        sampled_px: sampled as u64,
        faint_px: faint as u64,
        hidden_px_estimate: hidden as u64,
        shared_px: shared_px as u64,
        off_colour_px: off as u64,
        halo_px: halo as u64,
        noise_px: noise_px as u64,
        region: Region {
            x: ox0 as i32,
            y: oy0 as i32,
            w: ow,
            h: oh,
        },
        confidence: (in_frame * colour_conf).clamp(0.0, 1.0),
    }))
}

/// The EXTENTS a reading may take ([`measure`]; module docs, "Local peaks"):
/// the strokes' cores alone; with their edges; with the pixels beside them
/// too. Each is a whole: a reading takes every pixel of an extent or none.
const CORE: usize = 0;
const EDGE: usize = 1;
const BESIDE: usize = 2;
const EXTENTS: usize = 3;

/// The pixels a reading is taken over ([`measure`]; module docs, "Definite
/// and optional pixels"), by extent. Within the extents a reading takes,
/// each DEFINITE pixel is this text's glyph; each OPTIONAL one may or may
/// not be. A sampled pixel carries its best and its worst case; hidden ink,
/// which is never sampled, its worst case alone. The coverages the stroke
/// cores were read at ride along for [`TextOverImageContrast::glyph_alpha`].
#[derive(Default)]
struct Readings {
    levels: [Level; EXTENTS],
    alphas: Vec<f32>,
}

/// One extent's pixels ([`Readings`]).
#[derive(Default)]
struct Level {
    d_best: Vec<f64>,
    d_worst: Vec<f64>,
    o_best: Vec<f64>,
    o_worst: Vec<f64>,
}

impl Level {
    /// Move `other`'s pixels into this one, every list kept ascending;
    /// `other`'s storage is freed.
    fn absorb(&mut self, mut other: Level) {
        for (into, from) in [
            (&mut self.d_best, &mut other.d_best),
            (&mut self.d_worst, &mut other.d_worst),
            (&mut self.o_best, &mut other.o_best),
            (&mut self.o_worst, &mut other.o_worst),
        ] {
            if into.is_empty() {
                std::mem::swap(into, from);
            } else {
                into.reserve_exact(from.len());
                into.append(from);
            }
            // In place: values equal under `total_cmp` are the same bits.
            into.sort_unstable_by(f64::total_cmp);
        }
    }

    /// The best and the worst case over these pixels ([`Readings::bracket`]):
    /// `-inf` and `+inf` when no choice leaves `floor` glyph pixels.
    fn extremes(&self, floor: usize, percentile: f64) -> (f64, f64) {
        let n_d = self.d_best.len();
        let t0 = floor.max(1).saturating_sub(n_d);
        let at = |n: usize| nearest_rank_index(n, percentile) + 1;
        let ob = &self.o_best;
        let best = (t0..=ob.len())
            .map(|t| kth_of_two(&self.d_best, &ob[ob.len() - t..], at(n_d + t)))
            .fold(f64::NEG_INFINITY, f64::max);
        let worst = (t0..=self.o_worst.len())
            .map(|t| kth_of_two(&self.d_worst, &self.o_worst[..t], at(n_d + t)))
            .fold(f64::INFINITY, f64::min);
        (best, worst)
    }
}

impl Readings {
    /// A sampled pixel of `level`, definite or not.
    fn push(&mut self, level: usize, definite: bool, best: f64, worst: f64) {
        if definite {
            let l = &mut self.levels[level];
            l.d_best.push(best);
            l.d_worst.push(worst);
        } else {
            self.optional(level, Some(best), worst);
        }
    }

    /// An optional pixel of `level`: sampled (with a best case) or hidden
    /// (without).
    fn optional(&mut self, level: usize, best: Option<f64>, worst: f64) {
        let l = &mut self.levels[level];
        l.o_best.extend(best);
        l.o_worst.push(worst);
    }

    /// The sampled pixels, definite and optional, of every extent.
    fn sampled(&self) -> usize {
        self.levels
            .iter()
            .map(|l| l.d_best.len() + l.o_best.len())
            .sum()
    }

    /// The best case, the worst case and the lowest best-case ratio. A
    /// reading takes the pixels of the first `k` extents, for some `k`, and
    /// within them any choice of the optional pixels that leaves the text at
    /// least `floor` glyph pixels. The best case is the HIGHEST
    /// `percentile`-th percentile (nearest rank) any such reading gives, the
    /// worst case the LOWEST. Within the extents, for each count `t` of
    /// optional pixels taken, the `t` with the highest best cases raise
    /// every rank of the best reading at least as far as any other `t`
    /// would, and the `t` with the lowest worst cases lower every rank of
    /// the worst; so the extremes over the counts are the extremes over
    /// every choice. Needs at least `floor` sampled pixels. Each extent's
    /// pixels are moved into the reading, never copied: the hidden pixels
    /// of a box over a blind image are held once.
    fn bracket(self, floor: usize, percentile: f64) -> (f64, f64, f64) {
        let mut taken = Level::default();
        let (mut best, mut worst) = (f64::NEG_INFINITY, f64::INFINITY);
        for level in self.levels {
            taken.absorb(level);
            let (b, w) = taken.extremes(floor, percentile);
            (best, worst) = (best.max(b), worst.min(w));
        }
        let lowest = taken
            .d_best
            .first()
            .into_iter()
            .chain(taken.o_best.first())
            .fold(f64::INFINITY, |m, v| m.min(*v));
        (best, worst, lowest)
    }
}

/// The `k`-th smallest (1-based) of two ascending lists together, `k` at
/// most their total length: a binary search for how many of the `k` come
/// from `a`, logarithmic in the shorter of `a` and `k`.
fn kth_of_two(a: &[f64], b: &[f64], k: usize) -> f64 {
    debug_assert!(k >= 1 && k <= a.len() + b.len());
    // The most of `a` the `k` smallest can hold: `i` of them with `k - i` of
    // `b` when `a`'s `i`-th does not exceed the `b` value after those.
    let (mut lo, mut hi) = (k.saturating_sub(b.len()), k.min(a.len()));
    while lo < hi {
        let i = (lo + hi).div_ceil(2);
        if b.get(k - i).is_none_or(|&next| a[i - 1] <= next) {
            lo = i;
        } else {
            hi = i - 1;
        }
    }
    let from_a = lo.checked_sub(1).map_or(f64::NEG_INFINITY, |i| a[i]);
    let from_b = (k - lo).checked_sub(1).map_or(f64::NEG_INFINITY, |j| b[j]);
    from_a.max(from_b)
}

/// The colours a glyph's ink may render at full coverage, as a per-channel
/// box ([`measure`]).
#[derive(Debug, Clone, Copy)]
struct Ink {
    lo: [u8; 3],
    hi: [u8; 3],
}

impl Ink {
    fn at(c: Rgb) -> Self {
        Self {
            lo: [c.r, c.g, c.b],
            hi: [c.r, c.g, c.b],
        }
    }

    fn include(&mut self, p: Rgb) {
        for (i, v) in [p.r, p.g, p.b].into_iter().enumerate() {
            self.lo[i] = self.lo[i].min(v);
            self.hi[i] = self.hi[i].max(v);
        }
    }

    /// The squared distance from `p` to the nearest colour of the box.
    fn dist2(&self, p: Rgb) -> f64 {
        [p.r, p.g, p.b]
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let d = if v < self.lo[i] {
                    self.lo[i] - v
                } else {
                    v.saturating_sub(self.hi[i])
                } as f64;
                d * d
            })
            .sum()
    }

    fn contains(&self, p: Rgb) -> bool {
        self.dist2(p) == 0.0
    }

    fn describe(&self) -> String {
        describe_ink([self.lo, self.hi].map(|v| Rgb::new(v[0], v[1], v[2])))
    }
}

/// An ink box ([`TextOverImageContrast::ink_range`]) as `rgb(..)`, or
/// `rgb(..)..rgb(..)` from its low corner to its high.
pub fn describe_ink([lo, hi]: [Rgb; 2]) -> String {
    if lo == hi {
        format!("rgb({},{},{})", lo.r, lo.g, lo.b)
    } else {
        format!(
            "rgb({},{},{})..rgb({},{},{})",
            lo.r, lo.g, lo.b, hi.r, hi.g, hi.b
        )
    }
}

/// How far a set of glyph pixels moved, channel by channel, from frame B
/// in frame A: the brightest value one rose to and the darkest one fell to.
/// One ink painting them all lies at or past both ([`measure`]).
#[derive(Debug, Clone, Copy, Default)]
struct Reach {
    up: [Option<u8>; 3],
    down: [Option<u8>; 3],
}

impl Reach {
    fn add(&mut self, pa: Rgb, pb: Rgb) {
        for (i, (va, vb)) in [(pa.r, pb.r), (pa.g, pb.g), (pa.b, pb.b)]
            .into_iter()
            .enumerate()
        {
            if va > vb {
                self.up[i] = Some(self.up[i].map_or(va, |u| u.max(va)));
            } else if va < vb {
                self.down[i] = Some(self.down[i].map_or(va, |d| d.min(va)));
            }
        }
    }

    fn merge(&mut self, o: &Reach) {
        for i in 0..3 {
            self.up[i] = self.up[i].max(o.up[i]);
            self.down[i] = match (self.down[i], o.down[i]) {
                (Some(p), Some(q)) => Some(p.min(q)),
                (p, q) => p.or(q),
            };
        }
    }

    /// Per channel, where the MAJORITY of `strokes`' pixels reach: the median
    /// of the strokes' reaches weighted by their pixel counts, taken on the
    /// side nearer the background (the lower of a rise, the higher of a
    /// fall), so the bound it sets is the wider. Strokes that never moved
    /// that way in a channel do not vote there.
    fn majority(strokes: &[(Reach, usize)]) -> Reach {
        let median = |mut v: Vec<(u8, usize)>, rising: bool| {
            if rising {
                v.sort_by_key(|(x, _)| *x);
            } else {
                v.sort_by_key(|(x, _)| std::cmp::Reverse(*x));
            }
            let total: usize = v.iter().map(|(_, n)| n).sum();
            let mut acc = 0;
            v.into_iter().find_map(|(x, n)| {
                acc += n;
                (2 * acc >= total).then_some(x)
            })
        };
        let mut out = Reach::default();
        for i in 0..3 {
            let votes = |side: fn(&Reach, usize) -> Option<u8>| -> Vec<(u8, usize)> {
                strokes
                    .iter()
                    .filter_map(|(r, n)| side(r, i).map(|x| (x, *n)))
                    .collect()
            };
            out.up[i] = median(votes(|r, i| r.up[i]), true);
            out.down[i] = median(votes(|r, i| r.down[i]), false);
        }
        out
    }

    /// Widen `ink` to every colour from these bounds to `c`: the inks one
    /// fill painting these pixels may be (a channel that never moved stays
    /// at `c`).
    fn bound(&self, c: Rgb, ink: &mut Ink) {
        let at = |v: [Option<u8>; 3]| {
            Rgb::new(
                v[0].unwrap_or(c.r),
                v[1].unwrap_or(c.g),
                v[2].unwrap_or(c.b),
            )
        };
        ink.include(at(self.up));
        ink.include(at(self.down));
    }
}

/// The point where the ray from `from` through `c` leaves the RGB cube: the
/// most extreme ink a measured colour `c` may be a partial coverage of.
/// `None` when the ray has no direction or `c` already lies on the cube's
/// face.
fn ray_exit(from: Rgb, c: Rgb) -> Option<Rgb> {
    let d = [
        c.r as f64 - from.r as f64,
        c.g as f64 - from.g as f64,
        c.b as f64 - from.b as f64,
    ];
    let p = [c.r as f64, c.g as f64, c.b as f64];
    let t = (0..3)
        .filter(|&i| d[i] != 0.0)
        .map(|i| {
            if d[i] > 0.0 {
                (255.0 - p[i]) / d[i]
            } else {
                -p[i] / d[i]
            }
        })
        .fold(f64::INFINITY, f64::min);
    if !t.is_finite() || t <= 0.0 {
        return None;
    }
    let ch = |i: usize| (p[i] + t * d[i]).round().clamp(0.0, 255.0) as u8;
    let e = Rgb::new(ch(0), ch(1), ch(2));
    (e != c).then_some(e)
}

/// Each cell's maximum over the `(2r+1)²` window around it, in two separable
/// passes over a `w`×`h` row-major grid of non-negative values, each linear in
/// the grid whatever `r` (van Herk / Gil–Werman).
fn sliding_max(v: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut rows = vec![0.0f32; w * h];
    for y in 0..h {
        max_1d(&v[y * w..(y + 1) * w], r, &mut rows[y * w..(y + 1) * w]);
    }
    let mut out = vec![0.0f32; w * h];
    let (mut col, mut res) = (vec![0.0f32; h], vec![0.0f32; h]);
    for x in 0..w {
        for y in 0..h {
            col[y] = rows[y * w + x];
        }
        max_1d(&col, r, &mut res);
        for y in 0..h {
            out[y * w + x] = res[y];
        }
    }
    out
}

/// `out[i]` = the maximum of `v` over `i-r..=i+r` (clipped), for `v >= 0`.
/// Padded with `r` zeros each side so every window spans exactly `2r+1`
/// cells, which lie in at most two blocks of that length: the window's
/// maximum is the suffix maximum of the first block from its start and the
/// prefix maximum of the second up to its end.
fn max_1d(v: &[f32], r: usize, out: &mut [f32]) {
    let n = v.len();
    let k = 2 * r + 1;
    let m = n + 2 * r;
    let at = |i: usize| if i >= r && i < r + n { v[i - r] } else { 0.0 };
    let mut pre = vec![0.0f32; m];
    let mut suf = vec![0.0f32; m];
    for i in 0..m {
        pre[i] = if i % k == 0 {
            at(i)
        } else {
            pre[i - 1].max(at(i))
        };
    }
    for i in (0..m).rev() {
        suf[i] = if i + 1 == m || (i + 1) % k == 0 {
            at(i)
        } else {
            suf[i + 1].max(at(i))
        };
    }
    for (i, o) in out.iter_mut().enumerate().take(n) {
        *o = suf[i].max(pre[i + 2 * r]);
    }
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

    /// The linear-time window maximum agrees with the brute-force one at
    /// every radius, including radii past the grid and grids of one row or
    /// column, where every window is clipped.
    #[test]
    fn the_sliding_maximum_matches_brute_force() {
        let mut seed = 12345u32;
        let mut next = || {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            ((seed >> 16) % 1000) as f32 / 1000.0
        };
        for (w, h) in [(1usize, 1usize), (1, 9), (9, 1), (7, 5), (23, 17)] {
            let v: Vec<f32> = (0..w * h).map(|_| next()).collect();
            for r in [0usize, 1, 2, 3, 6, 30] {
                let got = sliding_max(&v, w, h, r);
                for y in 0..h {
                    for x in 0..w {
                        let mut want = 0.0f32;
                        for yy in y.saturating_sub(r)..(y + r + 1).min(h) {
                            for xx in x.saturating_sub(r)..(x + r + 1).min(w) {
                                want = want.max(v[yy * w + xx]);
                            }
                        }
                        assert_eq!(got[y * w + x], want, "{w}x{h} r{r} at ({x},{y})");
                    }
                }
            }
        }
    }

    /// The two-list order statistic agrees with sorting the union, for every
    /// rank, with ties, and with either list empty.
    #[test]
    fn the_kth_of_two_lists_matches_sorting_their_union() {
        let mut seed = 777u32;
        let mut next = || {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            ((seed >> 16) % 7) as f64
        };
        for (na, nb) in [
            (0usize, 1usize),
            (1, 0),
            (1, 1),
            (3, 8),
            (8, 3),
            (9, 9),
            (0, 5),
        ] {
            for _ in 0..20 {
                let mut a: Vec<f64> = (0..na).map(|_| next()).collect();
                let mut b: Vec<f64> = (0..nb).map(|_| next()).collect();
                a.sort_by(f64::total_cmp);
                b.sort_by(f64::total_cmp);
                let mut all = [a.clone(), b.clone()].concat();
                all.sort_by(f64::total_cmp);
                for k in 1..=all.len() {
                    assert_eq!(kth_of_two(&a, &b, k), all[k - 1], "{a:?} {b:?} k{k}");
                }
            }
        }
    }

    /// The bracket is the extreme over every leading run of extents and
    /// EVERY subset of their optional pixels that leaves the floor, checked
    /// against brute force on small lists.
    #[test]
    fn the_bracket_is_the_extreme_over_every_extent_and_choice_of_optional_pixels() {
        let mut seed = 4242u32;
        let mut next = || {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            seed >> 16
        };
        for (nd, no, floor, p) in [
            (0usize, 6usize, 3usize, 5.0),
            (2, 7, 4, 30.0),
            (5, 5, 1, 50.0),
            (6, 0, 6, 5.0),
            (1, 8, 2, 90.0),
            (7, 4, 5, 5.0),
        ] {
            for _ in 0..20 {
                let value = |r: u32| 1.0 + (r % 50) as f64 / 10.0;
                // (extent, best, worst); the last optional pixel is hidden.
                let d: Vec<(usize, f64, f64)> = (0..nd)
                    .map(|_| {
                        let w = value(next());
                        ((next() % 3) as usize, w + value(next()), w)
                    })
                    .collect();
                let o: Vec<(usize, Option<f64>, f64)> = (0..no)
                    .map(|i| {
                        let w = value(next());
                        let l = (next() % 3) as usize;
                        let up = value(next());
                        (l, (i + 1 < no).then_some(w + up), w)
                    })
                    .collect();
                let mut r = Readings::default();
                for &(l, bv, wv) in &d {
                    r.push(l, true, bv, wv);
                }
                for &(l, bv, wv) in &o {
                    r.optional(l, bv, wv);
                }
                if r.sampled() < floor {
                    continue;
                }
                let (best, worst, lowest) = r.bracket(floor, p);
                let (mut want_best, mut want_worst) = (f64::NEG_INFINITY, f64::INFINITY);
                for k in 1..=EXTENTS {
                    for set in 0u32..(1 << no) {
                        let pick = |i: usize| set & (1 << i) != 0 && o[i].0 < k;
                        if (0..no).any(|i| set & (1 << i) != 0 && o[i].0 >= k) {
                            continue;
                        }
                        let mut ws: Vec<f64> = d.iter().filter(|e| e.0 < k).map(|e| e.2).collect();
                        ws.extend((0..no).filter(|&i| pick(i)).map(|i| o[i].2));
                        if ws.len() >= floor {
                            ws.sort_by(f64::total_cmp);
                            want_worst = want_worst.min(nearest_rank(&ws, p).unwrap());
                        }
                        let mut bs: Vec<f64> = d.iter().filter(|e| e.0 < k).map(|e| e.1).collect();
                        bs.extend((0..no).filter(|&i| pick(i)).filter_map(|i| o[i].1));
                        if bs.len() >= floor && (0..no).all(|i| !pick(i) || o[i].1.is_some()) {
                            bs.sort_by(f64::total_cmp);
                            want_best = want_best.max(nearest_rank(&bs, p).unwrap());
                        }
                    }
                }
                let want_lowest = d
                    .iter()
                    .map(|e| e.1)
                    .chain(o.iter().filter_map(|e| e.1))
                    .fold(f64::INFINITY, f64::min);
                assert_eq!(best, want_best, "{d:?} {o:?} floor {floor} p{p}");
                assert_eq!(worst, want_worst, "{d:?} {o:?} floor {floor} p{p}");
                assert_eq!(lowest, want_lowest, "{d:?} {o:?}");
            }
        }
    }

    #[test]
    fn nearest_rank_reads_the_minimum_at_zero_and_the_maximum_at_100() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(nearest_rank(&v, 0.0), Some(1.0));
        assert_eq!(nearest_rank(&v, 5.0), Some(5.0));
        assert_eq!(nearest_rank(&v, 50.0), Some(50.0));
        assert_eq!(nearest_rank(&v, 100.0), Some(100.0));
        assert_eq!(nearest_rank(&[3.0], 5.0), Some(3.0));
        assert_eq!(nearest_rank::<f64>(&[], 5.0), None);
    }

    /// `7 / 100 * 100` is 7.000000000000001 in f64, whose ceiling is the
    /// eighth rank; multiplying first gives exactly 7.
    #[test]
    fn nearest_rank_is_not_off_by_one_from_float_error() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        for p in 1..=100 {
            assert_eq!(nearest_rank(&v, p as f64), Some(p as f64), "p{p}");
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

    /// The dilation matches the brute-force Chebyshev neighbourhood, at the
    /// grid's edges too.
    #[test]
    fn a_dilated_grid_is_the_chebyshev_neighbourhood() {
        let mut g = Grid::new((10, 20, 27, 31));
        for (x, y) in [(10, 20), (18, 25), (26, 30), (21, 22)] {
            g.set(x, y);
        }
        for r in 0..4 {
            let d = g.dilated(r);
            for y in 18..33 {
                for x in 8..29 {
                    let brute = (-r..=r).any(|dy| (-r..=r).any(|dx| g.get(x + dx, y + dy)))
                        && g.index(x, y).is_some();
                    assert_eq!(d.get(x, y), brute, "r={r} ({x},{y})");
                }
            }
        }
    }

    /// The alpha-blend residual is zero for any coverage of the colour itself,
    /// and large for a glyph of another colour.
    #[test]
    fn a_blend_toward_the_colour_has_no_residual() {
        let (b, c) = (Rgb::new(30, 60, 90), Rgb::new(250, 250, 250));
        for alpha in [0.2, 0.45, 0.7, 1.0] {
            assert!(residual(blend(b, c, alpha), b, c) <= 3.0, "{alpha}");
        }
        assert!(residual(Rgb::new(240, 40, 40), b, c) > 24.0 * 24.0);
    }

    /// The majority's reach is the pixel-weighted median of the strokes',
    /// taken on the side nearer the background at a tie (the lower rise, the
    /// higher fall), and a stroke that never moved one way does not vote.
    #[test]
    fn the_majority_reach_is_the_weighted_median_nearer_the_background() {
        let stroke = |pa: Rgb, pb: Rgb| {
            let mut r = Reach::default();
            r.add(pa, pb);
            r
        };
        let (dark, light) = (Rgb::new(40, 40, 40), Rgb::new(250, 250, 250));
        let rise = |v: u8| stroke(Rgb::new(v, v, v), dark);
        let fall = |v: u8| stroke(Rgb::new(v, v, v), light);
        // A few pixels at 255 against more at 240: 240.
        let m = Reach::majority(&[(rise(255), 56), (rise(240), 336)]);
        assert_eq!(m.up, [Some(240); 3]);
        assert_eq!(m.down, [None; 3]);
        // A tie: the lower rise, and the higher fall.
        let m = Reach::majority(&[(rise(255), 10), (rise(240), 10)]);
        assert_eq!(m.up, [Some(240); 3]);
        let m = Reach::majority(&[(fall(0), 10), (fall(30), 10), (rise(255), 500)]);
        assert_eq!(m.down, [Some(30); 3]);
        assert_eq!(m.up, [Some(255); 3]);
        assert_eq!(Reach::majority(&[]).up, [None; 3]);
    }
}
