import { EchoCore } from '@nemu/contracts';
import { digest, encodeCanonical, inputSchema } from '../../contracts/src/generated/echo/index.js';
import { frameLimits } from './framing.js';
import type { ChatEvent } from './outbox.js';

const enums = EchoCore.schema.enums;
function uploadObject<K extends 'UserText' | 'SourceMetadata'>(
  name: K,
  value: EchoCore.MessageMap[K],
) {
  const bytes = encodeCanonical(name, value, frameLimits);
  const object: EchoCore.ObjectRef = {
    schema_version: 1,
    object_type: EchoCore.schema.messages[name]!.tag,
    resource_version: '1',
    byte_length: BigInt(bytes.length),
    digest: Buffer.from(digest(name, value, frameLimits), 'hex'),
  };
  return { object, bytes };
}

export function textObservation(input: ChatEvent) {
  const payload = uploadObject('UserText', { schema_version: 1, text: input.payload.text });
  const source = uploadObject('SourceMetadata', {
    schema_version: 1,
    source_refs: [],
    source_span: payload.object,
    derivation: enums.Derivation!.VERBATIM as EchoCore.Derivation,
    // Raw user statement as admitted by gateway::validate_event. IRL supplies
    // the later distinction between a claim and a self-report.
    epistemic_type: enums.EpistemicType!.USER_ASSERTION as EchoCore.EpistemicType,
    resolution_state: enums.ResolutionState!.UNRESOLVED as EchoCore.ResolutionState,
    representation_version: '1',
  });
  const event: EchoCore.TerminalEvent = {
    schema_version: 1,
    id: input.id,
    stable_source_key: input.id,
    kind: enums.EventKind!.OBSERVATION as EchoCore.EventKind,
    event_time_ns: BigInt(Math.round(input.ts * 1000)) * 1000n,
    parents: [],
    correction_of: [],
    payload_ref: payload.object,
    payload_digest: payload.object.digest,
    provenance: source.object,
    derivation: enums.Derivation!.VERBATIM as EchoCore.Derivation,
  };
  inputSchema('TerminalEvent', frameLimits).parse(event);
  return { payload, source, event };
}
