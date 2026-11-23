//! CPython's `random.Random` (MT19937) for the few calls `check_law.py` uses,
//! so the ported property tests draw exactly the same trials with the same
//! seeds as the reference script (`reports/echo-core-v2-draft/check_law.py`).
//! Test-only; seeded; no global state.

pub struct PyRandom {
    mt: [u32; 624],
    index: usize,
}

impl PyRandom {
    /// `random.Random(seed)` for 0 ≤ seed < 2^32 (key = [seed]).
    pub fn new(seed: u32) -> Self {
        let mut r = Self {
            mt: [0; 624],
            index: 624,
        };
        r.init_genrand(19_650_218);
        let key = [seed];
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..624usize.max(key.len()) {
            let prev = r.mt[i - 1] ^ (r.mt[i - 1] >> 30);
            r.mt[i] = (r.mt[i] ^ prev.wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                r.mt[0] = r.mt[623];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..623 {
            let prev = r.mt[i - 1] ^ (r.mt[i - 1] >> 30);
            r.mt[i] = (r.mt[i] ^ prev.wrapping_mul(1_566_083_941)).wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                r.mt[0] = r.mt[623];
                i = 1;
            }
        }
        r.mt[0] = 0x8000_0000;
        r
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..624 {
            let prev = self.mt[i - 1] ^ (self.mt[i - 1] >> 30);
            self.mt[i] = 1_812_433_253u32.wrapping_mul(prev).wrapping_add(i as u32);
        }
        self.index = 624;
    }

    fn next_u32(&mut self) -> u32 {
        if self.index >= 624 {
            for k in 0..624 {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % 624] & 0x7fff_ffff);
                let mut v = self.mt[(k + 397) % 624] ^ (y >> 1);
                if y & 1 == 1 {
                    v ^= 0x9908_b0df;
                }
                self.mt[k] = v;
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `random()`: 53-bit float in [0, 1).
    pub fn random(&mut self) -> f64 {
        let a = f64::from(self.next_u32() >> 5);
        let b = f64::from(self.next_u32() >> 6);
        (a * 67_108_864.0 + b) * (1.0 / 9_007_199_254_740_992.0)
    }

    /// `uniform(a, b)`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    fn getrandbits(&mut self, k: u32) -> u32 {
        debug_assert!((1..=32).contains(&k));
        self.next_u32() >> (32 - k)
    }

    fn randbelow(&mut self, n: u32) -> u32 {
        let k = 32 - n.leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    /// `randint(a, b)` with b − a + 1 < 2^32.
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        let width = u32::try_from(b - a + 1).expect("small range");
        a + i64::from(self.randbelow(width))
    }

    /// `choice(seq)`: the caller has already evaluated every element.
    pub fn choice<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.randbelow(u32::try_from(items.len()).expect("small")) as usize]
    }
}

#[test]
fn matches_cpython_reference_draws() {
    // CPython 3.13: r = random.Random(20261019); r.random(), r.uniform(0.5, 6),
    // r.randint(1, 40), r.choice((1.0, -1.0, 0.5))
    let mut r = PyRandom::new(20_261_019);
    let got = (
        r.random(),
        r.uniform(0.5, 6.0),
        r.randint(1, 40),
        r.choice(&[1.0, -1.0, 0.5]),
    );
    assert_eq!(got, (0.5888913074339623, 5.802012126733248, 8, 1.0));
    // r = random.Random(20261019 + 7); [r.randint(1, 8) for _ in range(5)], r.uniform(-0.999, 0.999)
    let mut r = PyRandom::new(20_261_026);
    let ints: Vec<i64> = (0..5).map(|_| r.randint(1, 8)).collect();
    assert_eq!(ints, [2, 4, 3, 8, 7]);
    assert_eq!(r.uniform(-0.999, 0.999), 0.0838229536340801);
}
