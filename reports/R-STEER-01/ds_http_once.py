"""One official HTTP attempt in a killable child; no retry, parsing or billing."""
from __future__ import annotations

import argparse
import os
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

from ds_pilot import ENDPOINT, NoRedirect
from ds_protocol import write


def post(folder: Path, endpoint: str = ENDPOINT) -> int:
    # The CLI always uses ENDPOINT. A loopback endpoint is passed only by offline tests.
    key = os.environ.get('DEEPSEEK_API_KEY')
    if not key:
        return 2
    status: dict[str, object] = {'started_utc': datetime.now(timezone.utc).isoformat(),
                                'pid': os.getpid(), 'status': 'transport_failed'}
    try:
        request = urllib.request.Request(endpoint, data=(folder / 'request.json').read_bytes(),
            headers={'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json',
                     'User-Agent': 'R-STEER-01-frozen-high-resume'}, method='POST')
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        try:
            with opener.open(request, timeout=90) as response:
                raw = response.read(2_000_001)
                status['http_status'] = response.status
        except urllib.error.HTTPError as error:
            raw = error.read(2_000_001)
            if key.encode() not in raw:
                (folder / 'http-error.body').write_bytes(raw)
            status.update(status='http_error', http_status=error.code)
            return 2
        if len(raw) > 2_000_000 or key.encode() in raw:
            status['status'] = 'unsafe_or_oversized_not_persisted'
            return 2
        (folder / 'response.json').write_bytes(raw)
        status['status'] = 'response_saved'
        return 0
    except Exception as error:
        status['error_class'] = type(error).__name__
        return 2
    finally:
        status['ended_utc'] = datetime.now(timezone.utc).isoformat()
        write(folder / 'transport.json', status)
        os.environ.pop('DEEPSEEK_API_KEY', None)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--folder', type=Path, required=True)
    args = parser.parse_args()
    raise SystemExit(post(args.folder.resolve()))
