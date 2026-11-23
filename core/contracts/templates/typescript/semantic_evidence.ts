// ADR 0048. Wire-local checks only; the core binds real objects and dispatch.
import { isDeepStrictEqual as equal } from 'node:util';
import { normalize, type Limits, requireValue as check } from './runtime.js';
import {
  schema,
  type MessageMap,
  type StageAdoption,
  type EventAttributesRecordV2,
  type AttributeRule,
  type ConceptRegistry,
  type ConceptRef,
} from './schema.js';
import { checkSpeakThinkMessage } from './speak_think.js';

function registry(r: ConceptRegistry): void {
  const ids = new Set(r.entries.map((e) => e.concept_id));
  const member = (c: ConceptRef) => ids.has(c.concept_id) && c.registry_version === r.version;
  for (const e of r.entries)
    check(
      e.roles.every((role) => !role.domain || member(role.domain)),
      'role domain',
    );
  const aliases = new Map<string, string>();
  for (const link of [...r.links, ...r.aliases])
    check(member(link.origin) && member(link.target), 'unknown concept');
  for (const link of r.links)
    check(r.relation_kinds.includes(link.kind), 'unknown concept relation');
  for (const link of r.aliases) {
    check(link.kind === 'canonical' && !aliases.has(link.origin.concept_id), 'ambiguous alias');
    aliases.set(link.origin.concept_id, link.target.concept_id);
  }
  const done = new Set<string>();
  for (let id of aliases.keys()) {
    const path = new Set<string>();
    while (aliases.has(id) && !done.has(id)) {
      check(!path.has(id), 'alias cycle');
      path.add(id);
      id = aliases.get(id)!;
    }
    for (const visited of path) done.add(visited);
  }
}

function stage(s: StageAdoption): void {
  const b = s.binding;
  check(
    s.event.branch_id === b.branch_id &&
      s.interpretation.branch_id === b.branch_id &&
      !equal(s.event, s.interpretation),
    'interpretation endpoints',
  );
  check(
    equal(s.registry_before, b.environment.registry) &&
      equal(s.context.digest, b.c_core_digest) &&
      [s.context, ...s.definitions].every((d) => b.read_set.some((r) => equal(r, d))),
    'stage read set',
  );
  const res = [b.environment.irl, b.environment.nature_appraiser, b.environment.situation_matcher][
    s.stage - 1
  ];
  check(equal(res, s.producer_resource) && (!!res || s.outcome === 3), 'stage producer');
  check(s.stage === 3 || !s.definitions.length, 'definitions outside M');
  check(
    !!s.registry_after === !!s.staged_revisions.length && (!s.registry_after || s.stage === 1),
    'registry change',
  );
  check(!!s.reason === (s.outcome !== 1), 'stage reason');
  if (s.outcome === 3)
    check(!s.appraisal && !s.coverage.length && !s.registry_after && !s.revision, 'failed stage');
  else {
    check(!!s.candidate && !!s.actual_output, 'missing stage inputs');
    if (s.stage === 2) check(s.outcome === 1 && !!s.appraisal && !s.coverage.length, 'invalid A');
    else {
      check(!s.appraisal && (s.outcome !== 2 || !!s.coverage.length), 'partial stage');
      check(
        s.coverage.every((c) => (s.stage === 1 ? c.mu === undefined : !c.accepted)),
        'wrong stage coverage',
      );
    }
  }
  const edge = s.revision;
  check(!!edge === !!s.replaced_targets.length, 'replacement targets');
  if (edge)
    check(
      edge.kind === 14 &&
        equal(edge.origin, s.interpretation) &&
        !equal(edge.origin, edge.target) &&
        edge.target.branch_id === b.branch_id,
      'revision endpoints',
    );
  const q = s.relations?.result;
  if (q)
    check(
      equal(q.endpoint, edge?.target ?? s.event) &&
        q.frontier === b.base_commit &&
        (!edge || edge.purpose === q.purpose),
      'relation scope',
    );
}

