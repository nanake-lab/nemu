import { expect, test } from 'vitest';
import * as entry from './index';
test('desktop-sense has an importable scaffold entry point', () => {
  expect(entry).toBeDefined();
});
