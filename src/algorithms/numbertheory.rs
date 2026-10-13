//! Number theory: Sieve of Eratosthenes, extended Euclidean algorithm,
//! binary exponentiation, Chinese remainder theorem, matrix exponentiation,
//! baby-step giant-step discrete log, Miller–Rabin primality.

// ─── Sieve of Eratosthenes ────────────────────────────────────────────────────

pub fn sieve() {
    const N: usize = 100;
    let primes = sieve_primes(N);
    let trial: Vec<usize> = (2..=N).filter(|&x| trial_prime(x)).collect();

    println!("primes ≤ {N}:");
    println!("  {primes:?}");
    println!("count = {} (π(100) = 25)", primes.len());
    println!("matches trial division: {}", primes == trial);
    println!(
        "start crossing out at p² (p·k, k < p already gone) → O(n log log n);",
    );
    println!("trial division checks every candidate up to √x → O(n√n)");
}

fn sieve_primes(n: usize) -> Vec<usize> {
    let mut is_prime = vec![true; n + 1];
    is_prime[0] = false;
    if n >= 1 {
        is_prime[1] = false;
    }
    let mut p = 2;
    while p * p <= n {
        if is_prime[p] {
            let mut m = p * p; // smaller multiples already crossed out
            while m <= n {
                is_prime[m] = false;
                m += p;
            }
        }
        p += 1;
    }
    (2..=n).filter(|&x| is_prime[x]).collect()
}

fn trial_prime(x: usize) -> bool {
    x >= 2 && (2..x).take_while(|&d| d * d <= x).all(|d| x % d != 0)
}

// ─── Extended GCD ─────────────────────────────────────────────────────────────

pub fn extended_gcd() {
    for (a, b) in [(240i128, 46i128), (1071, 462), (3120, 17)] {
        let (g, x, y) = egcd(a, b);
        let identity = a * x + b * y;
        println!(
            "egcd({a}, {b}) → g = {g}, x = {x}, y = {y}   ({a}·{x} + {b}·{y} = {identity}, verified: {})",
            identity == g && g == gcd(a, b)
        );
    }

    // Killer app: modular inverse, the `d` of textbook RSA.
    let (e, m) = (17u64, 3120u64); // φ(3120) with gcd(e, φ) = 1
    match modinv(e, m) {
        Some(d) => println!(
            "modinv({e}, {m}) = {d}   ({} mod {m} = {})",
            e * d,
            e * d % m
        ),
        None => println!("modinv({e}, {m}) does not exist (gcd ≠ 1)"),
    }
    println!("Bézout coefficients come straight out of the Euclidean recursion,");
    println!("which is also what makes CRT and modular inverse possible in O(log n)");
}

/// Returns `(g, x, y)` with `a·x + b·y = g = gcd(a, b)`.
fn egcd(a: i128, b: i128) -> (i128, i128, i128) {
    let (mut old_r, mut r) = (a, b);
    let (mut old_s, mut s) = (1i128, 0i128);
    let (mut old_t, mut t) = (0i128, 1i128);
    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
        (old_t, t) = (t, old_t - q * t);
    }
    (old_r, old_s, old_t)
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Inverse of `a` mod `m`, if gcd(a, m) = 1.
fn modinv(a: u64, m: u64) -> Option<u64> {
    let (g, x, _) = egcd(a as i128, m as i128);
    (g == 1).then(|| x.rem_euclid(m as i128) as u64)
}

// ─── Binary (fast) exponentiation ────────────────────────────────────────────

fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// a^e mod m by repeated squaring: O(log e) multiplications.
fn modpow(mut a: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u64 % m;
    a %= m;
    while e > 0 {
        if e & 1 == 1 {
            r = mulmod(r, a, m);
        }
        a = mulmod(a, a, m);
        e >>= 1;
    }
    r
}

/// The honest O(e) way: multiply by `a`, `e` times.
fn modpow_naive(a: u64, e: u64, m: u64) -> u64 {
    let mut r = 1u64 % m;
    for _ in 0..e {
        r = mulmod(r, a % m, m);
    }
    r
}

pub fn fast_exponentiation() {
    const M: u64 = 1_000_000_007;
    let cases: &[(u64, u64)] = &[(2, 0), (2, 10), (3, 5), (7, 100), (123_456_789, 1000), (2, 63)];

    println!("a^e mod {M}");
    let mut all = true;
    for &(a, e) in cases {
        let fast = modpow(a, e, M);
        let naive = modpow_naive(a, e, M);
        let ok = fast == naive;
        all &= ok;
        println!("  {a}^{e} = {fast}   (naive agrees: {ok})");
    }
    let e = 1000u64;
    let ops = (64 - e.leading_zeros()) as usize + e.count_ones() as usize;
    println!("all {} cases match: {all}", cases.len());
    println!("for e = {e}: modpow performs {ops} modular multiplications, naive performs {e}");
    println!("each step squares the base while halving the exponent → O(log e);");
    println!("this is the engine behind RSA signatures, fast modular powers, and modular inverses via a^(m−2)");
}

