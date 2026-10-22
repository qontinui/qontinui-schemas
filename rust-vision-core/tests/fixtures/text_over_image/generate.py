#!/usr/bin/env python3
"""Generate the text-over-image fixture corpus (plan
2026-10-13-image-generation-and-visual-judgment, Phase 16 step 4).

Run ONCE, by hand; the outputs are committed and the Rust suite never runs
Python. Needs only Pillow with FreeType (written against Pillow 11.1.0):

    python3 generate.py            # writes next to this file

Each case is a PAIR of frames of one 320x180 page at DPR 1, as the two-pass
capture produces them: ``<case>_a.png`` renders the text, ``<case>_b.png`` is
the same page with the text hidden. ``manifest.json`` records, per case, the
image's box, the text element's box (its CSS line box, as a snapshot would
carry it) and its declared colour, so ``tests/text_over_image.rs`` can build
the snapshot.

- ``held``          the HELD case: white text over a dark photographic hero
                    sprinkled with sparse bright specks (about 2% of its
                    area). Its p5 contrast clears 4.5:1; its MINIMUM, set by
                    the glyph pixels that land on a speck, does not -- so it
                    passes at percentile 5 and fails at percentile 0.
- ``low_contrast``  the overlay mutation: near-white text over a LIGHT hero.
                    Must fail.
- ``scrim``         ``low_contrast`` with a solid dark scrim painted behind
                    the text. Must pass.
- ``wide_heading``  a block-level heading whose box spans the hero's full
                    width while its glyphs sit over the dark left part; the
                    rest of the box is over a light part. Passes under the
                    glyph mask and fails if the whole box is sampled.
"""

from __future__ import annotations

import json
import math
import pathlib
import random

from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = pathlib.Path(__file__).resolve().parent
W, H = 320, 180
PAGE = (248, 250, 252)
HERO = (0, 10, 320, 150)  # x, y, w, h
FONT = ImageFont.load_default(size=28)
TEXT = "Summer sale"
TEXT_AT = (24, 60)


def photo(rng: random.Random, base: float, amp: float, split: float | None = None) -> Image.Image:
    """A smooth, photographic-looking field with fine noise. With ``split``,
    the part right of ``split`` (a fraction of the width) is lifted to a light
    field instead."""
    x0, y0, w, h = HERO
    img = Image.new("RGB", (w, h))
    px = img.load()
    for y in range(h):
        for x in range(w):
            b = base if split is None or x < split * w else 215.0
            v = b + amp * (math.sin(x / 23) + math.cos(y / 17) + math.sin((x + y) / 31)) / 3
            n = rng.uniform(-6, 6)
            px[x, y] = tuple(
                max(0, min(255, round(v + n + t))) for t in (-8.0, 0.0, 10.0)
            )
    return img


def specks(img: Image.Image, spacing: int = 20, size: int = 3) -> None:
    d = ImageDraw.Draw(img)
    for y in range(4, img.height, spacing):
        for x in range(7, img.width, spacing):
            d.rectangle([x, y, x + size - 1, y + size - 1], fill=(245, 245, 245))


def page_with(hero: Image.Image) -> Image.Image:
    page = Image.new("RGB", (W, H), PAGE)
    page.paste(hero, (HERO[0], HERO[1]))
    return page


def line_box(text_at: tuple[int, int]) -> dict:
    """The CSS line box a snapshot would carry for the text: the ink bbox
    padded to a line height."""
    l, t, r, b = ImageDraw.Draw(Image.new("RGB", (1, 1))).textbbox(text_at, TEXT, font=FONT)
    return {"x": l - 2, "y": t - 6, "w": (r - l) + 4, "h": (b - t) + 12}


def glow(page: Image.Image, blur: float, alpha: int) -> Image.Image:
    """``page`` darkened by a blurred black ``text-shadow`` glow of TEXT."""
    layer = Image.new("L", page.size, 0)
    ImageDraw.Draw(layer).text(TEXT_AT, TEXT, font=FONT, fill=alpha)
    layer = layer.filter(ImageFilter.GaussianBlur(blur))
    return Image.composite(Image.new("RGB", page.size, (0, 0, 0)), page, layer)


def write_case(name: str, page_b: Image.Image, fg: tuple[int, int, int], text_box: dict,
               cases: dict, image: tuple[int, int, int, int] = HERO, shadow=None,
               extra=None) -> None:
    page_a = page_b.copy() if shadow is None else glow(page_b, *shadow)
    ImageDraw.Draw(page_a).text(TEXT_AT, TEXT, font=FONT, fill=fg)
    if extra is not None:
        extra(page_a)
    page_a.save(HERE / f"{name}_a.png", optimize=True)
    page_b.save(HERE / f"{name}_b.png", optimize=True)
    cases[name] = {
        "frame": f"{name}_a.png",
        "text_hidden": f"{name}_b.png",
        "image": {"x": image[0], "y": image[1], "w": image[2], "h": image[3]},
        "text": text_box,
        "fg": list(fg),
    }


