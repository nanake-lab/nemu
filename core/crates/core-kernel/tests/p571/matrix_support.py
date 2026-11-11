"""Protocol and evidence comparison, separate from all expected-value flows."""

import json
import hashlib
import time
from pathlib import Path
from typing import Any, Iterator

from arithmetic import Q, bits
from c_oracle import bit_tree, first_difference
from c_research import CResearch
from cf_research import CFResearch
from f_research import FResearch
from g_research import GResearch
from probe import Bridge

Model = CResearch | FResearch | CFResearch | GResearch


class MatrixBridge(Bridge):
    def trajectory(self, request: dict[str, Any]) -> Iterator[dict[str, Any]]:
        self.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
        self.stdin.flush()
        while True:
            line = self.stdout.readline()
            if not line:
                raise RuntimeError(f"bridge exited {self.p.poll()}")
            if line.startswith("P571-PREFIX:"):
                yield json.loads(line[len("P571-PREFIX:") :])
            elif line.startswith("P571:"):
                yield dict(completion=json.loads(line[5:]))
                return
            else:
                self.log.write(line)
                self.log.flush()


class Budget:
    def __init__(self, seconds: int, stop: Path) -> None:
        self.deadline = time.monotonic() + seconds
        self.stop = stop

    def check(self) -> None:
        if self.stop.exists():
            raise InterruptedError("explicit STOP file")
        if time.monotonic() >= self.deadline:
            raise InterruptedError("16 hour deadline")

    def remaining_ms(self) -> int:
        self.check()
        return max(1, int((self.deadline - time.monotonic()) * 1000))


def model_for(
    resources: dict[str, Any],
    case: dict[str, Any],
    family: str,
    bridge: Bridge,
    binding: dict[str, Any],
) -> Model:
    if family == "C":
        return CResearch(resources[case["resource"]], case, bridge.primitive)
    if family == "F":
        return FResearch(resources[case["resource"]], case, bridge.primitive)
    if family == "CF":
        return CFResearch(
            resources[case["c_resource"]],
            resources[case["f_resource"]],
            case,
            bridge.primitive,
        )
    if family == "G":
        return GResearch(resources[case["resource"]], case, binding)
    raise ValueError("no model for family")


def advance(model: Model, event: dict[str, Any], index: int, measure: bool) -> None:
    if isinstance(model, GResearch):
        model.step(event, index)
    elif isinstance(model, (CResearch, CFResearch)):
        model.step(event, measure)
    else:
        model.step(event)


def fast_view(snapshot: dict[str, Any]) -> dict[str, Any]:
    names = ("connection", "empathy", "curiosity", "achievement")

    def vector(v: dict[str, Any]) -> list[Any]:
        return [v["nature"][n] for n in names] + [c["value"] for c in v["learned"]]

    x = [snapshot["nature"][n] for n in names] + [
        c["value"] for c in snapshot["learned"]
    ]
    h, targets = {}, {}
    for slot in snapshot.get("slots", []):
        root = slot["basis"].get("record_id", "internal")
        h[root], targets[root] = vector(slot["current"]), vector(slot["target"])
    return dict(x=x, h=h, targets=targets, anchor=int(snapshot["anchor_time_ns"]))


def compare(
    model: Model, dut: dict[str, Any], family: str
) -> tuple[str, Any, Any] | None:
    actual = dut["snapshot"]
    if family == "C":
        assert isinstance(model, CResearch)
        expected = model.state()
        observed = {k: actual[k] for k in expected}
    elif family == "F":
        assert isinstance(model, FResearch)
        expected = (
            {k: model.state()[k] for k in ("x", "anchor") if model.base == "old"}
            if model.base == "old"
            else {k: model.state()[k] for k in ("x", "h", "targets", "anchor")}
        )
        observed = {k: fast_view(actual)[k] for k in expected}
    elif family == "CF":
        assert isinstance(model, CFResearch)
        d = compare(model.c, dict(snapshot=actual["c"]), "C")
        return d or compare(model.f, dict(snapshot=actual["f"]), "F")
    else:
        assert isinstance(model, GResearch)
        expected, observed = model.state(), actual
    return first_difference(bit_tree(expected), bit_tree(observed), "DUT-oracle")


