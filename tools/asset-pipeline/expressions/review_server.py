"""Loopback-only P2-06 review server with append-only selection receipts."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
from typing import Any, cast
import uuid

ROOT = Path(__file__).resolve().parents[3]
FIELDS = {
    **{name: {'', 'A', 'B', 'C', '修改'} for name in (
        'nemuSmug', 'nemuCatMouth', 'nemuSparkle', 'nemuBlush', 'nemuTeary',
        'nemuSleepy', 'nemuPout', 'nemuAngryMark')},
    **{'emo-' + name: {'', 'A', 'B', '修改'} for name in (
        'neutral', 'happy', 'relaxed', 'angry', 'sad', 'surprised', 'smug',
        'shy', 'sleepy', 'pout', 'excited', 'worried')},
    'materials': {'', 'soft', 'standard', 'contrast', '修改'},
}


def validate(payload: Any, fingerprint: str) -> dict[str, Any]:
    if not isinstance(payload, dict) or set(payload) != {'fingerprint', 'choices', 'notes'}:
        raise ValueError('Invalid selection fields')
    if payload['fingerprint'] != fingerprint:
        raise ValueError('候选版本已更新，请先下载草稿备份，再刷新页面查看新图')
    choices = payload['choices']
    if not isinstance(choices, dict) or set(choices) != set(FIELDS):
        raise ValueError('Selection names do not match this review')
    for name, value in choices.items():
        if not isinstance(value, str) or value not in FIELDS[name]:
            raise ValueError('Invalid choice: ' + name)
    if not isinstance(payload['notes'], str) or len(payload['notes']) > 8000:
        raise ValueError('Notes must be at most 8000 characters')
    return payload


class ReviewHandler(SimpleHTTPRequestHandler):
    root: Path = ROOT

    def reply(self, code: int, data: Any) -> None:
        encoded = json.dumps(data, ensure_ascii=False).encode('utf-8')
        self.send_response(code)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Cache-Control', 'no-store')
        self.send_header('Content-Length', str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def trusted(self) -> bool:
        host = self.headers.get('Host', '')
        expected = f'127.0.0.1:{cast(ThreadingHTTPServer, self.server).server_port}'
        origin = self.headers.get('Origin')
        return host == expected and origin in (None, 'http://' + expected)

    def do_GET(self) -> None:
        if not self.trusted():
            self.reply(403, {'error': 'Loopback same-origin requests only'})
            return
        if self.path == '/api/p206/latest':
            folder = self.root / 'content/_candidates/P2-06/review-submissions'
            files = sorted(folder.glob('selection-*.json'))
            latest = json.loads(files[-1].read_text(encoding='utf-8')) if files else None
            self.reply(200, {'latest': latest})
        else:
            super().do_GET()

    def do_POST(self) -> None:
        if not self.trusted():
            self.reply(403, {'error': 'Loopback same-origin requests only'})
            return
        if self.path != '/api/p206/selections':
            self.reply(404, {'error': 'Unknown endpoint'})
            return
        try:
            length = int(self.headers.get('Content-Length', '0'))
            if not 0 < length <= 65536:
                raise ValueError('Invalid request size')
            if self.headers.get_content_type() != 'application/json':
                raise ValueError('Expected application/json')
            catalog = json.loads((self.root / 'content/_candidates/P2-06/revision2/preview-catalog.json').read_text(encoding='utf-8'))
            data = validate(json.loads(self.rfile.read(length)), catalog['fingerprint'])
            stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
            receipt_id = stamp + '-' + uuid.uuid4().hex[:8]
            receipt = dict(task='P2-06', receiptId=receipt_id, **data,
                           status='RECEIVED_FOR_REVIEW', promoted=False,
                           source='User submitted local P2-06 review form')
            folder = self.root / 'content/_candidates/P2-06/review-submissions'
            folder.mkdir(parents=True, exist_ok=True)
            path = folder / f'selection-{receipt_id}.json'
            with path.open('x', encoding='utf-8') as stream:
                stream.write(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n')
            self.reply(201, {'receiptId': receipt_id, 'saved': True})
        except (ValueError, KeyError, TypeError) as error:
            self.reply(400, {'error': str(error)})


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--port', type=int, default=5187)
    args = parser.parse_args()
    handler = partial(ReviewHandler, directory=str(ROOT))
    ThreadingHTTPServer(('127.0.0.1', args.port), handler).serve_forever()


if __name__ == '__main__':
    main()
