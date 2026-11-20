"""Synthetic wire facts for ADR 0046. No service calls or persisted histories."""

from __future__ import annotations

import copy
from typing import Any
from fixture_law_state import ref
from fixture_j import reference

NAMES = {
    "Utf8Range",
    "RecordVersion",
    "QuoteLayer",
    "SpeakerAttribution",
    "RecordRelation",
    "RelationSet",
    "RelationQueryReceipt",
    "RemoteCognitiveReceipt",
    "InternalActivityRecordV2",
    "InternalActivityCommitV2",
    "CoreTransactionV4",
    "SpeechDeliveryV2",
}


def span(start: int, end: int) -> dict[str, Any]:
    return dict(schema_version=1, start=str(start), end=str(end))


def receipt(i: int, status: int = 1, adoption: int = 1) -> dict[str, Any]:
    from fixture_inputs import sample

    v = dict(
        schema_version=1,
        binding=sample("CognitionBinding", i),
        provider="fixture-service",
        requested_model="fixture-requested",
        mode=1,
        parameters=ref("fixture.parameters", 1),
        format=ref("fixture.format", 2),
        mapping=ref("fixture.mapping", 3),
        meter=ref("fixture.meter", 4),
        input_manifest=ref("fixture.input-manifest", 5),
        status=status,
        adoption=adoption,
    )
    if status != 6:
        v.update(
            capacity="4096",
            output_reserve="512",
            sent_request=ref("fixture.request", 6),
            request_sha256="5a" * 32,
        )
    if status in (1, 2) or adoption != 5:
        v.update(
            raw_response=ref("fixture.response", 7),
            returned_model="fixture-returned",
            finish_reason="length" if status == 2 else "stop",
            usage=ref("fixture.usage", 8),
        )
    if status >= 3:
        v["reason"] = "fixture interruption"
    if adoption != 5:
        product = sample("CognitiveMaterial", i)
        v.update(
            returned_text=copy.deepcopy(product["content"]),
            review=ref("fixture.review", 9),
        )
        if adoption == 1:
            v.update(
                product=product,
                accepted_range=span(0, int(product["content"]["byte_length"])),
            )
        if adoption == 2:
            product["content"] = ref("fixture.partial-text", 10) | dict(byte_length="4")
            v.update(product=product, accepted_range=span(1, 5))
    return v


def sample_wire(name: str, seed: int) -> dict[str, Any]:
    from fixture_inputs import sample

    i = seed % 3
    a = dict(
        schema_version=1,
        branch_id="synthetic",
        record_id=f"r-{i}",
        content=ref("fixture.utf8", i + 1),
    )
    b = dict(a, record_id=f"source-{i}", content=ref("fixture.utf8", i + 10))
    layer = dict(
        schema_version=1,
        source=a,
        visible=span(0, 8),
        principal_id="speaker",
        identity_evidence=ref("fixture.identity", 15),
    )
    edge = dict(
        schema_version=1,
        origin=a,
        target=b,
        kind=(1, 2, 27)[i],
        purpose="quote-in-dialogue",
        effective_commit="commit-1",
        governance=ref("fixture.governance", 16),
    )
    result: dict[str, Any] = dict(
        schema_version=1,
        endpoint=a,
        purpose=edge["purpose"],
        frontier="commit-2",
        relations=[] if i == 0 else [edge],
    )
    if name == "Utf8Range":
        return span(i, 5 + i)
    if name == "RecordVersion":
        return a
    if name == "QuoteLayer":
        return layer
    if name == "SpeakerAttribution":
        layers = [layer]
        if i:
            layers.append(
                dict(
                    layer,
                    source=b,
                    principal_id="quoted-speaker",
                    identity_evidence=ref("fixture.identity", 17),
                )
            )
        return dict(schema_version=1, layers=layers, governance=edge["governance"])
    if name == "RecordRelation":
        return edge
    if name == "RelationSet":
        return result
    if name == "RelationQueryReceipt":
        return dict(
            schema_version=1,
            result=result,
            manifest=reference("RelationSet", result),
            expected_count=str(len(result["relations"])),
        )
    if name == "SpeechDeliveryV2":
        spans = ([span(0, 100)], [span(0, 4), span(10, 14)], [])[i]
        return dict(
            schema_version=1,
            action_id="act",
            output_id="out",
            status=(1, 2, 4)[i],
            channel=(1, 2, 1)[i],
            presentation=a,
            delivered=spans,
        )
    if name == "RemoteCognitiveReceipt":
        return receipt(i, (1, 2, 6)[i], (1, 2, 5)[i])
    if name == "InternalActivityRecordV2":
        v = sample("InternalActivityRecord", i)
        local = v.pop("result")
        if i == 0:
            v["local"] = local
        else:
            v["remote"] = sample_wire("RemoteCognitiveReceipt", i)
        return v
    if name == "InternalActivityCommitV2":
        v = sample("InternalActivityCommit", i)
        del v["record"]
        return dict(v, activity=sample_wire("InternalActivityRecordV2", i))
    if name == "CoreTransactionV4":
        v = sample("CoreTransactionV3", i)
        if "internal_commit" in v:
            del v["internal_commit"]
            v["activity_commit"] = sample_wire("InternalActivityCommitV2", i)
        return v
    raise ValueError(name)


