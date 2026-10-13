//! Dynamic programming: fibonacci, 0/1 knapsack, LCS, edit distance, matrix chain,
//! Kadane, LIS, coin change, bitmask DP (TSP), digit DP.

use std::collections::HashMap;

/// Small deterministic LCG so demos are reproducible without dependencies.
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed >> 33
}

// ─── Fibonacci ────────────────────────────────────────────────────────────────

pub fn fibonacci() {
    let n = 30usize;

    // Bottom-up tabulation
    let mut dp = vec![0u64; n + 1];
    dp[1] = 1;
    for i in 2..=n {
        dp[i] = dp[i - 1] + dp[i - 2];
    }

    // Top-down memoization
    let mut memo = HashMap::new();
    let memo_val = fib_memo(n as u64, &mut memo);

    println!("tabulation: fib[0..={n}] = {dp:?}");
    println!("memoization: fib({n}) = {memo_val}");
    println!("iterative   fib({n}) = {}", dp[n]);
    println!("O(n) time, O(n) space (O(1) space possible with two rolling vars)");
}

fn fib_memo(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n < 2 {
        return n;
    }
    if let Some(&v) = memo.get(&n) {
        return v;
    }
    let v = fib_memo(n - 1, memo) + fib_memo(n - 2, memo);
    memo.insert(n, v);
    v
}

// ─── 0/1 Knapsack ─────────────────────────────────────────────────────────────

pub fn knapsack() {
    const WEIGHTS: &[u32] = &[2, 3, 4, 5];
    const VALUES: &[u32] = &[3, 4, 5, 6];
    const CAP: usize = 5;

    let n = WEIGHTS.len();
    // dp[i][w] = best value using the first i items with capacity w
    let mut dp = vec![vec![0u32; CAP + 1]; n + 1];
    for i in 1..=n {
        for w in 0..=CAP {
            dp[i][w] = dp[i - 1][w]; // skip item i
            let wt = WEIGHTS[i - 1] as usize;
            if wt <= w {
                let take = dp[i - 1][w - wt] + VALUES[i - 1];
                dp[i][w] = dp[i][w].max(take);
            }
        }
    }

    // Backtrack chosen items
    let mut w = CAP;
    let mut chosen = Vec::new();
    for i in (1..=n).rev() {
        if dp[i][w] != dp[i - 1][w] {
            chosen.push(i - 1);
            w -= WEIGHTS[i - 1] as usize;
        }
    }
    chosen.reverse();

    println!("items (0-based): (weight, value)");
    for i in 0..n {
        println!("  item {i}: ({}, {})", WEIGHTS[i], VALUES[i]);
    }
    println!("capacity = {CAP}");
    println!("optimal value = {}", dp[n][CAP]);
    println!(
        "take: {:?} → total weight {}",
        chosen,
        chosen.iter().map(|&i| WEIGHTS[i]).sum::<u32>()
    );
    println!("O(nW) time & space");
}

// ─── Longest Common Subsequence ───────────────────────────────────────────────

pub fn lcs() {
    let a = "ABCBDAB";
    let b = "BDCABA";
    let (lcs_str, len) = lcs_impl(a, b);
    println!("a = \"{a}\"");
    println!("b = \"{b}\"");
    println!("LCS  = \"{lcs_str}\" (length {len})");
    println!("O(mn) time & space — used by diff tools & bioinformatics");
}

