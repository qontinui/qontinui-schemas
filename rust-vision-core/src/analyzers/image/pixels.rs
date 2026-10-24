//! Pixel math for the [`super`] image analyzer: pure functions over an RGB
//! crop or a whole frame, no I/O, no model, no new dependency (plan
//! 2026-10-13-image-generation-and-visual-judgment, PD9).
//!
//! Every function here takes BUFFER pixels. Mapping a snapshot bbox (CSS
//! pixels) onto the buffer is the caller's job, through
//! [`crate::Frame::css_region_to_pixels`].

use ::image::imageops::{self, FilterType};
use ::image::{GrayImage, Luma, RgbImage};

use crate::element_snapshot::Rgb;

// ---------------------------------------------------------------------------
// Luma
// ---------------------------------------------------------------------------

/// Pillow's `convert("L")`: ITU-R 601-2 luma in fixed point,
/// `(19595 R + 38470 G + 7471 B + 0x8000) >> 16`.
///
/// Spelled exactly as Pillow spells it, not as the `image` crate's
/// `to_luma8` (which uses Rec. 709 weights), because the perceptual hashes
/// are cross-checked against Python `imagehash`, which hashes Pillow's luma.
pub fn pil_luma(r: u8, g: u8, b: u8) -> u8 {
    ((19595 * r as u32 + 38470 * g as u32 + 7471 * b as u32 + 0x8000) >> 16) as u8
}

/// The crop as an 8-bit luma image, through [`pil_luma`].
pub fn luma_image(rgb: &RgbImage) -> GrayImage {
    let mut out = GrayImage::new(rgb.width(), rgb.height());
    for (x, y, p) in rgb.enumerate_pixels() {
        out.put_pixel(x, y, Luma([pil_luma(p[0], p[1], p[2])]));
    }
    out
}

// ---------------------------------------------------------------------------
// Sharpness, blockiness, busyness
// ---------------------------------------------------------------------------

/// Variance of the 4-neighbour Laplacian over the interior pixels: the
/// standard focus measure. Higher is sharper. `None` below 3×3 pixels, where
/// there is no interior.
///
/// Read at the RENDERED scale (the frame crop), so it measures what a viewer
/// sees: an image that is sharp as a file but upscaled 3× on the page reads
/// soft here, as it should.
pub fn laplacian_variance(luma: &GrayImage) -> Option<f64> {
    let (w, h) = luma.dimensions();
    if w < 3 || h < 3 {
        return None;
    }
    let at = |x: u32, y: u32| luma.get_pixel(x, y)[0] as f64;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut n = 0.0;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let v = at(x - 1, y) + at(x + 1, y) + at(x, y - 1) + at(x, y + 1) - 4.0 * at(x, y);
            sum += v;
            sum_sq += v * v;
            n += 1.0;
        }
    }
    let mean = sum / n;
    Some((sum_sq / n - mean * mean).max(0.0))
}

/// Where an image's 8×8 JPEG coding grid lies on a crop, per axis, in the
/// crop's buffer pixels: grid lines sit at `phase + k × period` for every
/// integer `k`. The period is 8 source pixels at the DRAWN scale (8 ×
/// device px per source px, which differs per axis under `fill`), and the
/// phase is where the natural image's own top-left edge lands relative to the
/// crop — negative when the image starts before the crop (clipped by the frame
/// edge, or pushed out by `cover`'s object-position).
///
/// **Known limit: the box is the border box.** The placement is computed from
/// the element's bbox, which is the BORDER box (`getBoundingClientRect`), while
/// object-fit lays the image out in the CONTENT box. Padding or a border on an
/// `<img>` therefore shifts the true grid by the left/top inset and shrinks
/// the drawn scale by the content-to-border ratio, and nothing here can
/// correct for it: content-box insets are not on the wire. An image with a
/// thick border or padding can read cleaner than it is. Closing it needs the
/// producer to emit the content-box insets (a follow-up on the converters).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodingGrid {
    pub period_x: f64,
    pub phase_x: f64,
    pub period_y: f64,
    pub phase_y: f64,
}

/// The buffer positions `1..len` that a grid line crosses (a line at `b`
/// separates pixel `b − 1` from pixel `b`).
fn grid_lines(len: u32, period: f64, phase: f64) -> Vec<bool> {
    let mut lines = vec![false; len as usize];
    let mut k = ((1.0 - phase) / period).ceil();
    loop {
        let b = (phase + k * period).round();
        if b >= len as f64 {
            break;
        }
        if b >= 1.0 {
            lines[b as usize] = true;
        }
        k += 1.0;
    }
    lines
}

/// Compression blockiness: the mean absolute luma step ACROSS the image's
/// coding-grid lines, divided by the mean step everywhere else, over both
/// axes. A clean image reads near 1.0; JPEG at low quality, whose 8×8 blocks
/// are quantized independently, reads well above it.
///
/// The grid must be placed where the ENCODER put it ([`CodingGrid`]): a grid
/// sought at the crop's own origin, or at the wrong period, measures some
/// other set of columns and reads a blocky image as clean (measured on the
/// q10 fixture: 2.99 aligned, 1.38 three pixels off, 0.73 four off).
///
/// `None` when either period is under 4 pixels (block edges too close
/// together to separate from ordinary detail), when an axis holds fewer than
/// two grid lines, or when the image has no detail OFF the grid (flat art, or
/// a QR code drawn on an 8-pixel module): a ratio over a zero denominator is
/// not a measurement of compression.
pub fn blockiness(luma: &GrayImage, grid: CodingGrid) -> Option<f64> {
    let ok = |p: f64, ph: f64| p.is_finite() && p >= 4.0 && ph.is_finite();
    if !ok(grid.period_x, grid.phase_x) || !ok(grid.period_y, grid.phase_y) {
        return None;
    }
    let (w, h) = luma.dimensions();
    let bx = grid_lines(w, grid.period_x, grid.phase_x);
    let by = grid_lines(h, grid.period_y, grid.phase_y);
    if bx.iter().filter(|b| **b).count() < 2 || by.iter().filter(|b| **b).count() < 2 {
        return None;
    }
    let at = |x: u32, y: u32| luma.get_pixel(x, y)[0] as f64;
    let (mut on, mut on_n, mut off, mut off_n) = (0.0, 0.0, 0.0, 0.0);
    for y in 0..h {
        for x in 1..w {
            let d = (at(x, y) - at(x - 1, y)).abs();
            if bx[x as usize] {
                on += d;
                on_n += 1.0;
            } else {
                off += d;
                off_n += 1.0;
            }
        }
    }
    for x in 0..w {
        for y in 1..h {
            let d = (at(x, y) - at(x, y - 1)).abs();
            if by[y as usize] {
                on += d;
                on_n += 1.0;
            } else {
                off += d;
                off_n += 1.0;
            }
        }
    }
    let off_mean = off / off_n;
    if off_mean.is_nan() || off_mean <= 1e-6 {
        return None;
    }
    Some((on / on_n) / off_mean)
}

