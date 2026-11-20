import { checkGenesis } from '../../packages/contracts/src/generated/echo/genesis.js';
// Test-only JSON-lines adapter, run with Node's TypeScript stripping.
import { ingressProbe } from './ingress_probe.js';
import { checkThetaMessage } from '../../packages/contracts/src/generated/echo/theta.js';
import { checkLawStateMessage } from '../../packages/contracts/src/generated/echo/law_state.js';
import { checkCognitionMessage } from '../../packages/contracts/src/generated/echo/cognition.js';
import {
  checkJMessage,
  checkJResources,
  checkJEnvelope,
} from '../../packages/contracts/src/generated/echo/j_resources.js';
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
      formal: boolean;
    };
    if (q.op === 'ingress') {
      ingressProbe(q, q.limits);
      console.log(JSON.stringify({ ok: true }));
      continue;
    }
    if (q.op === 'j_genesis') {
      checkGenesis(q.name, fromJson(q.name, q.value), q.limits);
      console.log(JSON.stringify({ ok: true }));
      continue;
    }
    if (q.op === 'j_envelope') {
      checkJEnvelope(
        fromJson('ModulationEnvelopeV2', q.value['envelope'] as Record<string, unknown>),
        fromJson('JEffectReceipt', q.value['receipt'] as Record<string, unknown>),
        q.limits,
      );
      console.log(JSON.stringify({ ok: true }));
      continue;
    }
    if (q.op === 'j_resources') {
      const b = q.value;
      const get = (name: string, key: string): Record<string, unknown> =>
        fromJson(name, b[key] as Record<string, unknown>);
      checkJResources(
        get('JDirectionSet', 'direction_set'),
        get('JDirectionValidation', 'validation'),
        get('BodyReadoutParameters', 'body'),
        get('JEffectReceipt', 'receipt'),
        get('GenesisManifestV3', 'genesis'),
        (b['vectors'] as Record<string, unknown>[]).map((x) => fromJson('JDirectionVector', x)),
        q.formal,
        q.limits,
      );
      console.log(JSON.stringify({ ok: true }));
      continue;
    }
    const v =
      q.op === 'encode'
        ? normalize(q.name, fromJson(q.name, q.value), q.limits)
        : q.op === 'canonical'
          ? decodeCanonicalData(q.name, Buffer.from(q.hex, 'hex'), q.limits)
          : decodeProtobufData(q.name, Buffer.from(q.hex, 'hex'), q.limits);
    checkThetaMessage(q.name, v, q.limits);
    checkLawStateMessage(q.name, v, q.limits);
    checkCognitionMessage(q.name, v, q.limits);
    checkJMessage(q.name, v, q.limits);
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
