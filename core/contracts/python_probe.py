"""Test-only subprocess adapter. All runtime data is supplied on stdin."""

from __future__ import annotations
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "workers/irl/generated"))
from echo import runtime as r  # noqa: E402
from echo.bridge import from_json, to_json  # noqa: E402
from echo.typed_api import from_value, to_value  # noqa: E402
from echo.theta import check_theta_message  # noqa: E402

for line in sys.stdin:
    try:
        q = json.loads(line)
        limits = r.Limits(**q["limits"])
        if q["op"] == "ingress":
            import ingress_probe
            ingress_probe.check(q, limits)
            print(json.dumps(dict(ok=True)))
            continue
        name = q["name"]
        v = (
            r.normalize(name, from_json(name, q["value"]), limits)
            if q["op"] == "encode"
            else (
                r.decode_canonical(name, bytes.fromhex(q["hex"]), limits)
                if q["op"] == "canonical"
                else r.decode_protobuf(name, bytes.fromhex(q["hex"]), limits)
            )
        )
        typed = from_value(name, v, limits)
        check_theta_message(name, v, limits)
        assert r.normalize(name, to_value(typed), limits) == v
        print(
            json.dumps(
                dict(
                    ok=True,
                    value=to_json(name, v),
                    canonical=r.encode_canonical(name, v, limits).hex(),
                    protobuf=r.encode_protobuf(name, v, limits).hex(),
                    digest=r.digest(name, v, limits),
                )
            )
        )
    except (ValueError, TypeError, KeyError, OverflowError) as error:
        print(json.dumps(dict(ok=False, error=str(error))))
