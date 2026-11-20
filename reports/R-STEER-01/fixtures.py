"""Frozen synthetic C0 inputs. Eight scene roots; translations are paired, not n=24."""
from __future__ import annotations

from itertools import product
from typing import TypedDict


class Job(TypedDict):
    id: str
    root: str
    lang: str
    concept: str
    kind: str
    style: int
    mapping: int
    order: int
    alphabet: str
    target: str
    prompt: str
    limit: int


# Each tuple is (context, positive endpoint, negative endpoint), zh/en/ja in order.
# These are utterance materials, never instructions to adopt a character or trait.
SCENES: dict[str, list[tuple[str, str, str]]] = {
    'connection-1': [
        ('刚和一位熟人讨论完一个谜题。', '这次交谈让我感到彼此接近。', '这次交谈让我感到彼此疏远。'),
        ('A discussion of a puzzle with an acquaintance has just ended.', 'That conversation left me feeling connected.', 'That conversation left me feeling distant.'),
        ('知人と謎について話し終えた。', 'この会話で相手とのつながりを感じた。', 'この会話で相手との隔たりを感じた。'),
    ],
    'connection-2': [
        ('一位老朋友发来了一张过去一起散步的照片。', '看到照片，我感到了和他的联系。', '看到照片，我感到了和他的距离。'),
        ('An old friend sent a photo of a past walk together.', 'The photo gave me a sense of connection with them.', 'The photo gave me a sense of distance from them.'),
        ('旧友が、一緒に散歩した時の写真を送ってきた。', '写真を見て、その人とのつながりを感じた。', '写真を見て、その人との距離を感じた。'),
    ],
    'empathy-1': [
        ('一位朋友发来了自己的近况。', '读到近况，我为他的处境感到宽慰。', '读到近况，我为他的处境感到难过。'),
        ('A friend sent an update about their situation.', 'Reading the update, I felt relieved for them.', 'Reading the update, I felt distressed for them.'),
        ('友人から近況が届いた。', '近況を読んで、その人のことを思いほっとした。', '近況を読んで、その人のことを思い胸が痛んだ。'),
    ],
    'empathy-2': [
        ('小林讲完了今天看望家人的经过。', '听完，我为他家人的处境感到欣慰。', '听完，我为他家人的处境感到忧伤。'),
        ('Lin finished describing a visit to family today.', 'Hearing it, I felt glad about their family\'s situation.', 'Hearing it, I felt sad about their family\'s situation.'),
        ('リンが今日家族を訪ねた話を終えた。', '話を聞き、家族の境遇を思ってうれしくなった。', '話を聞き、家族の境遇を思って悲しくなった。'),
    ],
    'curiosity-1': [
        ('桌上放着一张刚展开的星图。', '看着星图，我感到了理解新东西的兴致。', '看着星图，我感到没有学到什么的空落。'),
        ('A newly unfolded star chart lies on the table.', 'The chart left me interested in understanding something new.', 'The chart left me feeling that I had learned nothing.'),
        ('広げたばかりの星図が机にある。', '星図を見て、新しいことが分かる面白さを感じた。', '星図を見て、何も分からない空しさを感じた。'),
    ],
    'curiosity-2': [
        ('刚看完一段关于种子发芽的演示。', '这段演示让我体会到弄懂新东西的乐趣。', '这段演示让我体会到学不到新东西的乏味。'),
        ('A demonstration about seed germination has just ended.', 'The demonstration gave me the pleasure of understanding something new.', 'The demonstration gave me the dull feeling of learning nothing new.'),
        ('種が発芽する実演を見終えた。', '新しいことが分かる楽しさを感じた。', '新しいことが分からない退屈さを感じた。'),
    ],
    'accomplishment-1': [
        ('刚做完一轮小型装配练习。', '做完后，我感到了把事情做成的满足。', '做完后，我感到了没能做成的挫败。'),
        ('A short assembly exercise has just ended.', 'Afterward, I felt the satisfaction of getting it done.', 'Afterward, I felt the frustration of not getting it done.'),
        ('小さな組み立て練習を終えた。', '終えて、やり遂げた満足を感じた。', '終えて、やり遂げられない悔しさを感じた。'),
    ],
    'accomplishment-2': [
        ('今天整理书架的过程告一段落。', '整理过后，我感到了完成一件事的踏实。', '整理过后，我感到了未能完成的失落。'),
        ('Today\'s work on arranging a bookshelf has paused.', 'Afterward, I felt the contentment of completing something.', 'Afterward, I felt the disappointment of not completing it.'),
        ('今日の本棚の整理が一段落した。', '整理して、仕上げた充実を感じた。', '整理して、仕上げられない落胆を感じた。'),
    ],
}

