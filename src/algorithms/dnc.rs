//! Divide & conquer: Strassen matrix multiplication, Karatsuba multiplication.

use std::time::Instant;

// ─── Strassen ─────────────────────────────────────────────────────────────────

pub fn strassen() {
    let n = 4;
    // Deterministic pseudo-random entries in -9..=9
    let mut seed = 0x1234u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) % 19) as i64 - 9
    };
    let a: Vec<Vec<i64>> = (0..n).map(|_| (0..n).map(|_| next()).collect()).collect();
    let b: Vec<Vec<i64>> = (0..n).map(|_| (0..n).map(|_| next()).collect()).collect();

    let naive = naive_mul(&a, &b);
    let fast = strassen_mul(&a, &b);
    let equal = naive == fast;

    println!("{n}×{n} matrices (entries −9..9):");
    println!("A = {}", fmt(&a));
    println!("B = {}", fmt(&b));
    println!("naive  A·B = {}", fmt(&naive));
    println!("strassen A·B = {}", fmt(&fast));
    println!("results identical: {equal}");
    println!(
        "scalar multiplications: naive = {} = n³, strassen = 7·(n/2)²·… = O(n^2.81)",
        n * n * n
    );
}

fn fmt(m: &[Vec<i64>]) -> String {
    let rows: Vec<String> = m
        .iter()
        .map(|r| {
            r.iter()
                .map(|v| format!("{v:>4}"))
                .collect::<Vec<_>>()
                .concat()
        })
        .collect();
    format!("[{}]", rows.join(" | "))
}

fn naive_mul(a: &[Vec<i64>], b: &[Vec<i64>]) -> Vec<Vec<i64>> {
    let n = a.len();
    let mut c = vec![vec![0i64; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0i64;
            for k in 0..n {
                sum += a[i][k] * b[k][j];
            }
            c[i][j] = sum;
        }
    }
    c
}

fn add(x: &[Vec<i64>], y: &[Vec<i64>]) -> Vec<Vec<i64>> {
    x.iter()
        .zip(y)
        .map(|(r1, r2)| r1.iter().zip(r2).map(|(a, b)| a + b).collect())
        .collect()
}

fn sub(x: &[Vec<i64>], y: &[Vec<i64>]) -> Vec<Vec<i64>> {
    x.iter()
        .zip(y)
        .map(|(r1, r2)| r1.iter().zip(r2).map(|(a, b)| a - b).collect())
        .collect()
}

/// Split into quadrants (input must be even-sized).
fn quadrants(m: &[Vec<i64>]) -> (Vec<Vec<i64>>, Vec<Vec<i64>>, Vec<Vec<i64>>, Vec<Vec<i64>>) {
    let h = m.len() / 2;
    let mut a11 = vec![vec![0i64; h]; h];
    let mut a12 = vec![vec![0i64; h]; h];
    let mut a21 = vec![vec![0i64; h]; h];
    let mut a22 = vec![vec![0i64; h]; h];
    for i in 0..h {
        for j in 0..h {
            a11[i][j] = m[i][j];
            a12[i][j] = m[i][j + h];
            a21[i][j] = m[i + h][j];
            a22[i][j] = m[i + h][j + h];
        }
    }
    (a11, a12, a21, a22)
}

fn join(
    c11: Vec<Vec<i64>>,
    c12: Vec<Vec<i64>>,
    c21: Vec<Vec<i64>>,
    c22: Vec<Vec<i64>>,
) -> Vec<Vec<i64>> {
    let h = c11.len();
    let n = h * 2;
    let mut c = vec![vec![0i64; n]; n];
    for i in 0..h {
        for j in 0..h {
            c[i][j] = c11[i][j];
            c[i][j + h] = c12[i][j];
            c[i + h][j] = c21[i][j];
            c[i + h][j + h] = c22[i][j];
        }
    }
    c
}

fn strassen_mul(a: &[Vec<i64>], b: &[Vec<i64>]) -> Vec<Vec<i64>> {
    let n = a.len();
    if n <= 2 {
        return naive_mul(a, b);
    }
    let (a11, a12, a21, a22) = quadrants(a);
    let (b11, b12, b21, b22) = quadrants(b);

    let p1 = strassen_mul(&add(&a11, &a22), &add(&b11, &b22));
    let p2 = strassen_mul(&add(&a21, &a22), &b11);
    let p3 = strassen_mul(&a11, &sub(&b12, &b22));
    let p4 = strassen_mul(&a22, &sub(&b21, &b11));
    let p5 = strassen_mul(&add(&a11, &a12), &b22);
    let p6 = strassen_mul(&sub(&a21, &a11), &add(&b11, &b12));
    let p7 = strassen_mul(&sub(&a12, &a22), &add(&b21, &b22));

    let c11 = add(&sub(&add(&p1, &p4), &p5), &p7);
    let c12 = add(&p3, &p5);
    let c21 = add(&p2, &p4);
    let c22 = add(&sub(&add(&p1, &p3), &p2), &p6);
    join(c11, c12, c21, c22)
}

// ─── Karatsuba ────────────────────────────────────────────────────────────────

/// Big non-negative integers as base-10⁴ digits, little-endian.
type Big = Vec<u32>;

const BASE: u64 = 10_000;
const CUTOFF: usize = 16;

