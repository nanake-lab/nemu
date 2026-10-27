"""Contract compatibility gate (P5-21; ADR 0007 §§2, 11).

Proves that published field numbers, enum values and frozen type tags are never
reused. The current .proto files are compared with every frozen baseline under
baselines/<name>/ (the .proto files as published) and, with --ref, with the
.proto files at a git ref. Checks:

1. every published message keeps its tag and its exact field table and rules,
   so its CanonicalV1 and Protobuf bytes are unchanged;
2. a new message never takes a published tag;
3. every published enum keeps every (name, value); a new name never takes a
   published value;
4. supersession is permanent: a superseded mark is never removed;
5. a successor (@supersedes) keeps each predecessor field number identical or
   reserved, and numbers its new fields above the predecessor's maximum;
6. no live message embeds a superseded message or a wholly superseded enum
   (old R/G types stay types only, not wired into new records);
7. the frozen golden vectors of each baseline are an unchanged prefix of the
   current golden files (byte-for-byte via a SHA-256 of their entries).

A self-test applies one deliberate violation per rule and requires each to be
reported, so the gate cannot pass vacuously. Exit status 1 on any violation.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from generate import schema

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASELINES = HERE / "baselines"

Schema = dict[str, Any]


def whole_enum_superseded(s: Schema, name: str) -> bool:
    return name in s.get("superseded_enums", {})


def violations(old: Schema, new: Schema) -> list[str]:
    out: list[str] = []
    old_m, new_m = old["messages"], new["messages"]
    for name, m in old_m.items():
        n = new_m.get(name)
        if n is None:
            out.append(f"message {name} removed")
            continue
        if n["tag"] != m["tag"]:
            out.append(f"message {name} tag changed {m['tag']} -> {n['tag']}")
        if n["fields"] != m["fields"]:
            before = {f["number"]: f for f in m["fields"]}
            after = {f["number"]: f for f in n["fields"]}
            for number in sorted(before.keys() | after.keys()):
                if before.get(number) != after.get(number):
                    out.append(f"message {name} field {number} changed")
            if before == after:
                out.append(f"message {name} field order changed")
        if m.get("transient", False) != n.get("transient", False):
            out.append(f"message {name} persistence classification changed")
        if m.get("retains") != n.get("retains"):
            out.append(f"message {name} retained predecessor changed")
        if m.get("resource") != n.get("resource"):
            out.append(f"message {name} resource projection changed")
        if n["rules"] != m["rules"]:
            out.append(f"message {name} rules changed")
        if set(m.get("reserved", [])) - set(n.get("reserved", [])):
            out.append(f"message {name} dropped a reserved number")
        if "superseded" in m and "superseded" not in n:
            out.append(f"message {name} lost its superseded mark")
    old_tags = {m["tag"]: name for name, m in old_m.items()}
    for name, n in new_m.items():
        if name not in old_m and n["tag"] in old_tags:
            out.append(f"tag {n['tag']} of {old_tags[n['tag']]} reused by {name}")
    for enum, values in old["enums"].items():
        current = new["enums"].get(enum)
        if current is None:
            out.append(f"enum {enum} removed")
            continue
        for key, value in values.items():
            if current.get(key) != value:
                out.append(f"enum {enum}.{key} = {value} removed or renumbered")
        used = set(values.values())
        for key, value in current.items():
            if key not in values and value in used:
                out.append(f"enum {enum}.{key} reuses published value {value}")
        if whole_enum_superseded(old, enum) and not whole_enum_superseded(new, enum):
            out.append(f"enum {enum} lost its superseded mark")
        for key in old.get("superseded_values", {}).get(enum, {}):
            if key not in new.get("superseded_values", {}).get(enum, {}):
                out.append(f"enum {enum}.{key} lost its superseded mark")
    for enum, aliases in old.get("json_enums", {}).items():
        if new.get("json_enums", {}).get(enum) != aliases:
            out.append(f"enum {enum} JSON aliases changed")
    return out + discipline(new)


def discipline(s: Schema) -> list[str]:
    """Rules 5 and 6, which hold within a single schema."""
    out: list[str] = []
    messages = s["messages"]
    for name, m in messages.items():
        field_numbers = [f["number"] for f in m["fields"]]
        if len(field_numbers) != len(set(field_numbers)):
            out.append(f"{name} reuses a field number")
        groups = m["rules"].get("oneofs", {})
        for group_name, group in groups.items():
            members = [f for f in m["fields"] if f.get("oneof") == group_name]
            if (
                [f["name"] for f in members] != group["fields"]
                or not members
                or any(f["required"] or f["repeated"] for f in members)
            ):
                out.append(f"{name} invalid oneof {group_name}")
        if any(f.get("oneof") not in groups for f in m["fields"] if "oneof" in f):
            out.append(f"{name} orphan oneof member")
        predecessor = m.get("supersedes") or m.get("retains")
        if predecessor:
            p = messages.get(predecessor, {})
            if not p or (
                "supersedes" in m and p.get("superseded", {}).get("by") != name
            ):
                out.append(f"{name} supersedes {predecessor}, which does not name it")
            kept = {f["number"]: f for f in m["fields"]}
            reserved = set(m.get("reserved", []))
            numbers = {f["number"] for f in p.get("fields", [])} | set(
                p.get("reserved", [])
            )
            # A field number is never reused, however many generations later.
            if kept.keys() & reserved:
                out.append(
                    f"{name} uses its own reserved numbers {sorted(kept.keys() & reserved)}"
                )
            for f in p.get("fields", []):
                if f["number"] in kept and kept[f["number"]] != f:
                    out.append(
                        f"{name} field {f['number']} reuses {predecessor}.{f['name']} differently"
                    )
                if f["number"] not in kept and f["number"] not in reserved:
                    out.append(f"{name} must reserve {predecessor} field {f['number']}")
            # The predecessor's reserved numbers are inherited in full and stay unused.
            for number in sorted(set(p.get("reserved", []))):
                if number in kept:
                    out.append(
                        f"{name} field {number} reuses {predecessor}'s reserved number {number}"
                    )
                elif number not in reserved:
                    out.append(
                        f"{name} must inherit {predecessor}'s reserved number {number}"
                    )
            for number in kept:
                if number not in numbers and number <= max(numbers, default=0):
                    out.append(
                        f"{name} new field {number} is not above {predecessor}'s numbers"
                    )
        if "superseded" in m:
            continue
        for f in m["fields"]:
            t = f["type"]
            if not m.get("transient") and messages.get(t, {}).get("transient"):
                out.append(
                    f"persistent {name}.{f['name']} embeds transient credentials"
                )
            if "superseded" in messages.get(t, {}) or whole_enum_superseded(s, t):
                out.append(f"live {name}.{f['name']} uses superseded {t}")
    return out


def entry_digest(entries: list[Any]) -> str:
    h = hashlib.sha256()
    for e in entries:
        h.update(
            json.dumps(
                e, sort_keys=True, ensure_ascii=True, separators=(",", ":")
            ).encode("ascii")
            + b"\n"
        )
    return h.hexdigest()


def golden_violations(baseline: Path) -> list[str]:
    path = baseline / "golden.json"
    if not path.exists():
        return []
    out = []
    for name, frozen in json.loads(path.read_text(encoding="utf-8")).items():
        current = json.loads((HERE / f"golden/{name}.json").read_text(encoding="utf-8"))
        prefix = current[: frozen["count"]]
        if len(prefix) != frozen["count"] or entry_digest(prefix) != frozen["sha256"]:
            out.append(
                f"golden {name}: frozen prefix of {frozen['count']} vectors changed"
            )
    return out


def schema_at(ref: str) -> Schema:
    names = subprocess.run(
        ["git", "ls-tree", "--name-only", ref, "core/contracts/"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.split()
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            if name.endswith(".proto"):
                text = subprocess.run(
                    ["git", "show", f"{ref}:{name}"],
                    cwd=ROOT,
                    check=True,
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                ).stdout
                (Path(tmp) / Path(name).name).write_text(text, encoding="utf-8")
        return schema(Path(tmp))


def _field(name: str, typ: str, number: int, required: bool = False) -> dict[str, Any]:
    return dict(
        name=name,
        type=typ,
        number=number,
        repeated=False,
        required=required,
        nonempty=False,
    )


def with_chain(s: Schema) -> Schema:
    """Add a correct synthetic three-generation chain SelfTestV1 -> V2 -> V3.

    The chain has its own tags, is in no baseline and is embedded in no live message, so
    the self-test never rewrites a published field, whichever baselines are frozen.
    V1 has fields 1, 2, 5; V2 keeps 1, 2, reserves 5 and adds 6; V3 keeps 1, 2, 6,
    inherits reserved 5 and adds 7."""
    s = copy.deepcopy(s)
    version = _field("schema_version", "uint32", 1, required=True)
    a, b = _field("a", "string", 2), _field("b", "double", 5)
    c, d = _field("c", "bool", 6), _field("d", "uint64", 7)

    def message(tag: str, fields: list[dict[str, Any]]) -> dict[str, Any]:
        return dict(
            tag=tag,
            source="check_compat self-test",
            file="self-test",
            rules={},
            fields=copy.deepcopy(fields),
        )

    v1 = message("selftest.v1", [version, a, b])
    v1["superseded"] = dict(by="SelfTestV2", reason="self-test")
    v2 = message("selftest.v2", [version, a, c])
    v2.update(
        reserved=[5],
        supersedes="SelfTestV1",
        superseded=dict(by="SelfTestV3", reason="self-test"),
    )
    v3 = message("selftest.v3", [version, a, c, d])
    v3.update(reserved=[5], supersedes="SelfTestV2")
    s["messages"].update(SelfTestV1=v1, SelfTestV2=v2, SelfTestV3=v3)
    return s


def self_test(old: Schema, new: Schema) -> list[str]:
    """Each mutation must produce its own expected violation; returns what was missed.

    A mutation counts as caught only when the rule it targets fires, so a mutation that
    happens to touch a field frozen in some baseline cannot pass for the wrong reason.
    The fixtures change no published field, so the self-test also passes with ``old``
    equal to ``new`` (the current schema frozen as a baseline)."""
    assert not violations(old, new), "self-test needs a clean schema"
    chained = with_chain(new)

    def field(s: Schema, message: str, name: str) -> dict[str, Any]:
        return next(f for f in s["messages"][message]["fields"] if f["name"] == name)

    def renumber(s: Schema) -> None:
        field(s, "EventEnvelope", "id")["number"] = 99

    def retype(s: Schema) -> None:
        field(s, "OutputEnd", "reason")["type"] = "uint32"

    def recycle_enum(s: Schema) -> None:
        s["enums"]["RunStatus"]["RECYCLED"] = 4

    def drop_enum_value(s: Schema) -> None:
        del s["enums"]["FailureCode"]["UNKNOWN_OVERLAP"]

    def reuse_tag(s: Schema) -> None:
        s["messages"]["SelfTestTag"] = dict(
            copy.deepcopy(s["messages"]["SelfTestV1"]), tag="echo.fast-state"
        )
        del s["messages"]["SelfTestTag"]["superseded"]

    def reuse_reserved(s: Schema) -> None:
        v2 = s["messages"]["SelfTestV2"]
        v2["fields"].append(_field("e", "string", 5))
        v2["reserved"] = []

    def unreserve(s: Schema) -> None:
        s["messages"]["SelfTestV2"]["reserved"] = []

    def low_number(s: Schema) -> None:
        field(s, "SelfTestV2", "c")["number"] = 3

    def embed_old(s: Schema) -> None:
        s["messages"]["SelfTestV3"]["fields"].append(_field("e", "SelfTestV1", 8))

    def embed_old_enum(s: Schema) -> None:
        s["messages"]["SelfTestV3"]["fields"].append(_field("f", "EffectEndpoint", 8))

    def unmark(s: Schema) -> None:
        del s["messages"]["EffectContract"]["superseded"]

    def unmark_value(s: Schema) -> None:
        del s["superseded_values"]["RunStatus"]["AX_DEGRADED"]

    def reuse_inherited_reserved(s: Schema) -> None:
        # Codex PR #36 counterexample, on the real messages: VersionedEnvironmentV3
        # succeeds V2 and turns V2's reserved 4 into a field. No published field changes;
        # the extra dependency-chain violations it causes are not what is checked here.
        m = s["messages"]
        v3 = copy.deepcopy(m["VersionedEnvironmentV2"])
        v3.update(tag="echo.environment-v3", supersedes="VersionedEnvironmentV2")
        del v3["reserved"]
        v3["fields"].append(_field("scope_v3", "double", 4))
        m["VersionedEnvironmentV3"] = v3
        m["VersionedEnvironmentV2"]["superseded"] = dict(
            by="VersionedEnvironmentV3", reason="self-test"
        )

    def reuse_inherited_reserved_chain(s: Schema) -> None:
        v3 = s["messages"]["SelfTestV3"]
        v3["fields"].append(_field("e", "double", 5))
        v3["reserved"] = []

    def drop_inherited_reserved(s: Schema) -> None:
        s["messages"]["SelfTestV3"]["reserved"] = []

    def reuse_own_reserved(s: Schema) -> None:
        s["messages"]["SelfTestV3"]["fields"].append(_field("e", "double", 5))

    def oneof_members(s: Schema) -> None:
        s["messages"]["ReadoutV2"]["rules"]["oneofs"]["body"]["fields"].pop()

    def oneof_number(s: Schema) -> None:
        field(s, "ReadoutV2", "v2")["number"] = 5

    def oneof_move(s: Schema) -> None:
        field(s, "ReadoutV2", "v2")["oneof"] = "other"

    # (mutation, compare against the current schema instead of the baseline, expected)
    mutations = (
        (oneof_members, True, "message ReadoutV2 rules changed"),
        (oneof_number, True, "ReadoutV2 reuses a field number"),
        (oneof_move, True, "message ReadoutV2 field 6 changed"),
        (renumber, False, "message EventEnvelope field 2 changed"),
        (retype, False, "message OutputEnd field 3 changed"),
        (recycle_enum, False, "enum RunStatus.RECYCLED reuses published value 4"),
        (drop_enum_value, False, "enum FailureCode.UNKNOWN_OVERLAP = 7 removed"),
        (
            reuse_tag,
            False,
            "tag echo.fast-state of FastStateSnapshot reused by SelfTestTag",
        ),
        (reuse_reserved, False, "SelfTestV2 field 5 reuses SelfTestV1.b differently"),
        (unreserve, False, "SelfTestV2 must reserve SelfTestV1 field 5"),
        (low_number, False, "SelfTestV2 new field 3 is not above SelfTestV1's numbers"),
        (embed_old, False, "live SelfTestV3.e uses superseded SelfTestV1"),
        (embed_old_enum, False, "live SelfTestV3.f uses superseded EffectEndpoint"),
        # Losing a mark is only a violation against a schema that already had the mark.
        (unmark, True, "message EffectContract lost its superseded mark"),
        (unmark_value, True, "enum RunStatus.AX_DEGRADED lost its superseded mark"),
        (
            reuse_inherited_reserved,
            False,
            "VersionedEnvironmentV3 field 4 reuses VersionedEnvironmentV2's reserved number 4",
        ),
        (
            reuse_inherited_reserved_chain,
            False,
            "SelfTestV3 field 5 reuses SelfTestV2's reserved number 5",
        ),
        (
            drop_inherited_reserved,
            False,
            "SelfTestV3 must inherit SelfTestV2's reserved number 5",
        ),
        (reuse_own_reserved, False, "SelfTestV3 uses its own reserved numbers [5]"),
    )
    missed = []
    for mutation, against_current, expected in mutations:
        mutated = copy.deepcopy(chained)
        mutation(mutated)
        found = violations(new if against_current else old, mutated)
        if not any(expected in v for v in found):
            missed.append(f"{mutation.__name__} (expected {expected!r}; got {found})")
    # Controls: the correct chain passes against the baseline and against the current
    # schema, so the chain mutations above fail for the rule they target, not for noise.
    for label, base in (("baseline", old), ("current", new)):
        found = violations(base, chained)
        if found:
            missed.append(
                f"control chain rejected against {label}: " + "; ".join(found)
            )
    return missed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--ref", help="also compare with the .proto files at this git ref"
    )
    parser.add_argument(
        "--baselines",
        type=Path,
        default=BASELINES,
        help="directory of frozen baselines (default: baselines/)",
    )
    args = parser.parse_args()
    current = schema()
    problems: list[str] = []
    checked = []
    for baseline in sorted(p for p in args.baselines.iterdir() if p.is_dir()):
        old = schema(baseline)
        found = violations(old, current)
        problems += [f"[{baseline.name}] {v}" for v in found]
        problems += [f"[{baseline.name}] {v}" for v in golden_violations(baseline)]
        if not found:
            missed = self_test(old, current)
            problems += [f"[{baseline.name}] self-test missed {m}" for m in missed]
        checked.append(baseline.name)
    # The next contract card freezes the current IDL as a baseline: the gate must pass
    # against itself, self-test included (Codex re-review on PR #36).
    found = violations(current, current)
    problems += [f"[current] {v}" for v in found]
    if not found:
        problems += [
            f"[current] self-test missed {m}" for m in self_test(current, current)
        ]
    checked.append("current")
    if args.ref:
        problems += [
            f"[{args.ref}] {v}" for v in violations(schema_at(args.ref), current)
        ]
        checked.append(args.ref)
    for p in problems:
        print("VIOLATION", p)
    superseded = sum("superseded" in m for m in current["messages"].values())
    print(
        f"Compatibility against {', '.join(checked)}: {len(problems)} violations; "
        f"{len(current['messages'])} messages ({superseded} superseded), "
        f"{len(current['enums'])} enums"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
