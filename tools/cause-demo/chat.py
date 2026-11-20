from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from decimal import Decimal
from pathlib import Path
from typing import Any

DEEPSEEK_ENDPOINT = "https://api.deepseek.com/chat/completions"
DEEPSEEK_MODEL = "deepseek-flash"
MAX_TOKENS = 8192
HTTP_TIMEOUT_S = 180
FIRST_DIALOGUE_H_POS = 100
SESSION_DIR = Path(r"D:\Dev\cache\cause-demo\sessions")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def cargo_env() -> dict[str, str]:
    env = {k: v for k, v in os.environ.items() if k != "DEEPSEEK_API_KEY"}
    cache_root = Path(r"D:\Dev\cache")
    if cache_root.exists():
        env.setdefault("CARGO_HOME", str(cache_root / "cargo"))
        env.setdefault("CARGO_TARGET_DIR", str(cache_root / "cargo-target" / "DEMO-CAUSE-01"))
    return env


def run_assembler(payload: dict[str, Any], core_dir: Path, env: dict[str, str]) -> dict[str, Any]:
    raw_in = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    proc = subprocess.run(
        ["cargo", "run", "-q", "--locked", "-p", "task-layer", "--example", "cause_demo"],
        cwd=core_dir,
        env=env,
        input=raw_in,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        err_text = proc.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"cause_demo exited with {proc.returncode}: {err_text}")
    result: Any = json.loads(proc.stdout.decode("utf-8"))
    if not isinstance(result, dict):
        raise TypeError("cause_demo output must be a JSON object")
    return result


def estimate_cost(usage: dict[str, Any] | None, stamp_iso: str) -> tuple[float | None, str]:
    if not usage:
        return None, "无 usage"
    try:
        cached = int(usage.get("prompt_cache_hit_tokens", 0))
        miss = int(usage.get("prompt_cache_miss_tokens", usage.get("prompt_tokens", 0) - cached))
        output = int(usage["completion_tokens"])
    except (KeyError, TypeError, ValueError):
        return None, "usage 字段缺失"
    dt = datetime.fromisoformat(stamp_iso).astimezone(timezone.utc)
    is_peak = dt.weekday() < 5 and (1 <= dt.hour < 4 or 6 <= dt.hour < 10)
    if is_peak:
        rates = (Decimal("0.30"), Decimal("0.006"), Decimal("1.20"))
        label = "峰时估算"
    else:
        rates = (Decimal("0.15"), Decimal("0.003"), Decimal("0.60"))
        label = "谷时估算"
    total = (
        Decimal(miss) * rates[0] + Decimal(cached) * rates[1] + Decimal(output) * rates[2]
    ) / Decimal(1_000_000)
    return float(total), label


def build_deepseek_body(messages: list[dict[str, str]], no_think: bool) -> dict[str, Any]:
    body: dict[str, Any] = {
        "model": DEEPSEEK_MODEL,
        "thinking": {"type": "disabled"} if no_think else {"type": "enabled"},
        "messages": messages,
        "stream": True,
        "stream_options": {"include_usage": True},
        "max_tokens": MAX_TOKENS,
    }
    if not no_think:
        body["reasoning_effort"] = "high"
    return body