/// Sobel gradient magnitude (luma units) at every interior pixel.
fn sobel(luma: &GrayImage) -> Vec<f64> {
    let (w, h) = luma.dimensions();
    if w < 3 || h < 3 {
        return Vec::new();
    }
    let at = |x: u32, y: u32| luma.get_pixel(x, y)[0] as f64;
    let mut out = Vec::with_capacity(((w - 2) * (h - 2)) as usize);
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let gx = at(x + 1, y - 1) + 2.0 * at(x + 1, y) + at(x + 1, y + 1)
                - at(x - 1, y - 1)
                - 2.0 * at(x - 1, y)
                - at(x - 1, y + 1);
            let gy = at(x - 1, y + 1) + 2.0 * at(x, y + 1) + at(x + 1, y + 1)
                - at(x - 1, y - 1)
                - 2.0 * at(x, y - 1)
                - at(x + 1, y - 1);
            out.push((gx * gx + gy * gy).sqrt());
        }
    }
    out
}

/// The Sobel magnitude above which a pixel counts as an edge: a step of 16
/// grey levels between neighbours.
pub const EDGE_MAGNITUDE: f64 = 64.0;

/// Fraction (0..=1) of interior pixels whose Sobel magnitude exceeds
/// [`EDGE_MAGNITUDE`]. `None` below 3×3 pixels.
pub fn edge_density(luma: &GrayImage) -> Option<f64> {
    let mags = sobel(luma);
    if mags.is_empty() {
        return None;
    }
    let edges = mags.iter().filter(|m| **m > EDGE_MAGNITUDE).count();
    Some(edges as f64 / mags.len() as f64)
}

/// Shannon entropy, in bits (0..=8), of the 256-bin luma histogram. `None`
/// for an empty image.
pub fn luma_entropy(luma: &GrayImage) -> Option<f64> {
    let n = luma.pixels().len();
    if n == 0 {
        return None;
    }
    let mut hist = [0u64; 256];
    for p in luma.pixels() {
        hist[p[0] as usize] += 1;
    }
    let n = n as f64;
    Some(
        hist.iter()
            .filter(|c| **c > 0)
            .map(|c| {
                let p = *c as f64 / n;
                -p * p.log2()
            })
            .sum(),
    )
}

/// Fraction (0..=1) of `tile`×`tile` tiles whose luma standard deviation is
/// under 2 grey levels: the page's EMPTY space in the design sense (flat
/// areas of any colour, not literally white). Partial edge tiles count.
pub fn flat_tile_fraction(luma: &GrayImage, tile: u32) -> Option<f64> {
    let (w, h) = luma.dimensions();
    let tile = tile.max(1);
    if w == 0 || h == 0 {
        return None;
    }
    let (mut flat, mut total) = (0u64, 0u64);
    let mut ty = 0;
    while ty < h {
        let mut tx = 0;
        while tx < w {
            let (mut s, mut s2, mut n) = (0.0, 0.0, 0.0);
            for y in ty..(ty + tile).min(h) {
                for x in tx..(tx + tile).min(w) {
                    let v = luma.get_pixel(x, y)[0] as f64;
                    s += v;
                    s2 += v * v;
                    n += 1.0;
                }
            }
            let mean = s / n;
            if (s2 / n - mean * mean).max(0.0).sqrt() < 2.0 {
                flat += 1;
            }
            total += 1;
            tx += tile;
        }
        ty += tile;
    }
    Some(flat as f64 / total as f64)
}

/// Hasler–Süsstrunk colourfulness (2003): `σ_rgyb + 0.3 μ_rgyb` over the
/// opponent channels `rg = R − G`, `yb = (R + G)/2 − B`. 0 for a greyscale
/// image; around 30 is "moderately colourful", above 80 "extremely".
pub fn colourfulness(rgb: &RgbImage) -> Option<f64> {
    let n = rgb.pixels().len();
    if n == 0 {
        return None;
    }
    let (mut rg_s, mut rg_s2, mut yb_s, mut yb_s2) = (0.0, 0.0, 0.0, 0.0);
    for p in rgb.pixels() {
        let (r, g, b) = (p[0] as f64, p[1] as f64, p[2] as f64);
        let rg = r - g;
        let yb = 0.5 * (r + g) - b;
        rg_s += rg;
        rg_s2 += rg * rg;
        yb_s += yb;
        yb_s2 += yb * yb;
    }
    let n = n as f64;
    let (rg_m, yb_m) = (rg_s / n, yb_s / n);
    let rg_var = (rg_s2 / n - rg_m * rg_m).max(0.0);
    let yb_var = (yb_s2 / n - yb_m * yb_m).max(0.0);
    Some((rg_var + yb_var).sqrt() + 0.3 * (rg_m * rg_m + yb_m * yb_m).sqrt())
}

// ---------------------------------------------------------------------------
// Colour
// ---------------------------------------------------------------------------

fn srgb_to_linear(v: u8) -> f64 {
    let s = v as f64 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f64) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

