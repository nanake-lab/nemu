"""Materialize review-only YAML files from explicit candidate design parameters."""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import yaml  # type: ignore[import-untyped]
from gltf import read_json

Json = dict[str, Any]


def prepare(source: Path, bible: Path, designs: Path, output: Path) -> None:
    model = read_json(source)
    palette = json.loads((bible / "03-palette.json").read_text(encoding="utf-8"))
    proposals = json.loads(designs.read_text(encoding="utf-8"))
    nodes = model["nodes"]
    output.mkdir(parents=True, exist_ok=True)
    for preset_name, preset in proposals["springPresets"].items():
        for tail_name, tail in proposals["tails"].items():
            cfg = copy.deepcopy(proposals["temporaryBuild"])
            cfg["source"] = proposals["sourceReference"]
            cfg["meta"]["authors"] = model["extensions"]["VRMC_vrm"]["meta"]["authors"]
            cfg["meta"]["version"] = f"P2-05-candidate-{tail_name}-{preset_name}"
            cfg["tail"] = {**copy.deepcopy(proposals["tailCommon"]), **tail}
            rgb = palette["colors"]["tailBase"].lstrip("#")
            # Palette values are sRGB; MToon baseColorFactor is linear.
            channels = [int(rgb[i:i + 2], 16) / 255 for i in (0, 2, 4)]
            cfg["tail"]["color"] = [c / 12.92 if c <= .04045 else ((c + .055) / 1.055) ** 2.4 for c in channels] + [1]
            def linear_hex(value: str) -> list[float]:
                rgb_hex = value.lstrip("#")
                srgb = [int(rgb_hex[i:i + 2], 16) / 255 for i in (0, 2, 4)]
                return [c / 12.92 if c <= .04045 else ((c + .055) / 1.055) ** 2.4 for c in srgb]

            cfg["materials"][cfg["tail"]["materialName"]] = {
                "baseColor": cfg["tail"]["color"],
                "mtoon": {**proposals["tailMaterialResponse"], "shade_color_factor": linear_hex(palette["colors"]["hairShadow"]),
                          "outline_color_factor": linear_hex(palette["colors"]["outline"])}}
            chains = []
            for index, spring in enumerate(model["extensions"]["VRMC_springBone"]["springs"]):
                bones = [nodes[j["node"]]["name"] for j in spring["joints"]]
                if any(b.startswith("J_Sec_Hair1_23") or b.startswith("J_Sec_Hair1_24") for b in bones):
                    category = "ear"
                elif any("Skirt" in b for b in bones):
                    category = "skirt"
                elif any("Hair" in b for b in bones):
                    category = "hair"
                else:
                    # Preserve unrelated source physics; terminal joints have no simulated segment.
                    first = spring["joints"][0]
                    keys = {"hitRadiusM": ("hitRadius", 0), "stiffness": ("stiffness", 1),
                            "gravityPower": ("gravityPower", 0), "dragForce": ("dragForce", .5),
                            "gravityDir": ("gravityDir", [0, -1, 0])}
                    values = {key: first.get(source_key, default) for key, (source_key, default) in keys.items()}
                    # Source vectors are glTF; spring configuration uses Blender coordinates.
                    x, y, z = values["gravityDir"]
                    values["gravityDir"] = [x, -z, y]
                    chains.append({"name": spring.get("name", f"source-{index}"), "bones": bones,
                                   "centerBone": nodes[spring["center"]]["name"], **values})
                    continue
                chains.append({"name": f"P205-{category}-{index:02d}", "bones": bones,
                               "centerBone": "Root", **preset[category]})
            chains.append({"name": "P205-tail", "bones": ["$tail"], "centerBone": "Root", **preset["tail"]})
            cfg["springs"]["chains"] = chains
            target = output / f"build-{tail_name}-{preset_name}.yaml"
            target.write_text("# CANDIDATE ONLY. Not approved; do not promote to content/nemu/build.yaml.\n"
                              "# expressions/materials/profile are TEMPORARY P2-06 placeholders; expression/material edits are empty.\n"
                              "# Geometry uses Blender coordinates: +Z up, +Y toward the avatar back.\n"
                              + yaml.safe_dump(cfg, sort_keys=False, allow_unicode=True), encoding="utf-8")
    metadata = {"status": "candidate-not-approved", "source": str(source.resolve()),
                "sourceSha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "bibleSha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in
                                [bible / "04-proportions.md", bible / "03-palette.json"]},
                "paletteStatus": palette.get("status"), "candidateParameters": proposals,
                "earMapping": {"left": ["J_Sec_Hair1_24", "J_Sec_Hair2_24"],
                               "right": ["J_Sec_Hair1_23", "J_Sec_Hair2_23"]}}
    (output / "candidates.json").write_text(json.dumps(metadata, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (output / "spring-presets.json").write_text(json.dumps(proposals["springPresets"], indent=2) + "\n", encoding="utf-8")
    manifest = [{"path": p.name, "kind": "other", "source": "P2-05 candidate generator; Nemu Bible and read-only VRoid export",
                 "license": "custom:Nemu private candidate; terms pending P7", "redistributable": False}
                for p in sorted(output.iterdir()) if p.is_file() and p.name != "manifest.json"]
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--bible", type=Path, required=True)
    parser.add_argument("--designs", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    prepare(args.source, args.bible, args.designs, args.output)
