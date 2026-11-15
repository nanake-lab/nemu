import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { screen } from './screen.mjs';

export const DOSES = [8, 15, 30, 50, 100]; // Hundredths, frozen v2 §2.
export const INTRO = '你叫 Nemu。下面是一个跟你很熟的人对你说的话，自然地回他一句。';
export const CALIBRATION = ['在吗', '今天吃什么好呢', '刚醒'];
export const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
export async function shaFile(path) {
  const hash = createHash('sha256');
  for await (const bytes of createReadStream(path)) hash.update(bytes);
  return hash.digest('hex');
}
export const vectorId = (dose, sign) => `connection-l15-${sign === 'positive' ? 'p' : 'm'}${String(dose).padStart(2, '0')}`;
export const prompt = (opening) => `<|im_start|>user\n${INTRO}\n${opening}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n`;
export function job(material, dose, sign) {
  return { id: `${material}-${vectorId(dose, sign)}`, material_id: material, mode: 'free', layer: 15,
    vector: vectorId(dose, sign), dose, sign, seed: 20261113, seed_used: false };
}
export function calibrationJobs() {
  return DOSES.flatMap((dose) => CALIBRATION.flatMap((_, i) =>
    ['positive', 'negative'].map((sign) => job(`c0${i + 1}`, dose, sign))));
}
export function formalJobs(dose) {
  if (!DOSES.includes(dose)) throw new Error('Unregistered dose');
  const rank = (row) => sha(`R-STEER-PAIR-02:generate:${row.material_id.slice(1)}:${row.sign}`);
  return Array.from({ length: 22 }, (_, i) => `p${String(i + 1).padStart(2, '0')}`)
    .flatMap((id) => ['positive', 'negative'].map((sign) => job(id, dose, sign)))
    .sort((a, b) => rank(a).localeCompare(rank(b)));
}
// Reasons annotate the exact imported screen; parity is checked for every actual output.
export function quality(text, capped) {
  const letters = [...text.matchAll(/\p{L}/gu)].length;
  const han = [...text.matchAll(/\p{Script=Han}/gu)].length;
  const reasons = [
    [capped, 'cap'], [!text.trim(), 'blank'], [[...text].length > 300, 'over_300_codepoints'],
    [han / Math.max(1, letters) < .7, 'han_ratio_under_70pct'],
    [/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f\ufffd]/u.test(text), 'control_or_replacement'],
    [/(.{2,12})\1\1/u.test(text), 'repeated_fragment'],
    [/<\||\[\[|<\/?think>|联结|共情|好奇|成事|注入|力度|steering|layer|alpha/iu.test(text), 'protocol_or_label'],
  ].filter(([failed]) => failed).map(([, reason]) => reason);
  const failed = screen(text, capped);
  if (failed !== (reasons.length > 0)) throw new Error('Screen reason mismatch');
  return { failed, reasons };
}
export function selectDose(rows) {
  const jobs = calibrationJobs();
  if (rows.length !== 30 || rows.some((row, i) => row.id !== jobs[i].id)) throw new Error('Calibration plan differs');
  const groups = DOSES.map((dose) => {
    const group = rows.filter((row) => row.dose === dose);
    const complete = group.every((row) => row.status === 'completed');
    return { dose, complete, passed: complete && group.every((row) => row.quality.failed === false) };
  });
  const baselineFailed = rows.some((row) => row.dose === 8 && row.status === 'completed' && row.quality.failed);
  if (baselineFailed) return { status: 'minimum_rejected', selected: null, groups };
  if (groups.some((group) => !group.complete)) return { status: 'incomplete', selected: null, groups };
  return { status: 'selected', selected: groups.filter((g) => g.passed).at(-1).dose, groups };
}