/// sRGB → OKLab (Ottosson 2020). `[L, a, b]`, L in 0..=1.
pub fn oklab(c: Rgb) -> [f64; 3] {
    let (r, g, b) = (
        srgb_to_linear(c.r),
        srgb_to_linear(c.g),
        srgb_to_linear(c.b),
    );
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    ]
}

/// OKLab → sRGB, clamped into gamut.
pub fn oklab_to_rgb(lab: [f64; 3]) -> Rgb {
    let l = lab[0] + 0.396_337_777_4 * lab[1] + 0.215_803_757_3 * lab[2];
    let m = lab[0] - 0.105_561_345_8 * lab[1] - 0.063_854_172_8 * lab[2];
    let s = lab[0] - 0.089_484_177_5 * lab[1] - 1.291_485_548 * lab[2];
    let (l, m, s) = (l * l * l, m * m * m, s * s * s);
    Rgb::new(
        linear_to_srgb(4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s),
        linear_to_srgb(-1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s),
        linear_to_srgb(-0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s),
    )
}

/// Pixels of `rgb`, subsampled on a regular stride so at most `max` are
/// returned. Deterministic: the same crop always yields the same sample.
pub fn sample_pixels(rgb: &RgbImage, max: usize) -> Vec<Rgb> {
    let n = rgb.pixels().len();
    if n == 0 {
        return Vec::new();
    }
    let stride = n.div_ceil(max.max(1)).max(1);
    rgb.pixels()
        .step_by(stride)
        .map(|p| Rgb::new(p[0], p[1], p[2]))
        .collect()
}

/// One palette cluster: its colour and the share of the image it covers.
#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    pub oklab: [f64; 3],
    pub coverage: f64,
}

/// k-means in OKLab over `pixels`, `k` clusters at most, seeded
/// deterministically from the `k` most populated 4-bit-per-channel colour
/// bins (so the same image always yields the same palette). Returned largest
/// first; empty clusters are dropped.
pub fn kmeans_oklab(pixels: &[Rgb], k: usize) -> Vec<Cluster> {
    if pixels.is_empty() || k == 0 {
        return Vec::new();
    }
    let labs: Vec<[f64; 3]> = pixels.iter().map(|p| oklab(*p)).collect();
    // Seeds: the most populated coarse bins, ties broken by bin index.
    let mut bins: std::collections::BTreeMap<u16, (u32, [f64; 3])> = Default::default();
    for (p, lab) in pixels.iter().zip(&labs) {
        let key = ((p.r as u16 >> 4) << 8) | ((p.g as u16 >> 4) << 4) | (p.b as u16 >> 4);
        let e = bins.entry(key).or_insert((0, [0.0; 3]));
        e.0 += 1;
        for (acc, v) in e.1.iter_mut().zip(lab) {
            *acc += v;
        }
    }
    let mut seeds: Vec<(u16, u32, [f64; 3])> =
        bins.into_iter().map(|(k, v)| (k, v.0, v.1)).collect();
    seeds.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut centroids: Vec<[f64; 3]> = seeds
        .iter()
        .take(k)
        .map(|(_, n, s)| [s[0] / *n as f64, s[1] / *n as f64, s[2] / *n as f64])
        .collect();
    let mut assign = vec![0usize; labs.len()];
    for _ in 0..12 {
        for (i, lab) in labs.iter().enumerate() {
            assign[i] = nearest(&centroids, lab);
        }
        let mut sums = vec![([0.0; 3], 0usize); centroids.len()];
        for (lab, a) in labs.iter().zip(&assign) {
            for (acc, v) in sums[*a].0.iter_mut().zip(lab) {
                *acc += v;
            }
            sums[*a].1 += 1;
        }
        for (c, (s, n)) in centroids.iter_mut().zip(&sums) {
            if *n > 0 {
                *c = [s[0] / *n as f64, s[1] / *n as f64, s[2] / *n as f64];
            }
        }
    }
    let mut counts = vec![0usize; centroids.len()];
    for a in &assign {
        counts[*a] += 1;
    }
    let total = labs.len() as f64;
    let mut out: Vec<Cluster> = centroids
        .into_iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|(c, n)| Cluster {
            oklab: c,
            coverage: n as f64 / total,
        })
        .collect();
    out.sort_by(|a, b| b.coverage.total_cmp(&a.coverage));
    out
}

fn nearest(centroids: &[[f64; 3]], lab: &[f64; 3]) -> usize {
    let d = |c: &[f64; 3]| (0..3).map(|i| (c[i] - lab[i]).powi(2)).sum::<f64>();
    let mut best = 0;
    for (i, c) in centroids.iter().enumerate() {
        if d(c) < d(&centroids[best]) {
            best = i;
        }
    }
    best
}

/// How many 4-bit-per-channel colour bins it takes to cover 90% of `pixels`.
/// A flat illustration needs a few dozen at most; a photograph needs
/// hundreds. The classifier in [`super`] reads this.
pub fn bins_for_90_percent(pixels: &[Rgb]) -> usize {
    if pixels.is_empty() {
        return 0;
    }
    let mut bins: std::collections::HashMap<u16, u32> = Default::default();
    for p in pixels {
        let key = ((p.r as u16 >> 4) << 8) | ((p.g as u16 >> 4) << 4) | (p.b as u16 >> 4);
        *bins.entry(key).or_default() += 1;
    }
    let mut counts: Vec<u32> = bins.into_values().collect();
    counts.sort_unstable_by(|a, b| b.cmp(a));
    let target = (pixels.len() as f64 * 0.9).ceil() as u64;
    let mut acc = 0u64;
    for (i, c) in counts.iter().enumerate() {
        acc += *c as u64;
        if acc >= target {
            return i + 1;
        }
    }
    counts.len()
}

// ---------------------------------------------------------------------------
// Known placeholder patterns
// ---------------------------------------------------------------------------

/// Smallest side, in CSS px, at which [`is_crossed_box`] answers at all. A
/// real placeholder fills a content slot; an icon is an X on a flat ground
/// too. In CSS px, not buffer px: at DPR 2 a 48-CSS-px close icon is 96
/// buffer px, and a buffer-px floor let it through.
pub const CROSSED_BOX_MIN_SIDE_CSS: f64 = 64.0;

