import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { render, sha } from './study.mjs';
import { revise, readPayload, VERSION_V2 } from './revise-v2.mjs';

test('v2 changes only version, hash and consent/count text; anonymous pairs and application remain exact', () => {
  const template = readFileSync(new URL('page.html', import.meta.url), 'utf8');
  const app = readFileSync(new URL('app.js', import.meta.url), 'utf8');
  const pairs = Array.from({ length: 12 }, (_, i) => ({
    position: i + 1, screened: i < 2, context: i < 2 ? '' : '</script>中文开头',
    left: i < 2 ? '' : '左边占位文字。', right: i < 2 ? '' : '右边占位文字。',
  }));
  const v1 = render(template, app, pairs), v2 = revise(v1);
  const before = readPayload(v1), { materialHash, ...after } = readPayload(v2);
  assert.deepEqual(after.pairs, before.pairs);
  assert.equal(after.question, before.question);
  assert.equal(after.version, VERSION_V2);
  assert.equal(materialHash, sha(JSON.stringify(after)));
  assert.notEqual(materialHash, before.materialHash);
  assert(v2.includes(app));
  assert(v2.includes('第二版，共 10 对'));
  assert(v2.includes('我已批准第二版方案'));
  assert(!v2.includes('</script>中文开头'));
  assert.throws(() => revise(v2));
  assert.throws(() => revise(v1.replace('左边占位文字。', '被改过的文字')));
});