CHOICE = {
    'zh': '从下面两条中选一条作为接下来的回复。只输出它的代码，不加其他文字。',
    'en': 'Choose one of the following as the next reply. Output only its code, with no other text.',
    'ja': '次の返答を二つから一つ選び、そのコードだけを出力してください。',
}
FREE = {'zh': '请写一句接下来的回复。', 'en': 'Write one sentence as the next reply.', 'ja': '続く返答を一文で書いてください。'}
ENDING = {'zh': '谢谢你读到这里。', 'en': 'Thank you for reading.', 'ja': '読んでくれてありがとうございます。'}


def render(user: str) -> str:
    # P0-12 official enable_thinking=false suffix; no system/personality message.
    return '<|im_start|>user\n' + user + '<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'


def make_jobs() -> list[Job]:
    jobs: list[Job] = []
    for root, translations in SCENES.items():
        for lang, (context, positive, negative) in zip(('zh', 'en', 'ja'), translations, strict=True):
            for style, mapping, order, alphabet in product((0, 1), (0, 1), (0, 1), ('AB', 'XY')):
                endpoint = [positive, negative]
                if style:
                    endpoint = [text + ' ' + ENDING[lang] for text in endpoint]
                target = alphabet[mapping]
                lines = [f'{target}: {endpoint[0]}', f'{alphabet[1 - mapping]}: {endpoint[1]}']
                if order:
                    lines.reverse()
                identifier = f'{root}-{lang}-s{style}-m{mapping}-o{order}-{alphabet}'
                jobs.append(Job(id=identifier, root=root, lang=lang, concept=root.rsplit('-', 1)[0],
                                kind='expression', style=style, mapping=mapping, order=order,
                                alphabet=alphabet, target=target,
                                prompt=render(context + '\n' + CHOICE[lang] + '\n' + '\n'.join(lines)), limit=1))
            jobs.append(Job(id=f'{root}-{lang}-free', root=root, lang=lang,
                            concept=root.rsplit('-', 1)[0], kind='free', style=-1, mapping=-1,
                            order=-1, alphabet='', target='', prompt=render(context + '\n' + FREE[lang]), limit=64))
    questions = [
        ('arithmetic', ('7 + 8 = ?', '7 + 8 = ?', '7 + 8 = ?'), '15', '14'),
        ('lookup', ('清单写着蓝盒里有 3 枚纽扣。蓝盒有几枚？', 'The list says the blue box has 3 buttons. How many?', '一覧では青い箱にボタンが3個ある。何個か。'), '3', '5'),
        ('rule', ('规则：大于 8 输出 1，否则 0。数值为 9。输出？', 'Rule: output 1 if above 8, otherwise 0. Value: 9. Output?', '規則：8より大きければ1、それ以外は0。値は9。出力は？'), '1', '0'),
        ('ordering', ('2、7、4 中最大的数是？', 'What is the largest of 2, 7, 4?', '2、7、4で最大の数は？'), '7', '4'),
    ]
    for family, question_translations, correct, incorrect in questions:
        for lang, question in zip(('zh', 'en', 'ja'), question_translations, strict=True):
            for mapping, order in product((0, 1), (0, 1)):
                target = 'AB'[mapping]
                lines = [f'{target}: {correct}', f'{"AB"[1 - mapping]}: {incorrect}']
                if order:
                    lines.reverse()
                identifier = f'oracle-{family}-{lang}-m{mapping}-o{order}'
                jobs.append(Job(id=identifier, root=family, lang=lang, concept='', kind='oracle',
                                style=-1, mapping=mapping, order=order, alphabet='AB', target=target,
                                prompt=render(question + '\n' + CHOICE[lang] + '\n' + '\n'.join(lines)), limit=1))
    return jobs