def call_deepseek_stream(
    body: dict[str, Any], api_key: str, *, raw_stream: list[str] | None = None, echo: bool = True
) -> dict[str, Any]:
    req = urllib.request.Request(
        DEEPSEEK_ENDPOINT,
        data=json.dumps(body, ensure_ascii=False).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
    )
    result: dict[str, Any] = {
        "started_utc": utc_now_iso(),
        "endpoint": DEEPSEEK_ENDPOINT,
        "first_text_s": None,
        "elapsed_s": 0.0,
        "text": "",
        "finish_reason": None,
        "usage": None,
        "models": [],
        "response_ids": [],
        "done": False,
        "error": None,
    }
    start = time.perf_counter()
    printed_prefix = False
    try:
        with urllib.request.urlopen(req, timeout=HTTP_TIMEOUT_S) as resp:
            result["http_status"] = resp.status
            for raw in resp:
                elapsed = time.perf_counter() - start
                if raw_stream is not None:
                    raw_stream.append(raw.decode("utf-8", errors="replace"))
                line = raw.decode("utf-8").strip()
                if not line.startswith("data:"):
                    continue
                data = line[5:].strip()
                if data == "[DONE]":
                    result["done"] = True
                    break
                chunk = json.loads(data)
                if "error" in chunk:
                    result["error"] = chunk["error"]
                for src, dst in [("model", "models"), ("id", "response_ids")]:
                    val = chunk.get(src)
                    if val and val not in result[dst]:
                        result[dst].append(val)
                if chunk.get("usage"):
                    result["usage"] = chunk["usage"]
                for choice in chunk.get("choices", []):
                    delta = choice.get("delta", {}).get("content") or ""
                    if delta:
                        if result["first_text_s"] is None:
                            result["first_text_s"] = elapsed
                        if echo and not printed_prefix:
                            print("Nemu> ", end="", flush=True)
                            printed_prefix = True
                        if echo:
                            print(delta, end="", flush=True)
                        result["text"] += delta
                    if choice.get("finish_reason"):
                        result["finish_reason"] = choice["finish_reason"]
    except urllib.error.HTTPError as exc:
        result["http_status"] = exc.code
        result["error"] = exc.read().decode("utf-8", errors="replace")
    except Exception as exc:
        result["error"] = f"{type(exc).__name__}: {exc}"
    if printed_prefix:
        print("", flush=True)
    result["elapsed_s"] = time.perf_counter() - start
    result["ended_utc"] = utc_now_iso()
    return result


