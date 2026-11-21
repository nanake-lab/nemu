"""Synthetic, process-local regression cases. Recorded numbers only exercise the interface."""

import asyncio
import json
from collections.abc import Awaitable, Callable
from dataclasses import dataclass, fields, replace
from pathlib import Path
from typing import cast

import pytest
from echo.bridge import from_json
from echo.runtime import SCHEMA, Limits, digest, encode_canonical
from echo.typed_api import from_value, to_value
from echo.types import (
    Act,
    ActionStatus,
    BodyActionResult,
    DesktopKind,
    DesktopObservation,
    EventEnvelope,
    EventKind,
    FactProjection,
    Lang,
    NatureAppraisalRecord,
    NatureVector,
    ObjectRef,
    ProposalBindingV2,
    QueryStatus,
    SemanticGraph,
    SourceMetadata,
    TouchKindV2,
    TouchObservationV2,
    UserText,
    UserUtterance,
)

from nemu_drive import (
    AppraisalFailure,
    AppraisalResult,
    AppraisalView,
    IncompleteAppraisal,
    LocalAppraiser,
    appraise,
)

# Fixture limits only. Neither a production budget nor an appraisal scale.
LIMITS = Limits(1_048_576, 4096, 32)
GOLD = json.loads(
    (
        Path(__file__).resolve().parents[3] / "core/contracts/golden/legal.json"
    ).read_text("utf-8")
)
COMPLETE: dict[str, float | None] = {
    "connection": -1.0,
    "empathy": 0.25,
    "curiosity": 0.0,
    "achievement": 1.0,
}
type Evaluate = Callable[[AppraisalView], Awaitable[dict[str, float | None]]]


def fixture(name: str) -> object:
    row = next(r["request"] for r in GOLD if r["request"]["name"] == name)
    return from_value(name, from_json(name, row["value"]), LIMITS)


def reference(value: object) -> ObjectRef:
    name = type(value).__name__
    data = to_value(value)
    return ObjectRef(
        1,
        bytes.fromhex(digest(name, data, LIMITS)),
        SCHEMA["messages"][name]["tag"],
        "1",
        len(encode_canonical(name, data, LIMITS)),
    )


@dataclass(frozen=True)
class Case:
    event: EventEnvelope
    binding: ProposalBindingV2
    view: AppraisalView
    resource: ObjectRef

    async def run(
        self, evaluate: Evaluate, deadline: float | None = None
    ) -> AppraisalResult:
        local = LocalAppraiser(self.resource, "offline-fixture", "1", evaluate)
        return await appraise(
            self.event,
            self.binding,
            self.view,
            local,
            record_id="fixture-record",
            limits=LIMITS,
            deadline=deadline,
        )


def example(payload: object, kind: EventKind = EventKind.OBSERVATION) -> Case:
    original = cast(ProposalBindingV2, fixture("ProposalBindingV2"))
    ref = reference(payload)
    event = replace(
        cast(EventEnvelope, fixture("EventEnvelope")),
        id="fixture-event",
        payload_ref=ref,
        payload_digest=ref.digest,
        kind=kind,
    )
    resource = replace(original.environment.irl, resource_version="offline-fixture-v1")
    metadata = replace(
        cast(SourceMetadata, fixture("SourceMetadata")), source_refs=[event.id]
    )
    z = SemanticGraph(
        1,
        "fixture-z",
        [],
        [],
        [Act(1, "fixture-act", metadata, "synthetic-observation")],
        [],
    )
    facts = FactProjection(
        1, original.base_commit, "fixture", QueryStatus.OK, [], "1", False
    )
    binding = replace(
        original,
        event_id=event.id,
        raw_log_position=event.raw_log_position,
        environment=replace(original.environment, nature_appraiser=resource),
        accepted_z_digest=reference(z).digest,
        c_core_digest=reference(facts).digest,
    )
    return Case(event, binding, AppraisalView(z, facts, ("fixture-person",)), resource)


class Replay:
    def __init__(self, row: dict[str, float | None]) -> None:
        self.row = row
        self.views: list[AppraisalView] = []

    async def __call__(self, view: AppraisalView) -> dict[str, float | None]:
        self.views.append(view)
        return self.row.copy()


