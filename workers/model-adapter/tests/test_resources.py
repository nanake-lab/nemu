"""P5-42 structural vectors; no qualification or GPU inference is asserted here."""
from __future__ import annotations

from copy import deepcopy
import importlib
from pathlib import Path
import sys
from typing import Any

import pytest
from echo.bridge import from_json
from echo.runtime import Limits, encode_canonical
from echo.typed_api import from_value
from echo.types import JHookBinding, Lineage, ObjectRef
from nemu_model_adapter.resources import Publication, Unmodulated, canonical_ref, receive

LIMITS = Limits(1048576, 4096, 32)


def fixture() -> dict[str, Any]:
    sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "core/contracts"))
    try:
        return dict(importlib.import_module("verify_j").resource_bundle(0))
    finally:
        sys.path.remove(str(Path(__file__).resolve().parents[3] / "core/contracts"))


def publication(b: dict[str, Any]) -> tuple[
    bytes, bytes, Publication, JHookBinding, dict[ObjectRef, bytes],
]:
    objects: dict[ObjectRef, bytes] = {}

    def store(name: str, value: dict[str, Any]) -> ObjectRef:
        native = from_json(name, value)
        ref = canonical_ref(name, native, LIMITS)
        objects[ref] = encode_canonical(name, native, LIMITS)
        return ref

    for v in b["vectors"]:
        store("JDirectionVector", v)
    s = store("JDirectionSet", b["direction_set"])
    vref = store("JDirectionValidation", b["validation"])
    store("BodyReadoutParameters", b["body"])
    g = store("GenesisManifestV3", b["genesis"])
    r = b["receipt"]
    rr = store("JEffectReceipt", r)
    e = {k: r[k] for k in ("schema_version", "snapshot_id", "projected_at_ns", "x_digest",
                             "r_digest", "theta_digest", "carrier", "model", "tokenizer",
                             "direction_set", "total_bound")}
    e["effect_receipt_digest"] = rr.digest.hex()
    e["nature_coefficients"] = dict(schema_version=1, **{
        k: row["coefficient"] for k, row in zip(
            ("connection", "empathy", "curiosity", "achievement"), r["directions"][:4], strict=True)})
    e["learned_coefficients"] = [dict(schema_version=1, tendency_id=row["tendency_id"],
                                      coefficient=row["coefficient"])
                                 for row in r["directions"][4:] if row["status"] == 1]
    er = store("ModulationEnvelopeV2", e)
    h = from_value("JHookBinding", from_json("JHookBinding", b["direction_set"]["hook"]), LIMITS)
    return objects[er], objects[rr], Publication(er, g, s, vref, Lineage.EXPERIMENT), h, objects


def test_j05_excluded_and_removed_columns() -> None:
    b = fixture()
    e, r, p, h, obj = publication(b)
    first = receive(e, r, p, h, obj.__getitem__, b"prefix", b"numeric", LIMITS)
    assert first.delta == (0.0625, -0.0625, 0.0625, -0.0625, 0.0625)
    # Shrink R+, then change its direction resource. No rescaling of retained columns.
    b["receipt"]["directions"] = b["receipt"]["directions"][:4]
    b["receipt"]["total_bound"] = 0.5
    for value in (b["vectors"], b["validation"]["directions"], b["direction_set"]["directions"]):
        value.pop()
    b["direction_set"]["version"] = "new"
    importlib.import_module("verify_j").rebind(b)
    e, r, p, h, obj = publication(b)
    changed = receive(e, r, p, h, obj.__getitem__, b"prefix", b"numeric", LIMITS)
    assert changed.key != first.key
    assert changed.delta == (*first.delta[:4], 0.0)
    assert changed.key != receive(e, r, p, h, obj.__getitem__, b"prefix2", b"numeric", LIMITS).key
    assert changed.key != receive(e, r, p, h, obj.__getitem__, b"prefix", b"numeric2", LIMITS).key


@pytest.mark.parametrize("fault", ["missing", "digest", "validation", "budget", "unverified",
                                   "envelope", "model", "dtype", "zero-policy", "formal"])
def test_unmodulated_without_fallback(fault: str) -> None:
    from dataclasses import replace

    b = deepcopy(fixture())
    if fault == "budget":
        b["receipt"]["directions"][0]["coefficient"] = 2
    if fault == "unverified":
        b["receipt"]["directions"][4]["validation"] = b["receipt"]["genesis"]
    e, r, p, h, obj = publication(b)
    if fault == "missing":
        del obj[p.direction_set]
    if fault == "digest":
        obj[p.validation] = obj[p.genesis]
    if fault == "validation":
        p = replace(p, validation=p.direction_set)
    if fault == "envelope":
        e = e[:-1] + bytes([e[-1] ^ 1])
    if fault == "model":
        h = replace(h, model=h.tokenizer)
    if fault == "dtype":
        h = replace(h, width=h.width + 1)
    if fault in ("zero-policy", "formal"):
        p = replace(p, lineage=Lineage.REHEARSAL if fault == "zero-policy" else Lineage.LIFE)
    with pytest.raises(Unmodulated):
        receive(e, r, p, h, obj.__getitem__, b"p", b"n", LIMITS)