/// Largest OKLab chroma at which a colour counts as NEUTRAL for
/// [`placeholder_colours`].
pub const CROSSED_BOX_MAX_CHROMA: f64 = 0.06;

/// Widest hue gap (degrees, OKLab) between a TINTED ground and its ink for
/// the pair to be one hue at two lightnesses ([`placeholder_colours`]).
pub const CROSSED_BOX_MAX_HUE_GAP: f64 = 30.0;

/// Below this OKLab chroma a colour has no meaningful hue.
const HUE_DEFINED_CHROMA: f64 = 0.01;

/// The central box, as fractions of the width and height, whose ink
/// [`is_crossed_box`] leaves out of its census: a placeholder's caption sits
/// there, and captions are wide ("Hero image 1200 × 630" runs past half the
/// width) and short.
pub const CROSSED_BOX_CAPTION_BOX: (f64, f64) = (0.6, 0.3);

/// The central CORE, as fractions of the width and height, over which the
/// caption box's ink density is measured. Narrower than
/// [`CROSSED_BOX_CAPTION_BOX`] on purpose: a long caption's ends are sparse
/// either way, but measured over the whole 60%-wide box a compact central
/// subject is diluted below the cap (a hub ~24% × 28% read 0.31).
pub const CROSSED_BOX_DENSITY_CORE: (f64, f64) = (0.3, 0.3);

/// The widest band, as a fraction of the diagonal tolerance, around each
/// diagonal whose core ink is excused as the cross's own stroke. A
/// placeholder's strokes are thin (a 64-px bordered box: stroke 3.5,
/// tolerance 2.6, band 1.6 clears it); rays as wide as twice the tolerance
/// must not excuse a filled subject between them.
pub const CROSSED_BOX_MAX_EXCUSED_BAND: f64 = 0.6;

/// Most ink density the central core may hold, counting only ink off the
/// cross's own strokes (it runs through the core). Real
/// captions measured 0.06–0.11 there; a filled subject (a hallway's back
/// wall, a hub with rays to the corners) 0.62–0.78, and refuses.
pub const CROSSED_BOX_CAPTION_MAX_DENSITY: f64 = 0.35;

/// Whether a ground and an ink colour are a placeholder's: both NEUTRAL (grey
/// on grey, chroma ≤ [`CROSSED_BOX_MAX_CHROMA`]), or one TINT at two
/// lightnesses — both chromatic and within [`CROSSED_BOX_MAX_HUE_GAP`] of hue
/// (Tailwind blue-100 under blue-300, pale yellow under amber). A coloured X
/// on a neutral field, or on a field of another hue, is a graphic: a crimson
/// saltire on white has an achromatic ground, whose hue cannot match anything.
pub fn placeholder_colours(ground: [f64; 3], ink: [f64; 3]) -> bool {
    let lab = |c: [f64; 3]| {
        oklab(Rgb::new(
            c[0].round() as u8,
            c[1].round() as u8,
            c[2].round() as u8,
        ))
    };
    let (g, i) = (lab(ground), lab(ink));
    let (gc, ic) = (g[1].hypot(g[2]), i[1].hypot(i[2]));
    if gc <= CROSSED_BOX_MAX_CHROMA && ic <= CROSSED_BOX_MAX_CHROMA {
        return true;
    }
    if gc < HUE_DEFINED_CHROMA || ic < HUE_DEFINED_CHROMA {
        return false;
    }
    let gap = (g[2].atan2(g[1]) - i[2].atan2(i[1]))
        .to_degrees()
        .rem_euclid(360.0);
    gap.min(360.0 - gap) <= CROSSED_BOX_MAX_HUE_GAP
}

