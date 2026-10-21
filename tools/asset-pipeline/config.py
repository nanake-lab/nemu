"""Configuration and geometry calculations; deliberately independent of bpy."""
from __future__ import annotations

import json
import math
import subprocess
from pathlib import Path
from typing import Any, cast

import jsonschema
import yaml  # type: ignore[import-untyped]  # PyYAML has no bundled stubs; boundary validated below.

from errors import fail

Json = dict[str, Any]
Vec3 = tuple[float, float, float]


class UniqueLoader(yaml.SafeLoader):  # type: ignore[misc]  # External dynamic YAML loader API.
    pass


def unique_mapping(loader: UniqueLoader, node: Any) -> Json:
    result: Json = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node)
        if not isinstance(key, str) or key in result:
            fail("config", str(key), "unique string key", "duplicate or non-string")
        result[key] = loader.construct_object(value_node)
    return result


UniqueLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, unique_mapping)


def finite_tree(value: Any, field: str = "build") -> None:
    if isinstance(value, float) and not math.isfinite(value):
        fail("config", field, "finite number", value)
    if isinstance(value, dict):
        for key, child in value.items():
            finite_tree(child, f"{field}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            finite_tree(child, f"{field}[{index}]")


def load_config(path: Path) -> Json:
    try:
        data = yaml.load(path.read_text(encoding="utf-8-sig"), Loader=UniqueLoader)
    except (OSError, yaml.YAMLError) as error:
        fail("config", str(path), "readable build.yaml", str(error))
    finite_tree(data)
    schema = json.loads(Path(__file__).with_name("build.schema.json").read_text(encoding="utf-8"))
    errors = sorted(jsonschema.Draft202012Validator(schema).iter_errors(data), key=lambda e: str(e.path))
    if errors:
        fail("config", ".".join(map(str, errors[0].absolute_path)) or "build", "build.schema.json", errors[0].message)
    checked = subprocess.run(["node", str(Path(__file__).with_name("validate-profile.mjs"))], input=json.dumps(data["profile"]), capture_output=True, text=True, check=False)
    if checked.returncode:
        fail("config", "profile", "@nemu/contracts NemuProfile v2 configuration", checked.stderr or checked.stdout)
    return cast(Json, data)


def writable_path(path: Path, root: Path) -> Path:
    resolved = path.resolve()
    for protected in (root / "content", root / "content-template"):
        if resolved.is_relative_to(protected.resolve()):
            fail("config", str(resolved), "output outside read-only content and content-template", "protected path")
    return resolved
