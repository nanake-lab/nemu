"""Static shapes of the JSON fields consumed and produced by the board."""

from typing import NotRequired, TypedDict


class PrHead(TypedDict):
    ref: str
    sha: str


class RawPr(TypedDict):
    number: int
    title: str
    state: str
    merged_at: NotRequired[str | None]
    draft: NotRequired[bool]
    labels: NotRequired[list[dict[str, str]]]
    head: PrHead
    html_url: str
    updated_at: str


class PrDetails(TypedDict, total=False):
    waiting_runner: bool
    verdict: str | None
    guarded: bool
    draft: bool


class CardPr(PrDetails):
    number: int
    url: str
    title: str
    state: str
    branch: str
    labels: list[str]
    updated_at: str


class Card(TypedDict):
    id: str
    title: str
    deps: list[str]
    difficulty: str
    suggested: str
    milestone: str
    status_override: NotRequired[dict[str, str]]
    you: NotRequired[str]
    note: NotRequired[str]
    prs: NotRequired[list[CardPr]]
    status: NotRequired[str]
    waiting_on: NotRequired[list[str]]
    status_label: NotRequired[str]


class Milestone(TypedDict):
    id: str
    name: str
    total: NotRequired[int]
    done: NotRequired[int]
    remaining: NotRequired[list[str]]


class Catalog(TypedDict):
    cards: list[Card]
    milestones: list[Milestone]


class Reminder(TypedDict):
    kind: str
    text: str


class AttentionRef(TypedDict):
    card: str
    pr: NotRequired[int]
    url: NotRequired[str]


class Attention(Reminder, AttentionRef):
    pass


class PausedPr(TypedDict):
    number: int
    title: str
    url: str


class Board(TypedDict):
    generated_at: str
    repo: str
    milestones: list[Milestone]
    attention: list[Attention]
    other_attention: list[Attention]
    paused_prs: list[PausedPr]
    next_up: dict[str, list[str]]
    cards: list[Card]


class WorkflowRun(TypedDict):
    id: int
    status: NotRequired[str]


class WorkflowJob(TypedDict):
    name: str
    status: str
