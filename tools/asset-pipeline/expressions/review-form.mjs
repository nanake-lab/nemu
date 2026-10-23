const data = JSON.parse(document.getElementById('review-data').textContent);
const controls = [...document.querySelectorAll('select[name]')];
const notes = document.getElementById('notes');
const status = document.getElementById('save-status');
const key = 'nemu-p206-review-' + data.fingerprint;
let hasDraft = false;
let dirty = false;
function payload() {
  return { fingerprint: data.fingerprint, choices: Object.fromEntries(controls.map(c => [c.name, c.value])), notes: notes.value };
}
function restore(value) {
  for (const c of controls) if (typeof value.choices?.[c.name] === 'string' && [...c.options].some(o => o.value === value.choices[c.name])) c.value = value.choices[c.name];
  notes.value = value.notes || '';
}
restore({ choices: data.prefill, notes: '' });
try {
  const draft = localStorage.getItem(key);
  if (draft) { hasDraft = true; restore(JSON.parse(draft)); status.textContent = '已恢复本轮浏览器草稿；尚未提交的新改动请点提交。'; }
} catch { status.textContent = '浏览器草稿不可用，可用“导出草稿”备份。'; }
function save() {
  dirty = true;
  delete status.dataset.receipt;
  try { localStorage.setItem(key, JSON.stringify(payload())); status.textContent = '浏览器草稿已保存 · 本次修改尚未提交'; }
  catch { status.textContent = '浏览器草稿保存失败，请导出草稿备份。'; }
}
for (const c of controls) c.addEventListener('change', save);
notes.addEventListener('input', save);
try {
  const response = await fetch('/api/p206/latest');
  if (response.ok) {
    const { latest } = await response.json();
    if (!hasDraft && !dirty && latest?.fingerprint === data.fingerprint) {
      restore(latest);
      status.textContent = '已恢复上次提交 · 回执 ' + latest.receiptId;
    }
  }
} catch { /* Draft/export still work while the save service is unavailable. */ }
document.getElementById('download').onclick = () => {
  const url = URL.createObjectURL(new Blob([JSON.stringify(payload(), null, 2)], { type: 'application/json' }));
  const link = document.createElement('a');
  link.href = url; link.download = 'P2-06-selection-' + data.revision + '.json'; link.click();
  URL.revokeObjectURL(url);
};
document.getElementById('submit').onclick = async () => {
  const button = document.getElementById('submit');
  button.disabled = true;
  try {
    const response = await fetch('/api/p206/selections', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload()) });
    const result = await response.json();
    if (!response.ok || !result.saved) throw Error(result.error || '服务未确认保存');
    try { localStorage.setItem(key, JSON.stringify(payload())); } catch { /* Server receipt is authoritative. */ }
    status.textContent = '已提交并保存到本机候选目录 · 回执 ' + result.receiptId;
    status.dataset.receipt = result.receiptId;
  } catch (error) {
    status.textContent = '提交未完成：' + error.message + '。草稿仍在页面中，也可导出备份。';
  } finally { button.disabled = false; }
};
