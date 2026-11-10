# P5-67 最小复算

以下仅为合成代数见证，数值来自复审反例或为检验恒等式选取；不是参数建议、实测数据或 E05 运行。使用 Python 标准库，浮点近似只用于展示，不能代替后继 ADR 的 binary64 规范与误差证明。

在仓库根目录执行（不读取任何生活存储）：

```powershell
python -X utf8 -c "from pathlib import Path; s=Path('reports/P5-67/CHECKS.md').read_text(encoding='utf-8'); exec(s.split(chr(10)+chr(96)*3+'python'+chr(10),1)[1].split(chr(10)+chr(96)*3,1)[0])"
```

```python
from math import isclose, sqrt

def update(state, q, s2, rho):
    p, n, s = state
    phi = (max(1-2*q, 0), 1-abs(2*q-1), max(2*q-1, 0))
    for k, share in enumerate(phi):
        f = 1-share*(1-rho)
        p[k] *= f
        n[k] = f*n[k]+share
        s[k] = f*s[k]+share*s2

def initial(p):
    return ([p]*3, [0.0]*3, [0.0]*3)

def ratios(state):
    p, n, s = state
    v = sum(s)/sum(n) if sum(n) else 0
    return v, [(p[k]+s[k]/v)/(p[k]+n[k])
               if n[k] and v else 1.0 for k in range(3)]

# 局部时钟下 Vbar 的定义与恒等式；零域先走约定，不做除零。
state = initial(1)
assert ratios(state) == (0, [1.0]*3)
update(state, .25, 1, .5)
update(state, 0, 4, .5)
v, r = ratios(state)
assert isclose(v, 19/7)
assert isclose(sum(x/v for x in state[2]), sum(state[1]))
assert all(0 <= n <= 2 for n in state[1])  # 固定 rho=.5 的质量界
mid = initial(1)
for s2 in (0, 1, .25, 4):
    update(mid, .25, s2, .5)
assert all(isclose(x, 1) for x in ratios(mid)[1])

# 先验 99.9% 仍可产生明显修正；精确恒等式的浮点复算。
state = initial(1000)
for q in (0, 1):
    for _ in range(10000):
        update(state, q, 0, .999)
update(state, .5, 1, .999)
v, r = ratios(state)
p, n, s = (values[1] for values in state)
assert isclose(p/(p+n), .999)
assert isclose(r[1]-1, n/(p+n)*(s/(n*v)-1))
assert r[1] > 2.9
print("prior witness:", round(v, 8), round(r[1], 6), round(.5/sqrt(r[1]), 6))

# 两处原始二阶矩相同，也可在不同先验份额下反向。
state = initial(.001)
for _ in range(100):
    update(state, 0, 0, .9)
update(state, .000005, 1, .9)
update(state, 1, 1, .9)
v, r = ratios(state)
assert isclose(state[2][1]/state[1][1], state[2][2]/state[1][2])
assert r[2]/r[1] > 4
print("reversal witness:", *(round(x, 6) for x in r[1:]))

def zeta_step(state, weights, r, z2, rho):
    m, b = max(weights), sum(weights)/len(weights)
    rm = sum(x for w, x in zip(weights, r) if w == m)/weights.count(m)
    rb = sum(r)/len(r)
    d = rm-rb
    if m == b or d == 0:
        return state
    p, a, total = state
    return (rho*p, rho*a+d*(z2-rb), rho*total+d*d)

def zeta(state):
    p, a, total = state
    return min(1, max(0, (p+a)/(p+total)))

birth = (1.0, 0.0, 0.0)
# 饱和同权、r 不同：旧式为 1/(1+.3**2)，修订后不更新。
assert zeta_step(birth, [1, 1], [.2, .8], .5, 1) == birth
average = zeta_step(birth, [.5, 1/sqrt(2)], [1, 2], 1.5, 1)
maximum = zeta_step(birth, [.5, 1/sqrt(2)], [1, 2], 2, 1)
assert isclose(zeta(average), .8) and zeta(maximum) == 1
assert zeta_step(average, [1, 1], [.2, .8], .5, .5) == average
assert zeta_step(average, [.5, 1], [1, 1], 1, .5) == average
# M>B 的并列最大者：取 r 均值不依赖名单顺序。
left = zeta_step(birth, [.5, 1, 1], [1, .2, .8], 2/3, 1)
right = zeta_step(birth, [1, .5, 1], [.8, 1, .2], 2/3, 1)
assert left == right
print("zeta witnesses:", zeta(birth), zeta(average), zeta(maximum))
print("synthetic algebra checks: passed")
```

结果：上述断言通过。它们只核对正参照恒等式、等份插值、两个反例，以及 ζ 的同权资格、正例、无信息保留与并列对称；不签发 E05 或机制验收。
