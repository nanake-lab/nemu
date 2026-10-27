"""Generate descriptors and language types from the annotated Protobuf IDL.

No runtime semantics are invented by this generator. Annotations implement the
approved ADR 0007 presence/order/validation policy. Run --check in CI.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


SUPERSEDED = r"@superseded (\S+) \| ([^\n]+)"


def superseded(by: str, reason: str) -> dict[str, str]:
    """Supersession metadata only; it never changes a field table or its bytes."""
    assert reason.strip(), "supersession needs a reason"
    return dict(by=by, reason=reason.strip())


def schema(directory: Path = HERE) -> dict[str, Any]:
    messages: dict[str, Any] = {}
    enums: dict[str, Any] = {}
    superseded_enums: dict[str, Any] = {}
    superseded_values: dict[str, Any] = {}
    json_enums: dict[str, Any] = {}
    for path in sorted(directory.glob("*.proto")):
        text = path.read_text(encoding="utf-8")
        for comments, name, body in re.findall(
            r"((?:^//[^\n]*\n)*)^enum (\w+)\s*\{([^}]+)\}", text, re.MULTILINE
        ):
            assert name not in enums, name
            code = re.sub(r"//[^\n]*", "", body)
            enums[name] = {k: int(v) for k, v in re.findall(r"(\w+)\s*=\s*(\d+)", code)}
            aliases = re.findall(r"// @json-values ([^\n]+)", comments)
            if aliases:
                mapping = json.loads(aliases[0])
                assert set(mapping) == {k for k, v in enums[name].items() if v}
                assert len(set(mapping.values())) == len(mapping)
                json_enums[name] = mapping
            for line in comments.splitlines():
                found = re.fullmatch(r"// " + SUPERSEDED, line)
                if found:
                    superseded_enums[name] = superseded(*found.groups())
            for value, by, reason in re.findall(
                r"(\w+)\s*=\s*\d+;[ \t]*// " + SUPERSEDED, body
            ):
                assert value in enums[name], (name, value)
                superseded_values.setdefault(name, {})[value] = superseded(by, reason)
        for tag, source, name, body in re.findall(
            r"// @message (\S+) \| ([^\n]+)\nmessage (\w+) \{(.*?)\n\}", text, re.DOTALL
        ):
            # ADR 0007 §6: explicit oneof membership; no change to existing fields.
            groups: dict[str, Any] = {}

            def expand_oneof(match: re.Match[str]) -> str:
                group, requirement, members = match.groups()
                assert requirement == "required", "only required oneofs are used"
                names = re.findall(r"^\s*\w+ (\w+) = ", members, re.MULTILINE)
                assert names, group
                groups[group] = dict(fields=names, required=True)
                return re.sub(
                    r"^(\s*)(\w+ \w+ = \d+; // optional[^\n]*)$",
                    lambda m: m[1] + "optional " + m[2] + ",oneof=" + group,
                    members,
                    flags=re.MULTILINE,
                )

            body = re.sub(
                r"oneof (\w+) \{ // (required)\n(.*?)^  \}",
                expand_oneof,
                body,
                flags=re.DOTALL | re.MULTILINE,
            )
            # P5-01 retrospective C1(b,c): fail closed instead of silently dropping
            # a valid Protobuf declaration from the authority descriptor.
            for line in body.splitlines():
                code = line.strip()
                if not code or code.startswith("//"):
                    continue
                if "=" in code:
                    assert re.fullmatch(
                        r"(?:optional|repeated) \w+ \w+ = \d+(?: \[[^\]]+\])?; // [^\n]+",
                        code,
                    ), (name, "unparsed declaration", code)
                if code.startswith("reserved"):
                    assert re.fullmatch(r"reserved [\d, ]+;", code), (
                        name,
                        "unparsed reserved",
                        code,
                    )
                assert not code.startswith(("oneof", "map<")), (
                    name,
                    "unparsed syntax",
                    code,
                )
            fields = []
            for label, typ, field, number, annotation in re.findall(
                r"(optional|repeated) (\w+) (\w+) = (\d+)(?: \[[^\]]+\])?; // ([^\n]+)",
                body,
            ):
                options = annotation.split(",")
                f: dict[str, Any] = dict(
                    name=field,
                    type=typ,
                    number=int(number),
                    repeated=label == "repeated",
                    required="required" in options,
                    nonempty="nonempty" in options,
                )
                for item in options:
                    if "=" in item:
                        key, value = item.split("=", 1)
                        f[key] = (
                            value
                            if key in ("set", "oneof", "json_name", "json_map")
                            else float(value)
                            if key in ("min", "max")
                            else int(value)
                        )
                fields.append(f)
            assert fields and fields[0]["name"] == "schema_version"
            assert [f["number"] for f in fields] == sorted(
                f["number"] for f in fields
            ), (name, "field number order")
            assert len({f["number"] for f in fields}) == len(fields)
            assert name not in messages
            rules = re.findall(r"// @rules ([^\n]+)", body)
            messages[name] = dict(
                tag=tag,
                source=source,
                fields=fields,
                file=path.name,
                rules=json.loads(rules[0]) if rules else {},
            )
            retained = re.findall(r"// @retains (\w+)", body)
            if retained:
                messages[name]["retains"] = retained[0]
            if "// @transient" in body:
                messages[name]["transient"] = True
            if re.search(r"// @resource\s*$", body, re.MULTILINE):
                messages[name]["resource"] = True
            if messages[name]["rules"].get("cases"):
                discriminator = messages[name]["rules"].get("discriminator", "kind")
                assert any(
                    f["name"] == discriminator and f["required"] and not f["repeated"]
                    for f in fields
                )
            if groups:
                messages[name]["rules"]["oneofs"] = groups
                for group in groups.values():
                    positions = [
                        i for i, f in enumerate(fields) if f["name"] in group["fields"]
                    ]
                    assert positions == list(range(min(positions), max(positions) + 1))
            # Lifecycle metadata (P5-21). Keys are added only when present, so
            # unchanged messages keep byte-identical descriptors.
            reserved = sorted(
                int(n)
                for line in re.findall(r"^\s*reserved ([\d, ]+);", body, re.MULTILINE)
                for n in line.split(",")
            )
            assert not {f["number"] for f in fields} & set(reserved), name
            gone = re.findall(r"// " + SUPERSEDED, body)
            successor = re.findall(r"// @supersedes (\w+)", body)
            assert len(gone) <= 1 and len(successor) <= 1, name
            if gone:
                messages[name]["superseded"] = superseded(*gone[0])
            if successor:
                messages[name]["supersedes"] = successor[0]
            if reserved:
                messages[name]["reserved"] = reserved
    assert len({m["tag"] for m in messages.values()}) == len(messages)
    scalars = {"uint32", "uint64", "sint64", "double", "bool", "string", "bytes"}
    for m in messages.values():
        # Numeric cross-field rules are explicit IDL metadata, on required doubles only.
        numeric = {
            f["name"]
            for f in m["fields"]
            if f["type"] == "double" and f["required"] and not f["repeated"]
        }
        assert set(m["rules"].get("positive", [])) <= numeric
        absolute = m["rules"].get("at_least_abs", {})
        assert set(absolute) | set(absolute.values()) <= numeric
        for f in m["fields"]:
            assert f["type"] in scalars | messages.keys() | enums.keys(), f
        by = m.get("superseded", {}).get("by", "-")
        assert by == "-" or by in messages, by
        assert m.get("supersedes", next(iter(messages))) in messages
    for enum, by in superseded_enums.items():
        assert by["by"] == "-" or by["by"] in enums, (enum, by)
    for enum, values in superseded_values.items():
        for value, by in values.items():
            assert by["by"] == "-" or by["by"] in enums[enum], (enum, value, by)
    result: dict[str, Any] = dict(messages=messages, enums=enums)
    if json_enums:
        result["json_enums"] = json_enums
    if superseded_enums:
        result["superseded_enums"] = superseded_enums
    if superseded_values:
        result["superseded_values"] = superseded_values
    return result


def outputs(s: dict[str, Any]) -> dict[Path, str]:
    result = {
        HERE / "generated/schema.json": json.dumps(s, ensure_ascii=False, indent=2)
        + "\n"
    }
    ts = [
        "// Generated by core/contracts/generate.py; DO NOT EDIT.",
        'import type { Schema } from "./runtime.js";',
        "export const schema: Schema = " + json.dumps(s, ensure_ascii=False) + ";",
    ]
    rust = [
        "// Generated by core/contracts/generate.py; DO NOT EDIT.",
        "use super::scalar::{DecimalU64, DecimalI64, HexBytes};",
    ]
    py = [
        "# Generated by core/contracts/generate.py; DO NOT EDIT.",
        "from __future__ import annotations",
        "from dataclasses import dataclass",
        "from enum import IntEnum",
    ]

    def note(meta: dict[str, str]) -> str:
        by = "no successor" if meta["by"] == "-" else f"successor {meta['by']}"
        return f"Superseded ({by}): {meta['reason']}"

    for name, values in s["enums"].items():
        valid = {k: v for k, v in values.items() if v != 0}
        whole = s.get("superseded_enums", {}).get(name)
        gone = s.get("superseded_values", {}).get(name, {})
        if whole:
            ts += [f"/** @deprecated {note(whole)} */"]
            rust += [f"/// {note(whole)}"]
            py += [f"# {note(whole)}"]
        ts += [
            f"export type {name} = " + " | ".join(str(v) for v in valid.values()) + ";"
        ]
        py += [f"class {name}(IntEnum):"]
        for k, v in valid.items():
            py += ([f"    # {note(gone[k])}"] if k in gone else []) + [f"    {k} = {v}"]
        rust += [
            "#[allow(non_camel_case_types)]",
            "#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]",
            '#[serde(try_from = "u32", into = "u32")]',
            f"pub enum {name} {{",
        ]
        for k, v in valid.items():
            rust += ([f"    /// {note(gone[k])}"] if k in gone else []) + [
                f"    {k} = {v},"
            ]
        rust += ["}"]
        rust += [
            f"impl TryFrom<u32> for {name} {{ type Error = String; fn try_from(v:u32)->Result<Self,String>{{match v {{"
        ]
        rust += [f"{v} => Ok(Self::{k})," for k, v in valid.items()] + [
            '_ => Err("unknown enum".into()), }}}'
        ]
        rust += [f"impl From<{name}> for u32 {{fn from(v:{name})->Self{{v as u32}}}}"]
        body = (
            "true"
            if whole
            else "matches!(self, " + " | ".join(f"Self::{k}" for k in gone) + ")"
            if gone
            else "false"
        )
        rust += [
            f"impl {name} {{",
            "    /// Superseded values stay decodable for old data and tests; producers must not emit them (P5-21).",
            f"    pub const fn is_superseded(self) -> bool {{ {body} }}",
            "}",
        ]
    ts_types = dict(
        uint32="number",
        uint64="bigint",
        sint64="bigint",
        double="number",
        bool="boolean",
        string="string",
        bytes="Uint8Array",
    )
    rs_types = dict(
        uint32="u32",
        uint64="DecimalU64",
        sint64="DecimalI64",
        double="f64",
        bool="bool",
        string="String",
        bytes="HexBytes",
    )
    py_types = dict(
        uint32="int",
        uint64="int",
        sint64="int",
        double="float",
        bool="bool",
        string="str",
        bytes="bytes",
    )
    for name, m in s["messages"].items():
        gone = m.get("superseded")
        lifecycle = (
            f" @deprecated {note(gone)}"
            if gone
            else f" Supersedes {m['supersedes']}."
            if "supersedes" in m
            else ""
        )
        ts += [
            f"/** {m['source']}; frozen tag {m['tag']}.{lifecycle} */",
            f"export interface {name} {{",
        ]
        rust += [
            f"/// {m['source']}; frozen tag `{m['tag']}`."
            + (f" {note(gone)}" if gone else "")
            + (f" Supersedes [`{m['supersedes']}`]." if "supersedes" in m else ""),
            "#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]"
            if m.get("transient")
            else "#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]",
            "#[serde(deny_unknown_fields)]",
            f"pub struct {name} {{",
        ]
        py += [
            "@dataclass(frozen=True, repr=False)"
            if m.get("transient")
            else "@dataclass(frozen=True)",
            f"class {name}:",
        ]
        if gone:
            py += [f"    # {note(gone)}"]
        for f in m["fields"]:
            typ = f["type"]
            optional = not f["required"] and not f["repeated"]
            t = ts_types.get(typ, typ)
            r = rs_types.get(typ, typ)
            p = py_types.get(typ, typ)
            if f["repeated"]:
                t += "[]"
                r = f"Vec<{r}>"
                p = f"list[{p}]"
            if optional:
                r = f"Option<{r}>"
                p += " | None"
            ts += [f"  {f['name']}{'?' if optional else ''}: {t};"]
            if optional:
                rust += ['#[serde(default, skip_serializing_if = "Option::is_none")]']
            if typ == "double":
                helper = (
                    "finite_vec"
                    if f["repeated"]
                    else "finite_option"
                    if optional
                    else "finite"
                )
                rust += [f'#[serde(serialize_with = "super::scalar::{helper}")]']
            rust += [f"pub r#{f['name']}: {r},"]
            pname = f["name"] + "_" if f["name"] in ("from",) else f["name"]
            py += [f"    {pname}: {p}"]
        ts += ["}"]
        rust += ["}"]
        if m.get("transient"):
            rust += [
                f'impl std::fmt::Debug for {name} {{fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {{f.write_str("{name}([REDACTED])")}}}}'
            ]
        rust += [
            f'impl super::WireMessage for {name} {{const NAME: &\'static str = "{name}";'
            + (" const SUPERSEDED: bool = true;" if gone else "")
            + (" const PERSISTABLE: bool = false;" if m.get("transient") else "")
            + "}"
        ]
    ts += (
        ["export interface MessageMap {"]
        + [f"{n}: {n};" for n in s["messages"]]
        + ["}"]
    )
    rust += [
        "/// Conformance dispatch through every generated DTO and both typed codecs.",
        "pub fn verify_typed(name: &str, value: &serde_json::Value, limits: super::contract::Limits) -> Result<(), String> {match name {",
    ]
    rust += [
        f'"{n}" => super::verify_typed::<{n}>(value, limits),' for n in s["messages"]
    ]
    rust += ['_ => Err("unknown message".into()), }}']
    rust += [
        "/// Message types kept only to read old data and for tests; no runtime path may produce them.",
        "pub const SUPERSEDED_MESSAGES: &[&str] = &["
        + ", ".join(f'"{n}"' for n, m in s["messages"].items() if "superseded" in m)
        + "];",
    ]
    result[ROOT / "packages/contracts/src/generated/echo/schema.ts"] = (
        "\n".join(ts) + "\n"
    )
    result[ROOT / "core/crates/core-types/src/generated.rs"] = "\n".join(rust) + "\n"
    for worker in ("irl", "drive", "model-adapter"):
        out = ROOT / f"workers/{worker}/generated/echo"
        result[out / "types.py"] = "\n".join(py) + "\n"
        result[out / "schema.json"] = json.dumps(s, ensure_ascii=False, indent=2) + "\n"
        result[out / "__init__.py"] = (
            "# Generated Echo Core contract package; no runtime state.\n"
        )
        for template in (HERE / "templates/python").glob("*.py"):
            result[out / template.name] = template.read_text(encoding="utf-8")
    for template in (HERE / "templates/typescript").glob("*.ts"):
        result[ROOT / "packages/contracts/src/generated/echo" / template.name] = (
            template.read_text(encoding="utf-8")
        )
    from generate_browser import resource_types

    result[ROOT / "packages/contracts/src/generated/echo/resources.ts"] = (
        resource_types(s)
    )
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    changed = []
    for path, content in outputs(schema()).items():
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            changed.append(str(path.relative_to(ROOT)))
            if not args.check:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8", newline="\n")
    if args.check and changed:
        raise SystemExit("Stale generated files: " + ", ".join(changed))
    print(f"{len(changed)} generated files {'stale' if args.check else 'updated'}")


if __name__ == "__main__":
    main()
