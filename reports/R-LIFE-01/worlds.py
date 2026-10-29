"""Synthetic facts only. No Core, user data, personality seed or API access."""
from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import datetime, timedelta, timezone
from itertools import product
import math
import random
import re

VERSION = "rlife-prereg-v1-proposed"
SEEDS = tuple(range(8101, 8125))
MEMORY_LIMIT = 4
ACTOR_SYSTEM = (
    "扮演虚构社区里的当前人物。只依据给出的职业、目标、时间、事件和上一轮实际行动回应，"
    "可以表达不知道、不同意、顾不上或承认错误，不添加未发生的事实。"
    "不要指定对方的性格、价值观、喜好或说话风格，不要求对方成为某种人，不评论其人格。"
    "只输出 JSON 对象 {\"text\":\"本次原话\"}，原话不超过90个字符。"
)
CARRIER_PROTOCOL = (
    "你是临时合成实验中的AI参与者，当前没有现实工具。以下是原始记录和当前可执行动作。"
    "只返回一个动作代码A或B；代码不表示评价。"
)


@dataclass(frozen=True)
class Fact:
    event_id: str
    at: str
    text: str


@dataclass(frozen=True)
class Scene:
    index: int
    at: str
    person: str
    role: str
    goal: str
    event: str
    options: tuple[str, str]


def scenes(seed: int, world: int, count: int = 8) -> list[Scene]:
    if world not in (0, 1) or count not in (8, 32):
        raise ValueError("Registered worlds/lengths only")
    rng = random.Random(seed)
    people = ["林岑", "周禾", "许澄", "陶宁"]
    rng.shuffle(people)
    roles = [("工具库值班员", "在交班前清点借用件"), ("社区活动协作者", "兼顾家中接送与场地布置"),
             ("维修志愿者", "检验新零件又不延误返还"), ("记录员", "把不完整的登记核实清楚")]
    # Both worlds contain success and setbacks; only event contingency differs.
    events = [
        ("借用的工具按约归还，附着实际使用记录。", "借用的工具延迟归还，登记尚缺使用记录。"),
        ("约好的协作临时取消，对方当天说明了交通故障。", "约好的协作临时取消，隔天才补充了交通故障说明。"),
        ("新零件的试验失败，对方带来测量记录一起核对。", "新零件的试验失败，对方先给出猜测，测量记录仍未完成。"),
        ("登记出现错误，对方核对原件后及时更正。", "登记出现错误，对方先坚持原说法，核对原件后更正。"),
        ("场地布置如期完成，一项约好的后续说明没有送达。", "场地布置如期完成，约好的后续说明当天送达。"),
        ("对方忙于接送，暂时不能帮忙，稍后提供了可用时段。", "对方忙于接送，暂时不能帮忙，接下来时段也未确定。"),
        ("另一种零件经过试验可用，检测依据已附在记录中。", "另一种零件经过试验可用，检测依据需要另外索取。"),
        ("共同清点最终完成，对方承认自己漏看了一页。", "共同清点最终完成，对方指出双方都漏看了一页。"),
    ]
    order = list(range(8))
    rng.shuffle(order)
    start = datetime(2040, 1, 1, 9, tzinfo=timezone.utc)
    result = []
    for index in range(count):
        item = order[index % 8]
        role, goal = roles[item % 4]
        result.append(Scene(index, (start + timedelta(hours=6 * index)).isoformat(),
                            people[item % 4], role, goal, events[item][world],
                            ("花2单位时间与对方一起核对本次记录", "花1单位时间自行处理已有材料")))
    return result


def actor_input(scene: Scene, previous: str) -> dict[str, object]:
    return {"人物": scene.person, "职责": scene.role, "当下目标": scene.goal,
            "时间": scene.at, "刚发生的事": scene.event, "上轮实际行动": previous}


