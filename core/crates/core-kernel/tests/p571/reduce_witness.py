"""Frozen deletion-order 1-minimal witnesses; no parameter or threshold search.

Invoked only after the original raw trajectory is durable. The original file
is never rewritten. All deletion attempts are logged, including invalid
premises, and share the formal run's remaining wall budget.
"""

import json
from pathlib import Path
from typing import Any, Callable

from c_oracle import bit_tree, first_difference
from criteria import measurement_mutation
from f_research import FResearch
from c_research import CResearch
from matrix_support import (
    Budget,
    MatrixBridge,
    advance,
    audit_difference,
    compare,
    model_for,
    read_difference,
    read_points,
)
from prepare import write


def premises(case: dict[str, Any], events: list[dict[str, Any]]) -> bool:
    roots = (
        {case["initial"]["root"]}
        if "initial" in case and "root" in case["initial"]
        else set()
    )
    ordinals: set[int] = set()
    for e in events:
        # CF stores F as a named substep; preserve its root prerequisites too.
        f_event = e.get("f")
        if f_event is not None:
            f_op = f_event.get("op")
            if f_op in ("new", "update", "revisit"):
                if (
                    f_op != "new"
                    and f_event["root"] not in roots
                    and f_event.get("kind") != "internal"
                ):
                    return False
                roots.add(f_event["root"])
        op = e.get("op")
        if op in ("new", "update", "revisit"):
            if op != "new" and e["root"] not in roots and e.get("kind") != "internal":
                return False
            roots.add(e["root"])
        if op == "merge" and not set(e["members"]) <= roots:
            return False
        if not set(e.get("invalidate_prior_ordinals", [])) <= ordinals:
            return False
        ordinals.add(e.get("ordinal", 0))
    return True


