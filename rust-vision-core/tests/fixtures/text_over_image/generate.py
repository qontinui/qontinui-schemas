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

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

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

    # Round 4 (review of f4c2e2df).

    # clipped: a white heading in a FULL-WIDTH block box on a 960 px hero,
    # with a clipped 255 highlight under the last ~12% of its ink. The ink
    # there is invisible and no difference shows it; averaged over the blank
    # run of the box its estimate was diluted into a pass. Must NOT pass.
    wide_w = 960
    rng = random.Random(22)
    wide_hero = Image.new("RGB", (wide_w, HERO[3]))
    wpx = wide_hero.load()
    for y in range(HERO[3]):
        for x in range(wide_w):
            v = 55.0 + 22.0 * (math.sin(x / 23) + math.cos(y / 17) + math.sin((x + y) / 31)) / 3
            n = rng.uniform(-6, 6)
            wpx[x, y] = tuple(max(0, min(255, round(v + n + t))) for t in (-8.0, 0.0, 10.0))
    l, t, r, b = ImageDraw.Draw(Image.new("RGB", (1, 1))).textbbox(TEXT_AT, TEXT, font=FONT)
    x0 = int(r - (r - l) * 0.12) - 2
    ImageDraw.Draw(wide_hero).rectangle(
        [x0, t - 4 - HERO[1], r + 3, b + 4 - HERO[1]], fill=(255, 255, 255)
    )
    wide_page = Image.new("RGB", (wide_w, H), PAGE)
    wide_page.paste(wide_hero, (HERO[0], HERO[1]))
    full = {"x": 0, "y": box["y"], "w": wide_w, "h": box["h"]}
    write_case("clipped", wide_page, (255, 255, 255), full, cases,
               image=(0, HERO[1], wide_w, HERO[3]))

    # parallax: white text over a finely textured mid-dark hero; its
    # text_hidden_parallax twin has the left 45% of the page moved 2 px to the
    # right between the captures, under the text. Must be Unknown with that
    # twin.
    rng = random.Random(23)
    tex = Image.new("RGB", (HERO[2], HERO[3]))
    tpx = tex.load()
    for y in range(HERO[3]):
        for x in range(HERO[2]):
            v = 105 + 40 * (math.sin(x / 9) + math.cos(y / 7) + math.sin((x + y) / 13)) / 3
            v += rng.uniform(-6, 6)
            tpx[x, y] = (max(0, min(255, round(v))),) * 3
    tex_page = page_with(tex)
    write_case("parallax", tex_page, (255, 255, 255), box, cases)
    strip = (0, 0, int(W * 0.45), H)
    moved = tex_page.copy()
    moved.paste(ImageChops.offset(tex_page, 2, 0).crop(strip), strip[:2])
    moved.save(HERE / "parallax_b_moved.png", optimize=True)
    cases["parallax"]["text_hidden_parallax"] = "parallax_b_moved.png"

    # mottled: white text over a broad band of glyph-coloured speckle (a
    # 253 / 225 mottle) in the IMAGE, reaching well past the text box. Frame
    # B's 253 pixels beside a glyph are the image's own: not a still-painted
    # glyph.
    mot = photo(random.Random(24), base=55.0, amp=22.0)
    mpx = mot.load()
    rng = random.Random(25)
    for y in range(max(0, box["y"] - HERO[1] - 50), min(HERO[3], box["y"] - HERO[1] + box["h"] + 50)):
        for x in range(0, min(HERO[2], box["x"] + box["w"] + 60)):
            mpx[x, y] = (253, 253, 253) if rng.random() < 0.5 else (225, 225, 225)
    write_case("mottled", page_with(mot), (255, 255, 255), box, cases)

    # Round 5 (review of 4a90023c).

    # parallax at 4 and 6 px: beyond the local check's 3 px shift search.
    for dx in (4, 6):
        moved = tex_page.copy()
        moved.paste(ImageChops.offset(tex_page, dx, 0).crop(strip), strip[:2])
        moved.save(HERE / f"parallax_b_moved{dx}.png", optimize=True)
        cases["parallax"][f"text_hidden_parallax{dx}"] = f"parallax_b_moved{dx}.png"

    # tagline: the parallax hero (registered) with an UNTRACKED 14 px tagline
    # just under the heading, inside its local ring, in frame A only.
    def tagline(page: Image.Image) -> None:
        f = ImageFont.load_default(size=14)
        ImageDraw.Draw(page).text((24, 104), "Limited stock - order now", font=f, fill=(255, 255, 255))

    write_case("tagline", tex_page, (255, 255, 255), box, cases, extra=tagline)

    # card: a plain dark card, white text, and a re-rasterised UNHIDDEN twin.
    card = page_with(Image.new("RGB", (HERO[2], HERO[3]), (32, 36, 44)))
    write_case("card", card, (255, 255, 255), box, cases)
    card_big = card.resize((W * 4, H * 4), Image.NEAREST)
    ImageDraw.Draw(card_big).text(
        (TEXT_AT[0] * 4 + 1, TEXT_AT[1] * 4), TEXT, font=ImageFont.load_default(size=112),
        fill=(255, 255, 255),
    )
    card_rr = card.copy()
    card_rr.paste(card_big.resize((W, H), Image.BOX).crop(tb), tb[:2])
    card_rr.save(HERE / "card_b_rerastered.png", optimize=True)
    cases["card"]["text_hidden_rerastered"] = "card_b_rerastered.png"

    # heavy: a long regular line, then a HEAVY word ("M", stroke 5) wholly on
    # a clipped 255 patch, in a full-width box on a 1280 px hero. The hidden
    # word is denser than any visible band: at the regular line's densest band
    # density its ink reads as 4% of the samples (a pass at p5); counted in
    # full, 8%. Must NOT pass.
    heavy_w = 1280
    hv_b = Image.new("RGB", (heavy_w, H), PAGE)
    hv_hero = Image.new("RGB", (heavy_w, HERO[3]))
    rng = random.Random(26)
    hpx = hv_hero.load()
    for y in range(HERO[3]):
        for x in range(heavy_w):
            v = 55.0 + 22.0 * (math.sin(x / 23) + math.cos(y / 17) + math.sin((x + y) / 31)) / 3
            n = rng.uniform(-6, 6)
            hpx[x, y] = tuple(max(0, min(255, round(v + n + t))) for t in (-8.0, 0.0, 10.0))
    hv_b.paste(hv_hero, (HERO[0], HERO[1]))
    d0 = ImageDraw.Draw(Image.new("RGB", (1, 1)))
    prefix, word, sw = "free delivery on every order placed this week only, while stocks last ", "M", 5
    pl, pt, pr, pb = d0.textbbox(TEXT_AT, prefix, font=FONT)
    word_at = (pr + 4, TEXT_AT[1])
    wl, wt, wr, wb = d0.textbbox(word_at, word, font=FONT, stroke_width=sw)
    ImageDraw.Draw(hv_b).rectangle([wl - 1, wt - 2, wr + 1, wb + 2], fill=(255, 255, 255))
    hv_a = hv_b.copy()
    da = ImageDraw.Draw(hv_a)
    da.text(TEXT_AT, prefix, font=FONT, fill=(255, 255, 255))
    da.text(word_at, word, font=FONT, fill=(255, 255, 255), stroke_width=sw,
            stroke_fill=(255, 255, 255))
    hv_a.save(HERE / "heavy_a.png", optimize=True)
    hv_b.save(HERE / "heavy_b.png", optimize=True)
    cases["heavy"] = {
        "frame": "heavy_a.png",
        "text_hidden": "heavy_b.png",
        "image": {"x": 0, "y": HERO[1], "w": heavy_w, "h": HERO[3]},
        "text": {"x": 0, "y": min(pt, wt) - 6, "w": heavy_w, "h": max(pb, wb) - min(pt, wt) + 12},
        "fg": [255, 255, 255],
    }

    manifest = {
        "generator": "generate.py",
        "page": {"w": W, "h": H, "scale_factor": 1.0},
        "text": TEXT,
        "cases": cases,
    }
    (HERE / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