def main() -> None:
    cases: dict = {}
    box = line_box(TEXT_AT)

    dark = photo(random.Random(16), base=55.0, amp=22.0)
    specks(dark)
    write_case("held", page_with(dark), (255, 255, 255), box, cases)

    light = photo(random.Random(17), base=215.0, amp=18.0)
    write_case("low_contrast", page_with(light), (250, 250, 250), box, cases)

    scrimmed = page_with(light)
    ImageDraw.Draw(scrimmed).rectangle(
        [box["x"] - 10, box["y"] - 8, box["x"] + box["w"] + 10, box["y"] + box["h"] + 8],
        fill=(16, 18, 24),
    )
    write_case("scrim", scrimmed, (250, 250, 250), box, cases)

    # The glyphs end before x = 0.6 * 320 = 192; the heading's box is the
    # hero's full width, as a block-level <h1> inside it would be.
    split = photo(random.Random(18), base=55.0, amp=22.0, split=0.6)
    wide = {"x": HERO[0], "y": box["y"], "w": HERO[2], "h": box["h"]}
    assert box["x"] + box["w"] < 0.6 * W, "the glyphs must stay over the dark part"
    write_case("wide_heading", page_with(split), (255, 255, 255), wide, cases)

    # Round 3 (review of 278a7ae6). Appended after the cases above so their
    # bytes do not move.

    # sky: white text whose second word sits on a near-white (253 +/- 1)
    # region of the hero -- glyph pixels the A/B difference can barely see.
    # Must NOT pass.
    sky = photo(random.Random(19), base=55.0, amp=22.0)
    px = sky.load()
    rng = random.Random(20)
    for y in range(sky.height):
        for x in range(110, sky.width):
            v = 253 + rng.choice((-1, 0, 0, 1))
            px[x, y] = (v, v, v)
    write_case("sky", page_with(sky), (255, 255, 255), box, cases)

    # glow_mid / glow_light: white text with a dark blurred text-shadow glow
    # (blur 10, alpha 150), which the hide removes along with the glyphs. On a
    # mid-tone hero the glyphs pass; on a light one they fail -- judged on the
    # glyph pixels only, never Unknown for the halo.
    mid = photo(random.Random(21), base=95.0, amp=18.0)
    write_case("glow_mid", page_with(mid), (255, 255, 255), box, cases, shadow=(10, 150))
    write_case("glow_light", page_with(light), (255, 255, 255), box, cases, shadow=(10, 150))

    # credit: a heading over a hero with an UNTRACKED 12 px photo credit in
    # its corner (in frame A only, absent from the snapshot). The frames are
    # still registered: the credit is a compact cluster of tiles.
    def credit(page: Image.Image) -> None:
        f = ImageFont.load_default(size=12)
        ImageDraw.Draw(page).text((210, 140), "Photo: J. Doe", font=f, fill=(255, 255, 255))

    write_case("credit", page_with(dark), (255, 255, 255), box, cases, extra=credit)

    # rerastered: the scrim case with a frame B that did NOT hide the text and
    # re-rasterised its glyphs a quarter pixel to the right (a
    # non-deterministic raster of an unhidden twin).
    scrim_b = Image.open(HERE / "scrim_b.png").convert("RGB")
    big = scrim_b.resize((W * 4, H * 4), Image.NEAREST)
    ImageDraw.Draw(big).text(
        (TEXT_AT[0] * 4 + 1, TEXT_AT[1] * 4), TEXT, font=ImageFont.load_default(size=112),
        fill=(250, 250, 250),
    )
    rerastered = scrim_b.copy()
    tb = (box["x"], box["y"], box["x"] + box["w"], box["y"] + box["h"])
    rerastered.paste(big.resize((W, H), Image.BOX).crop(tb), tb[:2])
    rerastered.save(HERE / "scrim_b_rerastered.png", optimize=True)
    cases["scrim"]["text_hidden_rerastered"] = "scrim_b_rerastered.png"

    # glow_strong: a heavy glow (blur 16, alpha 255) on the light hero, whose
    # halo reaches ~16 px past the line box. Over a banner-sized image box
    # every tile away from the text lies in it.
    write_case("glow_strong", page_with(light), (255, 255, 255), box, cases, shadow=(16, 255))

    manifest = {
        "generator": "generate.py",
        "page": {"w": W, "h": H, "scale_factor": 1.0},
        "text": TEXT,
        "cases": cases,
    }
    (HERE / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
