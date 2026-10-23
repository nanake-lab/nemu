"""Audit P2-05 products without weakening the global asset-lint result."""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from gltf import read_json


PERMISSIONS = ("avatarPermission", "allowExcessivelyViolentUsage", "allowExcessivelySexualUsage",
               "commercialUsage", "allowPoliticalOrReligiousUsage", "allowAntisocialOrHateUsage",
               "creditNotation", "allowRedistribution", "modification")


def audit(source: Path, products: Path, report: Path) -> None:
    original = read_json(source)["extensions"]["VRMC_vrm"]
    rows: list[dict[str, Any]] = []
    for candidate in sorted(products.iterdir()):
        if not (candidate / "lint.json").is_file():
            continue
        result = json.loads((candidate / "lint.json").read_text(encoding="utf-8"))
        model = read_json(candidate / "nemu.vrm")
        vrm = model["extensions"]["VRMC_vrm"]
        scoped_errors = [d for d in result["diagnostics"] if d["severity"] == "error" and d["rule"].startswith(("A1.4", "A1.5"))]
        permission_diff = {k: [original["meta"].get(k), vrm["meta"].get(k)] for k in PERMISSIONS
                           if original["meta"].get(k) != vrm["meta"].get(k)}
        override_fields = ("overrideMouth", "overrideBlink", "overrideLookAt", "isBinary")
        expression_diff = {name: field for name, expr in original["expressions"]["preset"].items()
                           for field in override_fields if expr.get(field, False if field == "isBinary" else "none") !=
                           vrm["expressions"]["preset"].get(name, {}).get(field, False if field == "isBinary" else "none")}
        row = {"candidate": candidate.name, "scopedErrors": scoped_errors,
               "allErrors": result["errors"], "warnings": result["warnings"],
               "permissionDifferences": permission_diff, "expressionOverrideDifferences": expression_diff,
               "customExpressionNamesUnchanged": sorted(original["expressions"].get("custom", {})) == sorted(vrm["expressions"].get("custom", {})),
               "licenseUrl": vrm["meta"].get("licenseUrl"), "otherLicenseUrl": vrm["meta"].get("otherLicenseUrl"),
               "sha256": hashlib.sha256((candidate / "nemu.vrm").read_bytes()).hexdigest(), "metrics": result["metrics"]}
        rows.append(row)
    evidence = {"sourceSha256": hashlib.sha256(source.read_bytes()).hexdigest(), "products": rows}
    report.write_text(json.dumps(evidence, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    if not rows or any(r["scopedErrors"] or r["permissionDifferences"] or r["expressionOverrideDifferences"] or
                       not r["customExpressionNamesUnchanged"] or r["otherLicenseUrl"] != "LICENSE.md" for r in rows):
        raise ValueError(f"P2-05 audit failed; raw evidence: {report}")
    print(f"{len(rows)} products: A1.4/A1.5 errors zero, permissions and expression definitions preserved")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--products", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    audit(args.source, args.products, args.report)
