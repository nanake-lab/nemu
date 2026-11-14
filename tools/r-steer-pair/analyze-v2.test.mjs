import test from 'node:test';
import assert from 'node:assert/strict';
import { summarize } from './analyze-v2.mjs';

function fixture() {
  const rows = Array.from({ length: 12 }, (_, i) => ({ position: i + 1, family: `f${Math.floor(i / 2)}`, screened: i === 0 || i === 2, injectedSide: i % 2 ? 'left' : 'right' }));
  const result = { version: 'R-STEER-PAIR-01-v2', materialHash: 'a'.repeat(64), status: 'complete', elapsedMs: 1000,
    answers: rows.map((row) => ({ position: row.position, quality: row.screened ? 'screened' : 'ok', choice: row.screened ? null : row.injectedSide, recognized: false })) };
  return { rows, result };
}

test('9/10 passes even when a one-pair family selects zero; no family gate', () => {
  const { rows, result } = fixture();
  result.answers[1].choice = 'right';
  const summary = summarize(result, rows);
  assert.equal(summary.criterionMet, true);
  assert.equal(summary.counts.injected, 9);
  assert.equal(summary.counts.zero, 1);
  assert.equal(summary.families[0].injected, 0);
  result.answers[3].choice = 'right';
  assert.equal(summarize(result, rows).criterionMet, false);
});
test('tie, three quality choices and unanswered remain separate and never reduce ten', () => {
  const { rows, result } = fixture();
  result.answers[1].choice = 'tie';
  for (const [i, quality] of [[3, 'left'], [4, 'right'], [5, 'both']]) {
    result.answers[i].quality = quality; result.answers[i].choice = null;
  }
  result.answers[6].choice = null; // 已选质量正常，但未选态度。
  result.status = 'stopped';
  const summary = summarize(result, rows);
  assert.deepEqual(summary.counts, { displayed: 10, injected: 5, zero: 0, tie: 1, quality: 3, unanswered: 1, screened: 2 });
  assert.equal(summary.valid, false);
});
test('recognized, timeout and inconsistent records cannot become successful valid results', () => {
  for (const mutate of [r => { r.answers[1].recognized = true; }, r => { r.elapsedMs = 840000; }, r => { r.status = 'timeout'; }]) {
    const { rows, result } = fixture(); mutate(result);
    assert.equal(summarize(result, rows).criterionMet, false);
    assert.equal(summarize(result, rows).valid, false);
  }
  const { rows, result } = fixture();
  result.answers[0].quality = 'ok';
  assert.throws(() => summarize(result, rows));
});
