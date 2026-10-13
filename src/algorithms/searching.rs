//! Searching: linear search, binary search, BFS, DFS.

use std::cmp::Ordering;
use std::collections::VecDeque;

pub fn linear_search() {
    let arr = [7, 3, 9, 1, 5, 8, 2];
    let target = 5;
    println!("array = {arr:?}");
    let idx = arr.iter().position(|&x| x == target);
    println!("linear_search({target}) -> {idx:?}");
    if let Some(i) = idx {
        println!("  arr[{i}] = {}", arr[i]);
    }
}

pub fn binary_search() {
    let arr = [1, 3, 5, 7, 9, 11, 13, 15];
    println!("array = {arr:?}  (sorted precondition)");
    for target in [7, 1, 16] {
        println!("binary_search({target:>2}) -> {:?}", binary(&arr, target));
    }
}

// ─── Binary search on the answer (parametric search) ─────────────────────────

pub fn binary_search_on_answer() {
    // "Ship all packages within `days` days — what is the smallest hull capacity?"
    let weights = [7i64, 3, 1, 5, 4, 8, 2, 6];
    let days = 4usize;

    let checks = std::cell::Cell::new(0usize);
    let feasible = |cap: i64| {
        checks.set(checks.get() + 1);
        let (mut used, mut cur) = (1usize, 0i64);
        for &w in &weights {
            if cur + w > cap {
                used += 1;
                cur = 0;
            }
            cur += w;
        }
        used <= days
    };

    let lo = *weights.iter().max().unwrap(); // must fit the heaviest package
    let hi = weights.iter().sum(); // one ship takes everything
    let (mut l, mut r) = (lo, hi);
    while l < r {
        let mid = l + (r - l) / 2;
        if feasible(mid) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    let found = l;
    let brute = (lo..=hi).find(|&cap| feasible(cap)).unwrap();

    println!("package weights = {weights:?}, deadline = {days} days");
    println!("search range = [{lo}, {hi}]  (max package … total weight)");
    println!("smallest feasible capacity = {found}");
    println!("linear scan over the same range agrees: {}", found == brute);
    println!(
        "{} predicate checks instead of {} — monotone predicate turns \"find min feasible\" into O(log R)",
        checks.get(),
        hi - lo + 1
    );
}

fn binary<T: Ord>(arr: &[T], target: T) -> Option<usize> {
    let (mut lo, mut hi) = (0usize, arr.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match arr[mid].cmp(&target) {
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
            Ordering::Equal => return Some(mid),
        }
    }
    None
}

/// Undirected demo graph (adjacency lists):
///
/// ```text
///  0 — 1 — 3 — 4 — 5
///  |       |
///  2 ——————+
/// ```
fn demo_graph() -> Vec<Vec<usize>> {
    vec![
        vec![1, 2],
        vec![0, 2, 3],
        vec![0, 1, 3],
        vec![1, 2, 4],
        vec![3, 5],
        vec![4],
    ]
}

pub fn bfs() {
    let g = demo_graph();
    println!("graph (adjacency) = {g:?}");
    println!("BFS order from 0   = {:?}", bfs_order(&g, 0));
    println!(
        "shortest path 0→5  = {:?}  (BFS level parents)",
        shortest_path(&g, 0, 5)
    );
}

fn bfs_order(g: &[Vec<usize>], start: usize) -> Vec<usize> {
    let mut seen = vec![false; g.len()];
    let mut queue = VecDeque::new();
    let mut out = Vec::new();
    seen[start] = true;
    queue.push_back(start);
    while let Some(u) = queue.pop_front() {
        out.push(u);
        for &v in &g[u] {
            if !seen[v] {
                seen[v] = true;
                queue.push_back(v);
            }
        }
    }
    out
}

fn shortest_path(g: &[Vec<usize>], s: usize, t: usize) -> Option<Vec<usize>> {
    let mut prev = vec![usize::MAX; g.len()];
    let mut seen = vec![false; g.len()];
    let mut queue = VecDeque::new();
    seen[s] = true;
    queue.push_back(s);
    while let Some(u) = queue.pop_front() {
        if u == t {
            break;
        }
        for &v in &g[u] {
            if !seen[v] {
                seen[v] = true;
                prev[v] = u;
                queue.push_back(v);
            }
        }
    }
    if !seen[t] {
        return None;
    }
    let mut path = vec![t];
    let mut cur = t;
    while cur != s {
        cur = prev[cur];
        path.push(cur);
    }
    path.reverse();
    Some(path)
}

pub fn dfs() {
    let g = demo_graph();
    println!("graph (adjacency) = {g:?}");
    let mut seen = vec![false; g.len()];
    let mut order = Vec::new();
    dfs_rec(&g, 0, &mut seen, &mut order);
    println!("DFS order from 0   = {order:?}");
}

fn dfs_rec(g: &[Vec<usize>], u: usize, seen: &mut [bool], out: &mut Vec<usize>) {
    seen[u] = true;
    out.push(u);
    for &v in &g[u] {
        if !seen[v] {
            dfs_rec(g, v, seen, out);
        }
    }
}