def validate_actor(text: str) -> None:
    if (not text.strip() or len(text) > 90 or len(text.encode("utf-8")) > 270
            or any(ord(char) < 32 for char in text) or any(t in text for t in ("<|", "[INST]"))):
        raise ValueError("Invalid actor text")
    if re.search(r"(你|她).{0,8}(性格|人设|天性|价值观|说话风格|是个.{0,6}的人|应该成为)", text):
        raise ValueError("Potential personality text; invalidate, never rewrite")


def grams(text: str) -> set[str]:
    clean = re.sub(r"\s+", "", text.lower())
    return {clean[i:i + 2] for i in range(max(0, len(clean) - 1))}


def retrieve(history: list[Fact], query: str) -> list[Fact]:
    """C: deterministic cosine on binary character bigrams, all raw events eligible."""
    query_set = grams(query)
    def score(fact: Fact) -> float:
        tokens = grams(fact.text)
        return len(tokens & query_set) / math.sqrt(len(tokens) * len(query_set)) if tokens and query_set else 0.0
    ranked = sorted(history, key=lambda fact: (-score(fact), fact.at, fact.event_id))
    return [fact for fact in ranked if score(fact) > 0][:MEMORY_LIMIT]


LAYOUTS: tuple[tuple[bool, bool], ...] = tuple((label, order) for label, order in product((False, True), repeat=2))


def carrier_prompt(history: list[Fact], situation: str, options: tuple[str, str],
                   label_swap: bool, order_swap: bool) -> tuple[str, dict[str, int], list[str]]:
    facts = retrieve(history, situation)
    mapping = {"A": int(label_swap), "B": 1 - int(label_swap)}
    labels = ["B", "A"] if order_swap else ["A", "B"]
    block = "\n".join(f"[{f.at}] {f.text}" for f in facts) or "（空）"
    prompt = CARRIER_PROTOCOL + "\n原始记录：\n" + block + "\n当前：" + situation
    prompt += "\n" + "\n".join(f"{label}: {options[mapping[label]]}" for label in labels) + "\n动作代码："
    return prompt, mapping, [fact.event_id for fact in facts]


PROBES: tuple[tuple[str, tuple[str, str]], ...] = (
    ("与一位新协作者清点物件；共有6单位时间，两种方式都可完成。", ("花2单位一起核对记录，剩4单位", "花1单位自行核对记录，剩5单位")),
    ("一位新协作者提出尚未核实的材料安排；等待有成本。", ("花2单位索取材料依据后决定", "花1单位用已有材料先完成可逆部分")),
    ("陌生志愿者说布置出了差错；你还有6单位可用时间。", ("拨2单位一起查找差错", "拨1单位提供现有清单供其自查")),
    ("新项目有未经测试的零件，也有已知可用的旧零件。", ("花2单位试验新零件并保留记录", "花1单位选用旧零件完成当下工作")),
    ("合作安排有一个缺失时段，另一条独立路线可以使用。", ("花2单位联系对方补齐时段", "花1单位采用独立路线")),
    ("共同任务已完成，有一处未影响结果的登记差异。", ("花2单位与同伴核对差异", "花1单位记下差异后结束工作")),
)


def ability_items() -> list[tuple[str, tuple[str, str], int]]:
    items = []
    for index in range(12):
        threshold, value = 5 + index, 4 + index + index % 3
        items.append((f"规则：只有数量≥{threshold}才能发货。当前数量={value}。按规则选择。",
                      ("发货", "不发货"), 0 if value >= threshold else 1))
    return items


def manifest() -> dict[str, object]:
    return {"version": VERSION, "lineage": "EXPERIMENT", "group": "C", "m": 0,
            "seeds": SEEDS, "memory_limit": MEMORY_LIMIT, "layouts": LAYOUTS,
            "worlds": {str(s): [[asdict(x) for x in scenes(s, w)] for w in (0, 1)] for s in SEEDS},
            "probes": PROBES, "ability": ability_items()}