@pytest.mark.parametrize(
    "payload",
    [UserText(1, "文字样本"), UserUtterance(1, "音声", Lang.JA, 0.9, 250, None)],
)
def test_text_and_speech_make_only_bound_proposals(payload: object) -> None:
    case = example(payload)
    replay = Replay(COMPLETE)
    result = asyncio.run(case.run(replay))
    assert isinstance(result, NatureAppraisalRecord)
    assert result.appraisal == NatureVector(1, -1.0, 0.25, 0.0, 1.0)
    assert result.binding == case.binding and result.binding is not case.binding
    assert result.event_id == case.event.id and result.record_id == "fixture-record"
    assert result.persons == ["fixture-person"]
    assert (result.producer, result.producer_version) == ("offline-fixture", "1")
    assert replay.views == [case.view]
    assert [f.name for f in fields(replay.views[0])] == ["z", "c_core", "persons"]
    assert replay.views[0].z is not case.view.z
    assert replay.views[0].c_core is not case.view.c_core


def test_repeated_unknown_execution_receipt_stays_unknown_without_appraisal() -> None:
    receipt = BodyActionResult(1, "fixture-action", ActionStatus.UNKNOWN_ACTION, None)
    case = example(receipt, EventKind.EXECUTION_RESULT)
    replay = Replay(COMPLETE)
    for _ in range(2):
        assert asyncio.run(case.run(replay)) is AppraisalFailure.UNSUPPORTED_SOURCE
        assert receipt.status is ActionStatus.UNKNOWN_ACTION
        assert case.event.payload_ref == reference(receipt)
    assert replay.views == []


@pytest.mark.parametrize(
    "payload",
    [
        TouchObservationV2(1, "head", None, TouchKindV2.TOUCH_V2_CLICK, None, None),
        DesktopObservation(1, DesktopKind.IDLE_STARTED, None, None, None, 250),
    ],
)
def test_uncovered_source_needs_no_semantic_input_or_evaluator(payload: object) -> None:
    case = example(payload)
    result = asyncio.run(
        appraise(
            case.event,
            case.binding,
            None,
            None,
            record_id="fixture-record",
            limits=LIMITS,
        )
    )
    assert result is AppraisalFailure.UNSUPPORTED_SOURCE


def test_text_payload_tag_does_not_admit_an_execution_event() -> None:
    case = example(UserText(1, "fixture"), EventKind.EXECUTION_RESULT)
    replay = Replay(COMPLETE)
    assert asyncio.run(case.run(replay)) is AppraisalFailure.UNSUPPORTED_SOURCE
    assert replay.views == []


def test_known_zero_is_confined_to_the_supported_coordinate() -> None:
    case = example(UserText(1, "本次测试材料只确认纯噪声。"))
    observed = replace(case.view.z.acts[0], interaction_kind="fixture-pure-noise")
    z = replace(case.view.z, acts=[observed])
    case = replace(
        case,
        view=replace(case.view, z=z),
        binding=replace(case.binding, accepted_z_digest=reference(z).digest),
    )
    row = {"connection": None, "empathy": None, "curiosity": 0.0, "achievement": None}
    replay = Replay(row)
    result = asyncio.run(case.run(replay))
    assert replay.views[0].z.acts == [observed]
    assert isinstance(result, IncompleteAppraisal) and result.known == {
        "curiosity": 0.0
    }


def test_assessed_all_unknown_is_distinct_from_failure_and_missing_keys() -> None:
    case = example(UserText(1, "fixture"))
    assert asyncio.run(
        case.run(Replay(dict.fromkeys(COMPLETE)))
    ) == IncompleteAppraisal({})
    assert asyncio.run(case.run(Replay({}))) is AppraisalFailure.INVALID_OUTPUT