fn lcs_impl(a: &str, b: &str) -> (String, usize) {
    let x: Vec<char> = a.chars().collect();
    let y: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; y.len() + 1]; x.len() + 1];
    for i in 0..x.len() {
        for j in 0..y.len() {
            dp[i + 1][j + 1] = if x[i] == y[j] {
                dp[i][j] + 1
            } else {
                dp[i][j + 1].max(dp[i + 1][j])
            };
        }
    }
    // Walk back to recover the subsequence
    let (mut i, mut j) = (x.len(), y.len());
    let mut out = String::new();
    while i > 0 && j > 0 {
        if x[i - 1] == y[j - 1] {
            out.push(x[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    let out: String = out.chars().rev().collect();
    let len = dp[x.len()][y.len()];
    (out, len)
}

// ─── Edit distance (Levenshtein) ──────────────────────────────────────────────

pub fn edit_distance() {
    let (a, b) = ("kitten", "sitting");
    let d = edit_impl(a, b);
    println!("edit_distance(\"{a}\", \"{b}\") = {d}");
    println!("  insert 's' → sitten, substitute 'e'→'i' → sittin, add 'g' → sitting");
    println!("O(mn) time, O(min(m,n)) space possible");
}

fn edit_impl(a: &str, b: &str) -> usize {
    let x: Vec<char> = a.chars().collect();
    let y: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; y.len() + 1]; x.len() + 1];
    for i in 0..=x.len() {
        dp[i][0] = i;
    }
    for j in 0..=y.len() {
        dp[0][j] = j;
    }
    for i in 0..x.len() {
        for j in 0..y.len() {
            let cost = if x[i] == y[j] { 0 } else { 1 };
            dp[i + 1][j + 1] = (dp[i][j + 1] + 1)
                .min(dp[i + 1][j] + 1)
                .min(dp[i][j] + cost);
        }
    }
    dp[x.len()][y.len()]
}

// ─── Matrix chain multiplication ──────────────────────────────────────────────

pub fn matrix_chain() {
    // Matrix i has dims[i-1] × dims[i]
    let dims = [30, 35, 15, 5, 10, 20, 25];
    let n = dims.len() - 1; // number of matrices

    // dp[i][j] = min cost to multiply matrices i..=j (0-based)
    let mut dp = vec![vec![0u64; n]; n];
    let mut split = vec![vec![0usize; n]; n];

    for len in 2..=n {
        for i in 0..=n - len {
            let j = i + len - 1;
            dp[i][j] = u64::MAX;
            for k in i..j {
                let cost = dp[i][k] + dp[k + 1][j] + dims[i] as u64 * dims[k + 1] as u64 * dims[j + 1] as u64;
                if cost < dp[i][j] {
                    dp[i][j] = cost;
                    split[i][j] = k;
                }
            }
        }
    }

    println!("matrix dimensions: {dims:?}");
    println!("  (matrix i is {}×{})", dims[0], dims[1]);
    print!("optimal parenthesization: ");
    print_chain(&split, 0, n - 1);
    println!();
    println!("min scalar multiplications = {}", dp[0][n - 1]);
    println!("O(n^3) time, O(n^2) space");
}

// ─── Kadane's algorithm ──────────────────────────────────────────────────────

pub fn kadane() {
    let a = [-2i64, 1, -3, 4, -1, 2, 1, -5, 4];
    let (best, s, e) = kadane_impl(&a);
    let brute = brute_max_sub(&a);
    println!("array = {a:?}");
    println!(
        "max subarray = {:?} → sum {best}  (a[{s}..{e}])",
        &a[s..e]
    );
    println!("O(n²) brute force agrees: {}", best == brute);

    let mut seed = 0x5eed_u64;
    let mut ok = true;
    for _ in 0..500 {
        let n = 1 + next_rand(&mut seed) % 40;
        let v: Vec<i64> = (0..n)
            .map(|_| next_rand(&mut seed) as i64 % 21 - 10)
            .collect();
        let (got, gs, ge) = kadane_impl(&v);
        ok &= got == brute_max_sub(&v) && v[gs..ge].iter().sum::<i64>() == got;
    }
    println!("500 random mixed-sign arrays verified (sum + window): {ok}");
    println!("O(n) time, O(1) space — keep adding while profitable, reset when the run turns negative");
}

/// Returns `(best_sum, start, end_exclusive)`.
fn kadane_impl(a: &[i64]) -> (i64, usize, usize) {
    let (mut best, mut cur) = (a[0], a[0]);
    let (mut bs, mut be, mut cs) = (0usize, 1usize, 0usize);
    for i in 1..a.len() {
        if cur < 0 {
            cur = a[i];
            cs = i; // previous run can never be worth extending
        } else {
            cur += a[i];
        }
        if cur > best {
            best = cur;
            bs = cs;
            be = i + 1;
        }
    }
    (best, bs, be)
}

fn brute_max_sub(a: &[i64]) -> i64 {
    let mut best = i64::MIN;
    for i in 0..a.len() {
        let mut s = 0i64;
        for j in i..a.len() {
            s += a[j];
            best = best.max(s);
        }
    }
    best
}

// ─── Longest increasing subsequence ─────────────────────────────────────────

pub fn lis() {
    let a = [10i64, 9, 2, 5, 3, 7, 101, 18];
    let idx = lis_indices(&a);
    let seq: Vec<i64> = idx.iter().map(|&i| a[i]).collect();
    let want = lis_length_dp(&a);

    let valid = idx.windows(2).all(|w| w[0] < w[1])
        && seq.windows(2).all(|w| w[0] < w[1]);

    println!("array = {a:?}");
    println!("LIS = {seq:?}  (indices {idx:?})");
    println!("is a real strictly-increasing subsequence: {valid}");
    println!("O(n²) DP length = {want}, match: {}", seq.len() == want);

    let mut seed = 0x1234_u64;
    let mut ok = true;
    for _ in 0..300 {
        let n = 1 + next_rand(&mut seed) % 30;
        let v: Vec<i64> = (0..n).map(|_| next_rand(&mut seed) as i64 % 15).collect();
        ok &= lis_indices(&v).len() == lis_length_dp(&v);
    }
    println!("300 random arrays verified against the O(n²) DP: {ok}");
    println!("tails[k] = smallest tail of any increasing run of length k+1 → O(n log n)");
}

/// Patience-style LIS: `tails[k]` holds the index of the smallest tail for an
/// increasing subsequence of length k+1; `prev` reconstructs the actual run.
fn lis_indices(a: &[i64]) -> Vec<usize> {
    let mut tails: Vec<usize> = Vec::new();
    let mut prev = vec![usize::MAX; a.len()];
    for i in 0..a.len() {
        // first pile whose tail is ≥ a[i] (strictly increasing ⇒ lower bound)
        let (mut lo, mut hi) = (0usize, tails.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if a[tails[mid]] < a[i] {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo > 0 {
            prev[i] = tails[lo - 1];
        }
        if lo == tails.len() {
            tails.push(i);
        } else {
            tails[lo] = i;
        }
    }

    let mut out = Vec::new();
    let mut cur = *tails.last().unwrap();
    loop {
        out.push(cur);
        if prev[cur] == usize::MAX {
            break;
        }
        cur = prev[cur];
    }
    out.reverse();
    out
}

fn lis_length_dp(a: &[i64]) -> usize {
    let mut dp = vec![1usize; a.len()];
    for i in 0..a.len() {
        for j in 0..i {
            if a[j] < a[i] {
                dp[i] = dp[i].max(dp[j] + 1);
            }
        }
    }
    dp.into_iter().max().unwrap_or(0)
}

// ─── Coin change ─────────────────────────────────────────────────────────────

pub fn coin_change() {
    let coins = [1i64, 3, 4];
    let amount = 6i64;

    // dp[a] = fewest coins that add up to a
    let mut dp = vec![i64::MAX / 2; amount as usize + 1];
    dp[0] = 0;
    for a in 1..=amount {
        for &c in &coins {
            if c <= a {
                dp[a as usize] = dp[a as usize].min(dp[(a - c) as usize] + 1);
            }
        }
    }

    // dp2[w] = combinations that add up to w (order-independent)
    let mut dp2 = vec![0u64; amount as usize + 1];
    dp2[0] = 1;
    for &c in &coins {
        for a in c..=amount {
            dp2[a as usize] += dp2[(a - c) as usize];
        }
    }

    let bfs = min_coins_bfs(&coins, amount);
    let rec = count_ways(&coins, 0, amount);

    println!("coins = {coins:?}, amount = {amount}");
    println!("min coins = {}  (BFS over amounts = {bfs}, match: {})", dp[amount as usize], dp[amount as usize] == bfs);
    println!("combinations = {}  (recursive enumeration = {rec}, match: {})", dp2[amount as usize], dp2[amount as usize] == rec);
    println!("min coins per amount 0..={amount}: {:?}", &dp[..=amount as usize]);
    println!("combinations per amount 0..={amount}: {:?}", &dp2[..=amount as usize]);
    println!("unbounded knapsack pattern → O(amount × #coins) time; loop coins outside for combinations,");
    println!("inside for permutations — the classic order-of-loops gotcha");
}

fn min_coins_bfs(coins: &[i64], amount: i64) -> i64 {
    if amount == 0 {
        return 0;
    }
    let mut seen = vec![false; amount as usize + 1];
    let mut frontier = vec![0i64];
    seen[0] = true;
    let mut steps = 0i64;
    while !frontier.is_empty() {
        steps += 1;
        let mut next = Vec::new();
        for &a in &frontier {
            for &c in coins {
                let na = a + c;
                if na == amount {
                    return steps;
                }
                if na < amount && !seen[na as usize] {
                    seen[na as usize] = true;
                    next.push(na);
                }
            }
        }
        frontier = next;
    }
    -1 // unreachable
}

/// Independent brute force: use coin `i` again, or move to coin `i+1`.
fn count_ways(coins: &[i64], i: usize, rem: i64) -> u64 {
    if rem == 0 {
        return 1;
    }
    if rem < 0 || i == coins.len() {
        return 0;
    }
    count_ways(coins, i, rem - coins[i]) + count_ways(coins, i + 1, rem)
}

// ─── Bitmask DP (TSP) ────────────────────────────────────────────────────────

pub fn bitmask_dp() {
    const N: usize = 9;
    let mut seed = 0x1234_5678_u64;
    let mut dist = vec![vec![0i64; N]; N];
    for i in 0..N {
        for j in i + 1..N {
            let w = next_rand(&mut seed) % 90 + 10;
            dist[i][j] = w as i64;
            dist[j][i] = w as i64;
        }
    }

    // dp[mask][i] = cheapest way to visit exactly `mask`, ending at city i
    let full = (1u32 << N) - 1;
    let mut dp = vec![vec![i64::MAX; N]; (1 << N) as usize];
    dp[1][0] = 0; // start at city 0
    for mask in 0..=full {
        for i in 0..N {
            if dp[mask as usize][i] == i64::MAX || mask & (1 << i) == 0 {
                continue;
            }
            for j in 0..N {
                if mask & (1 << j) != 0 {
                    continue;
                }
                let nm = (mask | (1 << j)) as usize;
                dp[nm][j] = dp[nm][j].min(dp[mask as usize][i] + dist[i][j]);
            }
        }
    }
    let best = (0..N).map(|i| dp[full as usize][i] + dist[i][0]).min().unwrap();

    let mut brute = i64::MAX;
    tsp_brute(&dist, 0, full ^ 1, 0, &mut brute);

    println!("symmetric tour costs for {N} cities (seeded random, 10…99):");
    for row in &dist {
        println!("  {:?}", row);
    }
    println!("shortest closed tour = {best}");
    println!("memo-free recursion over all (n−1)! = {} orderings = {brute}", (1..N as u64).product::<u64>());
    println!("match: {}", best == brute);
    println!("O(2ⁿ·n²) time / O(2ⁿ·n) space beats O(n!) — subsets become the DP index");
}

fn tsp_brute(dist: &[Vec<i64>], at: usize, remaining: u32, cost: i64, best: &mut i64) {
    if remaining == 0 {
        *best = (*best).min(cost + dist[at][0]);
        return;
    }
    for v in 0..dist.len() {
        if remaining & (1 << v) != 0 {
            tsp_brute(dist, v, remaining ^ (1 << v), cost + dist[at][v], best);
        }
    }
}

fn print_chain(split: &[Vec<usize>], i: usize, j: usize) {
    if i == j {
        print!("M{}", i + 1);
        return;
    }
    print!("(");
    print_chain(split, i, split[i][j]);
    print!("×");
    print_chain(split, split[i][j] + 1, j);
    print!(")");
}

// ─── Digit DP ─────────────────────────────────────────────────────────────────

fn contains_13(x: i64) -> bool {
    let s = x.to_string();
    s.as_bytes().windows(2).any(|w| w == b"13")
}

/// Count `0 ≤ x ≤ n` whose decimal digits contain "13" and x ≡ 0 (mod 13)
/// (the classic "B-number" pattern). Memoized over the digit position.
fn digit_dp_count(n: i64) -> i64 {
    let digits: Vec<u8> = n.to_string().bytes().map(|b| b - b'0').collect();
    let mut memo: HashMap<(usize, usize, bool, bool), i64> = HashMap::new();
    digit_dp_go(0, 0, false, false, true, &digits, &mut memo)
}

fn digit_dp_go(
    pos: usize,
    rem: usize,       // value of the prefix mod 13
    found: bool,       // "13" already seen in the prefix
    prev_one: bool,    // previous digit was 1
    tight: bool,       // prefix equals the bound so far
    digits: &[u8],
    memo: &mut HashMap<(usize, usize, bool, bool), i64>,
) -> i64 {
    if pos == digits.len() {
        return i64::from(found && rem == 0);
    }
    let key = (pos, rem, found, prev_one);
    if !tight {
        if let Some(&v) = memo.get(&key) {
            return v;
        }
    }
    let limit = if tight { digits[pos] } else { 9 };
    let mut total = 0i64;
    for d in 0..=limit {
        total += digit_dp_go(
            pos + 1,
            (rem * 10 + d as usize) % 13,
            found || (prev_one && d == 3),
            d == 1,
            tight && d == limit,
            digits,
            memo,
        );
    }
    if !tight {
        memo.insert(key, total);
    }
    total
}

pub fn digit_dp() {
    // Small bound: verify against a full scan.
    let small = 100_000i64;
    let dp_small = digit_dp_count(small);
    let brute: i64 = (0..=small).filter(|&x| contains_13(x) && x % 13 == 0).count() as i64;

    // Large bound: only the DP is fast enough.
    let big = 987_654_321i64;
    let dp_big = digit_dp_count(big);

    println!("count x ≤ N with \"13\" in the digits and x ≡ 0 (mod 13)");
    println!("  N = {small}: dp = {dp_small}, full scan = {brute}, match: {}", dp_small == brute);
    println!("  N = {big}: dp = {dp_big}  (a full scan would check {} numbers)", big + 1);
    println!("memo: (position, remainder mod 13, found \"13\", previous digit is 1) × tight");
    println!("→ O(digits × 13 × 4 × 10) ≈ {} transitions vs O(N) scanning",
        11 * 13 * 4 * 10);
}