/// The wireframe placeholder convention: a flat fill crossed by its two
/// diagonals (optionally with a border), at ANY size, aspect ratio and line
/// weight.
///
/// Structural rather than a pHash match on purpose: the hash of one reference
/// rendering lands 8–14 bits from the same pattern at another size or line
/// weight (measured with `imagehash` at 97×64, 200×150, 640×360 and a
/// thin-lined 400×300), so a hash list recognises the one picture it was cut
/// from and almost nothing else. The test here is what makes the pattern the
/// pattern:
///
/// 0. the short side is at least [`CROSSED_BOX_MIN_SIDE_CSS`] CSS px at the
///    frame's `scale_factor`;
/// 1. a flat, NEUTRAL ground — the most common 4-bit colour bin, and every
///    pixel within 24 (RGB distance) of its mean, is "ground"; the rest is
///    "ink", and ink is between 0.1% and 40% of the crop (a small, heavily
///    bordered box is about a third ink). Ground and mean ink are a
///    placeholder's colours ([`placeholder_colours`]: grey on grey, or one
///    tint at two lightnesses). Ink inside the central
///    [`CROSSED_BOX_CAPTION_BOX`] is left out of this census and of rule 2,
///    so a caption does not hide the pattern — but only while its central
///    [`CROSSED_BOX_DENSITY_CORE`] is as sparse as a caption
///    ([`CROSSED_BOX_CAPTION_MAX_DENSITY`]);
/// 2. at least 85% of the ink lies on a diagonal or the border (within
///    `max(2.5, 4% of the short side)` px);
/// 3. BOTH diagonals run CORNER TO CORNER: at 95% or more of the points
///    sampled along each over t = 0.02..0.98 (those clear of the border, so
///    a border cannot stand in for a diagonal), ink lies within a small
///    window (`max(2, 2% of the short side)` px). A placeholder's cross spans
///    its box; an icon's X is padded 20–25% inside its viewBox, so its
///    diagonals stop well short of the corners.
///
/// **Known limits — these still MATCH.** The rules are structural, so a
/// non-placeholder with the same structure passes them: a corner-to-corner
/// checkbox X drawn at 64 CSS px or more (smaller ones are refused only by
/// the size floor), and a monochrome chart whose two lines really do reach
/// the corners (an inset chart is refused by corner reach). Neither is common
/// at slot size, and the finding this feeds is Info; an assertion over it is
/// a judgement the caller opts into. The same holds for a TINTED brand
/// graphic drawn the same way — say blue-500 X ribbons running corner to
/// corner on a blue-100 field — which [`placeholder_colours`] accepts as one
/// hue at two lightnesses. And a compact central hub (about 16% × 20% of
/// the box) with rays to the corners matches at ANY ray width: in the 30%
/// core its density is indistinguishable from a caption's.
pub fn is_crossed_box(rgb: &RgbImage, scale_factor: f64) -> bool {
    let (w, h) = rgb.dimensions();
    if !(scale_factor.is_finite() && scale_factor > 0.0)
        || (w.min(h) as f64) < CROSSED_BOX_MIN_SIDE_CSS * scale_factor
    {
        return false;
    }
    let bin = |p: &::image::Rgb<u8>| {
        ((p[0] as u16 >> 4) << 8) | ((p[1] as u16 >> 4) << 4) | (p[2] as u16 >> 4)
    };
    let mut counts: std::collections::HashMap<u16, (u64, [u64; 3])> = Default::default();
    for p in rgb.pixels() {
        let e = counts.entry(bin(p)).or_insert((0, [0; 3]));
        e.0 += 1;
        for (acc, v) in e.1.iter_mut().zip(p.0) {
            *acc += v as u64;
        }
    }
    let Some((_, (n, sum))) = counts
        .iter()
        .max_by_key(|(k, (n, _))| (*n, std::cmp::Reverse(**k)))
    else {
        return false;
    };
    let ground = [
        sum[0] as f64 / *n as f64,
        sum[1] as f64 / *n as f64,
        sum[2] as f64 / *n as f64,
    ];
    let is_ink = |p: &::image::Rgb<u8>| {
        (0..3)
            .map(|i| (p[i] as f64 - ground[i]).powi(2))
            .sum::<f64>()
            .sqrt()
            > 24.0
    };
    let (wf, hf) = (w as f64, h as f64);
    let tol = (0.04 * wf.min(hf)).max(2.5);
    let diag_len = wf.hypot(hf);
    let diag_dist = |x: f64, y: f64| {
        let d1 = (x * hf - y * wf).abs() / diag_len;
        let d2 = (x * hf + y * wf - wf * hf).abs() / diag_len;
        d1.min(d2)
    };
    let border_dist = |x: f64, y: f64| x.min(y).min(wf - x).min(hf - y);
    let on_structure = |x: f64, y: f64| diag_dist(x, y) <= tol || border_dist(x, y) <= tol;
    // The central box is left out of the ink census: placeholders routinely
    // carry a caption there ("Hero image 1200 × 630"), and its glyphs are
    // neither diagonal nor border. The diagonals still cross it, but they are
    // measured everywhere else, and corner reach (below) is measured along
    // their whole length.
    let within = |(fw, fh): (f64, f64), x: f64, y: f64| {
        (x - wf / 2.0).abs() <= fw * wf / 2.0 && (y - hf / 2.0).abs() <= fh * hf / 2.0
    };
    let central = |x: f64, y: f64| within(CROSSED_BOX_CAPTION_BOX, x, y);
    let core = |x: f64, y: f64| within(CROSSED_BOX_DENSITY_CORE, x, y);
    let total = (w as u64) * (h as u64);
    let (mut ink, mut ink_on) = (0u64, 0u64);
    let mut central_px = 0u64;
    // Diagonal distance of every ink pixel in the core, judged once the
    // stroke width is known.
    let mut core_ink: Vec<f64> = Vec::new();
    // Ink on a diagonal (and off the border) outside the caption box: what
    // the stroke width is estimated from.
    let mut diag_ink = 0u64;
    let mut ink_sum = [0.0f64; 3];
    for (x, y, p) in rgb.enumerate_pixels() {
        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
        if central(px, py) {
            if core(px, py) {
                central_px += 1;
                if is_ink(p) {
                    core_ink.push(diag_dist(px, py));
                }
            }
            continue;
        }
        if is_ink(p) {
            ink += 1;
            if on_structure(px, py) {
                ink_on += 1;
            }
            if diag_dist(px, py) <= tol && border_dist(px, py) > tol {
                diag_ink += 1;
            }
            for (acc, v) in ink_sum.iter_mut().zip(p.0) {
                *acc += v as f64;
            }
        }
    }
    // A caption is sparse; a filled central subject is not, and the
    // exclusion above must not wave it through. The cross itself runs
    // through the core, so the ink ON its strokes is not counted: the stroke
    // width is estimated from the diagonals outside the caption box (about
    // 70% of their length, less the border bands), and only core ink farther
    // than half of it (plus a pixel of anti-aliasing) from a diagonal counts.
    // The diagonal tolerance itself would be far too wide here: it is sized
    // to find thin lines, and would excuse most of a filled subject.
    let measured_len = (2.0 * 0.7 * diag_len - 8.0 * tol).max(1.0);
    let stroke = diag_ink as f64 / measured_len;
    // Capped: a thick enough "stroke" (rays as wide as the diagonal
    // tolerance) would otherwise excuse most of a compact central subject.
    let band = (stroke / 2.0 + 1.0).min(CROSSED_BOX_MAX_EXCUSED_BAND * tol);
    let central_ink = core_ink.iter().filter(|d| **d > band).count() as u64;
    if central_px > 0 && central_ink as f64 > CROSSED_BOX_CAPTION_MAX_DENSITY * central_px as f64 {
        return false;
    }
    let share = ink as f64 / total as f64;
    if !(0.001..=0.40).contains(&share) || (ink_on as f64) < 0.85 * ink as f64 {
        return false;
    }
    let ink_mean = ink_sum.map(|v| v / ink as f64);
    if !placeholder_colours(ground, ink_mean) {
        return false;
    }
    // The corner-reach search window: small, so a sample near (but not on)
    // the border cannot borrow the border's ink.
    let r = (0.02 * wf.min(hf)).max(2.0).ceil() as i64;
    let inked_near = |cx: f64, cy: f64| {
        let (cx, cy) = (cx as i64, cy as i64);
        (cy - r..=cy + r).any(|y| {
            (cx - r..=cx + r).any(|x| {
                x >= 0
                    && y >= 0
                    && (x as u32) < w
                    && (y as u32) < h
                    && is_ink(rgb.get_pixel(x as u32, y as u32))
            })
        })
    };
    // Corner to corner: every sample point clear of the border (so the search
    // window cannot find border ink) must find ink.
    let clear = 2.0 * r as f64 + 1.0;
    let drawn = |anti: bool| {
        let (mut counted, mut inked) = (0u32, 0u32);
        for i in 0..48 {
            let t = 0.02 + 0.96 * i as f64 / 47.0;
            let (x, y) = (if anti { (1.0 - t) * wf } else { t * wf }, t * hf);
            if x.min(y).min(wf - x).min(hf - y) <= clear {
                continue;
            }
            counted += 1;
            if inked_near(x, y) {
                inked += 1;
            }
        }
        counted >= 8 && inked as f64 >= 0.95 * counted as f64
    };
    drawn(false) && drawn(true)
}

