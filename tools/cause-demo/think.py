"""Generate one or more independent solitary thoughts from a demo scene."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys
from typing import TYPE_CHECKING, Any

from chat import build_deepseek_body, call_deepseek_stream, cargo_env, repo_root, run_assembler

from think_plan import THOUGHT_SPEAKER as THOUGHT_SPEAKER, ThoughtPlan

if TYPE_CHECKING:
    from http_probe import Server

PROTOCOL = "这是 Nemu 独处时的内心。输出为她此刻的一个念头，用第一人称，一到三句。"
SENSES_FACT = "她在电脑里，能感知到的只有下面这些记录和现在的时间。"
TIME_FORMAT = "%Y-%m-%d %H:%M"
CACHE = Path("D:/Dev/cache")
RUN_DIR = CACHE / "cause-demo" / "think-02"
PLAN_RUN_DIR = CACHE / "cause-demo" / "think-05"
DEEPSEEK_RUN_DIR = CACHE / "cause-demo" / "think-03"
ENGINE = CACHE / "P0-10" / "llama.cpp" / "b11669"
MODEL = CACHE / "P0-12" / "gguf" / "carrier-Q4_K_M.gguf"


def new_record(
    records: list[dict[str, Any]], at: str, speaker: str, content: str, prefix: str
) -> dict[str, Any]:
    position = max((record["h_pos"] for record in records), default=0) + 1
    ids = {record["id"] for record in records}
    record_id = f"{prefix}_{position}"
    while record_id in ids:
        record_id += "_"
    return dict(id=record_id, h_pos=position, time=at, speaker=speaker, text=content)


def projection_payload(scenario: dict[str, Any], at: str) -> dict[str, Any]:
    cutoff = datetime.strptime(at, TIME_FORMAT)
    original_records = scenario["records"]
    records = [
        record for record in original_records
        if datetime.strptime(record["time"], TIME_FORMAT) <= cutoff
    ]
    available = {record["id"] for record in records}
    relations = [
        relation for relation in scenario.get("relations", [])
        if relation["from"] in available and relation["to"] in available
    ]
    slots = []
    for slot in scenario.get("slots", []):
        sources = [source for source in slot["sources"] if source in available]
        if sources:
            slots.append({**slot, "sources": sources})
    trigger = new_record(original_records, at, "（时间）", f"现在是 {at}，你一个人。", "alone")
    return dict(
        records=[*records, trigger], latest=trigger["id"], relations=relations,
        slots=slots, budget=scenario["budget"], dialogue=[], frontier_complete=True,
        decomposition_unknown=False, fact_mode="per_event", show_ids=False,
    )


def append_thought(
    scenario: dict[str, Any], binding: dict[str, Any], at: str, content: str,
    points: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    first = next((pack for pack in binding["packs"] if pack["fate"] == "accepted"), None)
    if first is None:
        raise ValueError("组包没有接纳任何槽，无法追加念头。")
    thought = new_record(scenario["records"], at, THOUGHT_SPEAKER, content, "thought")
    if points is not None:
        thought["points"] = points
    slots = [
        {**slot, "sources": [*slot["sources"], thought["id"]]}
        if slot["basis"] == first["basis"] else slot
        for slot in scenario["slots"]
    ]
    return {**scenario, "records": [*scenario["records"], thought], "slots": slots}


def thought_messages(
    messages: list[dict[str, str]], *, senses: str = "off"
) -> list[dict[str, str]]:
    if [message["role"] for message in messages] != ["system", "system", "user"]:
        raise ValueError("独处组包必须依次包含 P、E、o。")
    # cause_demo emits two system messages. Qwen's official template permits
    # one, so join the unchanged E to the replacement P inside that block.
    protocol = f"{PROTOCOL}\n{SENSES_FACT}" if senses == "on" else PROTOCOL
    system = f"{protocol}\n\n{messages[1]['content']}".strip()
    observation = messages[2]["content"].strip()
    return [{"role": "system", "content": system}, {"role": "user", "content": observation}]


def thought_request(
    assembled: dict[str, Any], payload: dict[str, Any], no_think: bool, plan2: bool,
) -> tuple[dict[str, Any], ThoughtPlan | None]:
    messages = thought_messages(assembled["messages"])
    plan = None
    if plan2:
        plan = ThoughtPlan(payload["records"][:-1], assembled["binding"])
        messages.append(plan.instruction())
    request = build_deepseek_body(messages, no_think=no_think)
    if plan is not None:
        request["response_format"] = {"type": "json_object"}
    return request, plan


def render_prompt(messages: list[dict[str, str]], *, senses: str = "off") -> str:
    system, observation = thought_messages(messages, senses=senses)
    return (
        f"<|im_start|>system\n{system['content']}<|im_end|>\n"
        f"<|im_start|>user\n{observation['content']}<|im_end|>\n"
        "<|im_start|>assistant\n<think>\n\n</think>\n\n"
    )


def deepseek_sample(request: dict[str, Any], api_key: str) -> dict[str, Any]:
    response = call_deepseek_stream(request, api_key, echo=False)
    valid = (
        response["error"] is None and response["done"] and response["finish_reason"] == "stop"
        and bool(response["text"].strip())
    )
    return {**response, "valid": valid}


def completion_request(sampling: str) -> dict[str, Any]:
    request = dict(
        stream=True, n_predict=256, return_tokens=True, repeat_penalty=1.0,
        frequency_penalty=0.0, cache_prompt=False, id_slot=0,
    )
    if sampling == "greedy":
        # DEMO-THINK-01 retained the P0-12 greedy parameters.
        return dict(request, temperature=0, seed=731, presence_penalty=0.0)
    # Qwen3.5-4B README, "Instruct (or non-thinking) mode for general tasks".
    # With seed omitted, llama.cpp chooses a fresh seed per request.
    return dict(request, temperature=0.7, top_p=0.8, top_k=20, min_p=0.0, presence_penalty=1.5)


def generate_sample(
    server: Server, request: dict[str, Any], tokens: list[int], folder: Path
) -> dict[str, Any]:
    result = server.completion({**request, "prompt": tokens}, folder)
    first_text_ms = next(
        (event["elapsedMs"] for event in map(
            json.loads, (folder / "events.jsonl").read_text(encoding="utf-8").splitlines()
        ) if event["data"].get("content")),
        None,
    )
    return dict(
        text=str(result["content"]), first_text_ms=first_text_ms,
        elapsed_ms=result["completedClientMs"], stop_type=result["final"].get("stop_type"),
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", required=True, type=Path)
    parser.add_argument("--at", required=True, help="YYYY-MM-DD HH:MM")
    parser.add_argument("--n", type=int, default=1)
    parser.add_argument("--append", type=Path, metavar="OUTPUT_SCENARIO")
    parser.add_argument("--sampling", choices=("greedy", "official"), default="official")
    parser.add_argument("--senses", choices=("on", "off"), default="off")
    parser.add_argument("--thinker", choices=("local", "deepseek"))
    parser.add_argument("--ds-think", choices=("off", "high"), default="off")
    parser.add_argument("--plan2", action="store_true", help="核对编号后按顺序拼接念头")
    args = parser.parse_args()
    if args.thinker is None:
        args.thinker = "deepseek" if args.plan2 else "local"
    if args.plan2 and args.thinker != "deepseek":
        parser.error("--plan2 需要 --thinker deepseek")
    if args.n < 1 or (args.append is not None and args.n != 1):
        parser.error("--n 必须为正整数，--append 只支持 --n 1")
    if args.append is not None and args.append.exists():
        parser.error("--append 需要一个新文件路径")
    if args.thinker == "deepseek" and args.senses != "off":
        parser.error("--thinker deepseek 只支持 --senses off")

    root = repo_root()

    try:
        scenario = json.loads(args.scenario.read_text(encoding="utf-8"))
        payload = projection_payload(scenario, args.at)
        assembled = run_assembler(payload, root / "core", cargo_env())
        if assembled["outcome"] != "sent":
            raise RuntimeError(f"组包失败：{assembled['outcome']} {assembled['reason']}")
        binding = assembled["binding"]
        if args.append is not None and not binding["accepted"]:
            raise ValueError("组包没有接纳任何槽，无法追加念头。")
        points = None
        if args.thinker == "deepseek":
            api_key = os.environ.get("DEEPSEEK_API_KEY", "").strip()
            if not api_key:
                raise ValueError("环境变量 DEEPSEEK_API_KEY 未设置或为空。")
            request, plan = thought_request(assembled, payload, args.ds_think == "off", args.plan2)
            run_dir = PLAN_RUN_DIR if plan else DEEPSEEK_RUN_DIR
            settings = dict(thinker="deepseek", ds_think=args.ds_think)
        else:
            sys.path.insert(0, str(root / "spikes" / "carrier-engine"))
            from http_probe import Server, quiet

            quiet()
            prompt = render_prompt(assembled["messages"], senses=args.senses)
            request = completion_request(args.sampling)
            run_dir = RUN_DIR
            settings = dict(prompt=prompt, senses=args.senses, sampling=args.sampling)
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
        run_dir.mkdir(parents=True, exist_ok=True)
        log_path = run_dir / f"{stamp}.jsonl"
        print(f"记录：{log_path}", file=sys.stderr, flush=True)
        with log_path.open("x", encoding="utf-8") as log:
            def log_row(row: dict[str, Any]) -> None:
                log.write(json.dumps(row, ensure_ascii=False) + "\n")
                log.flush()

            log_row(dict(
                scenario=str(args.scenario), at=args.at, n=args.n,
                binding=binding, request=request, **settings,
            ))
            try:
                if args.thinker == "deepseek":
                    for index in range(1, args.n + 1):
                        # Keep every failed attempt. Empty verified thoughts are completed rounds.
                        for _ in range(3 if plan else 1):
                            sample = deepseek_sample(request, api_key)
                            if plan:
                                sample["valid"] = bool(sample["valid"] and sample["usage"])
                                sample = plan.verify(sample)
                            log_row(dict(index=index, **sample))
                            if sample["valid"]:
                                break
                        if not sample["valid"]:
                            raise RuntimeError("DeepSeek 未返回正常结束的非空念头，详见请求日志。")
                        content = str(sample["text"])
                        if plan:
                            points = sample["verification"]["kept"]
                        print(content, flush=True)
                else:
                    with Server(ENGINE, MODEL, RUN_DIR / stamp / "server") as server:
                        # Explicitly parse ChatML markers, as in P0-12's text route.
                        tokens = server.post(
                            "/tokenize", dict(content=prompt, add_special=False, parse_special=True)
                        )["tokens"]
                        for index in range(1, args.n + 1):
                            folder = RUN_DIR / stamp / f"request-{index:02}"
                            sample = generate_sample(server, request, tokens, folder)
                            content = str(sample["text"])
                            log_row(dict(index=index, **sample))
                            print(content, flush=True)
                if args.append is not None and points != []:
                    if not content.strip():
                        raise ValueError("模型返回空文本，无法追加念头。")
                    output = append_thought(scenario, binding, args.at, content, points)
                    with args.append.open("x", encoding="utf-8") as stream:
                        json.dump(output, stream, ensure_ascii=False, indent=2)
                        stream.write("\n")
            except Exception as exc:
                log_row(dict(error=f"{type(exc).__name__}: {exc}"))
                raise
        return 0
    except (KeyError, OSError, RuntimeError, ValueError, TypeError) as exc:
        print(f"错误：{exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
