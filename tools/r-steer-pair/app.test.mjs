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
    study: { version: 'synthetic-only', materialHash: 'synthetic', question: '占位问题', pairs: Array.from({ length: 12 }, (_, i) => ({ position: i + 1, screened: i === 1, context: '测试', left: '左', right: '右' })) },
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

test('quality rejection skips preference, preserves fixed denominator and screened slot', () => {
  const h = harness(); h.start();
  h.qualities[1].events.click();
  assert.equal(h.state().position, 2);
  assert.equal(h.state().answers[0].quality, 'left');
  assert.equal(h.state().answers[0].choice, null);
  assert.equal(h.state().answers[1].quality, 'screened');
  h.click('stop');
  assert.equal(h.state().result.answers.length, 12);
  assert.equal(h.state().result.status, 'stopped');
});
test('approval, quality gate, tie and recognition are preserved without answers', () => {
  const h = harness(); h.click('start');
  assert.equal(h.get('intro').hidden, false);
  h.start();
  h.choices[0].events.click();
  assert.equal(h.state().answers[0].choice, null);
  h.qualities[0].events.click();
  h.get('recognized').checked = true; h.get('recognized').events.change();
  h.choices[2].events.click();
  h.click('stop');
  assert.equal(h.state().result.answers[0].choice, 'tie');
  assert.equal(h.state().result.answers[0].recognized, true);
});
test('overdue click cannot enter an answer even if timer did not run', () => {
  const h = harness(); h.start(); h.qualities[0].events.click();
  h.advanceClock(14 * 60 * 1000);
  h.choices[0].events.click();
  assert.equal(h.state().result.status, 'timeout');
  assert.equal(h.state().result.answers[0].choice, null);
  assert.equal(h.state().result.elapsedMs, 840000);
});
test('recognized material before timeout and unsaved-close warning are retained', () => {
  const h = harness(); h.start(); h.get('recognized').checked = true; h.get('recognized').events.change();
  h.advanceClock(840001); h.globalEvents.visibilitychange();
  assert.equal(h.state().result.answers[0].recognized, true);
  let prevented = false;
  h.globalEvents.beforeunload({ preventDefault: () => { prevented = true; } });
  assert.equal(prevented, true);
});
