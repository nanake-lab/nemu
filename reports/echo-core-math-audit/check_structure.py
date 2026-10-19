"""Structural properties: single-event order DAG, meta-influence acyclicity, lineage relations, Γ.

Spec references: F §4–§5, §13.1, §16.1, §16.10; O §4, §7.1, §17.1, §18 (无同事件环); X06; G06; I10.
"""

from __future__ import annotations

import random
from dataclasses import dataclass

from common import Check, Result

SEED = 20261026

# O §4 single-event order: (stage, reads, writes). Snapshot k provides *_k and the raw record.
ORDER: list[tuple[str, set[str], set[str]]] = [
    ("Gamma", {"raw", "U_k"}, {"r", "U_next"}),
    ("Phi", {"Delta", "x_k", "R_k", "theta_k"}, {"x_bar", "R_bar"}),
    ("H_next", {"H_k", "r"}, {"H_next"}),
    ("V", {"H_k", "r"}, {"c_core"}),
    ("IRL", {"r", "c_core"}, {"z"}),
    ("F_in", {"theta_k", "x_bar", "R_bar", "z"}, {"x_tilde"}),
    ("G", {"theta_k", "R_bar", "x_tilde", "r", "z"}, {"R_next"}),
    ("C", {"theta_k", "R_next", "H_next", "L_k"}, {"theta_next", "L_next"}),
    ("F_out", {"theta_next", "x_tilde", "R_next", "r", "z"}, {"x_next"}),
    ("commit", {"H_next", "x_next", "R_next", "theta_next", "L_next", "U_next"}, set()),
    ("J", {"theta_next", "x_next", "R_next"}, {"m"}),
]
SNAPSHOT = {"raw", "U_k", "Delta", "x_k", "R_k", "theta_k", "H_k", "L_k"}


def order_violations(order: list[tuple[str, set[str], set[str]]]) -> list[str]:
    available = set(SNAPSHOT)
    bad = []
    for stage, reads, writes in order:
        missing = reads - available
        if missing:
            bad.append(f"{stage} reads {sorted(missing)} before it is produced")
        clobber = writes & available
        if clobber:
            bad.append(f"{stage} rewrites {sorted(clobber)}")
        available |= writes
    if sum(1 for s, _, _ in order if s == "Phi") != 1:
        bad.append("time advanced more than once")
    return bad


def check_event_order_dag() -> Result:
    ok_order = order_violations(ORDER)
    swapped = list(ORDER)
    gi = next(i for i, s in enumerate(swapped) if s[0] == "G")
    swapped[gi], swapped[gi + 1] = swapped[gi + 1], swapped[gi]
    swap_bad = order_violations(swapped)
    twice = ORDER[:9] + [("Phi", {"Delta", "x_tilde", "R_next", "theta_next"}, {"x_bar"})] + ORDER[9:]
    twice_bad = order_violations(twice)
    g_reads_new_theta = "theta_next" in next(r for s, r, _ in ORDER if s == "G")
    return Result(
        "ORD-1",
        "O §4 / §18 无同事件环; F §5; X06 (swap G/C, repeated Φ)",
        "property",
        not ok_order and bool(swap_bad) and bool(twice_bad) and not g_reads_new_theta,
        f"spec order: {len(ok_order)} violations (every read is from snapshot k or an earlier stage; "
        f"G reads θ_k). Swapping G/C → {swap_bad[0]!r}; second Φ → {twice_bad[-1]!r}",
    )


def simulate_meta(rng: random.Random, transitive: bool, steps: int, nodes: int) -> bool:
    """Return True if some mechanism ends up in its own (true, transitive) ancestry.

    true_anc[t] is the full revision-level ancestry of t's current revision (inherited across
    revisions, O §21). The exclusion rule of O §7.1 is applied either with that transitive set or,
    for contrast, with only the direct modulators recorded on t.
    """
    true_anc: list[set[int]] = [set() for _ in range(nodes)]
    direct: list[set[int]] = [set() for _ in range(nodes)]
    for _ in range(steps):
        m, t = rng.randrange(nodes), rng.randrange(nodes)
        rule = true_anc[m] if transitive else direct[m]
        if t == m or t in rule:  # O §7.1 self / ancestor exclusion (distinct lineages here)
            continue
        true_anc[t] |= {m} | true_anc[m]
        direct[t].add(m)
        if t in true_anc[t]:
            return True  # t's revision was shaped, through others, by t itself
    return False


def check_meta_acyclic() -> Result:
    rng = random.Random(SEED)
    loops_transitive = sum(simulate_meta(rng, True, 60, 8) for _ in range(3000))
    loops_direct = sum(simulate_meta(rng, False, 60, 8) for _ in range(3000))
    # Deterministic A→B, B→C, C→A under direct-parent exclusion.
    direct: list[set[int]] = [set(), set(), set()]
    allowed = []
    for m, t in ((0, 1), (1, 2), (2, 0)):
        ok = t != m and t not in direct[m]
        allowed.append(ok)
        if ok:
            direct[t].add(m)
    return Result(
        "META-1",
        "O §7.1 / §21 meta exclusion of self + ancestors; I10; G06 (A→B→A)",
        "property",
        loops_transitive == 0 and loops_direct > 0 and all(allowed),
        f"3000 random runs: self-ancestry feedback loops with transitive ancestry = "
        f"{loops_transitive}; with direct-parent exclusion only = {loops_direct} "
        f"(A→B→C→A allowed: {all(allowed)})",
    )


@dataclass
class Ckpt:
    parent: int | None
    branch: str
    recorded: bool = True


