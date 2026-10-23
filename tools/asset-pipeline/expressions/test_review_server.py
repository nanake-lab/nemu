from __future__ import annotations

import http.client
from http.server import ThreadingHTTPServer
import json
from pathlib import Path
import tempfile
import threading
from typing import Any
import unittest

from review_server import FIELDS, ReviewHandler, validate


class ReviewServerTests(unittest.TestCase):
    def test_reject_unknown_fields_invalid_choices_and_stale_revision(self) -> None:
        valid: dict[str, Any] = dict(fingerprint='current', choices=dict.fromkeys(FIELDS, ''), notes='')
        self.assertEqual(validate(valid, 'current'), valid)
        for payload in [dict(valid, path='../elsewhere'), dict(valid, fingerprint='old'),
                        dict(valid, choices={**valid['choices'], 'materials': 'invented'}),
                        dict(valid, choices={}), dict(valid, notes='x' * 8001)]:
            with self.assertRaises(ValueError):
                validate(payload, 'current')

    def test_http_receipts_are_append_only_and_reject_cross_origin(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            catalog = root / 'content/_candidates/P2-06/revision2/preview-catalog.json'
            catalog.parent.mkdir(parents=True)
            catalog.write_text('{"fingerprint":"current"}', encoding='utf-8')

            class Handler(ReviewHandler):
                def log_message(self, format: str, *args: object) -> None:
                    pass

            Handler.root = root
            server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                port = server.server_port
                connection = http.client.HTTPConnection('127.0.0.1', port)
                payload = dict(fingerprint='current', choices=dict.fromkeys(FIELDS, ''), notes='test only')
                receipts = []
                for _ in range(2):
                    connection.request('POST', '/api/p206/selections', json.dumps(payload), {'Content-Type': 'application/json'})
                    response = connection.getresponse()
                    self.assertEqual(response.status, 201)
                    receipts.append(json.loads(response.read())['receiptId'])
                self.assertNotEqual(receipts[0], receipts[1])
                files = list((root / 'content/_candidates/P2-06/review-submissions').glob('selection-*.json'))
                self.assertEqual(len(files), 2)
                connection.request('GET', '/api/p206/latest')
                response = connection.getresponse()
                self.assertEqual(response.status, 200)
                self.assertEqual(json.loads(response.read())['latest']['receiptId'], receipts[-1])
                connection.request('POST', '/api/p206/selections', json.dumps(payload), {'Content-Type': 'application/json', 'Origin': 'https://other.example'})
                response = connection.getresponse()
                self.assertEqual(response.status, 403)
                response.read()
                self.assertEqual(len(list(files[0].parent.glob('selection-*.json'))), 2)
                connection.close()
            finally:
                server.shutdown()
                server.server_close()
                thread.join()


if __name__ == '__main__':
    unittest.main()
