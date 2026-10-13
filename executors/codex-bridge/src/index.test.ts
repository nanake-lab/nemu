import { expect, test } from 'vitest';
import * as entry from './index';
test('bridge scaffold imports without launching a process', () => {
  expect(entry).toBeDefined();
});
