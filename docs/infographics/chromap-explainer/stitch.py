#!/usr/bin/env python3
"""Stitch shoot.js SLICES (from y=0, full-width, fixed height) into the final
long-form PNGs, with the mandatory height assertion.

    node shoot.js "file://$PWD/index.html" render
    python3 stitch.py
      -> render/full@2x.png          (bitmap height MUST equal cssHeight × dpr)
      -> render/full-gray.png        (grayscale readability check)
      -> render/thumb.png            (1/4 scale hierarchy check)
      -> chromap-explainer@2x.png / -thumb.png (deliverable copies)

Per-section stitching is forbidden: it drops inter-section margins and yields
a bitmap shorter than the page (documented failure mode, once measured at
504 CSS px lost). Slices in file order are the single source of truth.
"""
from __future__ import annotations

import json
import shutil
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
RENDER = HERE / "render"
PAPER = "#F7F4EE"


def main() -> None:
    page = json.loads((RENDER / "page.json").read_text())
    slices = [Path(l) for l in
              (RENDER / "slices.txt").read_text().splitlines() if l.strip()]
    assert slices, "no slices found"

    imgs = [Image.open(p) for p in slices]
    width = imgs[0].width
    assert all(i.width == width for i in imgs), "ragged slice widths"

    canvas = Image.new("RGB", (width, sum(i.height for i in imgs)), PAPER)
    y = 0
    for im in imgs:
        canvas.paste(im, (0, y))
        y += im.height

    expected = page["cssHeight"] * page["dpr"]
    assert canvas.height == expected, (
        f"bitmap height {canvas.height} != page CSS height {page['cssHeight']} "
        f"x dpr {page['dpr']} = {expected}")
    assert canvas.width == page["cssWidth"] * page["dpr"], (
        f"bitmap width {canvas.width} != {page['cssWidth']} x {page['dpr']}")

    canvas.save(RENDER / "full@2x.png")
    canvas.convert("L").save(RENDER / "full-gray.png")
    canvas.resize((width // 4, canvas.height // 4), Image.LANCZOS).save(
        RENDER / "thumb.png")
    shutil.copy(RENDER / "full@2x.png", HERE / "chromap-explainer@2x.png")
    shutil.copy(RENDER / "thumb.png", HERE / "chromap-explainer-thumb.png")
    print(f"full@2x {canvas.size} (asserted = {page['cssHeight']} CSS px x "
          f"{page['dpr']}) -> thumb {width // 4}x{canvas.height // 4}")


if __name__ == "__main__":
    main()
