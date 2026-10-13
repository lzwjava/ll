//! String algorithms: KMP, Rabin-Karp, Boyer-Moore (Horspool variant),
//! Z-algorithm, Manacher (palindromes), Aho-Corasick (multi-pattern),
//! suffix array + LCP.

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

// ─── Manacher (longest palindromic substring) ──────────────────────────────────

/// Returns `(start, length)` of a longest palindromic substring in O(n).
fn longest_palindrome(s: &str) -> (usize, usize) {
    let bytes = s.as_bytes();
    // Transform "abc" → [0, '#', 'a', '#', 'b', '#', 'c', 1]: odd and even
    // palindromes both become odd runs around one center. Distinct sentinels
    // guarantee the expansion loop always stops at the borders.
    let mut t = vec![0u8];
    for &c in bytes {
        t.push(b'#');
        t.push(c);
    }
    t.push(b'#'); // mirror the leading '#' so both borders are symmetric
    t.push(1);
    let n = t.len();
    let mut p = vec![0usize; n];
    let (mut center, mut right) = (0usize, 0usize);
    for i in 1..n - 1 {
        if i < right {
            p[i] = p[2 * center - i].min(right - i); // mirror across `center`
        }
        while i > p[i] && i + p[i] + 1 < n && t[i + p[i] + 1] == t[i - p[i] - 1] {
            p[i] += 1;
        }
        if i + p[i] > right {
            center = i;
            right = i + p[i];
        }
    }
    // p[i] = radius in the transformed string = length in the original string.
    let best_i = (1..n - 1).max_by_key(|&i| p[i]).unwrap();
    let best_len = p[best_i];
    let left_t = best_i - best_len;
    let first_letter_t = if left_t % 2 == 0 { left_t } else { left_t + 1 };
    ((first_letter_t - 2) / 2, best_len)
}

fn brute_palindrome(s: &str) -> (usize, usize) {
    let b = s.as_bytes();
    let (mut bs, mut bl) = (0usize, 0usize);
    for i in 0..b.len() {
        for j in i..b.len() {
            if j - i + 1 > bl && (i..=j).all(|k| b[k] == b[j - (k - i)]) {
                bs = i;
                bl = j - i + 1;
            }
        }
    }
    (bs, bl)
}

pub fn manacher() {
    let mut all_ok = true;
    for text in ["forgeeksskeegfor", "bananas", "cbbd", "abacabadabacaba"] {
        let (start, len) = longest_palindrome(text);
        let (_, blen) = brute_palindrome(text);
        let ok = len == blen;
        all_ok &= ok;
        println!(
            "  \"{text}\" → \"{}\" at {start} (len {len}), brute force len {blen}: {ok}",
            &text[start..start + len]
        );
    }
    println!("all lengths match the O(n²) scan: {all_ok}");
    println!("the same expanding box as the Z-array, but centred on every position with a mirror");
    println!("→ O(n) time, O(n) space; answers both odd- and even-length palindromes");
}

// ─── Aho-Corasick (multi-pattern matching) ─────────────────────────────────────

/// Trie with failure links, flattened into a full transition table so matching
/// never walks back up. `out[v]` lists every pattern that ends at `v`.
fn build_aho(patterns: &[&str]) -> (Vec<[i32; 26]>, Vec<Vec<usize>>) {
    let mut goto = vec![[-1i32; 26]];
    let mut out: Vec<Vec<usize>> = vec![Vec::new()];
    for (pid, pat) in patterns.iter().enumerate() {
        let mut node = 0usize;
        for &c in pat.as_bytes() {
            let ci = (c - b'a') as usize;
            if goto[node][ci] == -1 {
                goto[node][ci] = goto.len() as i32;
                goto.push([-1; 26]);
                out.push(Vec::new());
            }
            node = goto[node][ci] as usize;
        }
        out[node].push(pid);
    }
    // BFS over depths: wire failure links, inherit their outputs, close the table.
    let mut fail = vec![0usize; goto.len()];
    let mut queue = std::collections::VecDeque::new();
    for c in 0..26 {
        let t = goto[0][c];
        if t != -1 {
            queue.push_back(t as usize);
        } else {
            goto[0][c] = 0; // root misses stay at the root
        }
    }
    while let Some(u) = queue.pop_front() {
        for c in 0..26 {
            let t = goto[u][c];
            if t != -1 {
                let v = t as usize;
                let f = goto[fail[u]][c];
                fail[v] = f as usize;
                let inherited = out[fail[v]].clone(); // e.g. "he" ends inside "she"
                out[v].extend(inherited);
                queue.push_back(v);
            } else {
                let f = goto[fail[u]][c];
                goto[u][c] = f; // follow the failure link in O(1)
            }
        }
    }
    (goto, out)
}

