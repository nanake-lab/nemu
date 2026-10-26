#!/usr/bin/env python3
"""Local progress board: http://127.0.0.1:8765

Every page load and every refresh asks GitHub for the current state, through
the GitHub CLI the user is already signed in to. Nothing is cached on disk and
the server only listens on this computer.

    pnpm board                       # from the repository root
    python tools/status/serve_board.py [--port 8765] [--no-browser]
"""

from __future__ import annotations

import argparse
import json
import sys
import threading
import time
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from build_status import (
    REPO,
    GitHubError,
    board_rows,
    compute,
    load_catalog,
)

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
PAGE = HERE / "board.html"

# Several tabs, or a click during an automatic refresh, share one GitHub round
# trip instead of each starting their own.
FRESH_FOR = 5.0
_lock = threading.Lock()
_last: tuple[float, bytes] | None = None


def board_json() -> tuple[int, bytes]:
    global _last
    with _lock:
        if _last and time.monotonic() - _last[0] < FRESH_FOR:
            return 200, _last[1]
        try:
            out = compute(REPO, load_catalog(REPO, ROOT / "tasks" / "cards.json"))
            body = json.dumps(
                {
                    "rows": board_rows(out),
                    "other_attention": out["other_attention"],
                    "paused_prs": out["paused_prs"],
                },
                ensure_ascii=False,
            ).encode()
        except GitHubError as exc:
            return 502, json.dumps({"error": str(exc)}, ensure_ascii=False).encode()
        _last = (time.monotonic(), body)
        return 200, body


class Handler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        if self.path in ("/", "/index.html"):
            self.send(200, "text/html; charset=utf-8", PAGE.read_bytes())
        elif self.path.startswith("/api/board"):
            code, body = board_json()
            self.send(code, "application/json; charset=utf-8", body)
        else:
            self.send(404, "text/plain; charset=utf-8", b"not found")

    def send(self, code: int, kind: str, body: bytes) -> None:
        self.send_response(code)
        self.send_header("Content-Type", kind)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format: str, *args: object) -> None:
        pass


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8765)
    ap.add_argument("--no-browser", action="store_true")
    args = ap.parse_args()

    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    url = f"http://127.0.0.1:{args.port}/"
    print(f"Nemu board: {url}  (Ctrl+C to stop)", flush=True)
    if not args.no_browser:
        webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