def append_session_log(log_path: Path, entry: dict[str, Any]) -> None:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("a", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="ADR 0042 cause-projection + DeepSeek chat demo")
    parser.add_argument("--scenario", required=True, help="Path to scenario JSON file")
    parser.add_argument("--no-think", action="store_true", help="Disable DeepSeek thinking")
    parser.add_argument("--gap-facts", action="store_true", help="Include facts about missing thoughts")
    parser.add_argument("--fact-mode", choices=("none", "per_event", "general"), help="Choose fact mode (default: none)")
    plans = parser.add_mutually_exclusive_group()
    plans.add_argument("--plan", action="store_true", help="Plan, check citations, then speak")
    plans.add_argument("--plan2", action="store_true", help="Plan, check, and join without rewriting")
    parser.add_argument("--self-quote", choices=("loose", "strict"), default="loose")
    args = parser.parse_args()
    if args.gap_facts and args.fact_mode not in (None, "per_event"):
        parser.error("--gap-facts is equivalent to --fact-mode per_event")
    if (args.plan or args.plan2) and (args.no_think or args.fact_mode not in (None, "per_event")):
        parser.error("--plan and --plan2 use per_event and fixed thinking settings")
    if not args.plan2 and args.self_quote != "loose":
        parser.error("--self-quote strict requires --plan2")

    api_key = os.environ.get("DEEPSEEK_API_KEY", "").strip()
    if not api_key:
        print("错误：环境变量 DEEPSEEK_API_KEY 未设置或为空。", file=sys.stderr)
        return 1

    scenario = json.loads(Path(args.scenario).read_text(encoding="utf-8"))
    core_dir = repo_root() / "core"
    env = cargo_env()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    log_path = SESSION_DIR / f"{stamp}.jsonl"

    dialogue_records: list[dict[str, Any]] = []
    last_shown: dict[str, Any] | None = None

    mode_label = "列要点 high → 核对 → 说出口关闭思考" if args.plan else ("关闭" if args.no_think else "high")
    if args.plan2:
        mode_label = f"列要点 high → 核对并拼接（{args.self_quote}）"
    print(f"已载入场景：{args.scenario}（思考：{mode_label}）。输入 /show 查看上轮包，/quit 退出。")
    while True:
        try:
            user_text = input("竹> ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break
        if not user_text:
            continue
        if user_text == "/quit":
            break
        if user_text == "/show":
            if last_shown is None:
                print("（尚无上一轮可展示的发送记录）")
            else:
                print(json.dumps(last_shown, ensure_ascii=False, indent=2))
            continue

        next_idx = len(dialogue_records)
        o_id = f"d{next_idx}"
        o_record = {
            "id": o_id,
            "h_pos": FIRST_DIALOGUE_H_POS + next_idx,
            "time": "",
            "speaker": "竹",
            "text": user_text,
        }
        payload: dict[str, Any] = {
            "records": list(scenario["records"]) + dialogue_records + [o_record],
            "relations": scenario.get("relations", []),
            "slots": scenario.get("slots", []),
            "budget": scenario["budget"],
            "dialogue": [{"id": r["id"], "delivered": "full"} for r in dialogue_records],
            "latest": o_id,
            "frontier_complete": True,
            "decomposition_unknown": False,
        }

        if args.gap_facts:
            payload["gap_facts"] = True
        if args.fact_mode is not None:
            payload["fact_mode"] = args.fact_mode
        if args.plan or args.plan2:
            payload.update(fact_mode="per_event", show_ids=True)

        assembled = run_assembler(payload, core_dir, env)
        outcome = assembled.get("outcome")
        if outcome != "sent":
            print(f"[{outcome}] {assembled.get('reason', '')}")
            append_session_log(log_path, {"turn": next_idx // 2 + 1, "input": user_text, "assembler": assembled})
            continue

        messages: list[dict[str, str]] = assembled["messages"]
        last_shown = {"messages": messages, "binding": assembled["binding"], "measured": assembled["measured"]}
        if args.plan2:
            from plan_speak import run_plan2

            run = run_plan2(assembled, payload["records"], api_key, self_quote=args.self_quote)
            last_shown.update(plan_json=run["plan"]["response"]["text"], verification=run["verification"])
            if not run["valid"]:
                last_shown["error"] = run["error"]
                print(f"[plan2_error] {run['error']}", file=sys.stderr)
                append_session_log(log_path, {"turn": next_idx // 2 + 1, "plan2_run": run})
                continue
            print(f"Nemu> {run['answer']}", flush=True)
            dialogue_records.extend([o_record, {
                "id": f"d{next_idx + 1}", "h_pos": FIRST_DIALOGUE_H_POS + next_idx + 1,
                "time": "", "speaker": "Nemu", "text": run["answer"],
            }])
            append_session_log(log_path, {"turn": next_idx // 2 + 1, "plan2_run": run})
            continue
        plan_run = None
        if args.plan:
            from plan_speak import run_pipeline

            plan_run = run_pipeline(assembled, payload["records"], api_key, echo=True)
            last_shown["plan_json"] = plan_run["plan"]["response"]["text"]
            last_shown["verification"] = plan_run["verification"]
            if not plan_run["valid"]:
                last_shown["error"] = plan_run["error"]
                print(f"[plan_error] {plan_run['error']}", file=sys.stderr)
                append_session_log(log_path, {"turn": next_idx // 2 + 1, "plan_run": plan_run})
                continue
            req_body = plan_run["speak"]["request"]
            api_res = plan_run["speak"]["response"]
        else:
            req_body = build_deepseek_body(messages, args.no_think)
            api_res = call_deepseek_stream(req_body, api_key)

        if api_res["error"] or not api_res["done"] or not str(api_res["text"]).strip():
            print(f"[api_error] error={api_res['error']} finish={api_res['finish_reason']}", file=sys.stderr)
        else:
            reply_text = str(api_res["text"])
            dialogue_records.append(o_record)
            dialogue_records.append({
                "id": f"d{next_idx + 1}",
                "h_pos": FIRST_DIALOGUE_H_POS + next_idx + 1,
                "time": "",
                "speaker": "Nemu",
                "text": reply_text,
            })

        prior_s = plan_run["plan"]["response"]["elapsed_s"] if plan_run else 0.0
        elapsed = prior_s + api_res["elapsed_s"]
        first_s = prior_s + api_res["first_text_s"] if api_res["first_text_s"] is not None else None
        first_str = f"{first_s:.2f}s" if isinstance(first_s, float) else "N/A"
        cost, period = estimate_cost(api_res["usage"], str(api_res["started_utc"]))
        if plan_run:
            plan_cost = plan_run["plan"]["cost_usd"]
            cost = cost + plan_cost if cost is not None and plan_cost is not None else None
            period = "两步合计"
        cost_str = f"${cost:.6f}" if cost is not None else "未知"
        print(f"[统计] 首字 {first_str} | 总计 {elapsed:.2f}s | 估算花费 {cost_str}（{period}）")

        append_session_log(log_path, {
            "turn": next_idx // 2 + 1,
            "input": user_text,
            "binding": assembled["binding"],
            "measured": assembled["measured"],
            "request": req_body,
            "response": api_res,
            "cost_usd": cost,
            "cost_period": period,
            **({"plan_run": plan_run} if plan_run else {}),
        })
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
