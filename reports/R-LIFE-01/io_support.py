"""Append-only synthetic evidence and paid request gates; no secrets in artifacts."""
from __future__ import annotations

from datetime import datetime, timezone
from decimal import Decimal
import hashlib
import json
import os
from pathlib import Path
from typing import cast
from urllib.request import HTTPRedirectHandler, Request, build_opener

CACHE = Path("D:/Dev/cache/R-LIFE-01")
RATES = {"actor": (Decimal("0.30"), Decimal("1.20"))}
MODELS = {"actor": "deepseek-flash"}


def save(path: Path, value: object) -> None:
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, allow_nan=False, indent=2)
        stream.write("\n")


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def new_run(path: Path) -> Path:
    root = path.resolve()
    if not root.is_relative_to(CACHE.resolve()) or root == CACHE.resolve():
        raise ValueError("Use a new directory under the experiment D-drive cache")
    root.mkdir(parents=True, exist_ok=False)
    save(root / "lineage.json", {"lineage": "EXPERIMENT", "group": "C", "m": 0,
                                 "created": datetime.now(timezone.utc).isoformat(),
                                 "core_store": False, "synthetic_only": True})
    return root


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req: Request, fp: object, code: int, msg: str,
                         headers: object, newurl: str) -> None:
        raise RuntimeError("Redirect refused")


class PaidClient:
    def __init__(self, root: Path, approved_usd: str, approval_ref: str) -> None:
        self.root, self.cap = root, Decimal(approved_usd)
        if not approval_ref.strip() or not self.cap.is_finite() or not 0 < self.cap <= Decimal("0.50"):
            raise ValueError("Explicit human budget approval required; actor-only R-LIFE-01 maximum $0.50")
        self.lock = (root / "paid.lock").open("x", encoding="utf-8")
        self.ledger = root / "budget.jsonl"
        self.reserved = Decimal(0)
        self.index = 0
        if self.ledger.exists():
            for row in self.ledger.read_text(encoding="utf-8").splitlines():
                entry = json.loads(row)
                self.reserved += Decimal(entry["reserved_usd"])
                self.index += 1
        self.approval_ref = approval_ref

    def close(self) -> None:
        self.lock.close()
        (self.root / "paid.lock").unlink()

    def call(self, role: str, system: str, payload: object) -> str:
        if role not in MODELS:
            raise ValueError("Unknown API role")
        text = json.dumps(payload, ensure_ascii=False, allow_nan=False)
        limit, output = 2500, 256
        # Conservative byte accounting, plus protocol allowance. No tools/thinking.
        bound = len((system + text).encode("utf-8")) + 1024
        if bound > limit:
            raise ValueError("Registered request budget exceeded; never truncate")
        cost = (bound * RATES[role][0] + output * RATES[role][1]) / 1000000
        if self.reserved + cost > self.cap:
            raise RuntimeError("Approved budget exhausted")
        key = os.environ.get("DEEPSEEK_API_KEY")
        if not key:
            raise RuntimeError("Required process environment key absent")
        body: dict[str, object] = {"model": MODELS[role], "max_tokens": output, "stream": False}
        url = "https://api.deepseek.com/chat/completions"
        body.update(messages=[{"role": "system", "content": system}, {"role": "user", "content": text}],
                    thinking={"type": "disabled"}, temperature=0.8,
                    response_format={"type": "json_object"})
        headers = {"Authorization": "Bearer " + key, "Content-Type": "application/json"}
        number = self.index
        save(self.root / f"api-{number:05d}-request.json", {"role": role, "body": body})
        with self.ledger.open("a", encoding="utf-8", newline="\n") as stream:
            stream.write(json.dumps({"index": number, "reserved_usd": str(cost),
                                     "approval_ref": self.approval_ref}) + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        self.reserved += cost  # Unknown/failed calls retain their reservation.
        self.index += 1
        try:
            request = Request(url, json.dumps(body, ensure_ascii=False).encode("utf-8"), headers)
            with build_opener(NoRedirect()).open(request, timeout=90) as response:
                data = cast(dict[str, object], json.load(response))
            save(self.root / f"api-{number:05d}-response.json", data)
            usage = data.get("usage")
            if not isinstance(usage, dict):
                raise ValueError("Missing usage; stop to reconcile budget")
            in_keys = ("prompt_tokens",)
            out_key = "completion_tokens"
            counts = [usage.get(k, 0) for k in in_keys] + [usage.get(out_key)]
            if any(type(count) is not int or count < 0 for count in counts):
                raise ValueError("Invalid usage")
            if sum(counts[:-1]) > bound or counts[-1] > output:
                raise ValueError("Usage exceeded reservation; stop before any further call")
            model = data.get("model")
            if not isinstance(model, str) or not model:
                raise ValueError("Missing returned model identity")
            identity_path = self.root / f"api-{role}-identity.json"
            if identity_path.exists():
                identity = json.loads(identity_path.read_text(encoding="utf-8"))
                if identity["model"] != model:
                    raise ValueError("Returned model identity changed")
            else:
                save(identity_path, {"requested": MODELS[role], "model": model})
            choices = cast(list[dict[str, object]], data["choices"])
            if choices[0]["finish_reason"] != "stop":
                raise ValueError("Truncated actor response")
            content = cast(dict[str, object], choices[0]["message"])["content"]
            if not isinstance(content, str):
                raise ValueError("Unexpected response")
            return content
        except Exception as error:
            save(self.root / f"api-{number:05d}-error.json", {"error_type": type(error).__name__, "retry": False})
            raise RuntimeError("API call failed; reservation retained; no automatic retry") from None
