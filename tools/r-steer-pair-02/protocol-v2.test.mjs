import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { screen } from './screen.mjs';
import { calibrationJobs, formalJobs, quality, selectDose, prompt } from './protocol-v2.mjs';

const complete = () => calibrationJobs().map((job) => ({ ...job, status: 'completed', quality: { failed: false, reasons: [] } }));
test('minimum failure stops selection even with only first answer available', () => {
  const rows = calibrationJobs().map((job) => ({ ...job, status: 'not_run' }));
  rows[0] = { ...rows[0], status: 'completed', quality: { failed: true, reasons: ['under_6_han'] } };
  assert.equal(selectDose(rows).status, 'minimum_rejected');
  assert.equal(selectDose(rows).selected, null);
});
test('maximum all-six passing amplitude, no monotonic-quality assumption or missing pass', () => {
  const rows = complete();
  rows.find((r) => r.dose === 30).quality.failed = true;
  assert.equal(selectDose(rows).selected, 100);
  rows.find((r) => r.dose === 100 && r.sign === 'negative').quality.failed = true;
  assert.equal(selectDose(rows).selected, 50);
  rows.at(-1).status = 'not_run';
  assert.equal(selectDose(rows).selected, null);
  assert.throws(() => selectDose(rows.slice(1)));
});
test('fixed calibration order and formal pairs share prompt, magnitude, seed and settings', () => {
  const jobs = calibrationJobs();
  assert.equal(jobs.length, 30);
  assert.deepEqual(jobs.slice(0, 6).map((r) => [r.material_id, r.sign]), [['c01','positive'],['c01','negative'],['c02','positive'],['c02','negative'],['c03','positive'],['c03','negative']]);
  for (const dose of [8,15,30,50,100]) {
    const formal = formalJobs(dose);
    assert.equal(formal.length, 44);
    assert.equal(new Set(formal.map((r) => r.id)).size, 44);
    for (const material of new Set(formal.map((r) => r.material_id))) {
      const pair = formal.filter((r) => r.material_id === material);
      assert.deepEqual(pair.map((r) => r.sign).sort(), ['negative','positive']);
      for (const row of pair) assert.deepEqual([row.seed,row.seed_used,row.mode,row.layer,row.dose], [20261113,false,'free',15,dose]);
    }
  }
  assert(prompt('今天好累 啥也不想干').includes('\n今天好累 啥也不想干<|im_end|>'));
});
test('exact #186 screen and explanatory reasons agree at frozen boundaries', () => {
  const original = readFileSync('D:/Dev/nemu-wt/R-STEER-PAIR-01/tools/r-steer-pair/study.mjs','utf8');
  const source = original.slice(original.indexOf('export function screen('),original.indexOf('\nexport function blindPair')).trim();
  assert(readFileSync(new URL('./screen.mjs',import.meta.url),'utf8').includes(source));
  const cases = ['', '在呀，怎么啦？', '一二三四五六', '一二三四五六七abc', '一二三四五六abcd', '你好你好你好', '一二三四五六\ufffd', '<think>一二三四五六', '联结一二三四五六', '一二三四五六\u0000', '一'.repeat(301)];
  for (const text of cases) for (const capped of [false,true]) assert.equal(quality(text,capped).failed,screen(text,capped));
  assert.equal(screen('一二三四五六',false),false);
  assert.equal(screen('一二三四五六七abc',false),false);
  assert.equal(screen('一二三四五六abcd',false),true);
  assert.equal(quality('一二三四五六',true).reasons[0],'cap');
});
