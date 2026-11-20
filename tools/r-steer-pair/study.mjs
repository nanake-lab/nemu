import { createHash } from 'node:crypto';

export const VERSION = 'R-STEER-PAIR-01-v1';
export const SOURCE = 'a8c57eb35cf5e823f8ee0a650efc8c3c707055e4';
export const QUESTION = '哪一句更想和对方亲近、继续来往？';
export const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const rank = (seed, domain, ...parts) => sha(JSON.stringify([seed, domain, ...parts]));

// PREREGISTRATION §2: selection never accepts output text or ratings.
export function schedule(roots, seed) {
  if (!/^[a-f0-9]{64}$/.test(seed)) throw new Error('Invalid seed');
  if (roots.length !== 12 || new Set(roots.map((r) => r.root)).size !== 12)
    throw new Error('Expected twelve unique roots');
  const families = new Map();
  for (const row of roots) families.set(row.root_family, (families.get(row.root_family) ?? 0) + 1);
  if (families.size !== 6 || [...families.values()].some((n) => n !== 2))
    throw new Error('Expected six two-root families');
  return [...roots].sort((a, b) => rank(seed, 'order', a.root).localeCompare(rank(seed, 'order', b.root)))
    .map((row, i) => ({
      position: i + 1, root: row.root, family: row.root_family,
      dose: ['p02', 'p04', 'p08'].sort((a, b) => rank(seed, 'dose', row.root, a).localeCompare(rank(seed, 'dose', row.root, b)))[0],
      injectedSide: parseInt(rank(seed, 'side', row.root).slice(-1), 16) % 2 ? 'right' : 'left',
    }));
}

// Symmetric mechanical screen, fixed in PREREGISTRATION §3. No semantic scoring.
export function screen(text, capped) {
  const letters = [...text.matchAll(/\p{L}/gu)].length;
  const han = [...text.matchAll(/\p{Script=Han}/gu)].length;
  return capped || !text.trim() || [...text].length > 300 || han < 6 || han / Math.max(1, letters) < 0.7
    || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f\ufffd]/u.test(text)
    || /(.{2,12})\1\1/u.test(text)
    || /<\||\[\[|<\/?think>|联结|共情|好奇|成事|注入|力度|steering|layer|alpha/iu.test(text);
}

export function blindPair(entry, context, injected, zero) {
  const blocked = screen(injected.text, injected.truncated) || screen(zero.text, zero.truncated);
  return {
    position: entry.position, screened: blocked,
    context: blocked ? '' : context,
    left: blocked ? '' : (entry.injectedSide === 'left' ? injected.text : zero.text),
    right: blocked ? '' : (entry.injectedSide === 'right' ? injected.text : zero.text),
  };
}

export function render(template, script, pairs) {
  const payload = { version: VERSION, question: QUESTION, pairs };
  payload.materialHash = sha(JSON.stringify(payload));
  const safe = JSON.stringify(payload).replaceAll('<', '\\u003c').replaceAll('\u2028', '\\u2028').replaceAll('\u2029', '\\u2029');
  return template.replace('/*__DATA__*/', `const study = ${safe};`).replace('/*__APP__*/', script);
}
