import { readFile, realpath } from 'node:fs/promises';
import { isAbsolute, relative } from 'node:path';
import { Bridge } from './bridge.js';
import { RpcClient } from './rpc.js';
import { startProcess, type LaunchOptions } from './process.js';

export { Bridge, type RunInput, type TurnReceipt, type UsageRecord, type Tier } from './bridge.js';
export { RpcClient } from './rpc.js';

/** Configuration must be explicitly provided. There is no deployment model/deadline fallback. */
export async function connect(options: LaunchOptions & {
  helpersPath: string; rpcTimeoutMs: number; maxUsageEntries: number; workRoot: string;
}): Promise<Bridge> {
  const [root, scratch] = await Promise.all([realpath(options.ownedRoot), realpath(options.workRoot)]);
  const rel = relative(root, scratch);
  if (!rel || rel === '..' || rel.startsWith('..\\') || rel.startsWith('../') || isAbsolute(rel)) throw new Error('unowned_work_root');
  const [helpers, catalog] = await Promise.all([
    readFile(options.helpersPath, 'utf8'), readFile(options.catalogPath, 'utf8'),
  ]);
  const transport = await startProcess({ ...options, cwd: scratch });
  const rpc = new RpcClient(transport, options.maxFrameBytes);
  try {
    const bridge = new Bridge(rpc, JSON.parse(helpers), JSON.parse(catalog), options.rpcTimeoutMs, options.maxUsageEntries, scratch);
    await bridge.initialize();
    return bridge;
  } catch (error) { await rpc.close(); throw error; }
}
