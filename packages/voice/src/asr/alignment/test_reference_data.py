"""Pure reference handling tests: no model, device, microphone or user's storage."""
from __future__ import annotations
import json
from pathlib import Path
import tempfile
import unittest
import wave
from import_continuous import extract
from reference_data import boundaries, freeze, inside, recordings


class ReferenceTests(unittest.TestCase):
    def test_pcm_extraction_preserves_samples_and_original(self) -> None:
        with tempfile.TemporaryDirectory(prefix='nemu-pcm-slice-') as tmp:
            source, target = Path(tmp)/'source.wav', Path(tmp)/'slice.wav'
            pcm = bytes(range(24))
            with wave.open(str(source), 'wb') as stream:
                stream.setnchannels(1)
                stream.setsampwidth(2)
                stream.setframerate(16000)
                stream.writeframes(pcm)
            original = source.read_bytes()
            extract(source, target, 4, 8)
            with wave.open(str(target), 'rb') as stream:
                self.assertEqual(stream.readframes(4), pcm[8:16])
            self.assertEqual(source.read_bytes(), original)
            with self.assertRaises(FileExistsError):
                extract(source, target, 4, 8)

    def test_first_last_lexical_units_only(self) -> None:
        items: list[dict[str, object]] = [
            {'text': '。', 'start_time': 0, 'end_time': 0},
            {'text': '今日', 'start_time': 0.8, 'end_time': 1.2},
            {'text': '休む', 'start_time': 1.28, 'end_time': 2.4},
            {'text': '！', 'start_time': 2.4, 'end_time': 2.4}]
        self.assertEqual(boundaries(items, 3.0), (800.0, 2400.0))

    def test_bad_spans_are_not_dropped_or_clamped(self) -> None:
        for start, end in [(1, 1), (-0.1, 1), (0, float('nan')), (0, 4)]:
            with self.subTest(start=start, end=end), self.assertRaises(ValueError):
                boundaries([{'text': 'voice', 'start_time': start, 'end_time': end}], 3)
        with self.assertRaises(ValueError):
            boundaries([], 3)
        with self.assertRaises(ValueError):
            boundaries([{'text': 'a', 'start_time': 1, 'end_time': 2},
                        {'text': 'b', 'start_time': 0, 'end_time': 1}], 3)

    def test_frozen_input_changes_fail_and_original_is_preserved(self) -> None:
        with tempfile.TemporaryDirectory(prefix='nemu-alignment-test-') as tmp:
            path = Path(tmp) / 'inputs.json'
            freeze(path, [{'sha256': 'original'}])
            self.assertEqual(json.loads(path.read_text()), [{'sha256': 'original'}])
            with self.assertRaises(ValueError):
                freeze(path, [{'sha256': 'changed'}])
            self.assertEqual(json.loads(path.read_text()), [{'sha256': 'original'}])

    def test_incomplete_coverage_and_path_escape_fail(self) -> None:
        with tempfile.TemporaryDirectory(prefix='nemu-alignment-test-') as tmp:
            root = Path(tmp)
            (root / 'user.json').write_text('[]')
            with self.assertRaisesRegex(ValueError, '60'):
                recordings(root, {'samples': [{'id': 'user-zh-001'}]})
            with self.assertRaises(ValueError):
                inside(root, '../not-a-user-recording.wav')


if __name__ == '__main__':
    unittest.main()
