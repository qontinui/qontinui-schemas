// SPDX-License-Identifier: MIT
//! `vision-audit hash` — the shared whole-file hasher, driven through the real
//! binary (plan
//! 2026-11-13-design-reference-library-stores-described-web-examples-for-any-project-to-query,
//! Phase 2). Fixtures are generated in-test so the repo carries no binaries.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::imageops::{self, FilterType};
use image::{ImageFormat, Rgb, RgbImage};
use qontinui_vision_core::analyzers::image::pixels::{hamming, parse_hash_hex};
use serde_json::Value;

/// A "logo": a gradient ground with a dark disc and a bright bar.
fn logo(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        let (fx, fy) = (x as f64 / w as f64, y as f64 / h as f64);
        let disc = (fx - 0.35).powi(2) + (fy - 0.45).powi(2) < 0.06;
        let bar = (0.6..0.9).contains(&fx) && (0.2..0.8).contains(&fy);
        if disc {
            Rgb([20, 30, 60])
        } else if bar {
            Rgb([240, 220, 40])
        } else {
            let g = (fx * 120.0 + fy * 80.0) as u8 + 40;
            Rgb([g, g / 2 + 60, 150])
        }
    })
}

/// An unrelated image: diagonal stripes, bright in the lower-left.
fn unrelated(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        let stripe = ((x + 2 * y) / 9) % 2 == 0;
        let base = if x < w / 2 && y > h / 2 { 230 } else { 70 };
        let v = if stripe { base } else { base / 3 };
        Rgb([v, v, v])
    })
}

struct Fixtures {
    _dir: tempfile::TempDir,
    original: PathBuf,
    copy: PathBuf,
    resized_jpeg: PathBuf,
    unrelated: PathBuf,
    text: PathBuf,
}

fn fixtures() -> Fixtures {
    let dir = tempfile::tempdir().unwrap();
    let p = |name: &str| dir.path().join(name);
    let img = logo(240, 160);
    img.save_with_format(p("original.png"), ImageFormat::Png)
        .unwrap();
    std::fs::copy(p("original.png"), p("copy.png")).unwrap();
    imageops::resize(&img, 150, 100, FilterType::Triangle)
        .save_with_format(p("resized.jpg"), ImageFormat::Jpeg)
        .unwrap();
    unrelated(240, 160)
        .save_with_format(p("unrelated.png"), ImageFormat::Png)
        .unwrap();
    std::fs::write(p("notes.txt"), "this is not an image\n").unwrap();
    Fixtures {
        original: p("original.png"),
        copy: p("copy.png"),
        resized_jpeg: p("resized.jpg"),
        unrelated: p("unrelated.png"),
        text: p("notes.txt"),
        _dir: dir,
    }
}

fn run(paths: &[&Path]) -> (i32, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_vision-audit"))
        .arg("hash")
        .args(paths)
        .output()
        .unwrap();
    let json: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap(), json)
}

fn hash(entry: &Value, key: &str) -> u64 {
    parse_hash_hex(entry[key].as_str().unwrap()).unwrap()
}

#[test]
fn hash_subcommand_fingerprints_and_compares_images() {
    let f = fixtures();
    let (code, report) = run(&[&f.original, &f.copy, &f.resized_jpeg, &f.unrelated, &f.text]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(report["hashed"], 4);
    assert_eq!(report["failed"], 1);
    let files = report["files"].as_array().unwrap();
    let (orig, copy, resized, other, text) =
        (&files[0], &files[1], &files[2], &files[3], &files[4]);

    // Argument order is preserved and every success carries the full record.
    assert_eq!(orig["path"], f.original.to_str().unwrap());
    assert_eq!(orig["ok"], true);
    assert_eq!(
        (orig["width"].as_u64(), orig["height"].as_u64()),
        (Some(240), Some(160))
    );
    assert_eq!(resized["width"], 150);
    assert_eq!(orig["sha256"].as_str().unwrap().len(), 64);

    // Identical bytes -> identical hashes and digest.
    assert_eq!(orig["phash"], copy["phash"]);
    assert_eq!(orig["dhash"], copy["dhash"]);
    assert_eq!(orig["sha256"], copy["sha256"]);

    // Resized + recompressed -> different bytes, small Hamming distance.
    assert_ne!(orig["sha256"], resized["sha256"]);
    let near_p = hamming(hash(orig, "phash"), hash(resized, "phash"));
    let near_d = hamming(hash(orig, "dhash"), hash(resized, "dhash"));
    assert!(near_p <= 6, "resized phash distance {near_p}");
    assert!(near_d <= 6, "resized dhash distance {near_d}");

    // Unrelated image -> a clearly larger distance.
    let far_p = hamming(hash(orig, "phash"), hash(other, "phash"));
    let far_d = hamming(hash(orig, "dhash"), hash(other, "dhash"));
    assert!(far_p >= 16, "unrelated phash distance {far_p}");
    assert!(far_d >= 16, "unrelated dhash distance {far_d}");
    assert!(far_p > near_p && far_d > near_d);

    // A non-image file is a per-file error, not a crash.
    assert_eq!(text["ok"], false);
    assert_eq!(text["error"]["code"], "decode_failed");
    assert!(text.get("phash").is_none());
}

#[test]
fn every_file_failing_exits_1_with_a_report() {
    let f = fixtures();
    let missing = f.text.with_file_name("missing.png");
    let (code, report) = run(&[&f.text, &missing]);
    assert_eq!(code, 1);
    assert_eq!(report["hashed"], 0);
    assert_eq!(report["failed"], 2);
    assert_eq!(report["files"][1]["error"]["code"], "read_failed");
}

#[test]
fn no_files_is_a_usage_error() {
    let (code, report) = run(&[]);
    assert_eq!(code, 1);
    assert_eq!(report, Value::Null, "usage errors print no JSON");
}

#[test]
fn library_hash_file_agrees_with_the_cli() {
    let f = fixtures();
    let lib = qontinui_vision_core::hash_file(&f.original).unwrap();
    let (_, report) = run(&[&f.original]);
    let cli = &report["files"][0];
    assert_eq!(lib.phash, hash(cli, "phash"));
    assert_eq!(lib.dhash, hash(cli, "dhash"));
    assert_eq!(lib.sha256, cli["sha256"].as_str().unwrap());
}
