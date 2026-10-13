//! Number theory: Sieve of Eratosthenes, extended Euclidean algorithm.

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