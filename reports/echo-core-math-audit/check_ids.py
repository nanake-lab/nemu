"""Acceptance-ID and invariant counts across the converted specifications (read-only).

Spec references: O §18 (I01–I22), O §20 (45 IDs), S §28 (same 45), S §29 (SYS01–SYS25),
S §30 (I01–I22), S §32 ("45 个算子项和 24 个系统项"; W00-C02).
"""

from __future__ import annotations

import re
from pathlib import Path

from common import Check, Result

SPEC = Path(__file__).resolve().parents[2] / "docs" / "echo-core"
ROW = re.compile(r"^\| ([FGCJX]\d\d)\s+\| (.*?)\s+\| (.*?)\s+\|$", re.M)
INV = re.compile(r"^\| (I\d\d)\s+\| (.*?)\s+\|", re.M)


def _section(text: str, start: str, end: str) -> str:
    return text[text.index(start): text.index(end)]


def _norm(s: str) -> str:
    return s.replace("，", "；").replace("stored state", "存储状态").replace(" ", "")


def check_acceptance_ids() -> Result:
    o = (SPEC / "ada-core-operators-spec-v1.0.md").read_text(encoding="utf-8")
    s = (SPEC / "echo-core-system-plan-v1.0.md").read_text(encoding="utf-8")
    o20 = {k: (a, b) for k, a, b in ROW.findall(_section(o, "# 20 可证伪验收矩阵", "# 21 "))}
    s28 = {k: (a, b) for k, a, b in ROW.findall(_section(s, "# 28 四算子验收追踪矩阵", "# 29 "))}
    per_family = {f: sum(k.startswith(f) for k in o20) for f in "FGCJX"}
    text_diffs = [k for k in o20 if _norm(f"{o20[k][0]}；应达到：{o20[k][1]}") != _norm(s28[k][0])]
    inv_o = INV.findall(o)
    inv_s = INV.findall(s)
    inv_diffs = [a for (a, b), (_c, d) in zip(inv_o, inv_s, strict=True) if _norm(b) != _norm(d)]
    sys_ids = sorted(set(re.findall(r"SYS\d\d", s)))
    ok = (len(o20) == 45 and set(o20) == set(s28) and not text_diffs and len(inv_o) == 22
          and not inv_diffs and len(sys_ids) == 25 and "24 个系统项" in s)
    return Result(
        "IDS-1",
        "O §18/§20 vs S §28/§30; S §29 vs S §32 (W00-C02, already resolved as a typo)",
        "property",
        ok,
        f"O §20 has {len(o20)} IDs {per_family}, same set and text as S §28 up to punctuation "
        f"(diffs: {text_diffs or 'none'}); I01–I22: {len(inv_o)} in O, {len(inv_s)} in S, "
        f"wording diffs: {inv_diffs or 'none'}; SYS IDs {sys_ids[0]}–{sys_ids[-1]} = {len(sys_ids)} "
        f"while S §32 says 24",
    )


CHECKS: list[Check] = [check_acceptance_ids]
