"""Read-only GLB measurements and exact canonical JSON comparison."""
from __future__ import annotations

import json
import struct
from pathlib import Path
from typing import Any, cast

from errors import fail

Json = dict[str, Any]


def read_json(path: Path) -> Json:
    blob = path.read_bytes()
    magic, version, size = struct.unpack_from("<III", blob)
    length, kind = struct.unpack_from("<II", blob, 12)
    if magic != 0x46546C67 or version != 2 or size != len(blob) or kind != 0x4E4F534A:
        fail("profile", str(path), "valid GLB 2.0 JSON chunk", "invalid header")
    return cast(Json, json.loads(blob[20:20 + length]))


def measurements(data: Json) -> tuple[float, float]:
    """Same POSITION accessor bounds contract as asset-lint, including node TRS."""
    nodes = data["nodes"]
    parents = {child: i for i, node in enumerate(nodes) for child in node.get("children", [])}

    def transform(index: int, point: list[float]) -> list[float]:
        node = nodes[index]
        if "matrix" in node:
            m = node["matrix"]
            point = [sum(m[c * 4 + r] * point[c] for c in range(3)) + m[12 + r] for r in range(3)]
        else:
            scaled = [x * s for x, s in zip(point, node.get("scale", [1, 1, 1]))]
            x, y, z, w = node.get("rotation", [0, 0, 0, 1])
            q = [x, y, z]
            dot = sum(a * b for a, b in zip(q, scaled))
            cross = [y * scaled[2] - z * scaled[1], z * scaled[0] - x * scaled[2], x * scaled[1] - y * scaled[0]]
            point = [2 * dot * q[k] + (w * w - sum(v * v for v in q)) * scaled[k] + 2 * w * cross[k] + node.get("translation", [0, 0, 0])[k] for k in range(3)]
        return transform(parents[index], point) if index in parents else point

    ys: list[float] = []
    for index, node in enumerate(nodes):
        if "mesh" not in node:
            continue
        for primitive in data["meshes"][node["mesh"]]["primitives"]:
            accessor = data["accessors"][primitive["attributes"]["POSITION"]]
            for x in (accessor["min"][0], accessor["max"][0]):
                for y in (accessor["min"][1], accessor["max"][1]):
                    for z in (accessor["min"][2], accessor["max"][2]):
                        ys.append(transform(index, [x, y, z])[1])
    if not ys:
        fail("profile", "meshes", "POSITION bounds", "none")
    head = data["extensions"]["VRMC_vrm"]["humanoid"]["humanBones"]["head"]["node"]
    return max(ys) - min(ys), max(ys) - transform(head, [0, 0, 0])[1]
