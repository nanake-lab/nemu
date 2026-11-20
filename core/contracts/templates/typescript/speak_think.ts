// ADR 0046. Structural decoding precedes these checks. Trusted H supplies authority.
import { isDeepStrictEqual } from 'node:util';
import { checkCognitionMessage } from './cognition.js';
import { digest, encodeCanonicalData, normalize, type Limits } from './runtime.js';
import { schema, type MessageMap, type RemoteCognitiveReceipt, type Utf8Range } from './schema.js';

function require(ok: boolean, message: string): void {
  if (!ok) throw new Error(message);
}

function interval(span: Utf8Range, length: bigint): void {
  require(span.start < span.end && span.end <= length, 'invalid UTF-8 range');
}

function remote(v: RemoteCognitiveReceipt): void {
  for (const hash of [v.model_sha256, v.tokenizer_sha256, v.request_sha256])
    require(hash === undefined || hash.some((b) => b !== 0), 'unavailable digest encoded as zero');
  if (v.status !== 6) require(v.output_reserve! <= v.capacity!, 'output reserve exceeds capacity');
  require(v.raw_response !== undefined ||
    (v.returned_text === undefined && v.returned_model === undefined), 'missing response');
  const accepted = v.adoption === 1 || v.adoption === 2;
  require((v.product !== undefined) === accepted &&
    (v.accepted_range !== undefined) === accepted, 'adoption payload');
  require(v.adoption === 5 || v.review !== undefined, 'missing adoption decision');
  const length = v.returned_text?.byte_length ?? 0n;
  if (accepted) {
    const span = v.accepted_range!;
    interval(span, length);
    const full = span.start === 0n && span.end === length;
    require(v.adoption !== 1 || full, 'incomplete full adoption');
    require(v.product!.content.byte_length === span.end - span.start, 'product length');
    require(!full || isDeepStrictEqual(v.product!.content, v.returned_text), 'full text identity');
  }
  require(v.adoption !== 5 || length === 0n, 'unaccounted returned text');
}

type Obj = Record<string, unknown>;

export function checkSpeakThinkMessage(name: string, value: Obj, limits: Limits): void {
  if (schema.messages[name as keyof MessageMap]?.file !== 'speak_think.proto') return;
  const v = normalize(name, value, limits);
  // The shape, enums, required oneofs, bytes and uint64 values were just validated.
  const typed = <K extends keyof MessageMap>(): MessageMap[K] => v as unknown as MessageMap[K];
  const child = (kind: string, item: unknown): void =>
    checkSpeakThinkMessage(kind, item as Obj, limits);
  switch (name) {
    case 'Utf8Range':
      interval(typed<'Utf8Range'>(), 2n ** 64n - 1n);
      break;
    case 'QuoteLayer': {
      const layer = typed<'QuoteLayer'>();
      interval(layer.visible, layer.source.content.byte_length);
      break;
    }
    case 'SpeakerAttribution': {
      const { layers } = typed<'SpeakerAttribution'>();
      require(layers.length > 0, 'missing speaker layers');
      for (const layer of layers) {
        require(layer.source.branch_id === layers[0]!.source.branch_id, 'quote branch');
        child('QuoteLayer', layer);
      }
      break;
    }
    case 'RecordRelation': {
      const edge = typed<'RecordRelation'>();
      require(edge.origin.branch_id === edge.target.branch_id, 'relation branch');
      break;
    }
    case 'RelationSet': {
      const set = typed<'RelationSet'>();
      const seen = new Set<string>();
      for (const edge of set.relations) {
        child('RecordRelation', edge);
        require(edge.purpose === set.purpose &&
          (isDeepStrictEqual(edge.origin, set.endpoint) ||
            isDeepStrictEqual(edge.target, set.endpoint)), 'relation query scope');
        const bytes = encodeCanonicalData('RecordRelation', edge, limits).toString('hex');
        require(!seen.has(bytes), 'duplicate relation');
        seen.add(bytes);
      }
      break;
    }
    case 'RelationQueryReceipt': {
      const { result, manifest, expected_count } = typed<'RelationQueryReceipt'>();
      child('RelationSet', result);
      require(expected_count === BigInt(result.relations.length), 'incomplete relation count');
      require(manifest.object_type === 'echo.relation-set' &&
        Buffer.from(manifest.digest).toString('hex') === digest('RelationSet', result, limits) &&
        manifest.byte_length ===
          BigInt(encodeCanonicalData('RelationSet', result, limits).length), 'relation manifest');
      break;
    }
    case 'RemoteCognitiveReceipt':
      remote(typed<'RemoteCognitiveReceipt'>());
      break;
    case 'InternalActivityRecordV2': {
      const record = typed<'InternalActivityRecordV2'>();
      require(record.finished_at_ns >= record.started_at_ns, 'execution interval');
      require(!record.previous ||
        ['echo.internal-activity-record', 'echo.internal-activity-record-v2'].includes(
          record.previous.object_type,
        ), 'previous record type');
      if (record.local)
        checkCognitionMessage('CognitiveResultAndReceipt', record.local as unknown as Obj, limits);
      else remote(record.remote!);
      break;
    }
    case 'InternalActivityCommitV2': {
      const commit = typed<'InternalActivityCommitV2'>();
      child('InternalActivityRecordV2', commit.activity);
      const result = commit.activity.local ?? commit.activity.remote!;
      require(isDeepStrictEqual(commit.binding, result.binding) &&
        commit.interpretations.length > 0, 'commit binding or interpretations');
      break;
    }
    case 'CoreTransactionV4': {
      const { activity_commit } = typed<'CoreTransactionV4'>();
      if (activity_commit) child('InternalActivityCommitV2', activity_commit);
      break;
    }
    case 'SpeechDeliveryV2': {
      const { presentation, delivered, status } = typed<'SpeechDeliveryV2'>();
      const length = presentation.content.byte_length;
      let previous = 0n;
      for (const [i, span] of delivered.entries()) {
        interval(span, length);
        require(i === 0 || previous < span.start, 'delivery ranges overlap or touch');
        previous = span.end;
      }
      const full = delivered.length === 1 && delivered[0]!.start === 0n && previous === length;
      if (status === 1)
        require(full || (length === 0n && delivered.length === 0), 'full delivery range');
      if (status === 2) require(delivered.length > 0 && !full, 'partial delivery range');
      if (status === 4) require(delivered.length === 0, 'unknown delivery claims text');
    }
  }
}
