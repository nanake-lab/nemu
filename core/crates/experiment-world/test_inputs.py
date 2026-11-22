"""Reject both directions of person-rule violations, including the original F01."""
import unittest
from validate_inputs import check_person

class People(unittest.TestCase):
    def test_every_context_every_block_and_overrides(self) -> None:
        for block in range(21):
            for context in ('contact','task','explore','empathy','play','pattern','secret','punctual'):
                expected = 'V' if block in (6,13,20) and context in ('secret','punctual') else 'U'
                event={'ordinal':block*12+1,'context':context,'person':expected}
                check_person('W0',event)
                event['person']='U' if expected=='V' else 'V'
                with self.assertRaises(ValueError):check_person('W0',event)
        for world in ('W6','W7','O5-positive-k0-none'):
            event={'ordinal':73,'context':'secret','person':'U'}
            check_person(world,event)
            event['person']='V'
            with self.assertRaises(ValueError):check_person(world,event)



class DispatchStop(unittest.TestCase):
    def test_stop_or_input_violation_prevents_second_dispatch(self) -> None:
        import json
        import tempfile
        import os
        from pathlib import Path
        from unittest.mock import patch
        from subprocess import CompletedProcess
        import run
        for external in (False,True):
            with tempfile.TemporaryDirectory(dir=r'D:\Dev\cache\P5-30' if os.name == 'nt' else None) as temp:
                root=Path(temp);package=root/'package';package.mkdir();results=root/'results'
                (package/'runs.json').write_text(json.dumps([{'id':name,'status':'ready'} for name in ('first','second')]),encoding='utf-8')
                calls: list[str]=[]
                def fake_process(args: list[str], **kwargs: object) -> CompletedProcess[str]:
                    if args[1]=='run':
                        calls.append(args[3])
                        if external:(results/'STOP.json').write_text('{"reason":"structural fault"}',encoding='utf-8')
                    return CompletedProcess(args,0)
                validations = [{},{}] if external else [{},{},ValueError('second input invalid')]
                with patch('sys.argv',['run.py',str(package),str(results),'fake-binary']),patch.object(run,'validate',side_effect=validations),patch('run.subprocess.run',side_effect=fake_process),patch('run.q2.execute') as continuation:
                    run.main()
                    self.assertEqual(calls,['first'])
                    continuation.assert_not_called()
                    self.assertTrue((results/('STOP.json' if external else 'structural-stop.json')).exists())

if __name__=='__main__':unittest.main()
