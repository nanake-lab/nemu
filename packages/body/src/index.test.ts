import { expect, test } from 'vitest';
import * as entry from './index';
test('body has an importable scaffold entry point', () => {
  expect(entry).toBeDefined();
});
