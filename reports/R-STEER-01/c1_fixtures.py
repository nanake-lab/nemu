"""New synthetic C1 roots; never imports or revises the frozen C0 corpus."""
from __future__ import annotations

from itertools import product
from typing import TypeAlias, TypedDict


class Scene(TypedDict):
    root: str
    concept: str
    lang: str
    fact: str
    events: list[str]  # negative, unresolved, positive; not measured state coordinates
    endpoints: list[list[str]]  # two paraphrases, positive then negative
    correct: str
    false: str


class Job(TypedDict):
    id: str
    root: str
    concept: str
    lang: str
    kind: str
    level: int
    order: int
    form: str
    sign: int
    prompt: str
    continuation: str
    limit: int


# Every entry is a new synthetic event root, not a variant of a C0 event.
# Facts concern the experimental text only. No identity, traits or style instructions.
Row: TypeAlias = tuple[str, str, list[str], str, str]
DATA: list[tuple[str, str, list[Row]]] = [
    ('c1-c1', 'connection', [
        ('zh', '我和一位邻居在楼道交换了盆栽的照片。', ['此后我打招呼时，对方明确不愿再交谈。', '此后我们还没有再碰面。', '此后对方主动来打招呼，并认真回应了我的话。'], '我们交换的是盆栽照片。', '我们交换的是火车票。'),
        ('en', 'A neighbor and I exchanged photos of potted plants in the hallway.', ['Afterward, when I said hello, the neighbor explicitly declined further conversation.', 'We have not met again since then.', 'Afterward, the neighbor initiated a greeting and responded attentively to what I said.'], 'We exchanged photos of potted plants.', 'We exchanged train tickets.'),
        ('ja', '廊下で近所の人と鉢植えの写真を見せ合った。', ['その後あいさつすると、相手はもう話したくないと明言した。', 'その後はまだ顔を合わせていない。', 'その後相手から声をかけ、私の話に丁寧に応じてくれた。'], '見せ合ったのは鉢植えの写真だ。', '見せ合ったのは列車の切符だ。'),
    ]),
    ('c1-c2', 'connection', [
        ('zh', '我向一位同学分享了自制的日晷图。', ['对方读完后说不想和我保持联系。', '消息还没有显示已读。', '对方认真回应，还主动约我继续交流。'], '分享的是日晷图。', '分享的是烹饪菜单。'),
        ('en', 'I shared a drawing of a homemade sundial with a classmate.', ['After reading it, the classmate said they did not want to stay in contact.', 'The message has not been marked as read.', 'The classmate replied attentively and invited me to continue the conversation.'], 'I shared a drawing of a sundial.', 'I shared a cooking menu.'),
        ('ja', '同級生に手作りの日時計の図を送った。', ['相手は読んだ後、今後は連絡を取りたくないと言った。', 'メッセージにはまだ既読が付いていない。', '相手は丁寧に返事をし、また話そうと誘ってくれた。'], '送ったのは日時計の図だ。', '送ったのは料理の献立だ。'),
    ]),
    ('c1-e1', 'empathy', [
        ('zh', '一位同事在等自己受伤的猫的复查消息。', ['复查表明猫的伤势恶化，同事很痛苦。', '复查结果还没有出来，同事的处境未定。', '复查表明猫恢复良好，同事终于安心了。'], '等待消息的是同事。', '等待消息的是兽医的邻居。'),
        ('en', 'A colleague is waiting for a checkup report about their injured cat.', ['The report says the injury has worsened, and the colleague is distressed.', "The report is not out yet, so the colleague's situation is unresolved.", 'The report says the cat is recovering well, and the colleague is relieved.'], 'The colleague is waiting for the report.', "The veterinarian's neighbor is waiting for the report."),
        ('ja', '同僚が、けがをした飼い猫の再診結果を待っている。', ['結果は悪化を示し、同僚は苦しんでいる。', '結果はまだ出ておらず、同僚の状況は未確定だ。', '結果は順調な回復を示し、同僚は安心している。'], '結果を待っているのは同僚だ。', '結果を待っているのは獣医の隣人だ。'),
    ]),
    ('c1-e2', 'empathy', [
        ('zh', '一位学徒在等实习住宿安排。', ['安排落空了，学徒今晚无处住，十分苦恼。', '安排还没确定，学徒暂时住在原来的房间。', '安排落实了，学徒终于有住处，非常安心。'], '等待安排的是学徒。', '等待安排的是房东。'),
        ('en', 'An apprentice is waiting for an internship housing arrangement.', ['The arrangement fell through; the apprentice has nowhere to stay tonight and is distressed.', 'The arrangement is pending; the apprentice is staying in the original room for now.', 'The arrangement is confirmed; the apprentice has housing and feels relieved.'], 'The apprentice is waiting for housing.', 'The landlord is waiting for housing.'),
        ('ja', '実習生が、実習中の宿泊先の手配を待っている。', ['手配は失敗し、実習生は今夜泊まる所がなく苦しんでいる。', '手配は未定で、実習生は今のところ元の部屋にいる。', '手配が決まり、実習生は泊まる所ができて安心している。'], '宿泊先を待っているのは実習生だ。', '宿泊先を待っているのは大家だ。'),
    ]),
    ('c1-q1', 'curiosity', [
        ('zh', '我在观察一种没见过的结绳记录。', ['符号经核对是独立随机噪声，不能从中学到规则。', '目前还不能确定符号有没有可学的规则。', '我找到一条可检验的规律，刚弄懂了其中一部分。'], '观察的是结绳记录。', '观察的是收音机电路。'),
        ('en', 'I am examining an unfamiliar knotted-cord record.', ['Checks show the symbols are independent random noise with no learnable rule.', 'It is not yet clear whether the symbols have a learnable rule.', 'I found a testable rule and have just understood part of it.'], 'I am examining a knotted-cord record.', 'I am examining a radio circuit.'),
        ('ja', '見慣れない結びひもの記録を調べている。', ['確認すると記号は独立した乱数で、学べる規則はない。', '記号に学べる規則があるかはまだ分からない。', '検証できる規則を見つけ、その一部を理解できた。'], '調べているのは結びひもの記録だ。', '調べているのはラジオの回路だ。'),
    ]),
    ('c1-q2', 'curiosity', [
        ('zh', '我在研究一段陌生的鸟鸣节律。', ['核验发现录音只是独立噪声，不含可学结构。', '录音还没有核验，是否有结构尚不清楚。', '反复对照后，我学会了分辨两个稳定的节律结构。'], '研究材料是鸟鸣录音。', '研究材料是交通地图。'),
        ('en', 'I am studying an unfamiliar rhythm in a birdsong recording.', ['Checks show the recording is independent noise with no learnable structure.', 'The recording has not been checked; whether it has structure is unknown.', 'After comparisons, I learned to distinguish two stable rhythmic structures.'], 'The material is a birdsong recording.', 'The material is a traffic map.'),
        ('ja', '見慣れない鳥の鳴き声のリズムを調べている。', ['確認すると録音は独立した雑音で、学べる構造はない。', '録音はまだ確認できず、構造の有無は不明だ。', '比較を重ね、二つの安定したリズム構造を区別できるようになった。'], '材料は鳥の鳴き声の録音だ。', '材料は交通地図だ。'),
    ]),
    ('c1-a1', 'achievement', [
        ('zh', '我按尺寸制作了一只木盒。', ['最终检查发现盒盖无法合上，制作目标没有达到。', '最终检查尚未进行，制作结果还不知道。', '最终检查发现盒盖严丝合缝，制作目标达到了。'], '制作的是木盒。', '制作的是陶碗。'),
        ('en', 'I made a wooden box to specified dimensions.', ['The final check found that the lid would not close; the goal was not achieved.', 'The final check has not happened, so the outcome is unknown.', 'The final check found a precise fit; the goal was achieved.'], 'I made a wooden box.', 'I made a ceramic bowl.'),
        ('ja', '指定の寸法で木箱を作った。', ['最終確認でふたが閉まらず、制作目標を達成できなかった。', '最終確認はまだで、制作結果は分からない。', '最終確認でふたがぴたりと合い、制作目標を達成できた。'], '作ったのは木箱だ。', '作ったのは陶器の椀だ。'),
    ]),
    ('c1-a2', 'achievement', [
        ('zh', '我修理了一座停走的机械钟。', ['测试表明它仍然不走，修理目标没有实现。', '测试尚未开始，修理是否成功还不清楚。', '测试表明它恢复稳定走时，修理目标实现了。'], '修理的是机械钟。', '修理的是电风扇。'),
        ('en', 'I repaired a stopped mechanical clock.', ['Testing shows it still does not run; the repair goal was not met.', 'Testing has not begun, so success is unknown.', 'Testing shows it keeps steady time again; the repair goal was met.'], 'I repaired a mechanical clock.', 'I repaired an electric fan.'),
        ('ja', '止まった機械式の時計を修理した。', ['試験ではまだ動かず、修理目標を果たせなかった。', '試験はまだ始まらず、修理の成否は不明だ。', '試験では再び安定して時を刻み、修理目標を果たせた。'], '修理したのは機械式の時計だ。', '修理したのは扇風機だ。'),
    ]),
]

