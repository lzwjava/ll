//! Sorting: bubble, merge, quick, heap, tim, radix.

use std::time::Instant;

fn demo_data() -> Vec<i32> {
    vec![
        9, -3, 7, 1, 8, 2, 5, 4, 6, 0, 12, -11, 15, 14, 13, 10,
    ]
}

fn report(name: &str, data: &[i32]) {
    let ok = data.windows(2).all(|w| w[0] <= w[1]);
    let head = &data[..data.len().min(20)];
    println!("{name:<30} sorted={ok}");
    println!("  first {:>2}: {head:?}", head.len());
}

/// Deterministic LCG — no `rand` dependency.
pub(crate) struct Lcg(pub u64);

impl Lcg {
    pub(crate) fn next_bounded(&mut self, bound: i32) -> i32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % bound as u64) as i32
    }
}

// ─── Bubble sort ──────────────────────────────────────────────────────────────

pub fn bubble_sort() {
    let mut data = demo_data();
    let n = data.len();
    for i in 0..n {
        let mut swapped = false;
        for j in 0..n - 1 - i {
            if data[j] > data[j + 1] {
                data.swap(j, j + 1);
                swapped = true;
            }
        }
        if !swapped {
            break; // already sorted — early exit
        }
    }
    report("bubble-sort", &data);
}

// ─── Merge sort ───────────────────────────────────────────────────────────────

pub fn merge_sort() {
    let mut data = demo_data();
    let mut buf = data.clone();
    let n = data.len();
    msort(&mut data, &mut buf, 0, n);
    report("merge-sort", &data);
}

fn msort(v: &mut [i32], buf: &mut [i32], lo: usize, hi: usize) {
    if hi - lo < 2 {
        return;
    }
    let mid = lo + (hi - lo) / 2;
    msort(v, buf, lo, mid);
    msort(v, buf, mid, hi);

    let (mut i, mut j, mut k) = (lo, mid, lo);
    while i < mid && j < hi {
        if v[i] <= v[j] {
            buf[k] = v[i];
            i += 1;
        } else {
            buf[k] = v[j];
            j += 1;
        }
        k += 1;
    }
    while i < mid {
        buf[k] = v[i];
        i += 1;
        k += 1;
    }
    while j < hi {
        buf[k] = v[j];
        j += 1;
        k += 1;
    }
    v[lo..hi].copy_from_slice(&buf[lo..hi]);
}

// ─── Quick sort (benchmark) ───────────────────────────────────────────────────

pub fn quick_sort() {
    const N: i32 = 10_000;

    let mut rng = Lcg(42);
    let mut nums: Vec<i32> = (0..N).map(|_| rng.next_bounded(10_000)).collect();

    let head: Vec<i32> = nums.iter().take(10).copied().collect();
    println!("  input : {N} random ints in 0..10000, first 10 = {head:?}");

    let start = Instant::now();
    let len = nums.len() as i32;
    quicksort(&mut nums, 0, len - 1);
    let elapsed = start.elapsed();

    let head: Vec<i32> = nums.iter().take(10).copied().collect();
    let ok = nums.windows(2).all(|w| w[0] <= w[1]);
    println!("  output: first 10 = {head:?}");
    println!(
        "  sorted {N} numbers in {:.3} ms (verified: {ok})",
        elapsed.as_secs_f64() * 1000.0
    );
}

fn quicksort(arr: &mut [i32], low: i32, high: i32) {
    if low < high {
        let p = partition(arr, low, high);
        quicksort(arr, low, p - 1);
        quicksort(arr, p + 1, high);
    }
}

fn partition(arr: &mut [i32], low: i32, high: i32) -> i32 {
    let pivot = arr[high as usize];
    let mut i = low;
    for j in low..high {
        if arr[j as usize] <= pivot {
            arr.swap(i as usize, j as usize);
            i += 1;
        }
    }
    arr.swap(i as usize, high as usize);
    i
}

// ─── Heap sort ────────────────────────────────────────────────────────────────

pub fn heap_sort() {
    let mut data = demo_data();
    let n = data.len();
    // build max-heap: sink every non-leaf node
    for i in (0..n / 2).rev() {
        sink(&mut data, i, n);
    }
    for end in (1..n).rev() {
        data.swap(0, end);
        sink(&mut data, 0, end);
    }
    report("heap-sort", &data);
}