def legal_wire() -> list[dict[str, Any]]:
    from fixture_inputs import request

    rows = [
        request("RemoteCognitiveReceipt", receipt(0, status, adoption))
        for status, choices in (
            (1, (2, 3, 4, 5)),
            (2, (3, 4, 5)),
            (3, (3, 4, 5)),
            (4, (3, 4, 5)),
            (5, (3, 4, 5)),
        )
        for adoption in choices
    ]
    rows.append(
        request(
            "RemoteCognitiveReceipt",
            dict(
                receipt(1),
                provider_revision="fixture-r1",
                model_sha256="ab" * 32,
                tokenizer_sha256="cd" * 32,
                local_pause_reason="local unavailable",
            ),
        )
    )
    rows.extend(
        request("RecordRelation", dict(sample_wire("RecordRelation", 1), kind=kind))
        for kind in range(1, 29)
    )
    return rows


def illegal_wire() -> list[dict[str, Any]]:
    from fixture_inputs import LIMITS, request
    from echo.bridge import from_json
    from echo.runtime import Limits, encode_canonical, encode_protobuf

    rows = []

    def bad(
        name: str, value: dict[str, Any], label: str, *, semantic: bool = True
    ) -> None:
        rows.append(dict(label=label, request=request(name, value)))
        if semantic:
            native = from_json(name, value)
            for op, encoder in (
                ("canonical", encode_canonical),
                ("protobuf", encode_protobuf),
            ):
                rows.append(
                    dict(
                        label=label + "-" + op,
                        request=dict(
                            name=name,
                            op=op,
                            limits=LIMITS,
                            hex=encoder(name, native, Limits(**LIMITS)).hex(),
                        ),
                    )
                )

    cloud = "RemoteCognitiveReceipt"
    bad(cloud, dict(receipt(0), action_plan={}), "cloud-action-plan", semantic=False)
    bad(cloud, dict(receipt(0), next_focus={}), "cloud-next-focus", semantic=False)
    bad(cloud, dict(receipt(0), queries=[]), "cloud-queries", semantic=False)
    for key in ("model_sha256", "tokenizer_sha256", "request_sha256"):
        bad(cloud, dict(receipt(0), **{key: "00" * 32}), "zero-" + key)
    bad(cloud, receipt(0, 2, 1), "truncated-full-adoption", semantic=False)
    # Locate the status discriminator using two valid encodings with identical
    # lengths. Change only that byte in a full-adoption receipt on each wire.
    full = receipt(0)
    ranged = dict(full, adoption=2)
    for op, encoder in (("canonical", encode_canonical), ("protobuf", encode_protobuf)):

        def encoded(value: dict[str, Any]) -> bytes:
            return encoder(cloud, from_json(cloud, value), Limits(**LIMITS))

        normal, cut = encoded(ranged), encoded(dict(ranged, status=2))
        offsets = [
            j for j, (a, b) in enumerate(zip(normal, cut, strict=True)) if a != b
        ]
        assert len(offsets) == 1
        offset = offsets[0]
        damaged = bytearray(encoded(full))
        assert damaged[offset] == normal[offset] == 1 and cut[offset] == 2
        damaged[offset] = cut[offset]
        rows.append(
            dict(
                label="truncated-full-adoption-" + op,
                request=dict(name=cloud, op=op, limits=LIMITS, hex=damaged.hex()),
            )
        )

    bad(
        cloud,
        dict(receipt(0, 6, 5), sent_request=ref("fixture.request", 6)),
        "not-sent-with-request",
        semantic=False,
    )
    bad(cloud, dict(receipt(0), accepted_range=span(1, 50)), "full-with-partial-range")
    bad(cloud, dict(receipt(0), adoption=3), "rejected-with-product")
    bad(
        cloud,
        dict(receipt(0, 4, 5), returned_text=ref("fixture.text", 10)),
        "text-without-response",
    )
    bad(cloud, dict(receipt(0), output_reserve="5000"), "reserve-over-capacity")
    relation = sample_wire("RecordRelation", 1)
    for key in ("origin", "target"):
        incomplete = dict(relation)
        del incomplete[key]
        bad("RecordRelation", incomplete, "missing-" + key, semantic=False)
    bad(
        "RecordRelation",
        dict(relation, target=dict(relation["target"], branch_id="other")),
        "cross-branch",
    )
    result = sample_wire("RelationQueryReceipt", 1)
    bad("RelationQueryReceipt", dict(result, expected_count="0"), "incomplete-query")
    result["result"]["relations"].clear()
    bad(
        "RelationQueryReceipt",
        dict(result, expected_count="0"),
        "omission-with-recount",
    )
    speaker = sample_wire("SpeakerAttribution", 1)
    speaker["layers"][1]["visible"] = span(10, 101)
    bad("SpeakerAttribution", speaker, "quote-out-of-bounds")
    delivery = sample_wire("SpeechDeliveryV2", 1)
    bad(
        "SpeechDeliveryV2",
        dict(delivery, delivered=[span(0, 11), span(10, 20)]),
        "overlap",
    )
    bad("SpeechDeliveryV2", dict(delivery, status=1), "partial-presented-as-full")
    txn = sample_wire("CoreTransactionV4", 1)
    txn["activity_commit"]["activity"]["remote"]["tokenizer_sha256"] = "00" * 32
    bad("CoreTransactionV4", txn, "nested-remote-zero")
    return rows