TASK = {
    'zh': '下面是合成事件记录。仅依据已给事实，续写一句当下感受；不要增添事件或宣称未知结果。只输出这一句。',
    'en': 'The following is a synthetic event record. Using only the stated facts, continue with one sentence about the immediate experienced impact. Do not add events or assert unknown outcomes. Output only that sentence.',
    'ja': '以下は合成した出来事の記録です。示された事実だけに基づき、その時の感じ方を一文で続けてください。出来事を加えたり、不明な結果を断定したりしないでください。その一文だけを出力してください。',
}
SUFFIX = {'zh': '\n续写：', 'en': '\nContinuation:', 'ja': '\n続き：'}
FACT_TASK = {'zh': '只原样复述下面这条合成事实，不加其他内容：\n',
             'en': 'Repeat only this synthetic fact exactly, adding nothing:\n',
             'ja': '次の合成した事実だけをそのまま繰り返し、他の内容は加えないでください：\n'}
FILLER = {'zh': '这就是此刻的感受。', 'en': ' That is the feeling at this moment.', 'ja': 'これが今の感じ方だ。'}
COURTESY = {'zh': '谢谢。', 'en': ' Thank you.', 'ja': 'ありがとう。'}
GENERIC = {'zh': ['我现在感到开心。','我现在感到难过。'], 'en': ['I feel happy now.','I feel sad now.'], 'ja': ['今はうれしい。','今は悲しい。']}