@pytest.mark.parametrize(
    "bad",
    [
        {**COMPLETE, "empathy": float("nan")},
        {**COMPLETE, "curiosity": float("inf")},
        {**COMPLETE, "achievement": 1.01},
        {**COMPLETE, "connection": -1.01, "curiosity": None},
        {**COMPLETE, "empathy": False},
        {**COMPLETE, "x": 0.2},
        {"curiosity": 0.0},
    ],
)
def test_invalid_complete_or_partial_output_never_yields_a(
    bad: dict[str, float | None],
) -> None:
    case = example(UserText(1, "fixture"))
    assert asyncio.run(case.run(Replay(bad))) is AppraisalFailure.INVALID_OUTPUT


def test_stale_inputs_are_rejected_before_the_evaluator() -> None:
    case = example(UserText(1, "fixture"))
    b = case.binding
    other_prefix = replace(case.view.c_core, history_prefix="other")
    mismatches = [
        replace(case, binding=replace(b, event_id="other")),
        replace(case, binding=replace(b, raw_log_position=b.raw_log_position + 1)),
        replace(case, event=replace(case.event, payload_digest=bytes(32))),
        replace(case, view=replace(case.view, z=replace(case.view.z, id="other"))),
        replace(
            case,
            view=replace(case.view, c_core=replace(case.view.c_core, truncated=True)),
        ),
        replace(
            case,
            view=replace(case.view, c_core=other_prefix),
            binding=replace(b, c_core_digest=reference(other_prefix).digest),
        ),
        replace(case, resource=replace(case.resource, resource_version="other")),
    ]
    replay = Replay(COMPLETE)
    for mismatch in mismatches:
        assert asyncio.run(mismatch.run(replay)) is AppraisalFailure.BINDING_MISMATCH
    assert replay.views == []


def test_snapshot_survives_mutation_during_local_call() -> None:
    case = example(UserText(1, "fixture"))
    original_read_set = [case.event.payload_ref, case.resource]
    case = replace(
        case, binding=replace(case.binding, read_set=original_read_set.copy())
    )

    async def local(view: AppraisalView) -> dict[str, float | None]:
        case.binding.read_set.clear()
        view.z.acts.clear()
        return COMPLETE

    result = asyncio.run(case.run(local))
    assert isinstance(result, NatureAppraisalRecord)
    assert result.binding.read_set == original_read_set
    assert len(case.view.z.acts) == 1


def test_private_caller_metadata_is_absent_from_evaluator_input() -> None:
    @dataclass(frozen=True)
    class CallerView(AppraisalView):
        x: str
        m: str
        theta: str
        tendency_strength: str

    case = example(UserText(1, "private raw text"))
    supplied = CallerView(
        case.view.z,
        case.view.c_core,
        case.view.persons,
        "private-x",
        "private-m",
        "private-theta",
        "private-strength",
    )
    replay = Replay(COMPLETE)
    result = asyncio.run(replace(case, view=supplied).run(replay))
    assert isinstance(result, NatureAppraisalRecord)
    (observed,) = replay.views
    assert type(observed) is AppraisalView and observed == case.view
    assert "private" not in repr(observed)


def test_unavailable_and_faults_give_no_record() -> None:
    case = example(UserText(1, "fixture"))
    assert (
        asyncio.run(
            appraise(
                case.event,
                case.binding,
                case.view,
                None,
                record_id="fixture",
                limits=LIMITS,
            )
        )
        is AppraisalFailure.UNAVAILABLE
    )

    async def broken(_: AppraisalView) -> dict[str, float | None]:
        raise OSError("fixture backend unavailable")

    assert asyncio.run(case.run(broken)) is AppraisalFailure.UNAVAILABLE


def test_deadline_cancels_pending_call_without_retry() -> None:
    case = example(UserText(1, "fixture"))
    calls = []

    async def pending(_: AppraisalView) -> dict[str, float | None]:
        calls.append("started")
        try:
            await asyncio.Future[None]()
        finally:
            calls.append("closed")
        return COMPLETE

    async def run() -> None:
        # Short real-time deadline only for this synthetic case.
        assert (
            await case.run(pending, asyncio.get_running_loop().time() + 0.25)
            is AppraisalFailure.TIMEOUT
        )

    asyncio.run(run())
    assert calls == ["started", "closed"]
