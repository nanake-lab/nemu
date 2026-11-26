import { expect, it } from 'vitest';
import { REQUIRED_PROCESSES, createProcessConfigs } from './process-config.js';

it('supervises the real voice entry with a liveness endpoint that permits degradation', () => {
  const spec = REQUIRED_PROCESSES.find((entry) => entry.name === 'voice-sidecar');
  expect(spec).toBeDefined();
  expect(spec?.args).toEqual(['src/main.py']);
  expect(spec?.healthProbe).toMatchObject({
    type: 'http-get',
    endpoint: 'http://127.0.0.1:17863/health',
  });
  const config = createProcessConfigs(spec ? [spec] : [])[0];
  expect(config?.command).toBe(spec?.command);
  expect(config?.healthCheck).toEqual(spec?.healthProbe);
});
