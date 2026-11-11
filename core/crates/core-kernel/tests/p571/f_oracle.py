"""Independent §8 F probe only; ADR0035 §4 A1 and v2 §6 old κ.

No core F helpers and no previously generated expected vectors. libm expm1 is
only an elementary numeric input; Φ's operation graph is evaluated here.
"""
from arithmetic import Q, rounded as r


class F:
    def __init__(self, resource, version, primitive):
        self.p, self.version, self.fn = resource, version, primitive
        self.x, self.h, self.targets, self.anchor = [Q(0)]*4, {}, {}, 0

    def step(self, event, index):
        dt = r(Q(event['time_ns']-self.anchor)/10**9)
        pull = -self.fn('expm1', -r(Q(self.p['lambda'])*dt))
        self.x = [max(Q(-1),min(Q(1),r(x+r(pull*r(Q(self.p['b'])-x))))) for x in self.x]
        self.h = {k:[r(h+r(pull*(-h))) for h in v] for k,v in self.h.items()}
        self.anchor = event['time_ns']
        target = [Q(event['target']),Q(0),Q(0),Q(0)]
        if self.version == 'old':
            exponent = 0 if index < 3 else index-2
            # Independent repeated product; κ=1/2 is exact on this frozen range.
            weight = Q(1)
            for _ in range(exponent): weight = r(weight*Q(self.p['kappa']))
            self.x = [max(Q(-1),min(Q(1),r(x+r(weight*v)))) for x,v in zip(self.x,target)]
            return
        root = event['root']
        current = self.h.get(root,[Q(0)]*4)
        updated=[]
        for j,(x,h,v) in enumerate(zip(self.x,current,target)):
            tentative=r(x+r(v-h))
            if -1 < tentative < 1:
                self.x[j]=tentative; updated.append(v)
            else:
                wall=Q(1 if tentative>=1 else -1)
                updated.append(max(min(h,v),min(max(h,v),r(h+r(wall-x)))))
                self.x[j]=wall
        self.h[root],self.targets[root]=updated,target

    def state(self):
        return dict(x=self.x,h=self.h,targets=self.targets,anchor=self.anchor)
