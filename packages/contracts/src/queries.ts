import { appBusPayloads } from './appbus.js';
import { wireInputSchema } from './generated/echo/browser.js';
import { schema, type QueryV2, type QueryResultV2 } from './generated/echo/schema.js';

/** Main process maps a correlated user_view result, preserving complete L entries for the panel. */
export function queryResultToPanel(query: QueryV2, result: QueryResultV2) {
  const q = wireInputSchema('QueryV2').parse(query);
  const r = wireInputSchema('QueryResultV2').parse(result);
  if (q.query_id !== r.query_id) throw new Error('Query correlation mismatch');
  const kinds = schema.enums.QueryKindV2!;
  const statuses = schema.enums.QueryStatus!;
  const kind =
    q.kind === kinds.QUERY_GROWTH
      ? 'growth'
      : q.kind === kinds.QUERY_MEMORY
        ? 'memory'
        : q.kind === kinds.QUERY_ACTIONS
          ? 'actions'
          : 'run_history';
  const names: Record<string, string> = {
    OK: 'ok',
    NO_MATCH: 'no_match',
    INDEX_LAGGING: 'index_lagging',
    DENIED: 'denied',
    OBJECT_MISSING: 'object_missing',
    BUDGET_EXHAUSTED: 'budget_exhausted',
  };
  const status = Object.entries(statuses).find(([, value]) => value === r.status)?.[0];
  return appBusPayloads['core.queryResult.v2'].parse({
    queryId: r.query_id,
    kind,
    status: names[status ?? ''],
    items: r.items.map((item) => ({
      ref: item.ref,
      at: item.at,
      kind: item.kind,
      summary: item.summary,
      source: item.source,
      retracted: item.retracted,
    })),
    growthEntries: r.growth_entries,
  });
}
