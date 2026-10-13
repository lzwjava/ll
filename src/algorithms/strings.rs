//! String algorithms: KMP, Rabin-Karp, Boyer-Moore (Horspool variant).

// ─── Knuth-Morris-Pratt ───────────────────────────────────────────────────────

pub fn kmp() {
    let text = "ababcababcabcab";
    let pattern = "abcab";

    let lps = build_lps(pattern);
    let hits = kmp_search(text, pattern, &lps);

    println!("text    = \"{text}\"");
    println!("pattern = \"{pattern}\"");
    println!("LPS table (longest proper prefix = suffix): {lps:?}");
    println!("matches at indices: {hits:?}");
    println!("O(n+m): a mismatch reuses prefix info instead of rescanning the text");
}

/// lps[i] = length of the longest proper prefix of pattern[..=i] that is also a suffix.
fn build_lps(pattern: &str) -> Vec<usize> {
    let p: Vec<char> = pattern.chars().collect();
    let mut lps = vec![0usize; p.len()];
    let mut len = 0usize;
    let mut i = 1;
    while i < p.len() {
        if p[i] == p[len] {
            len += 1;
            lps[i] = len;
            i += 1;
        } else if len > 0 {
            len = lps[len - 1]; // fallback instead of restarting
        } else {
            lps[i] = 0;
            i += 1;
        }
    }
    lps
}

fn kmp_search(text: &str, pattern: &str, lps: &[usize]) -> Vec<usize> {
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
    let mut hits = Vec::new();
    while i < t.len() {
        if t[i] == p[j] {
            i += 1;
            j += 1;
            if j == p.len() {
                hits.push(i - j);
                j = lps[j - 1];
            }
        } else if j > 0 {
            j = lps[j - 1];
        } else {
            i += 1;
        }
    }
    hits
}

// ─── Rabin-Karp ───────────────────────────────────────────────────────────────

pub fn rabin_karp() {
    let text = "ababcababcabcab";
    let pattern = "abcab";

    let hits = rk_search(text, pattern);
    println!("text    = \"{text}\"");
    println!("pattern = \"{pattern}\"");
    println!("matches at indices: {hits:?}");
    println!("sliding polynomial hash → O(n+m) expected; hash hits are re-verified,");
    println!("so collisions cost a direct compare but never a false match");
}

fn rk_search(text: &str, pattern: &str) -> Vec<usize> {
    const BASE: u64 = 256;
    const MOD: u64 = 1_000_000_007;

    let t: Vec<u8> = text.bytes().collect();
    let p: Vec<u8> = pattern.bytes().collect();
    if p.is_empty() || p.len() > t.len() {
        return vec![];
    }

    let mut high = 1u64;
    for _ in 0..p.len() - 1 {
        high = high * BASE % MOD;
    }
    let mut ph = 0u64;
    let mut th = 0u64;
    for i in 0..p.len() {
        ph = (ph * BASE + p[i] as u64) % MOD;
        th = (th * BASE + t[i] as u64) % MOD;
    }

    let mut hits = Vec::new();
    for i in 0..=t.len() - p.len() {
        if th == ph && t[i..i + p.len()] == p[..] {
            hits.push(i);
        }
        if i + p.len() < t.len() {
            // roll the window: drop t[i], add t[i+m]
            th = (th + MOD - t[i] as u64 * high % MOD) % MOD;
            th = (th * BASE + t[i + p.len()] as u64) % MOD;
        }
    }
    hits
}

// ─── Z-algorithm ─────────────────────────────────────────────────────────────

pub fn z_algorithm() {
    let text = "ababcababcabcab";
    let pattern = "abcab";

    // Z[i] = longest substring starting at i that matches a prefix of the input.
    let zpat = z_function(pattern.as_bytes());

    // Pattern matching: Z-array of "pattern$text" — a hit needs Z ≥ pattern length.
    let mut concat = pattern.as_bytes().to_vec();
    concat.push(b'$'); // separator that cannot appear in the pattern
    concat.extend_from_slice(text.as_bytes());
    let z = z_function(&concat);
    let sep = pattern.len() + 1;
    let hits: Vec<usize> = z
        .iter()
        .enumerate()
        .skip(sep)
        .filter(|&(_, &v)| v >= pattern.len())
        .map(|(i, _)| i - sep)
        .collect();

    let naive: Vec<usize> = (0..=text.len() - pattern.len())
        .filter(|&i| text.as_bytes()[i..i + pattern.len()] == *pattern.as_bytes())
        .collect();

    println!("text    = \"{text}\"");
    println!("pattern = \"{pattern}\"");
    println!("Z(pattern) = {zpat:?}");
    println!("matches at indices: {hits:?}");
    println!("matches naive scan: {}", hits == naive);
    println!("the Z-box [l, r) is reused exactly like KMP's prefix table → O(n+m),");
    println!("but the one-array formulation is usually easier to get right under time pressure");
}

/// Z-array: `z[0]` is 0 by convention, `z[i]` = longest common prefix of `s` and `s[i..]`.
fn z_function(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut z = vec![0usize; n];
    let (mut l, mut r) = (0usize, 0usize);
    for i in 1..n {
        if i < r {
            z[i] = (r - i).min(z[i - l]); // reuse what the previous box already proved
        }
        while i + z[i] < n && s[z[i]] == s[i + z[i]] {
            z[i] += 1;
        }
        if i + z[i] > r {
            l = i;
            r = i + z[i];
        }
    }
    z
}

// ─── Boyer-Moore (Horspool bad-character rule) ────────────────────────────────

pub fn boyer_moore() {
    let text = "ababcababcabcab";
    let pattern = "abcab";

    let table = bad_char_table(pattern);
    let hits = bm_search(text, pattern, &table);

    println!("text    = \"{text}\"");
    println!("pattern = \"{pattern}\"");
    let mut chars: Vec<(u8, usize)> = table
        .iter()
        .enumerate()
        .filter(|(b, _)| pattern.as_bytes().contains(&(*b as u8)))
        .map(|(b, &s)| (b as u8, s))
        .collect();
    chars.sort_unstable();
    println!("bad-character skips (pattern bytes only): {chars:?}");
    println!("matches at indices: {hits:?}");
    println!("scans the pattern right-to-left and skips whole blocks on mismatch,");
    println!("which is why grep-style search is sub-linear on average");
}

/// Horspool: for each byte, how far we may shift when a mismatch ends at that byte.
fn bad_char_table(pattern: &str) -> [usize; 256] {
    let p = pattern.as_bytes();
    let m = p.len();
    let mut table = [m; 256];
    // For every byte except the last position of the pattern, the shift aligns
    // that byte's last occurrence with the pattern's end.
    for i in 0..m - 1 {
        table[p[i] as usize] = m - 1 - i;
    }
    table
}

fn bm_search(text: &str, pattern: &str, table: &[usize; 256]) -> Vec<usize> {
    let t = text.as_bytes();
    let p = pattern.as_bytes();
    let (n, m) = (t.len(), p.len());
    let mut hits = Vec::new();
    let mut i = 0usize;
    while i + m <= n {
        let mut j = m;
        while j > 0 && t[i + j - 1] == p[j - 1] {
            j -= 1;
        }
        if j == 0 {
            hits.push(i);
            i += 1; // or shift by m
        } else {
            i += table[t[i + m - 1] as usize];
        }
    }
    hits
}
