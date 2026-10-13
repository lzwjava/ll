//! Array techniques: two pointers / sliding window, prefix sums & difference
//! arrays, monotonic stacks.

// ─── Two pointers / sliding window ───────────────────────────────────────────

pub fn two_pointers() {
    // (a) every pair summing to `target` in a sorted array
    let arr = [1, 2, 3, 4, 5, 7, 9, 11, 13, 15];
    let target = 16;
    let pairs = pair_sum(&arr, target);
    let mut brute: Vec<(i32, i32)> = Vec::new();
    for i in 0..arr.len() {
        for j in i + 1..arr.len() {
            if arr[i] + arr[j] == target {
                brute.push((arr[i], arr[j]));
            }
        }
    }
    let pairs_ok = pairs == brute;

    // (b) longest window with sum ≤ k (positive data ⇒ shrinking window works)
    let data = [2, 1, 5, 1, 3, 2, 6, 4];
    let k = 9i64;
    let (bs, be, bsum) = window_at_most(&data, k);
    let mut want_len = 0usize;
    for i in 0..data.len() {
        let mut s = 0i64;
        for j in i..data.len() {
            s += data[j];
            if s <= k {
                want_len = want_len.max(j - i + 1);
            }
        }
    }
    let win_ok = be - bs == want_len;

    println!("sorted array = {arr:?}, target = {target}");
    println!("  pairs: {pairs:?}");
    println!("  matches O(n²) brute force: {pairs_ok}");
    println!("data = {data:?}, k = {k}");
    println!(
        "  longest window summing ≤ k: data[{bs}..{be}] = {:?} (sum {bsum})",
        &data[bs..be]
    );
    println!("  brute-force max window length = {want_len}, matches: {win_ok}");
    println!("each index enters and leaves the window once → O(n) time, O(1) extra space");
}

/// Sorted input: advance the small end up when the sum is too small, the big
/// end down when it is too large — never revisit an index.
fn pair_sum(arr: &[i32; 10], target: i32) -> Vec<(i32, i32)> {
    let (mut lo, mut hi) = (0usize, arr.len() - 1);
    let mut out = Vec::new();
    while lo < hi {
        let s = arr[lo] + arr[hi];
        if s == target {
            out.push((arr[lo], arr[hi]));
            lo += 1;
            hi -= 1;
        } else if s < target {
            lo += 1;
        } else {
            hi -= 1;
        }
    }
    out
}

/// Longest `[lo, hi]` with sum ≤ k; returns `(lo, hi_exclusive, sum)`.
fn window_at_most(a: &[i64], k: i64) -> (usize, usize, i64) {
    let (mut lo, mut sum) = (0usize, 0i64);
    let (mut bs, mut be, mut bsum) = (0usize, 0usize, 0i64);
    for hi in 0..a.len() {
        sum += a[hi];
        while sum > k {
            sum -= a[lo];
            lo += 1;
        }
        if hi + 1 - lo > be - bs {
            (bs, be, bsum) = (lo, hi + 1, sum);
        }
    }
    (bs, be, bsum)
}

// ─── Prefix sums & difference arrays ─────────────────────────────────────────

pub fn prefix_sums() {
    let a = [3i64, 1, 4, 1, 5, 9, 2, 6];
    let queries = [(0, 4), (2, 8), (3, 4), (1, 7)]; // half-open [l, r)

    let mut pre = vec![0i64; a.len() + 1];
    for (i, &v) in a.iter().enumerate() {
        pre[i + 1] = pre[i] + v;
    }

    println!("array = {a:?}");
    let mut all_ok = true;
    for &(l, r) in &queries {
        let got = pre[r] - pre[l];
        let want: i64 = a[l..r].iter().sum();
        all_ok &= got == want;
        println!("  sum[{l}..{r}) = {got:>2}   (slice sum = {want:>2})");
    }
    println!("all {} queries match slice sums: {all_ok}", queries.len());

    // Difference array: O(1) range updates, materialized once at the end.
    let updates = [(0, 5, 10), (2, 8, -3), (3, 4, 7)];
    let mut naive = vec![0i64; a.len()];
    for &(l, r, d) in &updates {
        for x in &mut naive[l..r] {
            *x += d;
        }
    }
    let mut diff = vec![0i64; a.len() + 1];
    for &(l, r, d) in &updates {
        diff[l] += d;
        diff[r] -= d;
    }
    let mut built = vec![0i64; a.len()];
    let mut acc = 0i64;
    for i in 0..a.len() {
        acc += diff[i];
        built[i] = acc;
    }
    println!("range updates (l, r, delta): {updates:?}");
    println!("  difference array → {built:?}");
    println!("  matches per-element updates: {}", built == naive);
    println!("O(n) build, O(1) range query; range updates O(1) each (O(n) to materialize)");
}

// ─── Monotonic stack ─────────────────────────────────────────────────────────

pub fn monotonic_stack() {
    let arr = [2i64, 1, 5, 3, 4, 6, 2, 3];
    let nxt = next_greater(&arr);
    let mut ok = true;
    for i in 0..arr.len() {
        let want = (i + 1..arr.len()).find(|&j| arr[j] > arr[i]);
        ok &= want == nxt[i];
    }
    println!("array = {arr:?}");
    for (i, &j) in nxt.iter().enumerate() {
        match j {
            Some(j) => println!("  {i}: {} → next greater {j}: {}", arr[i], arr[j]),
            None => println!("  {i}: {} → none", arr[i]),
        }
    }
    println!("all next-greater answers match brute force: {ok}");

    let h = [2i64, 1, 5, 6, 2, 3];
    let area = largest_rectangle(&h);
    let mut want = 0i64;
    for i in 0..h.len() {
        let mut mn = h[i];
        for j in i..h.len() {
            mn = mn.min(h[j]);
            want = want.max(mn * (j - i + 1) as i64);
        }
    }
    println!("histogram heights = {h:?}");
    println!("largest rectangle = {area} (brute force = {want}, match: {})", area == want);
    println!("a monotonic stack pops each index once → O(n) time, O(n) space");
}

/// Indices with decreasing values; everything popped has found its next greater.
fn next_greater(a: &[i64]) -> Vec<Option<usize>> {
    let mut res = vec![None; a.len()];
    let mut st: Vec<usize> = Vec::new();
    for i in 0..a.len() {
        while let Some(&top) = st.last()
            && a[top] < a[i]
        {
            res[top] = Some(i);
            st.pop();
        }
        st.push(i);
    }
    res
}

/// Largest rectangle of aligned bars: extend each bar as far as its height allows.
fn largest_rectangle(h: &[i64]) -> i64 {
    let mut bars = h.to_vec();
    bars.push(0); // sentinel flushes the remaining stack
    let mut best = 0i64;
    let mut st: Vec<usize> = Vec::new();
    for i in 0..bars.len() {
        while let Some(&top) = st.last()
            && bars[top] > bars[i]
        {
            st.pop();
            let height = bars[top];
            let width = if st.is_empty() { i } else { i - *st.last().unwrap() - 1 };
            best = best.max(height * width as i64);
        }
        st.push(i);
    }
    best
}