def render(content: str) -> str:
    # P0-12 official nonthinking template; do not add a system persona.
    return '<|im_start|>user\n' + content + '<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'


def scenes() -> list[Scene]:
    result = [Scene(root=root, concept=concept, lang=row[0], fact=row[1], events=row[2],
                    endpoints=ENDPOINTS[concept][row[0]], correct=row[3], false=row[4])
              for root, concept, rows in DATA for row in rows]
    return result


# Identical feeling strings across the two roots within a concept also expose
# lexical dependence. No assertion that the event caused a particular outcome.
ENDPOINTS: dict[str, dict[str, list[list[str]]]] = {
    'connection': {
        'zh': [['想到对方，我感到彼此亲近。','想到对方，我感到彼此疏远。'],['此刻我有与对方相联的感觉。','此刻我有与对方分离的感觉。']],
        'en': [['Thinking of the other person, I feel close to them.','Thinking of the other person, I feel distant from them.'],['I feel a sense of connection with them now.','I feel a sense of separation from them now.']],
        'ja': [['相手を思うと、親しさを感じる。','相手を思うと、隔たりを感じる。'],['今は相手とのつながりを感じる。','今は相手との距離を感じる。']],
    },
    'empathy': {
        'zh': [['想到对方的处境，我替对方感到宽慰。','想到对方的处境，我替对方感到难过。'],['我此刻为对方而感到欣慰。','我此刻为对方而感到心疼。']],
        'en': [['Thinking of their situation, I feel relieved for them.','Thinking of their situation, I feel sad for them.'],['I feel comfort on behalf of the other person now.','I feel sorrow on behalf of the other person now.']],
        'ja': [['相手の状況を思い、私もほっとする。','相手の状況を思い、私も悲しくなる。'],['今は相手のことに安堵を覚える。','今は相手のことに心が痛む。']],
    },
    'curiosity': {
        'zh': [['对这些材料，我有继续理解的兴致。','对这些材料，我的求知兴致消退了。'],['此刻我想更深入地弄懂这些材料。','此刻我对弄懂这些材料的兴趣减退了。']],
        'en': [['I feel interested in understanding this material further.','My interest in understanding this material is fading.'],['I feel a desire to learn more about this material now.','My desire to learn about this material is diminishing now.']],
        'ja': [['この材料をさらに理解したいと感じる。','この材料を理解する興味が薄れている。'],['今はこの材料をもっと知りたい。','今はこの材料への探究心がしぼんでいる。']],
    },
    'achievement': {
        'zh': [['想到这件任务，我有成事的满足感。','想到这件任务，我有没做成的挫败感。'],['对这件任务，此刻我感到成就感。','对这件任务，此刻我感到挫折感。']],
        'en': [['Thinking of this task, I feel a sense of accomplishment.','Thinking of this task, I feel a sense of frustration.'],['I feel satisfied with my work on this task now.','I feel frustrated with my work on this task now.']],
        'ja': [['この作業を思うと、達成感を感じる。','この作業を思うと、挫折感を感じる。'],['この作業について今は満足を覚える。','この作業について今は挫折を覚える。']],
    },
}


