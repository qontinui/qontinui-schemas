#!/usr/bin/env python3
"""Generate the `image` analyzer's fixture corpus and its imagehash reference.

Run ONCE, by hand, from a throwaway virtualenv; the outputs are committed and
the Rust suite never runs Python:

    python3 -m venv /tmp/venv
    /tmp/venv/bin/pip install pillow numpy scipy imagehash
    /tmp/venv/bin/python generate.py            # writes next to this file

Writes, beside this script:

- ``base.png``        the HELD image: an illustration in three declared
                      colours (cream, navy, orange) with fine aperiodic
                      texture, its subject (the orange disc) in the lower
                      right.
- ``recolour.png``    ``base`` with every hue rotated 120 degrees: the
                      off-palette mutation.
- ``blur.png``        ``base`` scaled down 4x and back up: the soft mutation.
- ``jpeg_q10.jpg``    ``base`` encoded as JPEG at quality 10: the blocky
                      mutation.
- ``duplicate.png``   ``base`` rescaled to 80% (a near, not byte-identical,
                      copy): the duplicate mutation.
- ``other.png``       a different scene, the held partner of ``duplicate``.
- ``placeholder.png`` a flat grey fill: the placeholder mutation.
- ``stripes.png``, ``rings.png``  two extra structured images, only for the
                      hash cross-check.
- ``expected_hashes.json``  Python ``imagehash.phash`` / ``imagehash.dhash`` of
                      every PNG above except the flat placeholder, whose pHash
                      is degenerate (all AC coefficients are zero, so every bit
                      but the DC term is decided by floating-point noise).

The Rust test ``tests/image_corpus.rs`` recomputes the hashes in-crate and
requires agreement within a DECLARED Hamming tolerance, not bit for bit:
Pillow's Lanczos resampling and the ``image`` crate's differ numerically.
"""

from __future__ import annotations

import json
import pathlib
import sys

import imagehash
import numpy as np
import PIL
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
W, H = 128, 96
SEED = 15

CREAM = (254, 243, 199)
NAVY = (30, 58, 138)
ORANGE = (249, 115, 22)


def base_image() -> Image.Image:
    rng = np.random.default_rng(SEED)
    yy, xx = np.mgrid[0:H, 0:W].astype(np.float64)
    img = np.zeros((H, W, 3), dtype=np.float64)
    img[:] = CREAM
    # A navy "hill" across the bottom, with a wavy edge.
    ridge = 58 + 6 * np.sin(xx / 9.0)
    hill = yy > ridge
    img[hill] = NAVY
    # The orange subject: a disc in the lower right quadrant.
    disc = (xx - 92) ** 2 + (yy - 58) ** 2 < 15**2
    img[disc] = ORANGE
    # Fine texture so sharpness and the hashes have detail to read: scattered
    # light speckles inside the hill and low-amplitude noise everywhere. Both
    # are APERIODIC on purpose. A periodic texture aliases with the 8-pixel
    # coding grid the blockiness value looks for (stripes of period 6 put a
    # step on two of every three grid lines), and the held image would then
    # read as blocky before it was ever compressed.
    speckle = hill & ~disc & (rng.random((H, W)) < 0.12)
    img[speckle] += 22
    img += rng.normal(0.0, 5.0, size=img.shape)
    return Image.fromarray(np.clip(img, 0, 255).round().astype(np.uint8), "RGB")


def other_image() -> Image.Image:
    yy, xx = np.mgrid[0:H, 0:W].astype(np.float64)
    img = np.zeros((H, W, 3), dtype=np.float64)
    teal = np.array([13, 148, 136], dtype=np.float64)
    purple = np.array([126, 34, 206], dtype=np.float64)
    t = ((xx + yy) // 12) % 2
    img[:] = teal
    img[t == 1] = purple
    checker = ((xx // 32) + (yy // 32)) % 2 == 0
    img[checker] = img[checker] * 0.6 + 255 * 0.4
    return Image.fromarray(img.round().astype(np.uint8), "RGB")


def stripes_image() -> Image.Image:
    # Diagonal, so the pattern varies along BOTH axes: a stripe varying in x
    # only leaves every DCT row but the first at zero, and those bits would be
    # decided by floating-point noise rather than by the image.
    yy, xx = np.mgrid[0:H, 0:W].astype(np.float64)
    v = 128 + 100 * np.sin((xx + 0.6 * yy) / 5.0)
    return Image.fromarray(np.stack([v, v * 0.5, 255 - v], axis=-1).round().astype(np.uint8), "RGB")


def rings_image() -> Image.Image:
    yy, xx = np.mgrid[0:H, 0:W].astype(np.float64)
    r = np.hypot(xx - 40, yy - 30)
    v = 128 + 120 * np.cos(r / 6.0)
    return Image.fromarray(np.stack([v, 255 - v, v], axis=-1).round().astype(np.uint8), "RGB")


def rotate_hue(img: Image.Image, degrees: float) -> Image.Image:
    h, s, v = img.convert("HSV").split()
    shift = int(round(degrees / 360.0 * 256)) % 256
    h = h.point(lambda p: (p + shift) % 256)
    return Image.merge("HSV", (h, s, v)).convert("RGB")


def main() -> int:
    base = base_image()
    outputs = {
        "base.png": base,
        "recolour.png": rotate_hue(base, 120),
        "blur.png": base.resize((W // 4, H // 4), Image.Resampling.BILINEAR).resize(
            (W, H), Image.Resampling.BILINEAR
        ),
        "duplicate.png": base.resize((int(W * 0.8), int(H * 0.8)), Image.Resampling.LANCZOS),
        "other.png": other_image(),
        "placeholder.png": Image.new("RGB", (W, H), (229, 231, 235)),
        "stripes.png": stripes_image(),
        "rings.png": rings_image(),
    }
    for name, img in outputs.items():
        img.save(HERE / name, optimize=True)
    base.save(HERE / "jpeg_q10.jpg", quality=10)

    hashes = {}
    for name in sorted(outputs):
        if name == "placeholder.png":
            continue
        # Hash the file as written, exactly as the Rust side reads it.
        with Image.open(HERE / name) as img:
            hashes[name] = {
                "phash": str(imagehash.phash(img)),
                "dhash": str(imagehash.dhash(img)),
            }
    doc = {
        "generator": "rust-vision-core/tests/fixtures/image_corpus/generate.py",
        "reference": {
            "imagehash": imagehash.__version__,
            "pillow": PIL.__version__,
            "numpy": np.__version__,
            "python": sys.version.split()[0],
        },
        "hashes": hashes,
    }
    (HERE / "expected_hashes.json").write_text(json.dumps(doc, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
