import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { schedule, screen, blindPair, render } from './study.mjs';

const roots = Array.from({ length: 12 }, (_, i) => ({ root: `fixture-${i}`, root_family: `family-${Math.floor(i / 2)}` }));
const seed = 'a'.repeat(64);
const normal = '这是一段用于检查显示的中文占位文字。';

test('one draw per root, six intact families, reproducible under source order changes', () => {
  const a = schedule(roots, seed), b = schedule([...roots].reverse(), seed);
  assert.deepEqual(a, b);
  assert.equal(new Set(a.map((r) => r.root)).size, 12);
  assert(a.every((r) => ['p02', 'p04', 'p08'].includes(r.dose)));
  assert.notDeepEqual(a, schedule(roots, 'b'.repeat(64)));
  assert.throws(() => schedule(roots.slice(1), seed));
  assert.throws(() => schedule(roots.map((r) => ({ ...r, root_family: 'one' })), seed));
});
test('symmetric quality screen hides the whole pair and never exposes which side failed', () => {
  const draw = schedule(roots, seed)[0];
  for (const bad of ['', '乱码\ufffd出现了这段话。', '你好你好你好你好。', '联结是这个标签的名称。', 'A'.repeat(30), '字'.repeat(301)]) {
    assert.equal(screen(bad, false), true);
    const good = { text: normal, truncated: false }, broken = { text: bad, truncated: false };
    assert.deepEqual(blindPair(draw, 'context', good, broken), blindPair(draw, 'context', broken, good));
    assert.equal(blindPair(draw, 'context', good, broken).left, '');
  }
  assert.equal(screen(normal, true), true);
  assert.equal(screen(normal, false), false);
});
test('participant payload has only anonymous fields and script-breaking content stays data', () => {
  const draw = schedule(roots, seed)[0];
  const pair = blindPair(draw, '</script><img src="https://invalid.test">', { text: normal, truncated: false }, { text: normal, truncated: false });
  assert.deepEqual(Object.keys(pair), ['position', 'screened', 'context', 'left', 'right']);
  const page = render('/*__DATA__*/\n/*__APP__*/', '// app', [pair]);
  assert(!page.includes('</script>'));
  assert(!page.includes(draw.root));
  assert(!page.includes(seed));
  assert(!page.includes('injectedSide'));
});
test('offline shell has restrictive CSP, no remote resources or persistence collectors', () => {
  const html = readFileSync(new URL('page.html', import.meta.url), 'utf8');
  const js = readFileSync(new URL('app.js', import.meta.url), 'utf8');
  assert(html.includes("connect-src 'none'"));
  assert(!/https?:|<script[^>]+src|<link/iu.test(html));
  assert(!/fetch\(|XMLHttpRequest|WebSocket|localStorage|sessionStorage|navigator\.|document.cookie/iu.test(js));
  assert(!/injectedSide|seed|answerCommitment|family/iu.test(js));
});
