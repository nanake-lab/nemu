from __future__ import annotations

import copy
import json
from pathlib import Path
import struct

import pytest

from config import load_config, writable_path
from errors import BuildError
from geometry import chain_points, radius_at, weights_at
from gltf import measurements, read_json
from texture_fix import despeckle, edge_extend, quantize

BASE = Path(__file__).resolve().parents[1]


def test_example_config_uses_contract_profile() -> None:
    cfg = load_config(BASE / "build.example.yaml")
    assert cfg["tail"]["boneCount"] >= 8
    assert len(cfg["profile"]["emotionMap"]) == 12


@pytest.mark.parametrize("field", ["tail", "springs", "textures", "profile"])
def test_missing_section_fails_with_field(tmp_path: Path, field: str) -> None:
    cfg = load_config(BASE / "build.example.yaml")
    del cfg[field]
    path = tmp_path / "missing.yaml"
    path.write_text(json.dumps(cfg), encoding="utf-8")
    with pytest.raises(BuildError, match=field):
        load_config(path)


def test_missing_aesthetic_field(tmp_path: Path) -> None:
    cfg = load_config(BASE / "build.example.yaml")
    del cfg["tail"]["color"]
    path = tmp_path / "missing-color.yaml"
    path.write_text(json.dumps(cfg), encoding="utf-8")
    with pytest.raises(BuildError, match="color"):
        load_config(path)


def test_duplicate_keys_and_nan(tmp_path: Path) -> None:
    path = tmp_path / "bad.yaml"
    path.write_text("tail: 1\ntail: 2\n", encoding="utf-8")
    with pytest.raises(BuildError, match="duplicate"):
        load_config(path)
    path.write_text("tail: .nan", encoding="utf-8")
    with pytest.raises(BuildError, match="finite"):
        load_config(path)


def test_contract_rejects_missing_language(tmp_path: Path) -> None:
    cfg = load_config(BASE / "build.example.yaml")
    del cfg["profile"]["lipSyncProfiles"]["ja"]
    path = tmp_path / "missing-language.yaml"
    path.write_text(json.dumps(cfg), encoding="utf-8")
    with pytest.raises(BuildError, match="ja"):
        load_config(path)


def test_bone_chain_arc_and_no_input_mutation() -> None:
    curve = [[0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [1.0, 2.0, 0.0]]
    before = copy.deepcopy(curve)
    points = chain_points(curve, 3.0, 12)
    assert len(points) == 13
    assert points[-1] == pytest.approx((1, 2, 0))
    assert curve == before
    assert points[8] == pytest.approx((0, 2, 0))


def test_bad_chain_and_radius() -> None:
    with pytest.raises(BuildError, match="nonzero"):
        chain_points([[0, 0, 0], [0, 0, 0]], 1, 8)
    with pytest.raises(BuildError, match=">=8"):
        chain_points([[0, 0, 0], [0, 1, 0]], 1, 7)
    assert radius_at([[0, 0.1], [0.5, 0.2], [1, 0.0]], 0.75) == pytest.approx(0.1)
    with pytest.raises(BuildError, match="increasing"):
        radius_at([[0, 0.1], [0, 0.2]], 0)


@pytest.mark.parametrize("t", [0, 0.01, 0.125, 0.3, 0.5, 0.99, 1])
def test_skin_weights_normalized(t: float) -> None:
    weights = weights_at(t, 8)
    assert sum(w for _, w in weights) == pytest.approx(1)
    assert all(0 <= i < 8 and 0 <= w <= 1 for i, w in weights)
    assert len(weights) <= 2


def test_protected_outputs_and_junction_resolution(tmp_path: Path) -> None:
    for directory in ("content", "content-template"):
        with pytest.raises(BuildError, match="protected"):
            writable_path(tmp_path / directory / "nemu.vrm", tmp_path)
    assert writable_path(tmp_path / "output/model.vrm", tmp_path).name == "model.vrm"


def test_measurement_world_transform_and_glb(tmp_path: Path) -> None:
    data = {"nodes": [{"translation": [0, 1, 0], "scale": [1, 2, 1], "children": [1, 2]}, {"mesh": 0}, {"translation": [0, 0.75, 0]}], "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}], "accessors": [{"min": [0, 0, 0], "max": [1, 1, 1]}], "extensions": {"VRMC_vrm": {"humanoid": {"humanBones": {"head": {"node": 2}}}}}}
    assert measurements(data) == pytest.approx((2, 0.5))
    blob = json.dumps(data).encode()
    blob += b" " * (-len(blob) % 4)
    path = tmp_path / "fixture.vrm"
    path.write_bytes(struct.pack("<IIIII", 0x46546C67, 2, 20 + len(blob), len(blob), 0x4E4F534A) + blob)
    assert read_json(path) == data


def test_texture_rgb_extension_preserves_alpha_and_visible_pixels() -> None:
    pixels = [(200, 10, 20, 128), (0, 0, 0, 0), (0, 0, 0, 0)]
    assert edge_extend(pixels, 3, 1, 2) == [(200, 10, 20, 128), (200, 10, 20, 0), (200, 10, 20, 0)]
    assert pixels[1] == (0, 0, 0, 0)


def test_palette_and_despeckle_preserve_alpha() -> None:
    assert quantize([(100, 0, 0, 42)], [[0, 0, 0], [255, 0, 0]]) == [(0, 0, 0, 42)]
    pixels = [(255, 0, 0, 255)] * 9
    pixels[4] = (0, 0, 255, 128)
    assert despeckle(pixels, 3, 3, 6)[4] == (255, 0, 0, 128)
    assert pixels[4] == (0, 0, 255, 128)
