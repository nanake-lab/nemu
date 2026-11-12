"""C1 checks for token evidence, prior cancellation and design balance."""
from __future__ import annotations

import tempfile
import unittest
from collections import Counter
from pathlib import Path

from analyze_c1 import Score, gain, inside, load_score
from c1_fixtures import make_jobs


class C1Checks(unittest.TestCase):
    def test_static_string_prior_cancels_but_tokens_must_match(self) -> None:
        self.assertAlmostEqual(gain(Score(-6,2,(8,9)),Score(-10,2,(8,9))),2)
        self.assertAlmostEqual(gain(Score(-46,2,(8,9)),Score(-50,2,(8,9))),2)
        with self.assertRaisesRegex(ValueError,'Different continuation'):
            gain(Score(-6,2,(8,9)),Score(-10,2,(8,10)))

    def test_numeric_headroom_is_not_available_when_anchors_collapse(self) -> None:
        self.assertTrue(inside(-1,0,1))
        self.assertFalse(inside(1,1,1))
        self.assertFalse(inside(-1,.99,1))

    def test_reader_reconstructs_target_log_probability_and_rejects_corruption(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'prompt-ids.tsv').write_text('1\n2\n')
            (root/'complete-ids.tsv').write_text('1\n2\n7\n')
            header = 'step\ttarget\targmax\tlogZ\ttarget_logit\tlogp\teog\ttop12-id-logit\n'
            (root/'logits.tsv').write_text(header+'0\t7\t8\t4\t2\t-2\t0\t8:3,\n')
            (root/'completion.tsv').write_text('steps\teog\tlimit\tsum_logp\n1\t0\t1\t-2\n')
            self.assertEqual(load_score(root),Score(-2,1,(7,)))
            (root/'logits.tsv').write_text(header+'0\t7\t8\t4\t2\t-1\t0\t8:3,\n')
            with self.assertRaisesRegex(ValueError,'Invalid per-token'):
                load_score(root)

    def test_pairing_and_counterfactuals_do_not_change_the_scored_strings(self) -> None:
        jobs = make_jobs()
        self.assertEqual(len({j['id'] for j in jobs}),len(jobs))
        self.assertEqual(Counter(j['lang'] for j in jobs),{'zh':562,'en':562,'ja':562})
        priors = {(j['root'],j['lang'],j['form'],j['sign']):j['continuation'] for j in jobs if j['kind']=='prior'}
        for job in jobs:
            if job['kind']=='target':
                self.assertEqual(job['continuation'],priors[(job['root'],job['lang'],job['form'],job['sign'])])
        facts = {(j['root'],j['lang'],j['level'],j['sign']):j for j in jobs if j['kind']=='fact'}
        for (scene,lang,level,sign),job in facts.items():
            self.assertEqual(job['continuation'],facts[(scene,lang,-level,sign)]['continuation'])
            self.assertEqual(job['prompt'],facts[(scene,lang,level,-sign)]['prompt'])


if __name__ == '__main__':
    unittest.main()