// ---------------------------------------------------------------------------
// Perceptual hashes
// ---------------------------------------------------------------------------

/// Resize a luma image the way `imagehash` does (`Image.resize(…, LANCZOS)`)
/// as closely as the `image` crate allows: Lanczos3, support scaled on
/// downsampling. The two resamplers are not bit-identical, which is why the
/// cross-check against `imagehash` is within a Hamming tolerance.
fn resize_luma(luma: &GrayImage, w: u32, h: u32) -> GrayImage {
    imageops::resize(luma, w, h, FilterType::Lanczos3)
}

/// Unnormalized DCT-II of `x` (scipy's `fftpack.dct` default): `y[k] = 2 Σ
/// x[n] cos(π k (2n + 1) / 2N)`. The scale does not matter to the hash, which
/// compares coefficients to their own median; it is matched anyway.
fn dct_ii(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    (0..n)
        .map(|k| {
            2.0 * x
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    v * (std::f64::consts::PI * k as f64 * (2 * i + 1) as f64 / (2 * n) as f64)
                        .cos()
                })
                .sum::<f64>()
        })
        .collect()
}

/// Pack a row-major bit matrix into a u64, FIRST bit most significant — the
/// order `imagehash`'s `str()` writes its hex digits in.
fn pack_bits(bits: impl IntoIterator<Item = bool>) -> u64 {
    bits.into_iter()
        .fold(0u64, |acc, b| (acc << 1) | u64::from(b))
}

/// `imagehash.phash(img)` (hash_size 8, highfreq_factor 4): luma, resized to
/// 32×32, 2-D DCT-II, the top-left 8×8 coefficients compared with their
/// median (DC term included, as `imagehash` does).
pub fn phash(luma: &GrayImage) -> Option<u64> {
    if luma.width() == 0 || luma.height() == 0 {
        return None;
    }
    let small = resize_luma(luma, 32, 32);
    let rows: Vec<Vec<f64>> = (0..32)
        .map(|y| (0..32).map(|x| small.get_pixel(x, y)[0] as f64).collect())
        .collect();
    // DCT along axis 0 (columns) then axis 1 (rows), as imagehash does.
    let mut cols_dct = vec![vec![0.0; 32]; 32];
    for x in 0..32 {
        let col: Vec<f64> = (0..32).map(|y| rows[y][x]).collect();
        for (y, v) in dct_ii(&col).into_iter().enumerate() {
            cols_dct[y][x] = v;
        }
    }
    let full: Vec<Vec<f64>> = cols_dct.iter().map(|r| dct_ii(r)).collect();
    let low: Vec<f64> = (0..8)
        .flat_map(|y| (0..8).map(move |x| (y, x)))
        .map(|(y, x)| full[y][x])
        .collect();
    let mut sorted = low.clone();
    sorted.sort_by(f64::total_cmp);
    let median = (sorted[31] + sorted[32]) / 2.0;
    Some(pack_bits(low.iter().map(|v| *v > median)))
}

/// `imagehash.dhash(img)` (hash_size 8): luma resized to 9×8, each bit "is
/// the next pixel to the right brighter".
pub fn dhash(luma: &GrayImage) -> Option<u64> {
    if luma.width() == 0 || luma.height() == 0 {
        return None;
    }
    let small = resize_luma(luma, 9, 8);
    Some(pack_bits((0..8).flat_map(|y| {
        let small = &small;
        (0..8).map(move |x| small.get_pixel(x + 1, y)[0] > small.get_pixel(x, y)[0])
    })))
}