// ─── Chinese remainder theorem ────────────────────────────────────────────────

/// Solve `x ≡ a[i] (mod m[i])` for pairwise coprime moduli. Returns (x, M)
/// with the solution unique modulo M = ∏m[i].
fn crt(a: &[i64], m: &[i64]) -> Option<(i64, i64)> {
    let mut x = 0i64;
    let mut big_m = 1i64;
    for i in 0..a.len() {
        if gcd(big_m as i128, m[i] as i128) != 1 {
            return None; // moduli must be pairwise coprime
        }
        // Merge: x' = x + k·big_m with k ≡ (a[i] − x)·big_m⁻¹ (mod m[i]).
        let (g, inv, _) = egcd((big_m % m[i]) as i128, m[i] as i128);
        debug_assert_eq!(g, 1);
        let inv = inv.rem_euclid(m[i] as i128) as i64;
        let k = (a[i] - x).rem_euclid(m[i]) * inv % m[i];
        x += k * big_m;
        big_m *= m[i];
        x = x.rem_euclid(big_m);
    }
    Some((x, big_m))
}

pub fn chinese_remainder() {
    let instances: [(&[i64], &[i64]); 2] = [
        (&[2, 3, 2], &[3, 5, 7]),
        (&[5, 8, 4], &[9, 11, 13]),
    ];
    for (res, mods) in instances {
        print!("  ");
        for i in 0..res.len() {
            print!("x ≡ {} (mod {}){}", res[i], mods[i], if i + 1 == mods.len() { "\n" } else { ", " });
        }
        match crt(res, mods) {
            Some((x, mm)) => {
                let scan = (0..mm).find(|&k| {
                    res.iter().zip(mods).all(|(&ai, &mi)| k % mi == ai % mi)
                });
                println!("    smallest solution = {x} (mod {mm}), brute-force scan = {scan:?}, agrees: {}", scan == Some(x));
            }
            None => println!("    moduli not pairwise coprime — no unique solution guaranteed"),
        }
    }
    println!("each merge uses one extended-gcd inverse: O(k log m) for k congruences;");
    println!("classic reading: 2 mod 3, 3 mod 5, 2 mod 7 → 23 mod 105");
}

// ─── Matrix exponentiation ───────────────────────────────────────────────────

fn mat2_mul(a: [[i64; 2]; 2], b: [[i64; 2]; 2], m: i64) -> [[i64; 2]; 2] {
    let mut c = [[0i64; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            c[i][j] = ((a[i][0] as i128 * b[0][j] as i128 + a[i][1] as i128 * b[1][j] as i128)
                % m as i128) as i64;
        }
    }
    c
}

fn mat2_pow(mut a: [[i64; 2]; 2], mut e: u64, m: i64) -> [[i64; 2]; 2] {
    let mut r = [[1, 0], [0, 1]];
    while e > 0 {
        if e & 1 == 1 {
            r = mat2_mul(r, a, m);
        }
        a = mat2_mul(a, a, m);
        e >>= 1;
    }
    r
}

