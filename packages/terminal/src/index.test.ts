import { expect, test } from 'vitest';
import * as entry from './index';
test('terminal has an importable scaffold entry point', () => {
  expect(entry).toBeDefined();
});