def audit_difference(
    model: Model, dut: dict[str, Any], prefix: int
) -> tuple[str, Any, Any] | None:
    if prefix == 0:
        return None
    audit = dut["audit"]
    if isinstance(model, CResearch):
        return first_difference(bit_tree(model.last), bit_tree(audit), "C-audit")
    if isinstance(model, GResearch):
        return first_difference(bit_tree(model.last), bit_tree(audit), "G-audit")
    if isinstance(model, CFResearch):
        ca = audit.get("c")
        if ca is not None and model.last["c"] is not None:
            return first_difference(
                bit_tree(model.last["c"]), bit_tree(ca), "CF-C-audit"
            )
    if isinstance(model, FResearch):
        expected_rejection = model.last.get("fault") == "internal revisit"
        if bool(audit.get("fault")) != expected_rejection:
            return ("F-rejection", expected_rejection, audit.get("fault"))
        if model.case["id"] == "09-19" and model.last.get("applied"):
            event = model.case["events"][prefix - 1]
            resolved = model.last["resolved_inputs"]
            ti, ri = resolved["theta_revision"], resolved["effects_revision"]
            theta = model.case["historical_inputs"][ti]
            effect = model.case["historical_inputs"][ri]

            def identity(body: dict[str, Any], tag: str) -> dict[str, Any]:
                raw = json.dumps(body, sort_keys=True, separators=(",", ":")).encode()
                return dict(
                    schema_version=1,
                    digest=hashlib.sha256(raw).hexdigest(),
                    byte_length=str(len(raw)),
                    object_type=tag,
                    resource_version="P5-71-v2",
                )

            expected_refs = dict(
                theta_before=identity(
                    dict(
                        revision=ti,
                        b=[0] * 4,
                        g=theta["theta_g"],
                        lambda_bits=["3fe62e42fefa39ef"] * 4,
                    ),
                    "echo.theta-snapshot-v3",
                ),
                effects_before=identity(
                    dict(
                        revision=ri,
                        tendency_id="k",
                        s=effect["effect_s"],
                        r=effect["effect_r"],
                        direction=effect["effect_direction"],
                    ),
                    "p571.effects-view",
                ),
                interpretation_refs=[
                    identity(
                        dict(
                            id=event["interpretation_id"],
                            a=event["a"],
                            mu=event["mu"],
                            people=event["involved_people"],
                        ),
                        "p571.interpretation",
                    )
                ],
                governance_refs=[
                    identity(
                        dict(
                            id=event["governance_id"],
                            qualification=event["qualification"],
                        ),
                        "p571.governance",
                    )
                ],
            )
            record = audit.get("record") or audit["receipts"][0]
            return first_difference(
                expected_refs,
                {k: record[k] for k in expected_refs},
                "historical-H-refs",
            )
    return None


def read_points(
    model: FResearch | CResearch, case: dict[str, Any]
) -> list[dict[str, Any]]:
    if isinstance(model, CResearch):
        names = ("connection", "empathy", "curiosity", "achievement")
        theta = {
            k: [model.c.values[field][n] for n in names]
            for k, field in (
                ("b", "baseline"),
                ("g", "gain"),
                ("lambda", "recovery_per_s"),
            )
        }
        theta["lambda_bits"] = [bits(v) for v in theta["lambda"]]
        fast = FResearch({}, dict(case, theta=theta, version="A1"), model.fn)
        fast.x = [Q(v) for v in case["read_initial_x"]]
        return read_points(fast, case)
    points = []
    original_anchor = model.anchor
    for delta in case["reads_ns"]:
        at = original_anchor + delta
        if model.mutant == "read-reanchor":
            model.project(at)
            x, h = model.x, model.h
        else:
            x, h = model.projection(at)
        points.append(dict(delta_ns=delta, x=bit_tree(x), h=bit_tree(h)))
    return points


def read_difference(
    points: list[dict[str, Any]], dut: dict[str, Any]
) -> tuple[str, Any, Any] | None:
    if not dut.get("reads"):
        return None
    for expected, actual in zip(points, dut["reads"]):
        values = actual["nature"] + actual["learned"]
        observed = [dict(f64=f"{v:016x}") for v in values]
        d = first_difference(expected["x"], observed, "read-x")
        if d:
            return d
        if (
            actual["nature"] != actual["phi_nature"]
            or actual["learned"] != actual["phi_learned"]
        ):
            return ("original-Phi", actual["phi_nature"], actual["nature"])
    return None


def exact_metrics(model: CResearch) -> dict[str, Any]:
    values = model.metrics()
    real = values.pop("real_reference")
    beta = values.pop("real_beta")
    return dict(
        binary=bit_tree(values),
        exact_reference={k: str(v) for k, v in real.items()},
        real_beta=str(beta),
    )


def rational_json(value: Any) -> Any:
    if isinstance(value, Q):
        return dict(binary64=bits(value), exact=str(value))
    if isinstance(value, dict):
        return {k: rational_json(v) for k, v in value.items()}
    if isinstance(value, list):
        return [rational_json(v) for v in value]
    return value
