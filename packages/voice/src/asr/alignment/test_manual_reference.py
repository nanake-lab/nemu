"""Synthetic-only boundary import and distribution checks."""
from __future__ import annotations
import copy
import unittest
from manual_reference import PLAN_PATH, distribution, quantile, validate_annotations
from reference_data import file_sha


class ManualTests(unittest.TestCase):
    def fixture(self) -> tuple[dict[str,object],dict[str,object],list[dict[str,object]]]:
        ids=[f'synthetic-{i}' for i in range(26)]
        plan: dict[str,object]={'method':'synthetic','presentationOrder':ids}
        identities: list[dict[str,object]]=[{'id':i,'sha256':'a'*64,'frameCount':32000} for i in ids]
        rows=[dict(row,startSample=100,endSample=300) for row in identities]
        document: dict[str,object]={'method':'synthetic','planSha256':file_sha(PLAN_PATH),
                                   'status':'final','sampleRate':16000,'samples':rows}
        return plan,document,identities

    def test_linear_quantile_keeps_negative_tail(self) -> None:
        self.assertEqual(quantile([-10,0,10,100],.5),5)
        self.assertAlmostEqual(quantile([-10,0,10,100],.95),86.5)
        self.assertEqual(distribution([-1,0,250,1000])['histogram'],
            {'<0':1,'[0,200)':1,'[200,250)':0,'[250,300)':1,'[300,400)':0,
             '[400,600)':0,'[600,1000)':0,'>=1000':1})
        with self.assertRaises(ValueError):
            quantile([float('nan')],.5)

    def test_complete_valid_blind_document(self) -> None:
        plan,document,identities=self.fixture()
        self.assertEqual(len(validate_annotations(document,plan,identities)),26)

    def test_draft_and_wrong_method_cannot_become_reference(self) -> None:
        for key,value in [('status','draft'),('planSha256','changed'),('sampleRate',48000)]:
            plan,document,identities=self.fixture()
            document[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):
                validate_annotations(document,plan,identities)

    def test_missing_duplicate_tampered_and_invalid_boundaries_are_rejected(self) -> None:
        for change in ['missing','duplicate','hash','float','boolean','negative','reverse','outside']:
            plan,document,identities=self.fixture()
            rows=copy.deepcopy(identities)
            rows=[dict(row,startSample=100,endSample=300) for row in rows]
            if change=='missing': rows.pop()
            elif change=='duplicate': rows[-1]=dict(rows[0])
            elif change=='hash': rows[0]['sha256']='changed'
            elif change=='float': rows[0]['startSample']=1.5
            elif change=='boolean': rows[0]['startSample']=True
            elif change=='negative': rows[0]['startSample']=-1
            elif change=='reverse': rows[0]['endSample']=50
            elif change=='outside': rows[0]['endSample']=32001
            document['samples']=rows
            with self.subTest(change=change),self.assertRaises(ValueError):
                validate_annotations(document,plan,identities)


if __name__=='__main__':
    unittest.main()