fn fib_naive(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

pub fn matrix_exponentiation() {
    const M: i64 = 1_000_000_007;
    let base = [[1, 1], [1, 0]]; // [[F₂,F₁],[F₁,F₀]]

    for n in [0u64, 1, 2, 10, 30] {
        let f = mat2_pow(base, n, i64::MAX / 4)[0][1]; // exact, small n
        let naive = fib_naive(n);
        println!("  fib({n:>2}) = {f:>8}  (iterative: {naive}, match: {})", f as u64 == naive);
    }

    let n = 1_000_000u64;
    let fast = mat2_pow(base, n, M)[0][1];
    let (mut a, mut b) = (0i64, 1i64);
    for _ in 0..n {
        let c = (a + b) % M;
        a = b;
        b = c;
    }
    println!("  fib({n}) mod {M} = {fast}  (10⁶ additions agree: {})", fast == a);
    let squarings = 64 - n.leading_zeros();
    println!("{squarings} matrix squarings + a handful of multiplies instead of {n} additions → O(log n);");
    println!("same trick solves any linear recurrence (order-k → k×k matrix) and");
    println!("boolean matrix powers for graph transitive closure");
}

// ─── Baby-step giant-step (discrete logarithm) ───────────────────────────────

/// Smallest x ≥ 0 with g^x ≡ h (mod p), if any. O(√p) time and memory.
fn bsgs(g: i64, h: i64, p: i64) -> Option<i64> {
    let m = (p as f64).sqrt().ceil() as i64 + 1;
    let mut table = std::collections::HashMap::new();
    let mut e = 1i64;
    for j in 0..m {
        table.entry(e).or_insert(j); // keep the smallest j for each value
        e = e * g % p;
    }
    // g^(−m) via Fermat: g^(p−1) ≡ 1 → g^(p−1−m) ≡ g^(−m)
    let factor = modpow(g as u64, (p - 1 - (m % (p - 1))) as u64, p as u64) as i64;
    let mut gamma = h.rem_euclid(p);
    for i in 0..m {
        if let Some(&j) = table.get(&gamma) {
            return Some(i * m + j); // h = g^(i·m + j)
        }
        gamma = gamma * factor % p;
    }
    None // h is not in the subgroup generated by g
}

pub fn discrete_log() {
    let p = 1_000_003i64;
    let g = 5i64;
    println!("find the smallest x with {g}^x ≡ h (mod {p})");
    let mut all_ok = true;
    for x0 in [0i64, 1, 42, 12_345, 400_000, 999_999] {
        let h = modpow(g as u64, x0 as u64, p as u64) as i64;
        let found = bsgs(g, h, p);
        // independent scan: the true smallest exponent
        let mut cur = 1i64;
        let brute = (0..p).find(|_| {
            let hit = cur == h;
            cur = cur * g % p;
            hit
        });
        let ok = found == brute && found.map(|x| modpow(g as u64, x as u64, p as u64) as i64 == h) == Some(true);
        all_ok &= ok;
        println!("  h = {g}^{x0} mod {p} = {h:>7} → bsgs = {found:?}, scan = {brute:?}, ok: {ok}");
    }
    println!("all results verified: {all_ok}");
    println!("precompute m ≈ √p baby steps in a hash table, then walk down by g^(−m) giant steps;");
    println!("O(√p) time & memory (Pollard's variant drops memory to O(1)) — the basis of");
    println!("index-calculus attacks on DLP-based crypto");
}

// ─── Miller–Rabin primality ───────────────────────────────────────────────────

/// Deterministic Miller–Rabin: this base set proves primality for every u64.
fn is_prime_mr(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    const SMALL: &[u64] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &d in SMALL {
        if n % d == 0 {
            return n == d;
        }
    }
    let mut d = n - 1;
    let mut r = 0u32;
    while d % 2 == 0 {
        d /= 2;
        r += 1;
    }
    'witness: for &a in &[2u64, 325, 9375, 28178, 450775, 9_780_504, 1_795_265_022] {
        let a = a % n;
        if a == 0 {
            continue;
        }
        let mut x = modpow(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..r - 1 {
            x = mulmod(x, x, n);
            if x == n - 1 {
                continue 'witness; // strong witness pair — probably prime
            }
        }
        return false; // a is a witness: n is composite
    }
    true
}

fn prime_trial(n: u64) -> bool {
    n >= 2 && (2u64..).take_while(|&d| d * d <= n).all(|d| n % d != 0)
}

pub fn miller_rabin() {
    const LIMIT: u64 = 200_000;
    let mut mismatches = 0u64;
    let mut primes = 0u64;
    for n in 0..LIMIT {
        let a = is_prime_mr(n);
        let b = prime_trial(n);
        primes += u64::from(a);
        mismatches += u64::from(a != b);
    }
    println!("every n < {LIMIT} matches trial division: {} ({mismatches} mismatches)", mismatches == 0);
    println!("π({LIMIT}) = {primes}");
    println!("spot checks:");
    for n in [
        1_000_000_007u64,
        1_000_000_009,
        999_999_937,
        2_305_843_009_213_693_951, // 2⁶¹ − 1, a Mersenne prime
        561,                       // Carmichael number: fool every Fermat test
        2_047,                     // strong pseudoprime to base 2
        3_215_031_751,             // strong pseudoprime to bases 2,3,5,7,11
    ] {
        println!("  is_prime({n}) = {}", is_prime_mr(n));
    }
    println!("n−1 = d·2^r, square a^d up to r times per base → O(k·log³n) with k bases;");
    println!("these 7 bases are proven sufficient for all 64-bit inputs (no probability left)");
}