/// Hamming distance between two 64-bit hashes.
pub fn hamming(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// The hash as `imagehash` prints it: 16 lowercase hex digits.
pub fn hash_hex(h: u64) -> String {
    format!("{h:016x}")
}

/// Parse [`hash_hex`]'s form back.
pub fn parse_hash_hex(s: &str) -> Option<u64> {
    if s.len() != 16 {
        return None;
    }
    u64::from_str_radix(s, 16).ok()
}

// ---------------------------------------------------------------------------
// Saliency (spectral residual, Hou & Zhang 2007)
// ---------------------------------------------------------------------------

/// Side of the square grid the saliency map is computed on (the method's
/// customary 64).
pub const SALIENCY_GRID: u32 = 64;

/// 1-D DFT (forward when `sign == -1`, inverse without the 1/N when `+1`).
fn dft(re: &[f64], im: &[f64], sign: f64) -> (Vec<f64>, Vec<f64>) {
    let n = re.len();
    let mut out_re = vec![0.0; n];
    let mut out_im = vec![0.0; n];
    for k in 0..n {
        let (mut sr, mut si) = (0.0, 0.0);
        for t in 0..n {
            let ang = sign * 2.0 * std::f64::consts::PI * (k * t % n) as f64 / n as f64;
            let (s, c) = ang.sin_cos();
            sr += re[t] * c - im[t] * s;
            si += re[t] * s + im[t] * c;
        }
        out_re[k] = sr;
        out_im[k] = si;
    }
    (out_re, out_im)
}

fn dft2(re: &mut [Vec<f64>], im: &mut [Vec<f64>], sign: f64) {
    let n = re.len();
    for y in 0..n {
        let (r, i) = dft(&re[y], &im[y], sign);
        re[y] = r;
        im[y] = i;
    }
    for x in 0..n {
        let col_r: Vec<f64> = (0..n).map(|y| re[y][x]).collect();
        let col_i: Vec<f64> = (0..n).map(|y| im[y][x]).collect();
        let (r, i) = dft(&col_r, &col_i, sign);
        for y in 0..n {
            re[y][x] = r[y];
            im[y][x] = i[y];
        }
    }
}

/// Mean filter with edge replication, radius `r`.
fn box_blur(m: &[Vec<f64>], r: usize) -> Vec<Vec<f64>> {
    let n = m.len();
    let mut out = vec![vec![0.0; n]; n];
    for (y, row) in out.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let mut s = 0.0;
            let mut c = 0.0;
            for dy in -(r as i64)..=(r as i64) {
                for dx in -(r as i64)..=(r as i64) {
                    let yy = (y as i64 + dy).clamp(0, n as i64 - 1) as usize;
                    let xx = (x as i64 + dx).clamp(0, n as i64 - 1) as usize;
                    s += m[yy][xx];
                    c += 1.0;
                }
            }
            *cell = s / c;
        }
    }
    out
}

/// The spectral-residual saliency map of a whole frame on a
/// [`SALIENCY_GRID`]² grid, normalized so its maximum is 1.0. `None` for an
/// empty frame, or one whose map is identically zero (a flat frame has no
/// salient region).
///
/// The map is computed on a square grid, so a wide frame is squashed
/// horizontally first; element regions are mapped onto the grid with the
/// same per-axis scale (see [`grid_mean`]).
pub fn spectral_residual(frame_luma: &GrayImage) -> Option<Vec<Vec<f64>>> {
    if frame_luma.width() == 0 || frame_luma.height() == 0 {
        return None;
    }
    let n = SALIENCY_GRID as usize;
    let small = imageops::resize(
        frame_luma,
        SALIENCY_GRID,
        SALIENCY_GRID,
        FilterType::Triangle,
    );
    // A flat frame has no salient region. Its spectrum is a lone DC term, and
    // the residual of an otherwise empty spectrum inverts to a spike at the
    // origin, which would read as "the top-left corner is salient".
    let (lo, hi) = small.pixels().fold((u8::MAX, u8::MIN), |(lo, hi), p| {
        (lo.min(p[0]), hi.max(p[0]))
    });
    if hi <= lo {
        return None;
    }
    let mut re: Vec<Vec<f64>> = (0..n)
        .map(|y| {
            (0..n)
                .map(|x| small.get_pixel(x as u32, y as u32)[0] as f64 / 255.0)
                .collect()
        })
        .collect();
    let mut im = vec![vec![0.0; n]; n];
    dft2(&mut re, &mut im, -1.0);
    let log_amp: Vec<Vec<f64>> = (0..n)
        .map(|y| {
            (0..n)
                .map(|x| (re[y][x].hypot(im[y][x]) + 1e-9).ln())
                .collect()
        })
        .collect();
    let avg = box_blur(&log_amp, 1);
    for y in 0..n {
        for x in 0..n {
            let phase = im[y][x].atan2(re[y][x]);
            let mag = (log_amp[y][x] - avg[y][x]).exp();
            re[y][x] = mag * phase.cos();
            im[y][x] = mag * phase.sin();
        }
    }
    dft2(&mut re, &mut im, 1.0);
    let power: Vec<Vec<f64>> = (0..n)
        .map(|y| {
            (0..n)
                .map(|x| re[y][x].powi(2) + im[y][x].powi(2))
                .collect()
        })
        .collect();
    // ≈ a Gaussian of σ ≈ 2.5 grid cells: three passes of a radius-2 box.
    let map = box_blur(&box_blur(&box_blur(&power, 2), 2), 2);
    let max = map.iter().flatten().copied().fold(0.0_f64, f64::max);
    if max.is_nan() || max <= 0.0 || !max.is_finite() {
        return None;
    }
    Some(
        map.into_iter()
            .map(|row| row.into_iter().map(|v| v / max).collect())
            .collect(),
    )
}

