"""Stateless hosted judging, only after explicit provider selection and budget approval."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from decimal import Decimal
import json
import os
from pathlib import Path
from typing import cast
from urllib.request import Request, build_opener

from io_support import CACHE, NoRedirect, digest, save


@dataclass(frozen=True)
class Provider:
    model: str
    endpoint: str
    key_env: str
    input_rate: Decimal
    output_rate: Decimal
    pilot_cap: Decimal


# Public documentation checked 2026-10-29; selection is deliberately not defaulted.
PROVIDERS = {
    "glm": Provider("glm-4.6", "https://api.z.ai/api/paas/v4/chat/completions", "ZAI_API_KEY",
                    Decimal("0.60"), Decimal("2.20"), Decimal("7.00")),
    "minimax": Provider("MiniMax-M3", "https://api.minimax.io/v1/chat/completions", "MINIMAX_API_KEY",
                        Decimal("0.30"), Decimal("1.20"), Decimal("3.50")),
}
INPUT_LIMIT = 12000
OUTPUT_LIMIT = 512


def estimate(calls: int, provider: str, *, upper: bool = False) -> float:
    spec = PROVIDERS[provider]
    inputs, outputs = (INPUT_LIMIT, OUTPUT_LIMIT) if upper else (2500, 200)
    return float(Decimal(calls) * (inputs * spec.input_rate + outputs * spec.output_rate) / 1000000)


def request_body(provider: str, system: str, payload: object) -> tuple[dict[str, object], int]:
    spec = PROVIDERS[provider]
    text = json.dumps(payload, ensure_ascii=False, allow_nan=False)
    bound = len((system + text).encode("utf-8")) + 1024
    if bound > INPUT_LIMIT:
        raise ValueError("Anonymous material exceeds registered bound; never truncate")
    body: dict[str, object] = {"model": spec.model, "stream": False, "tools": [],
        "thinking": {"type": "disabled"},
        "messages": [{"role": "system", "content": system}, {"role": "user", "content": text}]}
    if provider == "glm":
        body.update(max_tokens=OUTPUT_LIMIT, temperature=.2, response_format={"type": "json_object"})
    else:
        # M3 ignores temperature/top_p overrides; official defaults are 1/.95.
        body.update(max_completion_tokens=OUTPUT_LIMIT, service_tier="standard")
    return body, bound


def response_text(data: dict[str, object], model: str, bound: int) -> str:
    if data.get("model") != model:
        raise ValueError("Returned model absent/changed; never mix versions")
    if data.get("web_search") or data.get("tool_calls"):
        raise ValueError("Unexpected search/tool response")
    usage = data.get("usage")
    if not isinstance(usage, dict):
        raise ValueError("Missing usage")
    for name, limit in (("prompt_tokens", bound), ("completion_tokens", OUTPUT_LIMIT)):
        value = usage.get(name)
        if type(value) is not int or not 0 <= value <= limit:
            raise ValueError("Invalid/excess usage; stop to reconcile")
    choices = data.get("choices")
    if not isinstance(choices, list) or len(choices) != 1 or not isinstance(choices[0], dict):
        raise ValueError("One completion required")
    choice = choices[0]
    message = choice.get("message")
    if (choice.get("finish_reason") != "stop" or not isinstance(message, dict)
            or message.get("tool_calls") or message.get("function_call") or message.get("web_search")):
        raise ValueError("Incomplete/tool response; no repair request")
    # Thinking is disabled in both selected APIs. Unexpected reasoning is not silently accepted.
    if message.get("reasoning_content"):
        raise ValueError("Unexpected reasoning mode")
    content = message.get("content")
    if not isinstance(content, str):
        raise ValueError("Text completion required")
    return content


class HostedJudge:
    def __init__(self, root: Path, provider: str, approved_usd: str, approval_ref: str,
                 prereg_sha256: str) -> None:
        self.root = root.resolve()
        if not self.root.is_relative_to(CACHE.resolve()) or self.root == CACHE.resolve():
            raise ValueError("Experiment cache only")
        if provider not in PROVIDERS:
            raise ValueError("Select an approved hosted provider; no subscription or automatic fallback")
        self.spec, self.provider = PROVIDERS[provider], provider
        self.cap = Decimal(approved_usd)
        if (not self.cap.is_finite() or not 0 < self.cap <= self.spec.pilot_cap or not approval_ref.strip()
                or prereg_sha256 != digest(Path(__file__).with_name("PREREG.md"))):
            raise ValueError("Current phase-two protocol, selection and human budget approval required")
        if (root / "judge-approval.json").exists():
            raise ValueError("Never resume/replace a judging attempt")
        self.index, self.reserved = 0, Decimal(0)
        self.approval_ref = approval_ref
        save(root / "judge-approval.json", {"provider": provider, "requested_model": self.spec.model,
            "approved_usd": approved_usd, "approval_ref": approval_ref, "prereg_sha256": prereg_sha256,
            "source_sha256": digest(Path(__file__)), "transport": "stateless_hosted_chat_completions"})

    def close(self) -> None:
        pass  # No model session, process, local history or persistent HTTP conversation.

    def call(self, role: str, system: str, payload: object) -> str:
        if role != "judge":
            raise ValueError("Judge role only")
        body, bound = request_body(self.provider, system, payload)
        cost = (bound * self.spec.input_rate + OUTPUT_LIMIT * self.spec.output_rate) / 1000000
        if self.reserved + cost > self.cap:
            raise RuntimeError("Approved judge budget exhausted")
        key = os.environ.get(self.spec.key_env)
        if not key:
            raise RuntimeError("Required process environment key absent")
        number = self.index
        request_path = self.root / f"judge-api-{number:05d}-request.json"
        save(request_path, {"body": body})  # No credentials, truth, paths or previous response.
        with (self.root / "judge-budget.jsonl").open("a", encoding="utf-8", newline="\n") as stream:
            stream.write(json.dumps({"index": number, "reserved_usd": str(cost),
                                     "approval_ref": self.approval_ref}) + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        self.reserved += cost
        self.index += 1
        try:
            request = Request(self.spec.endpoint, json.dumps(body, ensure_ascii=False).encode("utf-8"),
                              {"Authorization": "Bearer " + key, "Content-Type": "application/json"})
            with build_opener(NoRedirect()).open(request, timeout=120) as response:
                data = cast(dict[str, object], json.load(response))
            response_path = self.root / f"judge-api-{number:05d}-response.json"
            save(response_path, data)
            content = response_text(data, self.spec.model, bound)
            save(self.root / f"judge-api-{number:05d}-receipt.json", {
                "received_at": datetime.now(timezone.utc).isoformat(), "provider": self.provider,
                "requested_model": self.spec.model, "returned_model": data["model"],
                "provider_response_id": data.get("id"), "usage": data["usage"],
                "request_sha256": digest(request_path), "response_sha256": digest(response_path),
                "identity_limit": "Versioned provider ID, not a disclosed immutable weight hash."})
            return content
        except Exception as error:
            save(self.root / f"judge-api-{number:05d}-error.json", {"error_type": type(error).__name__, "retry": False})
            raise RuntimeError("Judge request failed; reservation retained; no retry/fallback") from None
