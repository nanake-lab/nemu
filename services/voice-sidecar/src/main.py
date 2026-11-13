"""Supervisor entry: dependency-free degraded health when Python extras are absent."""
from __future__ import annotations

import json
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def degraded_server(reason: str) -> None:
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            body = json.dumps({"status": "degraded", "reasons": [reason],
                               "vram": None, "ready": False}).encode()
            self.send_response(200 if self.path == "/health" else 503)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self) -> None:
            self.send_response(503)
            self.end_headers()

        def log_message(self, format: str, *args: object) -> None:
            pass  # No request paths, texts, or query parameters in logs.

    ThreadingHTTPServer(("127.0.0.1", int(os.getenv("NEMU_TTS_PORT", "17863"))), Handler).serve_forever()


def main() -> None:
    try:
        import uvicorn
        from nemu_voice_sidecar.app import create_app
    except ImportError:
        degraded_server("PYTHON_DEPENDENCIES_MISSING")
        return
    uvicorn.run(create_app(), host="127.0.0.1", port=int(os.getenv("NEMU_TTS_PORT", "17863")),
                access_log=False, log_level="warning", ws_max_size=65536)


if __name__ == "__main__":
    main()
