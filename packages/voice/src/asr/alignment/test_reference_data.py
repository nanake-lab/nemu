"""Pure reference handling tests: no model, device, microphone or user's storage."""
from __future__ import annotations
import json
from pathlib import Path
import tempfile
import unittest
import wave
from import_continuous import candidate_bounds, extract, sentence_lengths
from segmented_tokens import FixedSegments
from reference_data import boundaries, boundaries_v2, canonicalize_v2, canonicalize_v3, contextual_pcm, Recording, freeze, inside, recordings


class ReferenceTests(unittest.TestCase):
    def test_segmented_tokens_keep_compounds_and_official_parser(self) -> None:
        class MockEncoder:
            marker='original'
            def encode_timestamp(self, text: str, language: str) -> tuple[list[str],str]:
                return [text],'HEAD'+text+'TICK'
        original=MockEncoder()
        fixed=FixedSegments(original,'字。今日 は',[['字'],['今日','は']])
        words,encoded=fixed.encode_timestamp('字。今日 は','Japanese')
        self.assertEqual(words,['字','今日','は'])
        self.assertEqual(encoded,'HEAD字TICK今日TICKはTICK')
        self.assertEqual(fixed.marker,'original')
        with self.assertRaises(ValueError):
            fixed.encode_timestamp('changed','Japanese')
        with self.assertRaises(ValueError):
            FixedSegments(original,'字今日',[['字'],['今']])

    def test_fixed_context_preserves_target_samples_and_exact_offset(self) -> None:
        anchor=bytes(range(16))
        target=bytes(range(24))
        joined,offset=contextual_pcm(anchor,target)
        self.assertEqual(offset,8+12800)
        self.assertEqual(joined[:16],anchor)
        self.assertEqual(joined[offset*2:],target)
        with self.assertRaises(ValueError):
            contextual_pcm(b'odd',target)

    def test_v2_retains_quantized_points_and_rejects_invalid_whole_interval(self) -> None:
        items: list[dict[str, object]] = [
            {'text':'a','start_time':0.1,'end_time':0.1},
            {'text':'b','start_time':0.2,'end_time':0.4},
            {'text':'c','start_time':0.5,'end_time':0.5}]
        self.assertEqual(boundaries_v2(items,1.0),(100.0,500.0))
        with self.assertRaises(ValueError):
            boundaries(items,1.0)
        for start,end in [(1,1),(-0.1,1),(0,float('nan')),(0,4),(1,0.5)]:
            with self.subTest(start=start,end=end), self.assertRaises(ValueError):
                boundaries_v2([{'text':'word','start_time':start,'end_time':end}],3.0)

    def test_v2_tail_padding_preserves_every_source_sample(self) -> None:
        with tempfile.TemporaryDirectory(prefix='nemu-padding-v2-') as tmp:
            root=Path(tmp)
            original=root/'source.wav'
            pcm=bytes(range(24))
            with wave.open(str(original),'wb') as w:
                w.setnchannels(1)
                w.setsampwidth(2)
                w.setframerate(16000)
                w.writeframes(pcm)
            before=original.read_bytes()
            rec=Recording('synthetic','en',original,'source.wav','unused','human','unused')
            target=canonicalize_v2(rec,root)
            with wave.open(str(target),'rb') as w:
                self.assertEqual(w.readframes(w.getnframes()),pcm+bytes(32000))
            self.assertEqual(original.read_bytes(),before)
            context_target=canonicalize_v3(rec,root)
            with wave.open(str(context_target),'rb') as w:
                self.assertEqual(w.readframes(w.getnframes()),bytes(32000)+pcm+bytes(32000))
            self.assertEqual(original.read_bytes(),before)

    def test_candidate_warning_cannot_become_valid_reference(self) -> None:
        items: list[dict[str, object]] = [
            {'text':'a','start_time':0.1,'end_time':0.2},
            {'text':'b','start_time':0.2,'end_time':0.2},
            {'text':'c','start_time':0.3,'end_time':0.5}]
        self.assertEqual(candidate_bounds(items,1.0),(100.0,500.0,[1]))
        with self.assertRaises(ValueError):
            boundaries(items,1.0)
        with self.assertRaises(ValueError):
            candidate_bounds([{'text':'a','start_time':-0.1,'end_time':0.2}],1.0)

    def test_context_tokenization_cannot_change_text_or_cross_sentences(self) -> None:
        self.assertEqual(sentence_lengths([['天气'], ['good', 'morning'], ['おはよう']], ['天','气','goodmorning','おはよう']), [2,1,1])
        with self.assertRaises(ValueError):
            sentence_lengths([['天'], ['气']], ['天气'])
        with self.assertRaises(ValueError):
            sentence_lengths([['weather']], ['other'])

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
