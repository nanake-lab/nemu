"""Synthetic ADR 0049 inputs. Wire fixtures never open a store or authorize birth."""

from __future__ import annotations

from copy import deepcopy
from hashlib import sha256
from typing import Any

from fixture_law_state import ref

STAGES = ("irl", "appraiser", "matcher", "fast", "growth", "dynamics", "compiler")


def _runtime(stages: tuple[str, ...]) -> dict[str, Any]:
    bindings = {stage: ref("fixture." + stage, i) for i, stage in enumerate(STAGES, 1)}
    env = dict(
        schema_version=1,
        registry=ref("fixture.registry", 11),
        irl=bindings["irl"],
        carrier=ref("fixture.carrier", 12),
        model=ref("fixture.model", 13),
        tokenizer=ref("fixture.tokenizer", 14),
        config=ref("fixture.config", 15),
        implementations=[bindings[k] for k in STAGES[3:] if k in stages],
    )
    if "appraiser" in stages:
        env["nature_appraiser"] = bindings["appraiser"]
    if "matcher" in stages:
        env["situation_matcher"] = bindings["matcher"]
    if any(k in stages for k in ("fast", "growth", "dynamics")):
        env["law_version"] = ref("echo.law-version", 16)
    if "compiler" in stages:
        env["direction_set"] = ref("echo.j-direction-set", 17)
    return dict(schema_version=1, environment=env) | {k: bindings[k] for k in stages}


def _manifest(seed: int) -> tuple[str, dict[str, Any]]:
    from fixture_inputs import sample
    from verify_j import resource_bundle

    manifest = "GenesisManifestV3" if seed % 2 else "GenesisManifestV2"
    value = resource_bundle(seed % 3)["genesis"]
    if manifest == "GenesisManifestV2":
        value["proactivity_readout"] = sample(manifest, seed % 3)["proactivity_readout"]
    return manifest, value


def _birth(seed: int) -> dict[str, Any]:
    from fixture_j import reference
    from fixture_permission import permission_sample

    manifest, value = _manifest(seed)
    user = permission_sample("LocalUserBinding", seed)
    user["principal_id"] = value["caretaker_id"]
    raw = f"synthetic signature fixture {seed}".encode()
    signature = dict(
        schema_version=1,
        object_type="nemu.storage.raw-bytes-v1",
        resource_version="1",
        digest=sha256(raw).hexdigest(),
        byte_length=str(len(raw)),
    )
    return dict(
        schema_version=1,
        record_id=f"birth-{seed}",
        given_name=f"合成名字{seed}",
        manifest=reference(manifest, value),
        caretaker=user,
        signature_evidence=signature,
    )


def initial_sample(name: str, seed: int) -> dict[str, Any]:
    if name == "EmptyPrefixRoot":
        return dict(schema_version=1, kind=seed % 3 + 1)
    if name == "BirthNameRecord":
        return _birth(seed)
    if name == "InitialRuntimeRoot":
        return _runtime(((), ("irl", "fast", "compiler"), STAGES)[seed % 3])
    if name != "InitialPublication":
        raise ValueError(name)
    lineage = seed % 3 + 1
    header = dict(schema_version=1, lineage=lineage, created_at_ns="0")
    root: dict[str, Any] = dict(
        schema_version=1,
        store_id=f"fixture-store-{lineage}",
        branch_id=f"fixture-branch-{lineage}",
        header=header,
        x=ref("echo.fast-state-v3" if seed % 2 else "echo.fast-state-v2", 20),
        r=ref("echo.r-snapshot-v3", 21),
        theta=ref(
            "echo.theta-snapshot-v3" if seed % 2 else "echo.theta-snapshot-v2", 22
        ),
        q=_runtime(() if lineage == 1 else STAGES),
    )
    if lineage == 2:
        root["birth"] = _birth(seed)
        header["genesis_manifest_digest"] = root["birth"]["manifest"]["digest"]
    else:
        root["empty_history"] = initial_sample("EmptyPrefixRoot", 0)
        if lineage == 3:
            header.update(world_id="synthetic-only", world_seed=str(2**53 + 1))
    return root


