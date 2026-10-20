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
    return out + discipline(new)


def discipline(s: Schema) -> list[str]:
    """Rules 5 and 6, which hold within a single schema."""
    out: list[str] = []
    messages = s["messages"]
    for name, m in messages.items():
        predecessor = m.get("supersedes")
        if predecessor:
            p = messages.get(predecessor, {})
            if p.get("superseded", {}).get("by") != name:
                out.append(f"{name} supersedes {predecessor}, which does not name it")
            kept = {f["number"]: f for f in m["fields"]}
            numbers = {f["number"] for f in p.get("fields", [])} | set(
                p.get("reserved", [])
            )
            for f in p.get("fields", []):
                if f["number"] in kept and kept[f["number"]] != f:
                    out.append(
                        f"{name} field {f['number']} reuses {predecessor}.{f['name']} differently"
                    )
                if f["number"] not in kept and f["number"] not in m.get("reserved", []):
                    out.append(f"{name} must reserve {predecessor} field {f['number']}")
            for number in kept:
                if number not in numbers and number <= max(numbers, default=0):
                    out.append(
                        f"{name} new field {number} is not above {predecessor}'s numbers"
                    )
        if "superseded" in m:
            continue
        for f in m["fields"]:
            t = f["type"]
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


def self_test(old: Schema, new: Schema) -> list[str]:
    """Each mutation must be caught; returns the names of mutations that were not."""
    assert not violations(old, new), "self-test needs a clean schema"

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
        s["messages"]["RSnapshot"]["tag"] = "echo.fast-state"

    def reuse_reserved(s: Schema) -> None:
        f = copy.deepcopy(field(s, "VersionedEnvironmentV2", "irl"))
        s["messages"]["VersionedEnvironmentV2"]["fields"].append(
            f | dict(name="scope_v2", number=4)
        )
        s["messages"]["VersionedEnvironmentV2"]["reserved"].remove(4)

    def unreserve(s: Schema) -> None:
        s["messages"]["FastStateSnapshotV2"]["reserved"] = [4]

    def low_number(s: Schema) -> None:
        field(s, "ProposalBindingV2", "environment")["number"] = 16
        s["messages"]["ProposalBindingV2"]["reserved"] = []

    def embed_old(s: Schema) -> None:
        field(s, "RSnapshot", "law_version")["type"] = "EffectContract"

    def unmark(s: Schema) -> None:
        del s["messages"]["EffectContract"]["superseded"]

    def unmark_value(s: Schema) -> None:
        del s["superseded_values"]["RunStatus"]["AX_DEGRADED"]

    missed = []
    for mutation in (
        renumber,
        retype,
        recycle_enum,
        drop_enum_value,
        reuse_tag,
        reuse_reserved,
        unreserve,
        low_number,
        embed_old,
        unmark,
        unmark_value,
    ):
        mutated = copy.deepcopy(new)
        mutation(mutated)
        # unmark/unmark_value are only violations against a baseline that already had the mark.
        base = new if mutation in (unmark, unmark_value) else old
        if not violations(base, mutated):
            missed.append(mutation.__name__)
    return missed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--ref", help="also compare with the .proto files at this git ref"
    )
    args = parser.parse_args()
    current = schema()
    problems: list[str] = []
    checked = []
    for baseline in sorted(p for p in BASELINES.iterdir() if p.is_dir()):
        old = schema(baseline)
        found = violations(old, current)
        problems += [f"[{baseline.name}] {v}" for v in found]
        problems += [f"[{baseline.name}] {v}" for v in golden_violations(baseline)]
        if not found:
            missed = self_test(old, current)
            problems += [f"[{baseline.name}] self-test missed {m}" for m in missed]
        checked.append(baseline.name)
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
