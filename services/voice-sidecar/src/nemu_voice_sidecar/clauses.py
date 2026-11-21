"""Deterministic, reproducible clause splitting on punctuation for streaming TTS."""
from __future__ import annotations

import re

# Sentence and clause punctuation; keep repeated ellipses/dashes and trailing closing quotes attached.
_SPLIT_RE = re.compile(
    r"(?:(?<!\d)\.(?!\d)|[，。？！；：、…,\?!;:]|……|\.\.\.|——)+[”’\"'）\)】\]》」』]*"
)
_MIN_CLAUSE_CHARS = 5


def _has_speakable_text(segment: str) -> bool:
    return bool(re.search(r"[\w\u4e00-\u9fff]", segment))


def _non_space_len(segment: str) -> int:
    return sum(1 for ch in segment if not ch.isspace())


def split_clauses(text: str, *, min_chars: int = _MIN_CLAUSE_CHARS) -> list[str]:
    """Split text into deterministic speakable clauses at punctuation boundaries.

    Short fragments (< min_chars non-whitespace characters) are merged with adjacent
    clauses so split points stay stable and match GPT-SoVITS pre-segmentation.
    Single-clause inputs with no internal punctuation return ``[text.strip()]``.
    """
    cleaned = text.strip()
    if not cleaned:
        return []

    raw_parts: list[str] = []
    pos = 0
    for match in _SPLIT_RE.finditer(cleaned):
        end = match.end()
        part = cleaned[pos:end].strip()
        if part:
            raw_parts.append(part)
        pos = end
    tail = cleaned[pos:].strip()
    if tail:
        raw_parts.append(tail)

    if not raw_parts:
        return [cleaned]

    # Attach punctuation-only fragments to the preceding part; drop leading orphan punctuation.
    speakable: list[str] = []
    for part in raw_parts:
        if _has_speakable_text(part):
            speakable.append(part)
        elif speakable:
            speakable[-1] = f"{speakable[-1]}{part}"

    if len(speakable) <= 1:
        return speakable or [cleaned]

    # Deterministically merge short fragments (< min_chars) matching upstream merge_short_text_in_array.
    merged: list[str] = []
    pending = ""
    for part in speakable:
        pending = f"{pending} {part}" if pending and part[0].isascii() and pending[-1].isascii() else f"{pending}{part}"
        if _non_space_len(pending) >= min_chars:
            merged.append(pending)
            pending = ""
    if pending:
        if merged:
            sep = " " if pending[0].isascii() and merged[-1][-1].isascii() else ""
            merged[-1] = f"{merged[-1]}{sep}{pending}"
        else:
            merged.append(pending)
    return merged
