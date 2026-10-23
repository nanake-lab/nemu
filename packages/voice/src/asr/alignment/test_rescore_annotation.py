"""Synthetic tests for the zero-distance inference identity proof."""
import unittest
from rescore_annotation import correction


class RescoreTests(unittest.TestCase):
    def test_one_literal_extra_character(self) -> None:
        self.assertEqual(correction('あなた妊','あなた，\u6ca1\u6709\u598a',0,4),('あなた',1,3))

    def test_nonzero_or_other_text_cannot_be_inferred(self) -> None:
        for old,note,errors,count in [('あなた妊','あなた，\u6ca1\u6709\u598a',1,4),
                                    ('あなた妊','あなた，\u6ca1\u6709\u598a',0,5),
                                    ('あなた妊','わたし，\u6ca1\u6709\u598a',0,4),
                                    ('あなた','あなた，\u6ca1\u6709\u598a',0,3)]:
            with self.subTest(old=old,note=note,errors=errors,count=count),self.assertRaises(ValueError):
                correction(old,note,errors,count)


if __name__=='__main__':
    unittest.main()
