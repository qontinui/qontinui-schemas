// SPDX-License-Identifier: MIT
//! Whole-file perceptual hashing — the ONE hasher every caller shares.
//!
//! [`hash_file`] / [`hash_bytes`] decode a raster image and run the SAME
//! [`pixels::phash`] / [`pixels::dhash`] the per-element image analyzer runs
//! on its crops (plan 2026-10-13-image-generation-and-visual-judgment, PD9),
//! over the whole image instead of a crop. They exist so a caller that needs a
//! fingerprint of an asset file — the design-reference library's
//! `asset_fingerprint` and its asset guard (plan
//! 2026-11-13-design-reference-library-stores-described-web-examples-for-any-project-to-query,
//! Phase 2 / Phase 5) — gets byte-for-byte the hash the analyzer would have
//! computed, rather than a second hasher that drifts from it. The
//! `vision-audit hash` subcommand is a thin CLI over this module.
//!
//! **Formats.** Whatever this crate's `image` dependency decodes: PNG, JPEG
//! and WebP (its enabled features). SVG is NOT decoded here — a vector
//! document has no pixels until something rasterises it, and the rasteriser
//! choice is the caller's; rasterise first, then hash the raster.
//!
//! **Alpha.** The image is flattened to RGB by dropping the alpha channel
//! (`to_rgb8`), as Pillow's `convert("L")` does for the `imagehash` reference
//! the hashes are cross-checked against. A transparent region therefore hashes
//! as whatever colour its pixels carry, not as a background.

use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::analyzers::image::pixels;
use crate::cache::{hex_encode, sha256_of};

/// The fingerprint of one image file.
///
/// `phash` / `dhash` are 64-bit `imagehash`-compatible hashes; compare two with
/// [`pixels::hamming`]. They serialise as 16 lowercase hex digits
/// ([`pixels::hash_hex`]), which [`pixels::parse_hash_hex`] reads back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImageHashes {
    /// DCT perceptual hash (`imagehash.phash`, hash_size 8).
    #[serde(serialize_with = "serialize_hash_hex")]
    pub phash: u64,
    /// Difference hash (`imagehash.dhash`, hash_size 8).
    #[serde(serialize_with = "serialize_hash_hex")]
    pub dhash: u64,
    /// Decoded width in pixels.
    pub width: u32,
    /// Decoded height in pixels.
    pub height: u32,
    /// Lowercase hex SHA-256 of the file's BYTES (not of the decoded pixels),
    /// so an exact-copy check needs no decode.
    pub sha256: String,
}

fn serialize_hash_hex<S: serde::Serializer>(h: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&pixels::hash_hex(*h))
}

/// Why one file could not be hashed.
#[derive(Debug, Error)]
pub enum ImageHashError {
    /// The file could not be read.
    #[error("cannot read {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// The bytes are not an image this crate decodes (PNG, JPEG, WebP).
    #[error("cannot decode image: {0}")]
    Decode(String),
    /// The image decoded to zero pixels, which has no perceptual hash.
    #[error("image has zero width or height")]
    Empty,
}

impl ImageHashError {
    /// Stable machine code for the failure, for JSON consumers.
    pub fn code(&self) -> &'static str {
        match self {
            ImageHashError::Read { .. } => "read_failed",
            ImageHashError::Decode(_) => "decode_failed",
            ImageHashError::Empty => "empty_image",
        }
    }
}

/// Hash an image file: read, decode, then [`hash_bytes`].
pub fn hash_file(path: impl AsRef<Path>) -> Result<ImageHashes, ImageHashError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|source| ImageHashError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    hash_bytes(&bytes)
}

/// Hash encoded image bytes (PNG, JPEG or WebP; the format is sniffed).
pub fn hash_bytes(bytes: &[u8]) -> Result<ImageHashes, ImageHashError> {
    let sha256 = hex_encode(&sha256_of(bytes));
    let decoded =
        ::image::load_from_memory(bytes).map_err(|e| ImageHashError::Decode(e.to_string()))?;
    let rgb = decoded.to_rgb8();
    let luma = pixels::luma_image(&rgb);
    let (phash, dhash) = match (pixels::phash(&luma), pixels::dhash(&luma)) {
        (Some(p), Some(d)) => (p, d),
        _ => return Err(ImageHashError::Empty),
    };
    Ok(ImageHashes {
        phash,
        dhash,
        width: rgb.width(),
        height: rgb.height(),
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{ImageFormat, RgbImage};
    use std::io::Cursor;

    fn png(img: &RgbImage) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn hashes_match_the_analyzer_pixel_functions() {
        let img = RgbImage::from_fn(40, 30, |x, y| {
            ::image::Rgb([(x * 6) as u8, (y * 8) as u8, 90])
        });
        let h = hash_bytes(&png(&img)).unwrap();
        let luma = pixels::luma_image(&img);
        assert_eq!(h.phash, pixels::phash(&luma).unwrap());
        assert_eq!(h.dhash, pixels::dhash(&luma).unwrap());
        assert_eq!((h.width, h.height), (40, 30));
        assert_eq!(h.sha256.len(), 64);
    }

    #[test]
    fn json_spells_hashes_as_16_hex_digits() {
        let img = RgbImage::from_pixel(8, 8, ::image::Rgb([10, 20, 30]));
        let v = serde_json::to_value(hash_bytes(&png(&img)).unwrap()).unwrap();
        let p = v["phash"].as_str().unwrap();
        assert_eq!(p.len(), 16);
        assert!(pixels::parse_hash_hex(p).is_some());
    }

    #[test]
    fn non_image_bytes_are_a_decode_error() {
        let e = hash_bytes(b"not an image").unwrap_err();
        assert_eq!(e.code(), "decode_failed");
    }

    #[test]
    fn a_missing_file_is_a_read_error() {
        let e = hash_file("/nonexistent/definitely/missing.png").unwrap_err();
        assert_eq!(e.code(), "read_failed");
    }
}
