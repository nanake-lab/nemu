"""Adversarial preregistration checks; fabricated diagnostic rows, no life run."""
import copy
import unittest
from typing import Any
from analyze import endpoint

def rows(count: int, rotate: bool=False, marginal: float=0.25) -> list[dict[str, Any]]:
    out=[]
    for day in range(count+1):
        out.append({'day':day,'state':{'r':[],'theta':{}},'probes':[{'id':'p','delta':[0.25 if day else 0.,0.,0.,0.],'learned_delta':[],'marginal':[{'id':f't{day%2}' if rotate else 't','f':[0.25,0.,0.,0.],'nature':[marginal,0.,0.,0.],'learned':0.}]}]})
    return out

class Preregistration(unittest.TestCase):
    def test_right_censor_and_same_tendency_requirement(self) -> None:
        self.assertEqual(endpoint(rows(29),True)['M1']['status'],'failed')
        self.assertEqual(endpoint(rows(30),True)['M1']['first']['confirmation_day'],30)
        self.assertEqual(endpoint(rows(60,True),True)['M1']['status'],'failed')
    def test_zero_actual_margin_cannot_be_replaced_by_nonzero_raw_f(self) -> None:
        self.assertEqual(endpoint(rows(60,marginal=0.),True)['M1']['status'],'failed')
    def test_incomplete_is_not_a_negative_result_and_analysis_is_read_only(self) -> None:
        data=rows(15);original=copy.deepcopy(data)
        self.assertEqual(endpoint(data,False)['M1']['status'],'not_run')
        self.assertEqual(data,original)
if __name__=='__main__':unittest.main()
