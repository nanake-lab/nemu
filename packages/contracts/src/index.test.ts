import { expect, test } from 'vitest';
import * as entry from './index';
test('contracts has an importable scaffold entry point', () => {
  expect(entry).toBeDefined();
});
