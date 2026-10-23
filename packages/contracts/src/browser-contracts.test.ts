import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { fromJson } from './generated/echo/bridge.js';
import { inputSchema } from './generated/echo/runtime.js';
import { wireInputSchema } from './generated/echo/browser.js';
import type { MessageMap } from './generated/echo/schema.js';
const limits = { max_bytes: 1048576, max_items: 4096, max_depth: 32 };

test('browser validator accepts every frozen legal vector accepted by authority codecs', () => {
  const vectors = JSON.parse(
    readFileSync(new URL('../../../core/contracts/golden/legal.json', import.meta.url), 'utf8'),
  ) as Array<{ request: { name: keyof MessageMap; value: Record<string, unknown> } }>;
  for (const { request } of vectors) {
    const value = fromJson(request.name, request.value);
    expect(inputSchema(request.name, limits).safeParse(value).success, request.name).toBe(true);
    expect(wireInputSchema(request.name).safeParse(value).success, request.name).toBe(true);
  }
});