/// Mean of the saliency map over a BUFFER-pixel region of a `frame_w` ×
/// `frame_h` frame, mapped onto the grid per axis. A region smaller than one
/// grid cell reads the cell under its centre. The region is CLAMPED onto the
/// grid, so callers pass an in-frame region ([`super::Crop::px`]); `None` only
/// for an empty frame or an empty region.
pub fn grid_mean(
    map: &[Vec<f64>],
    frame_w: u32,
    frame_h: u32,
    (rx, ry, rw, rh): (u32, u32, u32, u32),
) -> Option<f64> {
    let n = map.len() as f64;
    if frame_w == 0 || frame_h == 0 || rw == 0 || rh == 0 {
        return None;
    }
    let sx = n / frame_w as f64;
    let sy = n / frame_h as f64;
    let last = map.len() - 1;
    let x0 = ((rx as f64 * sx).floor() as usize).min(last);
    let y0 = ((ry as f64 * sy).floor() as usize).min(last);
    let x1 = (((rx + rw) as f64 * sx).ceil() as usize).clamp(x0 + 1, last + 1);
    let y1 = (((ry + rh) as f64 * sy).ceil() as usize).clamp(y0 + 1, last + 1);
    let (mut s, mut c) = (0.0, 0.0);
    for row in &map[y0..y1] {
        for v in &row[x0..x1] {
            s += v;
            c += 1.0;
        }
    }
    Some(s / c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray(w: u32, h: u32, f: impl Fn(u32, u32) -> u8) -> GrayImage {
        GrayImage::from_fn(w, h, |x, y| Luma([f(x, y)]))
    }

    #[test]
    fn pil_luma_matches_pillow_on_known_values() {
        // Values Pillow 13 returns for Image.new("RGB",(1,1),c).convert("L").
        assert_eq!(pil_luma(255, 255, 255), 255);
        assert_eq!(pil_luma(0, 0, 0), 0);
        assert_eq!(pil_luma(255, 0, 0), 76);
        assert_eq!(pil_luma(0, 255, 0), 150);
        assert_eq!(pil_luma(0, 0, 255), 29);
        assert_eq!(pil_luma(249, 115, 22), 144);
        assert_eq!(pil_luma(30, 58, 138), 59);
    }

    #[test]
    fn a_flat_image_has_zero_laplacian_variance_and_entropy() {
        let g = gray(16, 16, |_, _| 120);
        assert_eq!(laplacian_variance(&g), Some(0.0));
        assert_eq!(luma_entropy(&g), Some(0.0));
        assert_eq!(edge_density(&g), Some(0.0));
        assert_eq!(flat_tile_fraction(&g, 8), Some(1.0));
    }

    #[test]
    fn a_checkerboard_is_sharp_busy_and_maximally_two_level() {
        let g = gray(16, 16, |x, y| if (x + y) % 2 == 0 { 0 } else { 255 });
        assert!(laplacian_variance(&g).unwrap() > 10_000.0);
        assert!((luma_entropy(&g).unwrap() - 1.0).abs() < 1e-9);
    }

    fn grid(period: f64, phase: f64) -> CodingGrid {
        CodingGrid {
            period_x: period,
            phase_x: phase,
            period_y: period,
            phase_y: phase,
        }
    }

    #[test]
    fn blockiness_finds_a_grid_where_one_is_drawn_and_not_elsewhere() {
        // Flat 8-px blocks with sub-grid texture: every big step is ON the
        // grid, and there is detail off it to compare against.
        let textured = gray(64, 64, |x, y| {
            let base: u8 = if (x / 8 + y / 8) % 2 == 0 { 100 } else { 140 };
            base + ((x * 7 + y * 3) % 3) as u8
        });
        let smooth = gray(64, 64, |x, y| ((x * 3 + y * 2) % 251) as u8);
        assert!(blockiness(&textured, grid(8.0, 0.0)).unwrap() > 5.0);
        // The same blocks sought half a period off read as clean.
        assert!(blockiness(&textured, grid(8.0, 4.0)).unwrap() < 1.0);
        // A phase a whole period back is the same grid.
        let a = blockiness(&textured, grid(8.0, 0.0)).unwrap();
        assert!((blockiness(&textured, grid(8.0, -8.0)).unwrap() - a).abs() < 1e-12);
        let s = blockiness(&smooth, grid(8.0, 0.0)).unwrap();
        assert!((0.5..1.6).contains(&s), "smooth image blockiness {s}");
        assert_eq!(blockiness(&smooth, grid(3.0, 0.0)), None);
    }

    #[test]
    fn blockiness_has_no_value_without_detail_off_the_grid() {
        // Flat blocks exactly on the grid: the off-grid step is zero, and a
        // ratio over it would be an infinity, not a measurement.
        let blocky = gray(
            64,
            64,
            |x, y| if (x / 8 + y / 8) % 2 == 0 { 100 } else { 140 },
        );
        assert_eq!(blockiness(&blocky, grid(8.0, 0.0)), None);
        assert_eq!(blockiness(&gray(32, 32, |_, _| 7), grid(8.0, 0.0)), None);
    }

    #[test]
    fn oklab_round_trips_srgb() {
        for c in [
            Rgb::new(0, 0, 0),
            Rgb::new(255, 255, 255),
            Rgb::new(249, 115, 22),
            Rgb::new(30, 58, 138),
        ] {
            assert_eq!(oklab_to_rgb(oklab(c)), c);
        }
        let white = oklab(Rgb::new(255, 255, 255));
        assert!((white[0] - 1.0).abs() < 1e-3 && white[1].abs() < 1e-3 && white[2].abs() < 1e-3);
    }

    #[test]
    fn kmeans_recovers_two_flat_colours_with_their_coverage() {
        let mut px = vec![Rgb::new(30, 58, 138); 300];
        px.extend(vec![Rgb::new(249, 115, 22); 100]);
        let clusters = kmeans_oklab(&px, 5);
        assert_eq!(clusters.len(), 2);
        assert!((clusters[0].coverage - 0.75).abs() < 1e-9);
        assert_eq!(oklab_to_rgb(clusters[0].oklab), Rgb::new(30, 58, 138));
        assert_eq!(oklab_to_rgb(clusters[1].oklab), Rgb::new(249, 115, 22));
    }

    #[test]
    fn hashes_are_stable_and_hex_round_trips() {
        let g = gray(64, 48, |x, y| ((x * 4 + y * 2) % 256) as u8);
        let p = phash(&g).unwrap();
        assert_eq!(phash(&g), Some(p));
        assert_eq!(parse_hash_hex(&hash_hex(p)), Some(p));
        assert_eq!(hamming(p, p), 0);
        assert_eq!(hamming(0, u64::MAX), 64);
    }

    #[test]
    fn spectral_residual_peaks_on_the_odd_one_out() {
        // A flat page with one small dark square: the square is the salient
        // region and the empty page around it is not.
        let g = gray(128, 128, |x, y| {
            if (90..100).contains(&x) && (20..30).contains(&y) {
                20
            } else {
                230
            }
        });
        let map = spectral_residual(&g).unwrap();
        let on = grid_mean(&map, 128, 128, (90, 20, 10, 10)).unwrap();
        let off = grid_mean(&map, 128, 128, (10, 90, 10, 10)).unwrap();
        assert!(on > 5.0 * off, "on {on} off {off}");
        assert!(spectral_residual(&gray(32, 32, |_, _| 128)).is_none());
    }
}
