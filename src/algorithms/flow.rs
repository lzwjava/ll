//! Network flow: Edmonds-Karp max flow / min cut, bipartite matching (Kuhn).

use std::collections::VecDeque;

// ─── Max flow (Edmonds-Karp) ──────────────────────────────────────────────────

const MF_NODES: usize = 6; // node 0 = source, node 5 = sink
const MF_CAP: &[(usize, usize, i64)] = &[
    (0, 1, 16),
    (0, 2, 13),
    (1, 3, 12),
    (2, 1, 4),
    (2, 4, 14),
    (3, 2, 9),
    (3, 5, 20),
    (4, 3, 7),
    (4, 5, 4),
];

pub fn max_flow() {
    let n = MF_NODES;
    let (s, t) = (0usize, 5usize);
    let mut cap = vec![vec![0i64; n]; n];
    for &(u, v, c) in MF_CAP {
        cap[u][v] += c;
    }
    let mut res = cap.clone(); // residual network

    let mut total = 0i64;
    let mut round = 0usize;
    loop {
        // BFS → the *shortest* augmenting path (that's the Edmonds-Karp bit).
        let mut prev = vec![usize::MAX; n];
        let mut q = VecDeque::new();
        prev[s] = s;
        q.push_back(s);
        while let Some(u) = q.pop_front() {
            for v in 0..n {
                if res[u][v] > 0 && prev[v] == usize::MAX {
                    prev[v] = u;
                    q.push_back(v);
                }
            }
        }
        if prev[t] == usize::MAX {
            break; // sink unreachable → flow is maximum
        }
        let mut aug = i64::MAX;
        let mut path = vec![t];
        let mut v = t;
        while v != s {
            let u = prev[v];
            aug = aug.min(res[u][v]);
            path.push(u);
            v = u;
        }
        path.reverse();
        let mut v = t;
        while v != s {
            let u = prev[v];
            res[u][v] -= aug;
            res[v][u] += aug; // pushes flow backwards for free later
            v = u;
        }
        total += aug;
        round += 1;
        println!("  round {round:>2}: {path:?} pushes {aug:>2} → total {total}");
    }

    let net: Vec<Vec<i64>> = (0..n)
        .map(|u| (0..n).map(|v| cap[u][v] - res[u][v]).collect())
        .collect();

    // 1. capacity: net flow stays within [−cap(v,u), +cap(u,v)] on every pair
    let cap_ok = (0..n).all(|u| (0..n).all(|v| net[u][v] <= cap[u][v] && net[u][v] >= -cap[v][u]));
    // 2. conservation: in = out everywhere except source/sink
    let inflow = |v: usize| (0..n).map(|u| net[u][v]).sum::<i64>();
    let cons_ok = (0..n).all(|u| u == s || u == t || inflow(u) == 0)
        && inflow(s) == -total
        && inflow(t) == total;
    // 3. independent check: enumerate every s/t cut and take the cheapest
    let mut min_cut = i64::MAX;
    let mut cut_count = 0usize;
    for mask in 0..(1u32 << (n - 2)) {
        let mut in_s = vec![false; n];
        in_s[s] = true;
        for i in 0..n - 2 {
            in_s[1 + i] = mask & (1 << i) != 0;
        }
        cut_count += 1;
        let cut: i64 = (0..n)
            .flat_map(|u| (0..n).map(move |v| (u, v)))
            .filter(|&(u, v)| in_s[u] && !in_s[v])
            .map(|(u, v)| cap[u][v])
            .sum();
        min_cut = min_cut.min(cut);
    }

    println!("max flow = {total}");
    println!("no arc carries more than its capacity: {cap_ok}");
    println!("flow conserved at all intermediate nodes; source −{total} / sink +{total}: {cons_ok}");
    println!(
        "brute-force minimum s–t cut over all {cut_count} cuts = {min_cut} → max-flow = min-cut: {}",
        min_cut == total
    );
    println!("BFS augmenting paths → O(V·E²); Dinic's level+blocking flow scheme → O(V²E)");
}

// ─── Bipartite matching (Kuhn / Hungarian DFS) ────────────────────────────────

const BM_LEFT: usize = 5;
const BM_RIGHT: usize = 6;
const BM_EDGES: &[(usize, usize)] = &[
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 2),
    (2, 3),
    (2, 4),
    (3, 0),
    (3, 4),
    (4, 2),
    (4, 5),
];

fn try_kuhn(u: usize, adj: &[Vec<usize>], seen: &mut [bool], match_r: &mut [usize]) -> bool {
    for &v in &adj[u] {
        if seen[v] {
            continue;
        }
        seen[v] = true;
        if match_r[v] == usize::MAX || try_kuhn(match_r[v], adj, seen, match_r) {
            match_r[v] = u; // free v, possibly by re-routing its current match
            return true;
        }
    }
    false
}

/// Independent optimum: try every assignment of the remaining left vertices.
fn brute_matching(u: usize, used: u32, adj: &[Vec<usize>]) -> usize {
    if u == adj.len() {
        return 0;
    }
    let mut best = brute_matching(u + 1, used, adj);
    for &v in &adj[u] {
        if used & (1 << v) == 0 {
            best = best.max(1 + brute_matching(u + 1, used | (1 << v), adj));
        }
    }
    best
}

pub fn bipartite_matching() {
    let mut adj = vec![Vec::new(); BM_LEFT];
    for &(u, v) in BM_EDGES {
        adj[u].push(v);
    }

    // Kuhn: try to grow the matching by one augmenting path per left vertex.
    let mut match_r = vec![usize::MAX; BM_RIGHT];
    let mut order: Vec<usize> = (0..BM_LEFT).collect();
    order.sort_by_key(|&u| adj[u].len()); // cheapest vertex first
    let mut size = 0usize;
    for u in order {
        let mut seen = vec![false; BM_RIGHT];
        if try_kuhn(u, &adj, &mut seen, &mut match_r) {
            size += 1;
        }
    }

    let mut pairs: Vec<(usize, usize)> = match_r
        .iter()
        .enumerate()
        .filter(|&(_, &l)| l != usize::MAX)
        .map(|(r, &l)| (l, r))
        .collect();
    pairs.sort_unstable();

    let edges_ok = pairs.iter().all(|&(l, r)| BM_EDGES.contains(&(l, r)));
    let left_ok = pairs.iter().map(|&(l, _)| l).collect::<std::collections::HashSet<_>>().len() == pairs.len();
    let right_ok = pairs.iter().map(|&(_, r)| r).collect::<std::collections::HashSet<_>>().len() == pairs.len();
    let best = brute_matching(0, 0, &adj);

    println!("bipartite graph: {BM_LEFT} left × {BM_RIGHT} right, {} edges", BM_EDGES.len());
    println!("left adjacency: {adj:?}");
    println!("Kuhn matching ({size} pairs): {pairs:?}");
    println!("every pair is an input edge: {edges_ok}, no vertex reused: {}", left_ok && right_ok);
    println!("brute-force optimum = {best} → optimal: {}", size == best);
    println!("one DFS attempt per left vertex, each O(E) → O(V·E);");
    println!("Hopcroft–Karp batches the shortest augmenting paths → O(E√V);");
    println!("unit-capacity bipartite matching *is* a max-flow instance — same augmentation idea");
}
