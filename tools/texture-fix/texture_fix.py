"""Deterministic, non-destructive RGBA helpers. Every adjustment is explicit."""
from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
from typing import cast

Pixel = tuple[int, int, int, int]


def edge_extend(pixels: list[Pixel], width: int, height: int, iterations: int) -> list[Pixel]:
    """Extend RGB under alpha=0, preserving every alpha and visible pixel."""
    if width * height != len(pixels) or iterations < 0:
        raise ValueError("step=texture-fix object=image expected=valid dimensions/iterations actual=invalid")
    result = pixels.copy()
    known = {i for i, p in enumerate(pixels) if p[3] > 0}
    for _ in range(iterations):
        additions: dict[int, Pixel] = {}
        for y in range(height):
            for x in range(width):
                index = y * width + x
                if index in known:
                    continue
                adjacent = [ny * width + nx for ny, nx in ((y - 1, x), (y, x - 1), (y, x + 1), (y + 1, x)) if 0 <= nx < width and 0 <= ny < height and ny * width + nx in known]
                if adjacent:
                    source = result[min(adjacent)]
                    additions[index] = (*source[:3], pixels[index][3])
        for index, value in additions.items():
            result[index] = value
        known.update(additions)
    return result


def quantize(pixels: list[Pixel], palette: list[list[int]]) -> list[Pixel]:
    if not palette or any(len(c) != 3 or any(not 0 <= v <= 255 for v in c) for c in palette):
        raise ValueError("step=texture-fix object=palette expected=nonempty RGB 0..255 actual=invalid")
    return [cast(Pixel, (*min(palette, key=lambda c: sum((c[k] - p[k]) ** 2 for k in range(3))), p[3])) for p in pixels]


def despeckle(pixels: list[Pixel], width: int, height: int, minimum_neighbors: int) -> list[Pixel]:
    if width * height != len(pixels) or not 1 <= minimum_neighbors <= 8:
        raise ValueError("step=texture-fix object=neighbors expected=1..8 and valid dimensions actual=invalid")
    result = pixels.copy()
    for y in range(height):
        for x in range(width):
            index = y * width + x
            if pixels[index][3] == 0:
                continue
            neighbors = [pixels[ny * width + nx][:3] for ny in range(max(0, y - 1), min(height, y + 2)) for nx in range(max(0, x - 1), min(width, x + 2)) if (nx, ny) != (x, y) and pixels[ny * width + nx][3] > 0]
            if neighbors:
                color, count = Counter(neighbors).most_common(1)[0]
                if count >= minimum_neighbors and pixels[index][:3] not in neighbors:
                    result[index] = (*color, pixels[index][3])
    return result


def main() -> None:
    from PIL import Image

    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--config", type=Path, required=True)
    args = parser.parse_args()
    if args.source.resolve() == args.output.resolve() or args.output.exists():
        raise ValueError("step=texture-fix object=output expected=new output file actual=existing/source")
    root = Path(__file__).resolve().parents[2]
    for protected in (root / "content", root / "content-template"):
        if args.output.resolve().is_relative_to(protected.resolve()):
            raise ValueError("step=texture-fix object=output expected=outside protected content actual=protected")
    config = json.loads(args.config.read_text(encoding="utf-8"))
    image = Image.open(args.source).convert("RGBA")
    pixels = cast(list[Pixel], list(image.get_flattened_data()))
    for operation in config["operations"]:
        match operation["kind"]:
            case "edgeExtend":
                pixels = edge_extend(pixels, image.width, image.height, operation["iterations"])
            case "palette":
                pixels = quantize(pixels, operation["colors"])
            case "despeckle":
                pixels = despeckle(pixels, image.width, image.height, operation["minimumNeighbors"])
            case other:
                raise ValueError(f"step=texture-fix object=operation expected=known kind actual={other}")
    image.putdata(pixels)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    image.save(args.output, format="PNG")
    print(f"step=texture-fix output={args.output} status=ok")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        raise ValueError(f"step=texture-fix object=CLI/config expected=valid explicit operation and image actual={type(error).__name__}: {error}") from error
