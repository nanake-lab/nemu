// Only anonymous study material is bundled above; no answer key or scoring code.
const element = (id) => document.getElementById(id);
let started = 0;
let startedMonotonic = 0;
let position = 0;
let result = null;
let downloaded = false;
const answers = study.pairs.map((pair) => ({
  position: pair.position, quality: pair.screened ? 'screened' : 'unanswered',
  choice: null, conditionKnown: false, familiar: false,
}));
const elapsed = () => Math.max(Date.now() - started, performance.now() - startedMonotonic, 0);

function finish(status) {
  if (result) return;
  result = {
    version: study.version, materialHash: study.materialHash, status,
    elapsedMs: Math.round(elapsed()), answers,
  };
  element('trial').hidden = true;
  element('done').hidden = false;
  element('status').textContent = status === 'complete' ? '所有可比较的题目已提交。'
    : '已保留目前的选择。';
}
function show() {
  while (position < study.pairs.length && study.pairs[position].screened) position++;
  if (position === study.pairs.length) { finish('complete'); return; }
  const pair = study.pairs[position];
  element('progress').textContent = `第 ${pair.position} / ${study.pairs.length} 对`;
  for (const name of ['context', 'left', 'right']) element(name).textContent = pair[name];
  element('conditionKnown').checked = false;
  element('familiar').checked = false;
  element('quality').hidden = false;
  element('choice').hidden = true;
}
function advance() {
  answers[position].conditionKnown = element('conditionKnown').checked;
  answers[position].familiar = element('familiar').checked;
  position++;
  show();
}
element('approved').addEventListener('change', () => { element('start').disabled = !element('approved').checked; });
element('start').addEventListener('click', () => {
  if (started || !element('approved').checked) return;
  started = Date.now(); startedMonotonic = performance.now();
  element('intro').hidden = true; element('trial').hidden = false;
  element('question').textContent = study.question;
  show();

});
document.querySelectorAll('[data-quality]').forEach((button) => button.addEventListener('click', () => {
  if (!started || result || element('quality').hidden) return;
  const quality = button.dataset.quality;
  answers[position].quality = quality;
  if (quality !== 'ok') { advance(); return; }
  element('quality').hidden = true; element('choice').hidden = false;
}));
document.querySelectorAll('[data-choice]').forEach((button) => button.addEventListener('click', () => {
  if (!started || result || element('choice').hidden) return;
  answers[position].choice = button.dataset.choice;
  advance();
}));
element('qualityBack').addEventListener('click', () => {
  if (!started || result) return;
  answers[position].quality = 'unanswered';
  element('quality').hidden = false; element('choice').hidden = true;
});
element('conditionKnown').addEventListener('change', () => {
  if (started && !result) answers[position].conditionKnown = element('conditionKnown').checked;
});
element('familiar').addEventListener('change', () => {
  if (started && !result) answers[position].familiar = element('familiar').checked;
});
element('stop').addEventListener('click', () => { if (started && !result) finish('stopped'); });
element('download').addEventListener('click', () => {
  if (!result) return;
  const url = URL.createObjectURL(new Blob([JSON.stringify(result, null, 2) + '\n'], { type: 'application/json' }));
  const link = document.createElement('a');
  link.href = url; link.download = 'R-STEER-PAIR-02-results.json';
  document.body.append(link); link.click(); link.remove();
  // The browser releases this download URL when the document closes.
  downloaded = true;
  element('saveNotice').textContent = '已请求下载；请检查浏览器的下载列表或所选文件夹。';
});
element('saved').addEventListener('change', () => {
  if (!downloaded) { element('saved').checked = false; element('saveNotice').textContent = '请先下载结果文件。'; }
});
addEventListener('beforeunload', (event) => {
  if (started && !(downloaded && element('saved').checked)) { event.preventDefault(); event.returnValue = ''; }
});