function rule(r: AttributeRule): void {
  const s = r.source;
  check(
    r.exogenous === s <= 3 &&
      r.self_narrative === (s === 6) &&
      r.growth_access === [2, 2, 2, 2, 3, 1, 1][s - 1],
    'source partition',
  );
  const types = new Map<string, number>([
    ['echo.user-text', 1],
    ['echo.user-utterance', 1],
    ['echo.touch-v2', 2],
    ['echo.desktop-observation', 4],
    ['echo.body-action-result', 5],
    ['echo.terminal-maintenance', 7],
    ['echo.system-maintenance-record', 7],
    ['echo.internal-activity-record', 6],
    ['echo.internal-activity-record-v2', 6],
  ]);
  check(
    (types.get(r.payload_type) ?? s) === s &&
      (r.event_kind !== 8 || s === 7) &&
      (r.event_kind !== 4 || s === 5),
    'event source',
  );
}

function attributes(v: EventAttributesRecordV2): void {
  const { event: e, applied: r } = v;
  const among = (x: unknown) => v.inputs.some((i) => equal(x, i));
  check(
    v.event_id === e.record_id &&
      r.payload_type === e.content.object_type &&
      v.rule_table.object_type === 'echo.attribute-rule-table' &&
      v.rule_table_version === v.rule_table.resource_version &&
      v.exogenous === r.exogenous &&
      v.self_narrative === r.self_narrative &&
      !!v.c_unit === r.exogenous,
    'attribute rule',
  );
  check(
    among(e) &&
      v.inputs.every((i) => i.branch_id === e.branch_id) &&
      v.source.locations.some((l) => equal(l.source, e)),
    'event inputs',
  );
  check(
    [v.c_unit, v.directed_to_n, v.cause].every((x) => !x || among(x)),
    'classification evidence',
  );
  if (r.source === 1 || r.source === 2)
    check(
      !!v.directed_to_n && !!v.person_mapping && (r.source !== 1 || !!v.source.speaker),
      'unverified actor',
    );
  check(
    (r.source !== 5 || !!v.cause) && (![2, 3].includes(r.source) || !!v.episode_phase_fact),
    'missing origin',
  );
  for (const [axis, requirements] of [
    ['occurred', 'occurrence_scopes'],
    ['truth', 'truth_scopes'],
    ['subjective', 'subjective_scopes'],
  ] as const) {
    check(!!v[axis].length, 'unassessed axis');
    for (const i of v[axis]) {
      check(among(i.scope.source) && i.evidence.every(among), 'verification evidence');
      check(
        axis === 'subjective' ? i.value !== 1 || !!i.experiencer : !i.experiencer,
        'verification experiencer',
      );
    }
    for (const t of v.targets)
      check(
        t[requirements].every((p) => v[axis].some((i) => equal(i.scope, p.scope))),
        'unobserved premise',
      );
  }
}