def build_tree(rng: random.Random, n: int) -> list[Ckpt]:
    nodes = [Ckpt(None, "b0"), Ckpt(None, "c0")]  # two independent origins
    nb = 1
    for _ in range(n):
        p = rng.randrange(len(nodes))
        children = [c for c in nodes if c.parent == p]
        if children:  # second child of the same parent is a fork → new branch id
            nb += 1
            branch = f"b{nb}"
        else:
            branch = nodes[p].branch
        nodes.append(Ckpt(p, branch, rng.random() > 0.05))
    return nodes


def chain(nodes: list[Ckpt], j: int) -> list[int]:
    out: list[int] = [j]
    while nodes[out[-1]].parent is not None:
        out.append(nodes[out[-1]].parent)  # type: ignore[arg-type]
    return out


def valid_cont(nodes: list[Ckpt], i: int, j: int) -> bool:
    return i in chain(nodes, j)


def same_lineage(nodes: list[Ckpt], i: int, j: int) -> bool:
    return chain(nodes, i)[-1] == chain(nodes, j)[-1]


def verifier(nodes: list[Ckpt], i: int, j: int) -> str:
    """F §13.1: TRUE / FALSE / UNKNOWN; missing records on the path yield UNKNOWN."""
    path = chain(nodes, j)
    if i in path:
        seg = path[: path.index(i) + 1]
        return "TRUE" if all(nodes[k].recorded for k in seg) else "UNKNOWN"
    return "FALSE" if all(nodes[k].recorded for k in path) else "UNKNOWN"


def check_lineage_relations() -> Result:
    rng = random.Random(SEED + 1)
    bad = 0
    forks_checked = 0
    for _ in range(200):
        nodes = build_tree(rng, 25)
        n = len(nodes)
        for i in range(n):
            for j in range(n):
                sl, vc = same_lineage(nodes, i, j), valid_cont(nodes, i, j)
                bad += sl != same_lineage(nodes, j, i)
                bad += vc and valid_cont(nodes, j, i) and i != j
                bad += vc and not sl
                v = verifier(nodes, i, j)
                bad += (v == "TRUE" and not vc) or (v == "FALSE" and vc)
                p_i, p_j = nodes[i].parent, nodes[j].parent
                if i != j and p_i is not None and p_i == p_j:
                    forks_checked += 1
                    bad += vc or valid_cont(nodes, j, i)
                    bad += nodes[i].branch == nodes[j].branch
        for _t in range(300):
            a, b, c = rng.randrange(n), rng.randrange(n), rng.randrange(n)
            bad += same_lineage(nodes, a, b) and same_lineage(nodes, b, c) and not same_lineage(nodes, a, c)
            bad += valid_cont(nodes, a, b) and valid_cont(nodes, b, c) and not valid_cont(nodes, a, c)
    return Result(
        "LIN-1",
        "F §13.1 SameLineage / ValidContinuation; F §16.10; SYS16",
        "property",
        bad == 0 and forks_checked > 0,
        f"200 random checkpoint trees (forks get new branch ids, 5% records missing): {bad} "
        f"violations; SameLineage is an equivalence, ValidContinuation a partial order, "
        f"{forks_checked} sibling-fork pairs are never continuations; verifier never misreports",
    )


def gamma(record: tuple[str, str], u: dict[str, str]) -> tuple[tuple[str, str] | None, str]:
    rid, payload = record
    if rid in u:
        return None, "duplicate" if u[rid] == payload else "conflict"
    u[rid] = payload
    return record, "accepted"


def project(d: list[tuple[str, str]]) -> tuple[list[tuple[str, str]], list[str]]:
    u: dict[str, str] = {}
    h: list[tuple[str, str]] = []
    decisions = []
    for rec in d:
        r, why = gamma(rec, u)
        decisions.append(why)
        if r is not None:
            h.append(r)
    return h, decisions


def check_ingress() -> Result:
    d = [("e1", "a"), ("e2", "b"), ("e2", "b")]
    h, dec = project(d)
    first = h == [("e1", "a"), ("e2", "b")] and dec[-1] == "duplicate" and len(dec) == 3
    rng = random.Random(SEED + 2)
    bad = 0
    for _ in range(2000):
        raw = [(f"e{rng.randint(0, 8)}", rng.choice("xyz")) for _ in range(rng.randint(0, 30))]
        h_all, _ = project(raw)
        bad += project(raw)[0] != h_all  # deterministic replay
        bad += len({r for r, _ in h_all}) != len(h_all)  # one entry per stable id
        for k in range(len(raw) + 1):  # H_(k+1) = H_k ⊕ r_k on prefixes
            hk, _ = project(raw[:k])
            if k < len(raw):
                hk1, dk = project(raw[: k + 1])
                bad += hk1 != (hk + [raw[k]] if dk[-1] == "accepted" else hk)
        firsts: dict[str, str] = {}
        for rid, p in raw:
            firsts.setdefault(rid, p)
        bad += dict(h_all) != firsts  # conflicts never overwrite the first payload
    return Result(
        "ING-1",
        "F §4 Γ / Π_Γ; F §5 filtered commit; F §16.1 D=(e1,e2,e2); SYS01",
        "property",
        first and bad == 0,
        f"16.1 case: H={h}, decisions={dec}, ℓ=3; 2000 random raw logs: {bad} violations of "
        "replay determinism, id uniqueness, prefix recursion or no-overwrite",
    )


CHECKS: list[Check] = [
    check_event_order_dag,
    check_meta_acyclic,
    check_lineage_relations,
    check_ingress,
]