def initial_legal() -> list[dict[str, Any]]:
    from fixture_inputs import request

    rehearsal = initial_sample("InitialPublication", 0)
    rehearsal["q"] = _runtime(("irl", "appraiser", "matcher", "fast", "compiler"))
    return [
        request("InitialPublication", rehearsal),
        request("InitialPublication", initial_sample("InitialPublication", 4)),
        # The Rust consumer checks the exact bytes named by both LIFE controls.
        request(*_manifest(1)),
        request(*_manifest(4)),
    ]


def initial_illegal() -> list[dict[str, Any]]:
    from fixture_inputs import LIMITS, request
    from echo.bridge import from_json
    from echo.runtime import Limits, encode_canonical, encode_protobuf

    rows: list[dict[str, Any]] = []

    def reject(
        name: str, label: str, value: dict[str, Any], *, wire: bool = True
    ) -> None:
        rows.append(dict(label="P5-52-" + label, request=request(name, value)))
        if (
            wire
        ):  # Structurally valid bytes must also reach semantic checks after decoding.
            for op, encode in (
                ("canonical", encode_canonical),
                ("protobuf", encode_protobuf),
            ):
                data = encode(name, from_json(name, value), Limits(**LIMITS))
                rows.append(
                    dict(
                        label=f"P5-52-{label}-{op}",
                        request=dict(name=name, op=op, hex=data.hex(), limits=LIMITS),
                    )
                )

    name = "InitialPublication"
    rehearsal, life, experiment = [initial_sample(name, i) for i in range(3)]
    for base in (life, experiment):
        for stage in STAGES:
            v = deepcopy(base)
            del v["q"][stage]
            reject(name, f"{v['header']['lineage']}-missing-{stage}", v)
    for stage in ("growth", "dynamics"):
        reject(name, "rehearsal-" + stage, rehearsal | dict(q=_runtime((stage,))))
    for base, field, tag in [
        (life, "x", "echo.user-text"),
        (rehearsal, "r", "echo.theta-snapshot-v2"),
        (experiment, "theta", "echo.r-snapshot-v3"),
    ]:
        reject(
            name, "wrong-" + field, base | {field: base[field] | dict(object_type=tag)}
        )
    for base in (rehearsal, experiment):
        v = deepcopy(base)
        del v["empty_history"]
        reject(
            name, f"birth-in-{v['header']['lineage']}", v | dict(birth=life["birth"])
        )
    v = deepcopy(life)
    del v["birth"]
    reject(
        name, "life-empty-history", v | dict(empty_history=rehearsal["empty_history"])
    )
    for kind in (2, 3):
        reject(
            name,
            f"other-empty-role-{kind}",
            rehearsal | dict(empty_history=dict(schema_version=1, kind=kind)),
        )
    reject(
        name,
        "manifest-digest",
        life | dict(header=life["header"] | dict(genesis_manifest_digest="00" * 32)),
    )
    reject(
        name,
        "both-origins",
        life | dict(empty_history=rehearsal["empty_history"]),
        wire=False,
    )
    reject(name, "missing-origin", v, wire=False)
    for stage in STAGES:
        q = _runtime(STAGES)
        q[stage] = q[stage] | dict(resource_version="same digest, different version")
        reject("InitialRuntimeRoot", "rebound-" + stage, q)
    for missing in ("law_version", "direction_set"):
        q = _runtime(STAGES)
        del q["environment"][missing]
        reject("InitialRuntimeRoot", "missing-" + missing, q)
    b = life["birth"]
    for key, changes in [
        ("manifest", dict(object_type="echo.user-text")),
        ("signature_evidence", dict(object_type="echo.user-text")),
        ("signature_evidence", dict(resource_version="2")),
        ("signature_evidence", dict(byte_length="0")),
    ]:
        bad = b | {key: b[key] | changes}
        reject("BirthNameRecord", "birth-" + key + "-" + next(iter(changes)), bad)
        reject(
            name, "nested-" + key + "-" + next(iter(changes)), life | dict(birth=bad)
        )
    return rows
