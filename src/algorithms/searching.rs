//! Searching: linear search, binary search, BFS, DFS, IDA* (8-puzzle).

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

// ─── IDA* (iterative-deepening A* on the 8-puzzle) ───────────────────────────

const GOAL: u32 = 123_456_780; // 1 2 3 / 4 5 6 / 7 8 _

fn decode(mut s: u32) -> [u8; 9] {
    let mut out = [0u8; 9];
    for i in (0..9).rev() {
        out[i] = (s % 10) as u8;
        s /= 10;
    }
    out
}

fn encode(d: &[u8; 9]) -> u32 {
    d.iter().fold(0u32, |acc, &x| acc * 10 + x as u32)
}

/// Blank moves: (new blank index, direction) — direction 0=up 1=down 2=left
/// 3=right, so `dir ^ 1` is the way back.
fn neighbours(b: usize) -> Vec<(usize, u8)> {
    let (r, c) = (b / 3, b % 3);
    let mut v = Vec::new();
    if r > 0 {
        v.push((b - 3, 0));
    }
    if r < 2 {
        v.push((b + 3, 1));
    }
    if c > 0 {
        v.push((b - 1, 2));
    }
    if c < 2 {
        v.push((b + 1, 3));
    }
    v
}

/// Admissible heuristic: summed Manhattan distance of every tile.
fn manhattan(s: u32) -> i32 {
    let d = decode(s);
    let mut h = 0;
    for (i, &tile) in d.iter().enumerate() {
        if tile == 0 {
            continue;
        }
        let goal = tile as usize - 1; // tile k belongs at index k−1
        let (gr, gc) = ((goal % 3) as i32, (goal / 3) as i32);
        h += ((i % 3) as i32 - gr).abs() + ((i / 3) as i32 - gc).abs();
    }
    h as i32
}

fn ida_search(
    path: &mut Vec<u32>,
    moves: &mut Vec<u8>,
    g: i32,
    bound: i32,
    exceeded: &mut i32,
    nodes: &mut usize,
) -> bool {
    *nodes += 1;
    let state = *path.last().unwrap();
    let f = g + manhattan(state);
    if f > bound {
        *exceeded = (*exceeded).min(f); // next threshold
        return false;
    }
    if state == GOAL {
        return true;
    }
    let mut d = decode(state);
    let blank = d.iter().position(|&x| x == 0).unwrap();
    for (nb, dir) in neighbours(blank) {
        if let Some(&last) = moves.last() {
            if last == dir ^ 1 {
                continue; // don't undo the move we just made
            }
        }
        d.swap(blank, nb);
        path.push(encode(&d));
        moves.push(dir);
        if ida_search(path, moves, g + 1, bound, exceeded, nodes) {
            return true;
        }
        d.swap(blank, nb); // undo
        path.pop();
        moves.pop();
    }
    false
}

fn shortest_len_bfs(start: u32) -> Option<usize> {
    use std::collections::{HashSet, VecDeque};
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert(start);
    q.push_back((start, 0usize));
    while let Some((s, dist)) = q.pop_front() {
        if s == GOAL {
            return Some(dist);
        }
        let mut d = decode(s);
        let blank = d.iter().position(|&x| x == 0).unwrap();
        for (nb, _) in neighbours(blank) {
            d.swap(blank, nb);
            let next = encode(&d);
            d.swap(blank, nb);
            if seen.insert(next) {
                q.push_back((next, dist + 1));
            }
        }
    }
    None
}

pub fn ida_star() {
    let start = 876_543_210u32; // 8 7 6 / 5 4 3 / 2 1 _
    let h0 = manhattan(start);
    println!("start = {:?}   goal = {:?}", decode(start), decode(GOAL));
    println!("h(start) = {h0} (Manhattan distance)");

    let mut bound = h0;
    let mut moves: Vec<u8> = Vec::new();
    let mut nodes = 0usize;
    let solution = loop {
        let mut path = vec![start];
        let mut exceeded = i32::MAX;
        if ida_search(&mut path, &mut moves, 0, bound, &mut exceeded, &mut nodes) {
            break Some(moves.clone());
        }
        if exceeded == i32::MAX {
            break None;
        }
        bound = exceeded;
        moves.clear();
    };

    let dirs = ['↑', '↓', '←', '→'];
    match solution {
        Some(mv) => {
            println!("IDA* solved it: {} moves, {nodes} nodes expanded", mv.len());
            println!("  moves: {}", mv.iter().map(|&d| dirs[d as usize]).collect::<String>());
            // replay the move list from scratch
            let mut s = start;
            for &d in &mv {
                let mut cur = decode(s);
                let blank = cur.iter().position(|&x| x == 0).unwrap();
                let (nb, _) = neighbours(blank)
                    .into_iter()
                    .find(|&(_, dir)| dir == d)
                    .unwrap();
                cur.swap(blank, nb);
                s = encode(&cur);
            }
            println!("  replaying the moves reaches the goal: {}", s == GOAL);
            let opt = shortest_len_bfs(start);
            println!("  BFS optimum = {opt:?} → optimal: {}", opt == Some(mv.len()));
        }
        None => println!("no solution (unsolvable parity)"),
    }
    println!("IDA* = depth-first search with a cost bound, deepened only when f > bound;");
    println!("memory O(depth) instead of BFS's O(b^d), while an admissible h (Manhattan)");
    println!("prunes hopeless branches — the standard technique for 15-puzzle & SAT-style searches");
}