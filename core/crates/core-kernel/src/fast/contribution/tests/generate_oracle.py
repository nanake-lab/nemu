"""ADR0035 §9(2,5,10): exact rational oracle; standard library only.

Synthetic vectors are code fixtures. No production routines are imported.
Every binary64 operation rounds an exact Fraction, independently of Rust.
The exact A1 oracle uses clip(x+v-h), h+(next-x), not the branch implementation.
"""
from fractions import Fraction as Q
from itertools import product
from pathlib import Path
import json
import random
import struct


def bits(x):
    return struct.pack('>d', x).hex()


def rounded(x):
    return Q(float(x))


def exact(x, h, v):
    y = max(Q(-1), min(Q(1), x + v - h))
    return y, h + y - x


def binary(x, h, v):
    d = rounded(v-h)
    s = rounded(x+d)
    if -1 < s < 1:
        return s, v
    y = Q(1) if s >= 1 else Q(-1)
    t = rounded(h+rounded(y-x))
    return y, max(min(h,v), min(max(h,v), t))


def generate():
    rng = random.Random(5070)
    cases = list(product([-1., -.75, 0., .5, 1.], [-2., -.5, 0., .75, 2.], [-1., 0., .25, 1.]))
    cases += [(rng.uniform(-1,1), rng.uniform(-2,2), rng.uniform(-1,1)) for _ in range(128)]
    cases += [(0.44103831552214, -0.07324038289205337, 0.0002230830984472614)]
    vectors = []
    max_residual = Q(0)
    for raw in cases:
        x,h,v = map(Q,raw)
        y,j = binary(x,h,v)
        ey,ej = exact(x,h,v)
        # A rational, outward error enclosure from actual rounding errors:
        # clip is 1-Lipschitz; the slot's branch/interpolation rounding is
        # recorded separately, never asserted to preserve exact decomposition.
        err_d = abs(rounded(v-h)-(v-h))
        err_s = abs(rounded(x+rounded(v-h))-(x+rounded(v-h)))
        x_bound = err_d+err_s
        s = rounded(x+rounded(v-h))
        h_bound = x_bound
        if s >= 1 or s <= -1:
            bound = Q(1) if s >= 1 else Q(-1)
            gap = rounded(bound-x)
            h_bound += abs(gap-(bound-x)) + abs(rounded(h+gap)-(h+gap))
        assert abs(y-ey) <= x_bound
        assert abs(j-ej) <= h_bound
        residual = (y-x)-(j-h)
        max_residual=max(max_residual,abs(residual))
        vectors.append(dict(inputs=[bits(z) for z in raw], expected=[bits(float(y)),bits(float(j))],
            exact=[str(ey),str(ej)], error_bounds=[str(x_bound),str(h_bound)], residual=str(residual)))
    # Exact d_diamond non-expansion, including different decompositions.
    grid=list(map(Q,[-1.,-.5,0.,.5,1.]))
    comparisons=0
    for x,h,xx,hh,v in product(grid,repeat=5):
        y,j=exact(x,h,v); yy,jj=exact(xx,hh,v)
        assert abs(y-yy)+abs(j-jj) <= abs(x-xx)+abs(h-hh)
        comparisons+=1
    # B6' merge envelopes: interval endpoints are rounded at each addition.
    members=[Q(.1),Q(.2),Q(.3)]
    upper=Q(0); current=Q(0); merge=[]
    for v in members:
        current=rounded(current+v); upper=rounded(upper+v)
        assert 0 <= current <= upper
        merge.append(bits(float(current)))
    return dict(seed=5070, vectors=vectors, exact_distance_comparisons=comparisons,
        max_decomposition_residual=str(max_residual), merge_expected=merge)


if __name__ == '__main__':
    Path(__file__).with_name('oracle.json').write_text(
        json.dumps(generate(),indent=2)+'\n',encoding='utf-8')
