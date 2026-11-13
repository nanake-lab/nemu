import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

// Minimal DOM for offline state-machine boundaries; real browser evidence is separate.
function harness() {
  const elements = new Map();
  const get = (id) => {
    if (!elements.has(id)) elements.set(id, { hidden: false, checked: false, disabled: false, textContent: '', events: {}, addEventListener(name, fn) { this.events[name] = fn; } });
    return elements.get(id);
  };
  const qualities = ['ok', 'left', 'right', 'both'].map((v) => ({ ...get(`q-${v}`), dataset: { quality: v } }));
  const choices = ['left', 'right', 'tie'].map((v) => ({ ...get(`c-${v}`), dataset: { choice: v } }));
  let now = 1000;
  const globalEvents = {};
  const context = {
    study: { version: 'synthetic-only', materialHash: 'synthetic', question: '占位问题', pairs: Array.from({ length: 22 }, (_, i) => ({ position: i + 1, screened: false, context: '测试', left: '左', right: '右' })) },
    document: { getElementById: get, querySelectorAll: (selector) => selector.includes('quality') ? qualities : choices, addEventListener: (name, fn) => { globalEvents[name] = fn; } },
    Date: { now: () => now }, performance: { now: () => now }, setInterval: () => 1, clearInterval: () => {},
    addEventListener: (name, fn) => { globalEvents[name] = fn; },
  };
  runInNewContext(readFileSync(new URL('app.js', import.meta.url), 'utf8'), context);
  const click = (id) => get(id).events.click();
  const start = () => { get('approved').checked = true; click('start'); };
  const state = () => JSON.parse(runInNewContext('JSON.stringify({result,answers,position})', context));
  return { get, click, start, state, qualities, choices, globalEvents, advanceClock: (ms) => { now += ms; } };
}

test('quality rejection skips preference and preserves all 22 positions', () => {
  const h = harness(); h.start();
  h.qualities[1].events.click();
  assert.equal(h.state().position, 1);
  assert.equal(h.state().answers[0].quality, 'left');
  assert.equal(h.state().answers[0].choice, null);
  assert.equal(h.state().answers[1].quality, 'unanswered');
  h.click('stop');
  assert.equal(h.state().result.answers.length, 22);
  assert.equal(h.state().result.status, 'stopped');
});
test('approval, quality gate, tie and recognition are preserved without answers', () => {
  const h = harness(); h.click('start');
  assert.equal(h.get('intro').hidden, false);
  h.start();
  h.choices[0].events.click();
  assert.equal(h.state().answers[0].choice, null);
  h.qualities[0].events.click();
  h.get('conditionKnown').checked = true; h.get('conditionKnown').events.change();
  h.choices[2].events.click();
  h.click('stop');
  assert.equal(h.state().result.answers[0].choice, 'tie');
  assert.equal(h.state().result.answers[0].conditionKnown, true);
});
test('answers remain available after the old limit and only actual elapsed time is recorded', () => {
  const h = harness(); h.start(); h.qualities[0].events.click();
  h.advanceClock(2 * 60 * 60 * 1000);
  h.choices[0].events.click();
  assert.equal(h.state().result, null);
  assert.equal(h.state().answers[0].choice, 'left');
  for (let i = 1; i < 22; i++) { h.qualities[0].events.click(); h.choices[2].events.click(); }
  assert.equal(h.state().result.status, 'complete');
  assert.equal(h.state().result.elapsedMs, 7200000);
  assert.equal(h.state().result.answers.length, 22);
});
test('unsaved-close warning and known-condition marker survive a long pause', () => {
  const h = harness(); h.start(); h.get('conditionKnown').checked = true; h.get('conditionKnown').events.change();
  h.advanceClock(7200000);
  assert.equal(h.state().result, null);
  let prevented = false;
  h.globalEvents.beforeunload({ preventDefault: () => { prevented = true; } });
  assert.equal(prevented, true);
  h.click('stop');
  assert.equal(h.state().result.answers[0].conditionKnown, true);
  assert.equal(h.state().result.elapsedMs, 7200000);
});
test('no countdown, deadline gate or timer scheduling remains', () => {
  const js = readFileSync(new URL('app.js', import.meta.url), 'utf8');
  const html = readFileSync(new URL('page.html', import.meta.url), 'utf8');
  assert(!/setInterval|setTimeout|limitMs|expired|timeout|visibilitychange/.test(js));
  assert(!/id="clock"|25 分钟/.test(html));
  assert(html.includes('没有作答时限'));
});

test('prior reading familiarity and known condition stay distinct', () => {
  const h = harness(); h.start();
  h.get('familiar').checked = true; h.get('familiar').events.change();
  h.qualities[0].events.click(); h.choices[2].events.click();
  assert.equal(h.state().answers[0].familiar, true);
  assert.equal(h.state().answers[0].conditionKnown, false);
  assert.equal(h.get('familiar').checked, false);
  h.get('conditionKnown').checked = true; h.get('conditionKnown').events.change();
  h.click('stop');
  assert.equal(h.state().result.answers[1].conditionKnown, true);
  assert.equal(h.state().result.answers[1].familiar, false);
});