pub fn karatsuba() {
    // Big multiplication: two 40-digit numbers
    let x = "3141592653589793238462643383279502884197";
    let y = "2718281828459045235360287471352662497757";
    let a = from_dec(x);
    let b = from_dec(y);
    let fast = karatsuba_mul(&a, &b);
    let slow = schoolbook_mul(&a, &b);
    println!("{x}");
    println!("×  {y}");
    println!("karatsuba = {}", to_dec(&fast));
    println!("schoolbook = {}", to_dec(&slow));
    println!("match: {}\n", fast == slow);

    // Randomized correctness check against u128 arithmetic
    let mut seed = 0xabcdefu64;
    let mut ok = true;
    for _ in 0..1000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let p = (seed >> 11) & 0x7fff_ffff_ffff;
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let q = (seed >> 11) & 0x7fff_ffff_ffff;
        let expected = p as u128 * q as u128;
        if to_dec(&karatsuba_mul(&from_u64(p), &from_u64(q))) != expected.to_string() {
            ok = false;
        }
    }
    println!("1000 random 64-bit products verified: {ok}");
    println!("O(n^1.585) vs O(n²) schoolbook (here: 3 multiplications per split instead of 4)\n");

    println!("timing on ~4000-digit operands:");
    bench_mul();
}

fn trim(mut v: Big) -> Big {
    while v.len() > 1 && *v.last().unwrap() == 0 {
        v.pop();
    }
    v
}

fn from_u64(mut x: u64) -> Big {
    let mut v = Vec::new();
    while x > 0 {
        v.push((x % BASE as u64) as u32);
        x /= BASE as u64;
    }
    if v.is_empty() {
        v.push(0);
    }
    v
}

fn from_dec(s: &str) -> Big {
    let bytes = s.as_bytes();
    let mut v = Vec::new();
    let mut i = bytes.len();
    while i > 0 {
        let start = i.saturating_sub(4);
        let chunk = std::str::from_utf8(&bytes[start..i]).unwrap();
        v.push(chunk.parse::<u32>().unwrap());
        i = start;
    }
    trim(v)
}

fn to_dec(v: &[u32]) -> String {
    let mut out = format!("{}", v.last().unwrap());
    for &d in v[..v.len() - 1].iter().rev() {
        out.push_str(&format!("{d:04}"));
    }
    out
}

fn add_big(a: &[u32], b: &[u32]) -> Big {
    let n = a.len().max(b.len());
    let mut out = vec![0u32; n + 1];
    let mut carry = 0u64;
    for i in 0..n {
        let av = *a.get(i).unwrap_or(&0) as u64;
        let bv = *b.get(i).unwrap_or(&0) as u64;
        let s = av + bv + carry;
        out[i] = (s % BASE) as u32;
        carry = s / BASE;
    }
    out[n] = carry as u32;
    trim(out)
}

/// Requires a ≥ b.
fn sub_big(a: &[u32], b: &[u32]) -> Big {
    let mut out = vec![0u32; a.len()];
    let mut borrow = 0i64;
    for i in 0..a.len() {
        let av = a[i] as i64;
        let bv = *b.get(i).unwrap_or(&0) as i64;
        let mut d = av - bv - borrow;
        if d < 0 {
            d += BASE as i64;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = d as u32;
    }
    debug_assert_eq!(borrow, 0, "subtraction underflow");
    trim(out)
}

fn schoolbook_mul(a: &[u32], b: &[u32]) -> Big {
    let mut out = vec![0u32; a.len() + b.len() + 1];
    for i in 0..a.len() {
        let mut carry = 0u64;
        for j in 0..b.len() {
            let cur = out[i + j] as u64 + a[i] as u64 * b[j] as u64 + carry;
            out[i + j] = (cur % BASE) as u32;
            carry = cur / BASE;
        }
        let mut idx = i + b.len();
        while carry > 0 {
            let cur = out[idx] as u64 + carry;
            out[idx] = (cur % BASE) as u32;
            carry = cur / BASE;
            idx += 1;
        }
    }
    trim(out)
}

fn split(v: &[u32], m: usize) -> (Big, Big) {
    let low = if v.len() > m { v[..m].to_vec() } else { v.to_vec() };
    let high = if v.len() > m {
        v[m..].to_vec()
    } else {
        vec![0]
    };
    (trim(low), trim(high))
}

fn karatsuba_mul(a: &[u32], b: &[u32]) -> Big {
    let n = a.len().max(b.len());
    if n <= CUTOFF {
        return schoolbook_mul(a, b);
    }
    let m = n / 2;
    let (a0, a1) = split(a, m);
    let (b0, b1) = split(b, m);

    let z0 = karatsuba_mul(&a0, &b0);
    let z2 = karatsuba_mul(&a1, &b1);
    let s1 = add_big(&a0, &a1);
    let s2 = add_big(&b0, &b1);
    let z1 = sub_big(&sub_big(&karatsuba_mul(&s1, &s2), &z0), &z2);

    // result = z2·B^(2m) + z1·B^m + z0
    let mut result = z0;
    result = add_big(&result, &shift(&z1, m));
    result = add_big(&result, &shift(&z2, 2 * m));
    result
}

fn shift(v: &[u32], k: usize) -> Big {
    let mut out = vec![0u32; k];
    out.extend_from_slice(v);
    trim(out)
}

/// Timing comparison: big Karatsuba vs schoolbook on larger operands.
fn bench_mul() {
    let mut seed = 7u64;
    let digits = 4000usize;
    let mut a = Vec::with_capacity(digits);
    let mut b = Vec::with_capacity(digits);
    for _ in 0..digits {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        a.push(((seed >> 33) % BASE as u64) as u32);
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        b.push(((seed >> 33) % BASE as u64) as u32);
    }
    let t0 = Instant::now();
    let _ = schoolbook_mul(&a, &b);
    let t1 = Instant::now();
    let _ = karatsuba_mul(&a, &b);
    let t2 = Instant::now();
    println!(
        "  schoolbook: {:.1?}   karatsuba: {:.1?}",
        t1 - t0,
        t2 - t1
    );
}
