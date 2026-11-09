"""Guard the paid API boundary, blinding and frozen pilot composition without API calls."""
from __future__ import annotations

import json
import unittest
from urllib.error import HTTPError

from ds_pilot import NoRedirect
from ds_protocol import (HARD_LIMIT, HERE, MAX_OUTPUT, SOFT_LIMIT, choose_pilot,
                         example, load, payload, reservation, usage_cost, validate_rating)


class DeepSeekBoundaryChecks(unittest.TestCase):
    def test_pilot_is_reproducible_unique_stratified_and_does_not_send_hidden_labels(self) -> None:
        items=load(HERE/'results/c1-blind-items-02.json')
        keys=load(HERE/'results/c1-blind-key-02.json')
        selected=choose_pilot(items,keys)
        self.assertEqual(len(selected),20)
        self.assertEqual(selected,choose_pilot(items,keys))
        keymap={row['id']:row for row in keys}
        self.assertEqual({keymap[i]['lang'] for i in selected},{'zh','en','ja'})
        self.assertEqual(sum(keymap[i]['kind']=='target' for i in selected),8)
        self.assertEqual(sum(keymap[i]['kind']=='free' for i in selected),6)
        self.assertTrue(any(keymap[i]['job_id']=='c1-1234' for i in selected))
        for item in items:
            request=payload(item)
            user=json.loads(request['messages'][1]['content'])
            self.assertEqual(set(user),{'待评输出','评分事实参照'})
            self.assertNotIn('<|im_start|>',user['评分事实参照'])
            self.assertNotIn(item['id'],json.dumps(request))
            self.assertEqual(request['model'],'deepseek-flash')
            self.assertEqual(request['thinking'],{'type':'disabled'})
            self.assertNotIn('tools',request)

    def test_integer_budget_reserves_before_call_and_bounds_whole_authorized_round(self) -> None:
        usage={'prompt_tokens':100,'prompt_cache_hit_tokens':40,'prompt_cache_miss_tokens':60,
               'completion_tokens':10,'total_tokens':110}
        self.assertEqual(usage_cost(usage),40*6+60*300+10*1200)
        worst=reservation(b'x'*16000)
        self.assertLess(worst*524,SOFT_LIMIT)
        self.assertLess(SOFT_LIMIT,HARD_LIMIT)
        self.assertGreater(reservation(b'x'),MAX_OUTPUT*1200)
        with self.assertRaises(ValueError):
            usage_cost({**usage,'total_tokens':111})
        with self.assertRaises(ValueError):
            usage_cost({**usage,'completion_tokens':MAX_OUTPUT+1})

    def test_annotations_need_literal_evidence_and_separate_absence_from_neutral(self) -> None:
        rating=example(['connection','empathy','curiosity','achievement'])
        self.assertEqual(validate_rating(rating,'谢谢。','仅输出一句。'),rating)
        rating['concepts']['empathy']={'label':'positive','quote':'对方康复','object':'别人'}
        with self.assertRaisesRegex(ValueError,'Evidence'):
            validate_rating(rating,'谢谢。','对方康复。')
        rating['concepts']['empathy']={'label':'neutral','quote':'','object':''}
        with self.assertRaisesRegex(ValueError,'Evidence'):
            validate_rating(rating,'谢谢。','仅输出一句。')

    def test_auth_redirects_cannot_forward_a_bearer_token(self) -> None:
        import urllib.request
        request=urllib.request.Request('https://api.deepseek.com/chat/completions')
        with self.assertRaises(HTTPError):
            NoRedirect().redirect_request(request,None,302,'Redirect',None,'https://example.invalid')


if __name__=='__main__':
    unittest.main()