type Obj = Record<string, unknown>;
export function checkSemanticEvidence(name: string, value: Obj, limits: Limits): void {
  const m = schema.messages[name];
  if (m?.file !== 'semantic_evidence.proto') return;
  const v = normalize(name, value, limits);
  for (const f of m.fields) {
    if (!(f.name in v) || !(f.type in schema.messages)) continue;
    const items = f.repeated ? (v[f.name] as Obj[]) : [v[f.name] as Obj];
    for (const child of items) {
      checkSpeakThinkMessage(f.type, child, limits);
      checkSemanticEvidence(f.type, child, limits);
    }
  }
  const as = <K extends keyof MessageMap>(): MessageMap[K] => v as unknown as MessageMap[K];
  switch (name) {
    case 'SourceLocation': {
      const x = as<'SourceLocation'>();
      check(!x.text || x.text.start < x.text.end, 'empty range');
      break;
    }
    case 'SourceMetadataV2': {
      const x = as<'SourceMetadataV2'>(),
        seen = [...x.roots],
        branch = seen[0]?.branch_id;
      check(
        !!branch && !!x.locations.length && seen.every((r) => r.branch_id === branch),
        'missing or cross-branch roots',
      );
      for (const t of x.transformations) {
        check(
          !!t.inputs.length &&
            t.inputs.every((i) => seen.some((r) => equal(i, r))) &&
            !seen.some((r) => equal(t.output, r)) &&
            t.output.branch_id === branch,
          'transform chain',
        );
        seen.push(t.output);
      }
      check(
        x.locations.every((l) => seen.some((r) => equal(l.source, r))),
        'location binding',
      );
      if (x.speaker)
        check(
          x.locations.some((l) => equal(l.source, x.speaker?.layers[0]?.source)),
          'speaker source',
        );
      break;
    }
    case 'ConceptRevision': {
      const x = as<'ConceptRevision'>();
      check(
        x.lifecycle !== 2 || (!!x.label && !!x.definition && !!x.grounding.length),
        'stable grounding',
      );
      break;
    }
    case 'ConceptRegistry':
      registry(as<'ConceptRegistry'>());
      break;
    case 'ClaimV2': {
      const x = as<'ClaimV2'>();
      check(
        !!x.reported_text.text &&
          x.source.locations.some((l) => equal(l, x.reported_text)) &&
          !!x.time_fact === (x.time_kind === 3),
        'claim operand',
      );
      break;
    }
    case 'SemanticGraphV2': {
      const x = as<'SemanticGraphV2'>(),
        nodes = [...x.references, ...x.statements, ...x.actions].map((n) => n.id),
        ids = new Set(nodes);
      check(
        ids.size === nodes.length &&
          x.edges.every((e) => ids.has(e.from_node) && ids.has(e.to_node)) &&
          x.statements.every((c) => c.roles.every((r) => ids.has(r.node_ref))),
        'graph nodes',
      );
      break;
    }
    case 'TextAdapterEntry': {
      const x = as<'TextAdapterEntry'>();
      check(
        ['echo.user-text', 'echo.user-utterance'].includes(x.payload_tag) &&
          x.payload_schema === 1 &&
          equal(x.path, [{ schema_version: 1, field_number: 2 }]),
        'unsupported adapter',
      );
      break;
    }
    case 'TextAdapterTable': {
      const x = as<'TextAdapterTable'>();
      check(
        x.registry.object_type === 'echo.concept-registry' &&
          x.entries.length === 2 &&
          x.entries.every((e) => e.predicate.registry_version === x.registry.resource_version),
        'adapter table',
      );
      break;
    }
    case 'StageCoverage':
      check(
        ['accepted', 'mu', 'unknown_reason'].filter((k) => k in v).length === 1,
        'ambiguous coverage',
      );
      break;
    case 'StageAdoption':
      stage(as<'StageAdoption'>());
      break;
    case 'AttributeRule':
      rule(as<'AttributeRule'>());
      break;
    case 'AttributeRuleTable': {
      const x = as<'AttributeRuleTable'>(),
        keys = new Set(
          x.rules.map((r) => JSON.stringify([r.source, r.event_kind, r.payload_type])),
        );
      check(keys.size === x.rules.length && keys.size > 0, 'rule keys');
      break;
    }
    case 'ScopedVerification': {
      const x = as<'ScopedVerification'>();
      check(x.value === 3 ? !!x.reason : !!x.evidence.length, 'verification basis');
      break;
    }
    case 'VerificationRequirement':
      check(v.expected !== 3, 'unknown prerequisite');
      break;
    case 'TargetPremise': {
      const x = as<'TargetPremise'>();
      check(
        !!(x.truth_scopes.length || x.subjective_scopes.length || x.occurrence_scopes.length),
        'empty premise',
      );
      break;
    }
    case 'EventAttributesRecordV2':
      attributes(as<'EventAttributesRecordV2'>());
      break;
  }
}
