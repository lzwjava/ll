//! Dynamic programming: fibonacci, 0/1 knapsack, LCS, edit distance, matrix chain.

use std::collections::HashMap;

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
