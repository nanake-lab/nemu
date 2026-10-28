"""Frozen synthetic inputs; no user data, personality instructions, or live tools."""
from __future__ import annotations

from dataclasses import asdict, dataclass

CONTRASTS = [
    ("A historian encounters a date with two conflicting citations.", "She consults the original manuscript before concluding.", "She adopts the first citation without resolving the conflict."),
    ("A translator encounters an unfamiliar phrase with two possible meanings.", "He examines its source passage before interpreting it.", "He selects the first meaning without examining its source passage."),
    ("A curator receives a painting with an uncertain attribution.", "She examines the provenance records before assigning an artist.", "She assigns an artist without examining the provenance records."),
    ("An astronomer notices a faint unexplained point in an image.", "He compares a second exposure before classifying the point.", "He classifies the point without comparing a second exposure."),
    ("A botanist sees a leaf whose species is ambiguous.", "She examines its vein pattern before identifying the species.", "She identifies the species without examining its vein pattern."),
    ("An editor finds two versions of a quotation.", "He opens the cited original before choosing the wording.", "He chooses the wording without opening the cited original."),
    ("An archaeologist finds an object of uncertain age.", "She examines the excavation record before dating the object.", "She dates the object without examining the excavation record."),
    ("A musician hears a note that could come from two instruments.", "He listens to the isolated recording before identifying the instrument.", "He identifies the instrument without listening to the isolated recording."),
    ("A geologist finds a mineral that resembles two known types.", "She measures its hardness before naming the mineral.", "She names the mineral without measuring its hardness."),
    ("A lexicographer finds an ambiguous usage in an old letter.", "He reads the surrounding paragraph before defining the usage.", "He defines the usage without reading the surrounding paragraph."),
    ("A photographer sees an unexpected patch of color.", "She examines the original negative before explaining the patch.", "She explains the patch without examining the original negative."),
    ("A conservator finds an unfamiliar surface on an antique.", "He examines a material sample before describing the surface.", "He describes the surface without examining a material sample."),
]


@dataclass(frozen=True)
class Case:
    id: str
    family: str
    missing: bool
    context: str
    relevant: str
    evidence: str
    answer_a: str
    answer_b: str
    correct: str
    hidden: int


def cases() -> list[Case]:
    result: list[Case] = []
    for i in range(6):
        missing = i < 4
        threshold, actual = 8 + i, (8 + i) + (-2 if i % 2 == 0 else 2)
        hint = (8 + i) + (1 if i % 2 == 0 else -1)
        observed = f"The old estimate was {hint}; current count is unreported." if missing else f"The current count is {actual}, already confirmed."
        result.append(Case(f"stock{i}", "stock", missing,
            f"Bin S{i}: restock if and only if current count is below {threshold}. {observed}",
            "read the current bin count", f"Current count = {actual}.", "RESTOCK", "KEEP", "A" if actual < threshold else "B", actual))
        base, extra = 14 + i, 2 if i % 2 == 0 else 8
        observed = "The prior delay estimate is 5; today's delay is unreported." if missing else f"Today's delay is {extra}, already confirmed."
        result.append(Case(f"route{i}", "route", missing,
            f"For parcel R{i}, choose the faster route. North takes {base} minutes plus today's delay. South takes {base + 5} minutes. {observed}",
            "read today's North delay", f"Today's North delay = {extra} minutes.", "NORTH", "SOUTH", "A" if extra < 5 else "B", extra))
        bit = i % 2
        observed = f"The old diagnostic was {1-bit}; current diagnostic is unreported." if missing else f"The current diagnostic is {bit}, already confirmed."
        result.append(Case(f"device{i}", "device", missing,
            f"For unit D{i}, select RESET if the current diagnostic is 1; otherwise select RESEAT. {observed}",
            "read the current diagnostic", f"Current diagnostic = {bit}.", "RESET", "RESEAT", "A" if bit == 1 else "B", bit))
    return result


def tools(case: Case, order: int) -> list[tuple[str, str]]:
    choices = [("relevant", case.relevant), ("irrelevant", "read the decorative badge color"), ("submit", "submit a decision using only the visible information")]
    return choices if order == 0 else list(reversed(choices))


def choice_prompt(case: Case, order: int) -> str:
    listing = "\n".join(f"{chr(65+i)}: {description}" for i, (_, description) in enumerate(tools(case, order)))
    return ("Complete this synthetic task. A correct final decision earns 10 points; each read costs 1 point. "
            "You can use at most one read before the final decision.\n" + case.context +
            "\nAvailable next operations:\n" + listing + "\nChoose the next operation. Output exactly one letter: A, B, or C.")


def final_prompt(case: Case, action: str) -> str:
    returned = case.evidence if action == "relevant" else "Decorative badge color = violet." if action == "irrelevant" else "No read was performed."
    return ("Complete this synthetic task using the visible information and the executed operation result.\n" + case.context +
            "\nExecuted operation result: " + returned + f"\nFinal decisions:\nA: {case.answer_a}\nB: {case.answer_b}" +
            "\nOutput exactly one letter: A or B.")


def export() -> dict[str, object]:
    return {"contrasts": CONTRASTS, "cases": [asdict(c) for c in cases()],
            "orders": [0, 1], "seeds": [0, 11, 29, 47], "temperature": 0.7,
            "amplitudes": [1.0, 4.0], "random_directions": [1907, 2718]}