fn ac_search(text: &str, patterns: &[&str], goto: &[[i32; 26]], out: &[Vec<usize>]) -> Vec<(usize, usize)> {
    let mut hits = Vec::new();
    let mut node = 0usize;
    for (i, &c) in text.as_bytes().iter().enumerate() {
        node = goto[node][(c - b'a') as usize] as usize;
        for &pid in &out[node] {
            hits.push((i + 1 - patterns[pid].len(), pid));
        }
    }
    hits.sort_unstable();
    hits
}

fn ac_naive(text: &str, patterns: &[&str]) -> Vec<(usize, usize)> {
    let mut hits = Vec::new();
    for (pid, p) in patterns.iter().enumerate() {
        for i in 0..=text.len() - p.len() {
            if &text[i..i + p.len()] == *p {
                hits.push((i, pid));
            }
        }
    }
    hits.sort_unstable();
    hits
}

pub fn aho_corasick() {
    let patterns = ["he", "she", "his", "hers"];
    let text = "ahishers";
    let (goto, out) = build_aho(&patterns);
    let hits = ac_search(text, &patterns, &goto, &out);
    let naive = ac_naive(text, &patterns);

    println!("patterns: {patterns:?}");
    println!("text = \"{text}\"");
    for (i, pid) in &hits {
        let p = patterns[*pid];
        println!("  \"{p}\" at index {i} (ends at {})", i + p.len());
    }
    println!("matches the naive per-pattern scan: {}", hits == naive);
    println!("one pass over the text: goto[] resolves fail links in O(1) → O(text + matches);");
    println!("{} separate KMP scans would re-read the text {} times — AC reads it once", patterns.len(), patterns.len());
}

// ─── Suffix array ──────────────────────────────────────────────────────────────

/// Prefix doubling: rank suffixes by their first 2^k characters each round.
fn build_suffix_array(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<i32> = s.iter().map(|&c| c as i32).collect();
    let mut tmp = vec![0i32; n];
    let mut k = 1usize;
    loop {
        sa.sort_by_key(|&i| (rank[i], if i + k < n { rank[i + k] } else { -1 }));
        tmp[sa[0]] = 0;
        for w in 1..n {
            let (a, b) = (sa[w - 1], sa[w]);
            let ka = (rank[a], if a + k < n { rank[a + k] } else { -1 });
            let kb = (rank[b], if b + k < n { rank[b + k] } else { -1 });
            tmp[b] = tmp[a] + i32::from(ka < kb);
        }
        std::mem::swap(&mut rank, &mut tmp);
        if rank[sa[n - 1]] == (n - 1) as i32 {
            break; // every rank is distinct — the order is settled
        }
        k *= 2;
    }
    sa
}

/// Kasai: lcp[i] = longest common prefix of suffixes sa[i], sa[i+1] in O(n).
fn kasai(s: &[u8], sa: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut rank = vec![0usize; n];
    for (i, &p) in sa.iter().enumerate() {
        rank[p] = i;
    }
    let mut lcp = vec![0usize; n - 1];
    let mut h = 0usize;
    for i in 0..n {
        let r = rank[i];
        if r == n - 1 {
            h = 0;
            continue;
        }
        let j = sa[r + 1];
        while i + h < n && j + h < n && s[i + h] == s[j + h] {
            h += 1;
        }
        lcp[r] = h;
        if h > 0 {
            h -= 1; // the next suffix is at most one character shorter
        }
    }
    lcp
}

pub fn suffix_array() {
    let text = "banana";
    let s = text.as_bytes();
    let sa = build_suffix_array(s);
    let lcp = kasai(s, &sa);

    let mut naive: Vec<usize> = (0..s.len()).collect();
    naive.sort_by(|&a, &b| s[a..].cmp(&s[b..]));
    let naive_lcp: Vec<usize> = (0..sa.len() - 1)
        .map(|i| {
            let (a, b) = (&text[sa[i]..], &text[sa[i + 1]..]);
            a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
        })
        .collect();

    println!("text = \"{text}\"");
    println!("  rank  sa  suffix          LCP with previous");
    for (r, &i) in sa.iter().enumerate() {
        let l = if r == 0 {
            "-".to_string()
        } else {
            lcp[r - 1].to_string()
        };
        println!("  {r:>4} {i:>4}  {:<15} {l}", &text[i..]);
    }
    println!("matches a plain sort of all suffixes: {}", sa == naive);
    println!("LCP matches direct comparison: {}", lcp == naive_lcp);
    println!("prefix doubling re-ranks (i, i+k) pairs → O(n log² n) here (radix sort → O(n log n), SA-IS → O(n));");
    println!("Kasai's counter never grows twice → O(n); together they power longest repeated");
    println!("substring, substring indexing / search, and the Burrows–Wheeler transform");
}