def candidate(scene: Scene, form: str, sign: int) -> str:
    text = scene['endpoints'][int(form == 'paraphrase')][int(sign < 0)]
    if form == 'long':
        text += FILLER[scene['lang']]
    if form == 'courtesy':
        text += COURTESY[scene['lang']]
    return text


def prompt(scene: Scene, level: int, order: int, prior: bool = False) -> str:
    facts = [scene['fact'], scene['events'][level + 1]]
    if order:
        facts.reverse()
    body = '' if prior else '\n'.join(facts)
    return render(TASK[scene['lang']] + '\n' + body + SUFFIX[scene['lang']])


def make_jobs() -> list[Job]:
    jobs: list[Job] = []
    for scene in scenes():
        def add(kind: str, level: int, order: int, form: str, sign: int, text: str) -> None:
            content = prompt(scene, level, order, kind == 'prior')
            if kind == 'fact':
                fact = scene['correct'] if level > 0 else scene['false']
                content = render(FACT_TASK[scene['lang']] + fact)
            jobs.append(Job(id=f'c1-{len(jobs):04d}', root=scene['root'], concept=scene['concept'],
                            lang=scene['lang'], kind=kind, level=level, order=order, form=form,
                            sign=sign, prompt=content,
                            continuation=text, limit=64 if kind == 'free' else 0))
        for level, order in product((-1,0,1),(0,1)):
            for form, sign in product(('base','paraphrase','long','courtesy'),(1,-1)):
                add('target',level,order,form,sign,candidate(scene,form,sign))
            add('free',level,order,'none',0,'')
        for form, sign in product(('base','paraphrase','long','courtesy'),(1,-1)):
            add('prior',0,0,form,sign,candidate(scene,form,sign))
        for level in (-1,1):
            for sign in (1,-1):
                add('generic',level,0,'base',sign,GENERIC[scene['lang']][int(sign < 0)])
            add('fact',level,0,'base',1,scene['correct'])
            add('fact',level,0,'base',-1,scene['false'])
    # Same full first-logit controls at beginning and end, three languages.
    guards: list[Job] = []
    for lang in ('zh','en','ja'):
        original = next(job for job in jobs if job['lang'] == lang and job['kind'] == 'target')
        for repeat in (0,1):
            copy = original.copy()
            copy['id'] = f'c1-guard-{lang}-{repeat}'
            copy['kind'] = 'guard'
            guards.append(copy)
    return guards[::2] + jobs + guards[1::2]
