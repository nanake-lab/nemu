"""Read-only step trace of the frozen Python oracle and explicit binary64 recurrence.
No fixture, assertion, acceptance threshold or production source is modified.
"""
from fractions import Fraction
import json
import math
from pathlib import Path
import platform
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
DATA = b''.join((ROOT / 'evidence/tests' / f'theta1.bin.part-{i:03}').read_bytes() for i in range(19))

def bits(x):
    return struct.unpack('<Q', struct.pack('<d', x))[0]

def record(x):
    return {'value': repr(x), 'hex': x.hex(), 'bits': f'{bits(x):016x}'}

def nearest_square(x):
    # Integer quotient/remainder rounding of the exact rational square. This is
    # independent of float multiplication, pow, sqrt and any tolerance.
    n, d = x.as_integer_ratio()
    n, d = n * n, d * d
    e = n.bit_length() - d.bit_length()
    if (Fraction(n, d) < (Fraction(2) ** e)):
        e -= 1
    shift = 52 - e
    numerator, denominator = (n << shift, d) if shift >= 0 else (n, d << -shift)
    q, rem = divmod(numerator, denominator)
    round_up = 2 * rem > denominator or (2 * rem == denominator and q % 2 == 1)
    m = q + round_up
    if m == 1 << 53:
        m >>= 1
        e += 1
    assert -1022 <= e <= 1023
    correct_bits = ((e + 1023) << 52) | (m - (1 << 52))
    mul = x * x
    power = x ** 2
    return {'exact_numerator': str(n), 'exact_denominator': str(d),
            'quotient_before_round': str(q), 'remainder': str(rem), 'divisor': str(denominator),
            'tie': 2 * rem == denominator, 'round_up': round_up,
            'nearest_even_bits': f'{correct_bits:016x}',
            'multiply_bits': f'{bits(mul):016x}', 'pow_bits': f'{bits(power):016x}',
            'multiply_is_nearest_even': bits(mul) == correct_bits,
            'pow_is_nearest_even': bits(power) == correct_bits,
            'multiply_error_in_ulps': str(abs(Fraction.from_float(mul) - Fraction(n,d)) / Fraction.from_float(math.ulp(mul))),
            'pow_error_in_ulps': str(abs(Fraction.from_float(power) - Fraction(n,d)) / Fraction.from_float(math.ulp(mul)))}

def step(old, a, nb, ns, rho, b0, power):
    sw, sa, sq, b = old
    v = {}
    v['residual'] = a - b
    v['residual_square'] = v['residual'] ** 2 if power else v['residual'] * v['residual']
    v['decayed_square_sum'] = rho * sq
    v['square_sum'] = v['decayed_square_sum'] + v['residual_square']
    v['decayed_weight'] = rho * sw
    v['weight'] = v['decayed_weight'] + 1.0
    v['decayed_reaction_sum'] = rho * sa
    v['reaction_sum'] = v['decayed_reaction_sum'] + a
    v['prior_baseline_numerator'] = nb * b0
    v['baseline_numerator'] = v['prior_baseline_numerator'] + v['reaction_sum']
    v['baseline_denominator'] = nb + v['weight']
    v['baseline'] = v['baseline_numerator'] / v['baseline_denominator']
    v['prior_sigma_square'] = 0.3 ** 2 if power else 0.3 * 0.3
    v['weighted_prior_variance'] = ns * v['prior_sigma_square']
    v['variance_numerator'] = v['weighted_prior_variance'] + v['square_sum']
    v['variance_denominator'] = ns + v['weight']
    v['variance'] = v['variance_numerator'] / v['variance_denominator']
    v['sigma'] = math.sqrt(v['variance'])
    v['gain_numerator'] = 1.0 * 0.3
    v['raw_gain'] = v['gain_numerator'] / v['sigma']
    v['gain'] = min(3.0, max(0.2, v['raw_gain']))
    return (v['weight'], v['reaction_sum'], v['square_sum'], v['baseline']), v

offset = 0
first_power = first_sigma = None
power_differences = sigma_differences = 0
nearest_even_violations_mul = nearest_even_violations_pow = 0
for stream in range(500):
    nb, ns, rho, b0, count = struct.unpack_from('<ddddI', DATA, offset)
    offset += 36
    original = ordered = (0.0, 0.0, 0.0, b0)
    original_v = ordered_v = {'baseline': b0, 'variance': 0.3 * 0.3, 'sigma': 0.3, 'gain': 1.0}
    for event in range(count):
        a, exo, expected_b, expected_var, expected_g = struct.unpack_from('<dBddd', DATA, offset)
        offset += 33
        before_original, before_ordered = original, ordered
        if exo:
            original, original_v = step(original, a, nb, ns, rho, b0, True)
            ordered, ordered_v = step(ordered, a, nb, ns, rho, b0, False)
            x = original_v['residual']
            if bits(x ** 2) != bits(x * x):
                power_differences += 1
                exact = nearest_square(x)
                nearest_even_violations_mul += not exact['multiply_is_nearest_even']
                nearest_even_violations_pow += not exact['pow_is_nearest_even']
                if first_power is None:
                    first_power = {'stream_zero_based':stream, 'event_zero_based':event,
                                   'residual':record(x), 'integer_rounding_proof':exact}
        assert bits(original_v['baseline']) == bits(expected_b)
        assert bits(original_v['variance']) == bits(expected_var)
        assert bits(original_v['gain']) == bits(expected_g)
        if bits(original_v['sigma']) != bits(ordered_v['sigma']):
            sigma_differences += 1
            if first_sigma is None:
                first_sigma = {'stream_zero_based':stream, 'event_zero_based':event,
                    'input':{k:record(v) for k,v in dict(nb=nb,ns=ns,rho=rho,b0=b0,a=a).items()},
                    'before':{k:{'python':record(p),'rust_order':record(r),'same':bits(p)==bits(r)} for k,p,r in zip(['weight','reaction_sum','square_sum','baseline'],before_original,before_ordered)},
                    'steps':{k:{'python':record(v),'rust_order':record(ordered_v[k]),'same':bits(v)==bits(ordered_v[k])} for k,v in original_v.items()},
                    'integer_rounding_proof':nearest_square(original_v['residual'])}
assert offset == len(DATA)
result = {'status':'diagnosis_only_not_fixed','python':sys.version,'libc':platform.libc_ver(),
          'original_oracle_values_verified':72822,'power_differences':power_differences,
          'sigma_differences':sigma_differences,'nearest_even_violations_multiply_at_differing_powers':nearest_even_violations_mul,
          'nearest_even_violations_pow_at_differing_powers':nearest_even_violations_pow,
          'first_power_difference':first_power,'first_sigma_difference':first_sigma}
print(json.dumps(result,ensure_ascii=False,indent=2))
