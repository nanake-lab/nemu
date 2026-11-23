"""Independent calibration fixtures; I0/I1 fixed before observing any outputs."""
from __future__ import annotations

from dataclasses import asdict, dataclass
from itertools import permutations


@dataclass(frozen=True)
class Case:
    id: str
    family: str
    missing: bool
    rule: str
    visible: str
    evidence: str
    query: str
    yes: str
    no: str
    truth: bool


def cases() -> list[Case]:
    result = []
    for i in range(6):
        missing = i >= 4
        yes_color, no_color = [("amber", "blue"), ("silver", "purple"), ("coral", "jade")][i // 2]
        color = yes_color if i % 2 == 0 else no_color
        threshold = 10 + i
        value = threshold + (-4 if i % 2 == 0 else 4)
        enabled = "yes" if i % 2 == 0 else "no"
        credential = ["badge", "pass", "permit", "coupon", "voucher", "stamp"][i]
        facts = [
            ("sorting", f"Put a {yes_color} tile in LEFT; put a {no_color} tile in RIGHT.",
             f"Current tile color is {color}.", "read the current tile color", "LEFT", "RIGHT", i % 2 == 0),
            ("heating", f"Select WARM if the current temperature is below {threshold}; otherwise select COOL.",
             f"Current temperature is {value}.", "read the current temperature", "WARM", "COOL", value < threshold),
            ("admission", f"Select ENTER if both {credential} and ticket are valid; otherwise select WAIT.",
             f"{credential.capitalize()} valid: yes. Ticket valid: {enabled}.", "read ticket validity", "ENTER", "WAIT", i % 2 == 0),
            ("budget", f"Select WITHIN if the sum of item costs is at most {threshold+2}; otherwise select OVER.",
             f"First item cost is 5. Second item cost is {value}.", "read the second item cost", "WITHIN", "OVER", 5 + value <= threshold+2),
        ]
        for family, rule, evidence, query, yes, no, truth in facts:
            visible = evidence
            if missing:
                visible = {"sorting": "Current tile color is unreported.",
                           "heating": "Current temperature is unreported.",
                           "admission": f"{credential.capitalize()} valid: yes. Ticket validity is unreported.",
                           "budget": "First item cost is 5. Second item cost is unreported."}[family]
            result.append(Case(f"cal-{family}-{i}", family, missing, rule, visible, evidence, query, yes, no, truth))
    return result


LAYOUTS = [(mapping, reverse) for mapping in permutations(("relevant", "irrelevant", "submit")) for reverse in (False, True)]


def prompt(case: Case, layout: int, version: str, final: bool) -> str:
    mapping, reverse = LAYOUTS[layout]
    descriptions = {"relevant": case.query, "irrelevant": "read the decorative border color",
                    "submit": "submit a decision using the visible information"}
    if final:
        answers = (case.no, case.yes) if reverse else (case.yes, case.no)
        entries = [("A", answers[0]), ("B", answers[1])]
        facts = case.evidence
    else:
        entries = [(chr(65 + i), descriptions[action]) for i, action in enumerate(mapping)]
        if reverse:
            entries.reverse()
        facts = case.visible
    listing = "\n".join(f"{label}: {description}" for label, description in entries)
    objective = ("A correct final decision earns 10 points; each read costs 1 point. "
                 "At most one read is available before the final decision.")
    if version == "I0":
        header = "Complete this synthetic task. " + ("All current facts are provided." if final else objective)
        return (f"{header}\n{case.rule} {facts}\n" + ("Final decisions:" if final else "Available next operations:") +
                f"\n{listing}\n" + ("Choose the final decision. Output exactly one letter: A or B." if final else
                                        "Choose the next operation. Output exactly one letter: A, B, or C."))
    if version != "I1":
        raise ValueError("Only the preregistered interfaces are allowed")
    header = "Synthetic rule task. " + ("All current facts are provided." if final else objective)
    return (f"{header}\nRULE: {case.rule}\nCURRENT FACTS: {facts}\n" +
            ("ANSWER CODES:" if final else "OPERATION CODES:") + f"\n{listing}\n" +
            ("Select the answer code that follows the rule and current facts. " if final else "Select the next operation code. ") +
            "The response slot reads exactly one token: " + ("A or B." if final else "A, B, or C.") +
            " Write only that code, with no other text.")


def export() -> dict[str, object]:
    return {"cases": [asdict(c) for c in cases()], "layouts": LAYOUTS, "versions": ["I0", "I1"],
            "prompts": {v: [prompt(c, layout, v, final) for c in cases() for layout in range(12) for final in (False, True)]
                        for v in ("I0", "I1")}}