fn sink(v: &mut [i32], mut root: usize, len: usize) {
    loop {
        let l = 2 * root + 1;
        let r = l + 1;
        let mut largest = root;
        if l < len && v[l] > v[largest] {
            largest = l;
        }
        if r < len && v[r] > v[largest] {
            largest = r;
        }
        if largest == root {
            return;
        }
        v.swap(root, largest);
        root = largest;
    }
}

// ─── Tim sort (simplified: natural runs + insertion sort + pairwise merge) ────

const MIN_RUN: usize = 32;

pub fn tim_sort() {
    let mut rng = Lcg(0x5eed);
    let mut data: Vec<i32> = (0..500).map(|_| rng.next_bounded(1000)).collect();
    let n = data.len();

    // 1. Find natural runs; strictly descending runs get reversed.
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < n {
        let mut j = i;
        if j + 1 < n && data[j] > data[j + 1] {
            while j + 1 < n && data[j] > data[j + 1] {
                j += 1;
            }
            data[i..=j].reverse();
        } else {
            while j + 1 < n && data[j] <= data[j + 1] {
                j += 1;
            }
        }
        runs.push((i, j + 1));
        i = j + 1;
    }
    let natural = runs.len();

    // 2. Extend short runs to MIN_RUN with an insertion sort.
    let mut extended: Vec<(usize, usize)> = Vec::new();
    let mut pos = 0;
    for (s, e) in runs {
        if s < pos {
            // overlap with a previous extension: keep only the untouched tail
            if e > pos {
                extended.push((pos, e));
                pos = e;
            }
            continue;
        }
        let mut e = e;
        if e - s < MIN_RUN {
            e = (s + MIN_RUN).min(n);
            insertion_sort(&mut data[s..e]);
        }
        extended.push((s, e));
        pos = e;
    }

    // 3. Pairwise merge adjacent runs until one remains.
    let mut cur = data.clone();
    let mut runs = extended;
    while runs.len() > 1 {
        let mut next = Vec::new();
        let mut merged = Vec::with_capacity(n);
        let mut k = 0;
        while k < runs.len() {
            let start = merged.len();
            if k + 1 < runs.len() {
                let (s1, e1) = runs[k];
                let (s2, e2) = runs[k + 1];
                debug_assert_eq!(e1, s2, "runs must stay contiguous");
                merge_two(&cur[s1..e1], &cur[s2..e2], &mut merged);
                next.push((start, merged.len()));
                k += 2;
            } else {
                let (s, e) = runs[k];
                merged.extend_from_slice(&cur[s..e]);
                next.push((start, merged.len()));
                k += 1;
            }
        }
        cur = merged;
        runs = next;
    }

    if let Some(&(s, e)) = runs.first() {
        data = cur[s..e].to_vec();
    }
    println!("  natural runs found: {natural}, minrun = {MIN_RUN}");
    report("tim-sort (simplified)", &data);
}

fn insertion_sort(v: &mut [i32]) {
    for i in 1..v.len() {
        let mut j = i;
        while j > 0 && v[j] < v[j - 1] {
            v.swap(j, j - 1);
            j -= 1;
        }
    }
}

fn merge_two(a: &[i32], b: &[i32], out: &mut Vec<i32>) {
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            out.push(a[i]);
            i += 1;
        } else {
            out.push(b[j]);
            j += 1;
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
}

// ─── Radix sort (LSD, base 256) ───────────────────────────────────────────────

pub fn radix_sort() {
    let data = demo_data();
    // Map i32 → u32 so unsigned radix order matches signed order:
    // XOR with the sign bit puts i32::MIN at 0 and i32::MAX at u32::MAX.
    let mut u: Vec<u32> = data.iter().map(|&x| (x as u32) ^ 0x8000_0000).collect();
    let mut tmp = vec![0u32; u.len()];
    for shift in [0u32, 8, 16, 24] {
        radix_pass(&u, &mut tmp, shift);
        std::mem::swap(&mut u, &mut tmp);
    }
    let out: Vec<i32> = u.iter().map(|&x| (x ^ 0x8000_0000) as i32).collect();
    report("radix-sort (LSD base 256)", &out);
}

fn radix_pass(v: &[u32], out: &mut [u32], shift: u32) {
    let mut count = [0u32; 256];
    for &x in v {
        count[((x >> shift) & 0xff) as usize] += 1;
    }
    let mut sum = 0u32;
    for c in count.iter_mut() {
        let old = *c;
        *c = sum;
        sum += old;
    }
    for &x in v {
        let b = ((x >> shift) & 0xff) as usize;
        out[count[b] as usize] = x;
        count[b] += 1;
    }
}