def reduce_all(
    out: Path,
    resources: dict[str, Any],
    refs: dict[str, Any],
    bridge: MatrixBridge,
    budget: Budget,
    request_for: Callable[..., dict[str, Any]],
    law_ref: Callable[..., dict[str, Any]],
    selected_version: Callable[..., str | None],
    comparable_state: Callable[..., dict[str, Any]],
) -> None:
    witnesses = sorted(
        list(out.glob("*-first-failure.json"))
        + list(out.glob("*-mutation-witness.json"))
    )
    for path in witnesses:
        budget.check()
        source = json.loads(path.read_text(encoding="utf-8"))
        row = source["plan"]
        if row["family"] not in ("C", "F", "CF", "G"):
            continue
        original = row["input"]
        events = source["events"]
        mutation = path.name.endswith("mutation-witness.json")
        selected = selected_version(row)
        name = original.get("resource", original.get("f_resource"))

        def fails(subset: list[dict[str, Any]]) -> tuple[str, Any, Any] | None:
            budget.check()
            case = dict(original, events=subset)
            model = model_for(
                resources, case, row["family"], bridge, law_ref(refs[name])
            )
            baseline = (
                model_for(
                    resources,
                    dict(case, version=selected, negative_control=""),
                    row["family"],
                    bridge,
                    law_ref(refs[name]),
                )
                if mutation and selected
                else None
            )
            dut_rows = []
            if not mutation and source.get("comparison_layer") == "DUT_oracle":
                altered = dict(row, input=case)
                # References retain original H/input provenance. Deletion is an
                # explicitly logged witness derivation, never a new input claim.
                for value in bridge.trajectory(
                    request_for(altered, resources, refs, budget)
                ):
                    if "prefix" in value:
                        dut_rows.append(value)
            for prefix in range(len(subset) + 1):
                budget.check()
                if prefix:
                    advance(model, subset[prefix - 1], prefix - 1, True)
                    if baseline:
                        advance(baseline, subset[prefix - 1], prefix - 1, True)
                if baseline and prefix:
                    expected = dict(
                        state=bit_tree(baseline.state()), audit=bit_tree(baseline.last)
                    )
                    actual = dict(
                        state=bit_tree(model.state()), audit=bit_tree(model.last)
                    )
                    d = first_difference(
                        comparable_state(expected["state"], row["family"]),
                        comparable_state(actual["state"], row["family"]),
                        "selected-law-criterion",
                    )
                    d = d or measurement_mutation(
                        row["version"], prefix, len(subset), case, expected, actual
                    )
                    if d:
                        return d
                elif dut_rows:
                    dut = dut_rows[prefix]
                    d = compare(model, dut, row["family"]) or audit_difference(
                        model, dut, prefix
                    )
                    if not dut["i23"]:
                        d = ("DUT-I23", True, False)
                    if d:
                        return d
                elif not mutation and prefix:
                    if (
                        isinstance(model, CResearch)
                        and model.last.get("fault")
                        and not isinstance(subset[prefix - 1].get("a"), str)
                    ):
                        return (
                            "finite-input-numerical-fault",
                            "finite admitted transition",
                            model.last["fault"],
                        )
                    replay = model_for(
                        resources, case, row["family"], bridge, law_ref(refs[name])
                    )
                    for j, event in enumerate(subset[:prefix]):
                        budget.check()
                        advance(replay, event, j, False)
                    d = first_difference(
                        bit_tree(model.state()),
                        bit_tree(replay.state()),
                        "independent-I23",
                    )
                    if d:
                        return d
            if isinstance(model, FResearch):
                points = read_points(model, case)
                if baseline and isinstance(baseline, FResearch):
                    return first_difference(
                        read_points(baseline, case), points, "same-anchor-read"
                    )
                if dut_rows:
                    return read_difference(points, dut_rows[-1])
            return None

        attempt_path = out / (path.stem + "-reduction.jsonl")
        with attempt_path.open("x", encoding="utf-8", newline="\n") as stream:
            initial = fails(events)
            stream.write(
                json.dumps(
                    dict(kind="reproduce", difference=initial), ensure_ascii=False
                )
                + "\n"
            )
            stream.flush()
            if initial is None:
                write(
                    out / (path.stem + "-minimum.json"),
                    dict(
                        status="not_run",
                        reason="witness predicate did not reproduce; original preserved",
                        original=path.name,
                    ),
                )
                continue
            changed = True
            while changed:
                changed = False
                for index in range(len(events)):
                    budget.check()
                    trial = events[:index] + events[index + 1 :]
                    valid = premises(original, trial)
                    difference = fails(trial) if valid else None
                    record = dict(
                        delete_index=index,
                        deleted_event=events[index],
                        premises_valid=valid,
                        still_fails=difference is not None,
                        difference=difference,
                    )
                    stream.write(
                        json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n"
                    )
                    stream.flush()
                    if difference is not None:
                        events = trial
                        changed = True
                        break
                if not changed:
                    roots = list(
                        dict.fromkeys(e["root"] for e in events if "root" in e)
                    )
                    for root in roots:
                        budget.check()
                        trial = [e for e in events if e.get("root") != root]
                        valid = premises(original, trial)
                        difference = fails(trial) if valid else None
                        stream.write(
                            json.dumps(
                                dict(
                                    delete_root=root,
                                    premises_valid=valid,
                                    still_fails=difference is not None,
                                    difference=difference,
                                ),
                                ensure_ascii=False,
                            )
                            + "\n"
                        )
                        stream.flush()
                        if difference is not None:
                            events = trial
                            changed = True
                            break
            # Coordinate/root deletion: all worlds have exactly one active F
            # direction or four equal C channels. The input schema requires four
            # C channels; removing an involved root is covered by its event
            # deletions above. Zero coordinates are implicit and have no input
            # events to delete. We do not replace values to seek a new witness.
            write(
                out / (path.stem + "-minimum.json"),
                dict(
                    status="completed",
                    minimality="1-minimal under registered event deletion order and preserved premises",
                    original=path.name,
                    events=events,
                    difference=fails(events),
                    attempts=attempt_path.name,
                    coordinate_root_scope="fixed 4-channel schema; one active F coordinate; roots removed with all corresponding events",
                ),
            )
