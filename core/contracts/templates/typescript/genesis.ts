// Pure V2/V3 rules from core-types semantic.rs (v2.0 §§3,8,13), no authority.
import { normalize, type Limits } from './runtime.js';

type Obj = Record<string, unknown>;
function require(ok: boolean, message: string): void {
  if (!ok) throw new Error(message);
}
function checkLawParameters(p: Obj): void {
  const n = (k: string): number => p[k] as number;
  require(n('n0') > 0, 'n0 must be positive');
  require(n('w_teach') <= n('w_max'), 'w_teach must not exceed w_max');
  require(n('v_u') < 1, 'v_u must be below 1');
  require(n('r_min') <= n('r_max'), 'r_min must not exceed r_max');
  require(n('g_min') <= n('g_max'), 'g_min must not exceed g_max');
  require(n('recovery_min_per_s') > 0, 'lambda_min must be positive');
  require(n('recovery_min_per_s') <= n('recovery_max_per_s'), 'lambda_min must not exceed lambda_max');
}
export function checkGenesis(name: string, value: Obj, limits: Limits): void {
  require(name === 'GenesisManifestV2' || name === 'GenesisManifestV3', 'genesis type');
  const g = normalize(name, value, limits);
  const p = (g['law_version'] as Obj)['parameters'] as Obj;
  checkLawParameters(p);
  const initial = g['initial_gain'] as Obj;
  const gain = ['connection', 'empathy', 'curiosity', 'achievement'].map((k) => initial[k] as number);
  require(gain.every((x) => x >= (p['g_min'] as number) && x <= (p['g_max'] as number)), 'initial gain outside [g_min, g_max]');
  require(gain.slice(0, 3).every((x) => gain[3]! < x), 'achievement must have the weakest initial gain');
  require((g['sigma0'] as number) > 0, 'sigma0 must be positive');
  require((g['tau0_s'] as number) > 0, 'tau0 must be positive');
}
