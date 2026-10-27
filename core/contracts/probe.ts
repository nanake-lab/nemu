// Test-only JSON-lines adapter, run with Node's TypeScript stripping.
import { ingressProbe } from './ingress_probe.js';
import { createInterface } from 'node:readline';
import { fromJson, toJson } from '../../packages/contracts/src/generated/echo/bridge.js';
import {
  encodeCanonicalData,
  decodeCanonicalData,
  encodeProtobufData,
  decodeProtobufData,
  digest,
  normalize,
  type Limits,
} from '../../packages/contracts/src/generated/echo/runtime.js';

for await (const line of createInterface({ input: process.stdin })) {
  try {
    const q = JSON.parse(line) as {
      name: string;
      op: string;
      value: Record<string, unknown>;
      hex: string;
      limits: Limits;
    };
    if (q.op === 'ingress') {
      ingressProbe(q, q.limits);
      console.log(JSON.stringify({ ok: true }));
      continue;
    }
    const v =
      q.op === 'encode'
        ? normalize(q.name, fromJson(q.name, q.value), q.limits)
        : q.op === 'canonical'
          ? decodeCanonicalData(q.name, Buffer.from(q.hex, 'hex'), q.limits)
          : decodeProtobufData(q.name, Buffer.from(q.hex, 'hex'), q.limits);
    console.log(
      JSON.stringify({
        ok: true,
        value: toJson(q.name, v),
        canonical: encodeCanonicalData(q.name, v, q.limits).toString('hex'),
        protobuf: encodeProtobufData(q.name, v, q.limits).toString('hex'),
        digest: digest(q.name, v, q.limits),
      }),
    );
  } catch (error) {
    console.log(JSON.stringify({ ok: false, error: String(error) }));
  }
}
