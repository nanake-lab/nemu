"""Single HTTP attempt with bounded partial evidence and syntactic no-retry guard."""
from __future__ import annotations

import argparse
import http.client
import json
import os
import re
import ssl
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

from ds_pilot import ENDPOINT, NoRedirect
from ds_protocol import write

NETWORK = (http.client.IncompleteRead, TimeoutError, ConnectionResetError,
           ConnectionAbortedError, http.client.RemoteDisconnected, ssl.SSLEOFError)


def parseable_content(raw: bytes) -> bool:
    # Conservative only: any complete JSON content value blocks retry, irrespective of schema/labels.
    text = raw.decode('utf-8', errors='replace')
    decoder = json.JSONDecoder()
    for match in re.finditer(r'"content"\s*:\s*', text):
        try:
            value, _ = decoder.raw_decode(text, match.end())
            if isinstance(value, str):
                json.loads(value)
                return True
        except (ValueError, TypeError):
            pass
    return False


def post(folder: Path, endpoint: str = ENDPOINT) -> int:
    key = os.environ.get('DEEPSEEK_API_KEY')
    if not key:
        return 3
    status: dict[str, object] = {'started_utc': datetime.now(timezone.utc).isoformat(), 'pid': os.getpid(),
                                'status': 'fatal_transport', 'response_complete': False}
    raw = bytearray()
    try:
        request = urllib.request.Request(endpoint, data=(folder / 'request.json').read_bytes(),
            headers={'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json',
                     'User-Agent': 'R-STEER-01-frozen-network-retry'}, method='POST')
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        with opener.open(request, timeout=90) as response:
            status['http_status'] = response.status
            expected = response.length
            while True:
                try:
                    chunk = response.read1(8192)
                except http.client.IncompleteRead as error:
                    raw.extend(error.partial)
                    raise
                if not chunk:
                    break
                raw.extend(chunk)
                if len(raw) > 2_000_000:
                    raise ValueError('Response byte bound')
            if expected is not None and len(raw) != expected:
                raise http.client.IncompleteRead(b'', expected - len(raw))
        if key.encode() in raw:
            raise ValueError('Unsafe response not persisted')
        (folder / 'response.json').write_bytes(raw)
        status.update(status='response_saved', response_complete=True, received_bytes=len(raw), retry_eligible=False)
        return 0
    except Exception as error:
        if isinstance(error, urllib.error.HTTPError):
            # Non-2xx is fatal, but preserve its bounded error body for diagnosis too.
            try:
                raw.extend(error.read(2_000_001))
            except Exception as body_error:
                status['error_body_read_error_class'] = type(body_error).__name__
        cause = error.reason if isinstance(error, urllib.error.URLError) else error
        status.update(error_class=type(error).__name__, cause_class=type(cause).__name__, received_bytes=len(raw))
        if isinstance(error, urllib.error.HTTPError):
            status['http_status'] = error.code
        safe = len(raw) <= 2_000_000 and key.encode() not in raw
        eligible = isinstance(cause, NETWORK) and not isinstance(error, urllib.error.HTTPError) and safe
        found = parseable_content(bytes(raw)) if safe else False
        status.update(parseable_score_content_found=found, retry_eligible=eligible and not found)
        if safe and raw:
            name = 'http-error.body' if isinstance(error, urllib.error.HTTPError) else 'partial-response.bin'
            (folder / name).write_bytes(raw)
        if eligible:
            status['status'] = 'network_incomplete'
            return 2
        return 3
    finally:
        status['ended_utc'] = datetime.now(timezone.utc).isoformat()
        write(folder / 'transport.json', status)
        os.environ.pop('DEEPSEEK_API_KEY', None)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--folder', type=Path, required=True)
    post_args = parser.parse_args()
    raise SystemExit(post(post_args.folder))
