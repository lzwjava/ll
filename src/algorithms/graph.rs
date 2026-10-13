//! Graph algorithms: Dijkstra, Bellman-Ford, Floyd-Warshall, Kruskal, Prim,
//! topological sort (Kahn), A* grid pathfinding.

use super::trees::Dsu;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const NODES: usize = 6;

/// Undirected weighted edges: (from, to, weight).
const EDGES: &[(usize, usize, i64)] = &[
    (0, 1, 4),
    (0, 2, 1),
    (1, 2, 2),
    (1, 3, 5),
    (2, 3, 8),
    (2, 4, 10),
    (3, 4, 2),
    (3, 5, 6),
    (4, 5, 1),
];

/// Directed edges (contains a negative weight for Bellman-Ford).
const DIR_EDGES: &[(usize, usize, i64)] = &[
    (0, 1, 4),
    (0, 2, 1),
    (1, 3, 5),
    (2, 1, -2),
    (2, 4, 10),
    (3, 4, 2),
    (3, 5, -1),
    (4, 5, 1),
];

fn undirected_adj() -> Vec<Vec<(usize, i64)>> {
    let mut g = vec![Vec::new(); NODES];
    for &(u, v, w) in EDGES {
        g[u].push((v, w));
        g[v].push((u, w));
    }
    g
}

fn print_dist(label: &str, dist: &[i64]) {
    println!("{label}");
    for (i, &d) in dist.iter().enumerate() {
        let d = if d == i64::MAX { "∞".to_string() } else { d.to_string() };
        println!("  from 0 → node {i} = {d:>4}");
    }
}

// ─── Dijkstra ─────────────────────────────────────────────────────────────────

pub fn dijkstra() {
    let g = undirected_adj();
    println!("graph: {NODES} nodes, {} undirected weighted edges", EDGES.len());
    let dist = dijkstra_impl(&g, 0);
    print_dist("dijkstra single-source distances:", &dist);
}

fn dijkstra_impl(g: &[Vec<(usize, i64)>], s: usize) -> Vec<i64> {
    let mut dist = vec![i64::MAX; g.len()];
    let mut heap = BinaryHeap::new();
    dist[s] = 0;
    heap.push(Reverse((0i64, s)));
    while let Some(Reverse((d, u))) = heap.pop() {
        if d > dist[u] {
            continue; // stale entry
        }
        for &(v, w) in &g[u] {
            let nd = d + w;
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(Reverse((nd, v)));
            }
        }
    }
    dist
}

// ─── Bellman-Ford ─────────────────────────────────────────────────────────────

pub fn bellman_ford() {
    let n = NODES;
    println!("graph: {n} nodes, {} directed edges (one negative: 2→1 = -2)", DIR_EDGES.len());
    let mut dist = vec![i64::MAX; n];
    dist[0] = 0;

    for _ in 0..n - 1 {
        let mut updated = false;
        for &(u, v, w) in DIR_EDGES {
            if dist[u] != i64::MAX && dist[u] + w < dist[v] {
                dist[v] = dist[u] + w;
                updated = true;
            }
        }
        if !updated {
            break;
        }
    }

    // One more relaxation round → negative cycle detection.
    let neg_cycle = DIR_EDGES
        .iter()
        .any(|&(u, v, w)| dist[u] != i64::MAX && dist[u] + w < dist[v]);

    print_dist("bellman-ford distances:", &dist);
    println!("  negative cycle detected: {neg_cycle}");
}

// ─── Floyd-Warshall ───────────────────────────────────────────────────────────

pub fn floyd_warshall() {
    let n = NODES;
    let mut d = vec![vec![i64::MAX; n]; n];
    for i in 0..n {
        d[i][i] = 0;
    }
    for &(u, v, w) in DIR_EDGES {
        if w < d[u][v] {
            d[u][v] = w;
        }
    }
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if d[i][k] != i64::MAX && d[k][j] != i64::MAX {
                    let nd = d[i][k] + d[k][j];
                    if nd < d[i][j] {
                        d[i][j] = nd;
                    }
                }
            }
        }
    }
    println!("all-pairs shortest paths (row = src, col = dst):");
    print!("     ");
    for j in 0..n {
        print!("{j:>4}");
    }
    println!();
    for (i, row) in d.iter().enumerate() {
        print!("{i:>4} ");
        for &v in row {
            if v == i64::MAX {
                print!("   ∞");
            } else {
                print!("{v:>4}");
            }
        }
        println!();
    }
}

// ─── Kruskal ──────────────────────────────────────────────────────────────────

pub fn kruskal() {
    let mut edges: Vec<(i64, usize, usize)> =
        EDGES.iter().map(|&(u, v, w)| (w, u, v)).collect();
    edges.sort_unstable();

    let mut dsu = Dsu::new(NODES);
    let mut total = 0i64;
    let mut picked: Vec<(usize, usize, i64)> = Vec::new();
    for (w, u, v) in edges {
        if dsu.union(u, v) {
            total += w;
            picked.push((u, v, w));
        }
    }
    println!("edges sorted by weight, greedy accept if no cycle:");
    for (u, v, w) in &picked {
        println!("  {u} — {v}  (w = {w})");
    }
    println!("MST weight = {total}  ({} edges for {NODES} nodes)", picked.len());
}

// ─── Prim ─────────────────────────────────────────────────────────────────────

pub fn prim() {
    let g = undirected_adj();
    let mut in_mst = vec![false; NODES];
    let mut heap = BinaryHeap::new();
    let mut total = 0i64;
    let mut picked: Vec<(usize, usize, i64)> = Vec::new();

    in_mst[0] = true;
    for &(v, w) in &g[0] {
        heap.push(Reverse((w, v, 0usize)));
    }

    while let Some(Reverse((w, v, from))) = heap.pop() {
        if in_mst[v] {
            continue;
        }
        in_mst[v] = true;
        total += w;
        picked.push((from, v, w));
        for &(to, wt) in &g[v] {
            if !in_mst[to] {
                heap.push(Reverse((wt, to, v)));
            }
        }
    }

    println!("grow MST from node 0, always taking the cheapest crossing edge:");
    for (u, v, w) in &picked {
        println!("  {u} — {v}  (w = {w})");
    }
    println!("MST weight = {total}  ({} edges)", picked.len());
}

// ─── Topological sort (Kahn) ──────────────────────────────────────────────────

pub fn topological_sort() {
    // DAG: 0→1, 0→2, 1→3, 2→3, 2→4, 3→5, 4→5
    let dag: Vec<Vec<usize>> = vec![
        vec![1, 2],
        vec![3],
        vec![3, 4],
        vec![5],
        vec![5],
        vec![],
    ];
    let n = dag.len();
    let mut indeg = vec![0usize; n];
    for outs in &dag {
        for &v in outs {
            indeg[v] += 1;
        }
    }

    let mut queue: Vec<usize> = (0..n).filter(|&v| indeg[v] == 0).collect();
    let mut order = Vec::new();
    while let Some(u) = queue.pop() {
        order.push(u);
        for &v in &dag[u] {
            indeg[v] -= 1;
            if indeg[v] == 0 {
                queue.push(v);
            }
        }
    }

    if order.len() == n {
        println!("DAG adjacency = {dag:?}");
        println!("topological order = {order:?}");
    } else {
        println!("cycle detected — topological sort impossible");
    }
}

// ─── A* ───────────────────────────────────────────────────────────────────────

pub fn a_star() {
    let h = 8;
    let w = 12;
    // false = free cell, true = wall
    let mut grid = vec![vec![false; w]; h];
    for y in 0..h {
        if !(2..=5).contains(&y) {
            grid[y][6] = true; // wall with a gap at rows 2..=5
        }
    }
    let start = (0usize, 0usize);
    let goal = (7usize, 11usize);

    println!("grid 8×12 with a wall at column 6 (gap at rows 2-5):");
    for (y, row) in grid.iter().enumerate() {
        let mut line: String = row.iter().map(|&c| if c { '#' } else { '.' }).collect();
        if y == start.0 {
            line.replace_range(start.1..start.1 + 1, "S");
        }
        if y == goal.0 {
            line.replace_range(goal.1..goal.1 + 1, "E");
        }
        println!("  {line}");
    }

    match astar_impl(&grid, start, goal) {
        Some(path) => {
            println!("path found: {} cells, cost = {}", path.len(), path.len() - 1);
            println!("  {path:?}");
        }
        None => println!("no path"),
    }
}

fn astar_impl(
    grid: &[Vec<bool>],
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    let h = grid.len();
    let w = grid[0].len();
    let idx = |p: (usize, usize)| p.0 * w + p.1;

    let heuristic = |p: (usize, usize)| (p.0.abs_diff(goal.0) + p.1.abs_diff(goal.1)) as i64;

    let mut gscore = vec![i64::MAX; h * w];
    let mut prev: Vec<Option<usize>> = vec![None; h * w];
    let mut open: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();

    gscore[idx(start)] = 0;
    open.push(Reverse((heuristic(start), idx(start))));

    const DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some(Reverse((_, u))) = open.pop() {
        if u == idx(goal) {
            let mut path = Vec::new();
            let mut cur = Some(u);
            while let Some(c) = cur {
                path.push((c / w, c % w));
                cur = prev[c];
            }
            path.reverse();
            return Some(path);
        }
        let (uy, ux) = (u / w, u % w);
        for (dy, dx) in DIRS {
            let (vy, vx) = (uy as i32 + dy, ux as i32 + dx);
            if vy < 0 || vx < 0 || vy as usize >= h || vx as usize >= w {
                continue;
            }
            let (vy, vx) = (vy as usize, vx as usize);
            if grid[vy][vx] {
                continue;
            }
            let v = vy * w + vx;
            let tentative = gscore[u] + 1;
            if tentative < gscore[v] {
                gscore[v] = tentative;
                prev[v] = Some(u);
                open.push(Reverse((tentative + heuristic((vy, vx)), v)));
            }
        }
    }
    None
}

// ─── 0/1 BFS ─────────────────────────────────────────────────────────────────

/// Directed graph whose edge weights are only 0 or 1.
const ZERO_ONE_EDGES: &[(usize, usize, i64)] = &[
    (0, 1, 0),
    (0, 2, 1),
    (1, 3, 1),
    (2, 3, 0),
    (1, 4, 1),
    (2, 4, 1),
    (3, 4, 0),
    (3, 5, 1),
    (4, 5, 0),
];

fn directed_adj(weights: &[(usize, usize, i64)], n: usize) -> Vec<Vec<(usize, i64)>> {
    let mut g = vec![Vec::new(); n];
    for &(u, v, w) in weights {
        g[u].push((v, w));
    }
    g
}

pub fn zero_one_bfs() {
    let g = directed_adj(&ZERO_ONE_EDGES, 6);
    let fast = zero_one_impl(&g, 0);
    let djk = dijkstra_impl(&g, 0);

    println!("directed graph with 0/1 weights: {ZERO_ONE_EDGES:?}");
    println!("  node | 0/1 BFS | Dijkstra");
    for i in 0..g.len() {
        println!("  {i:>4} | {:>7} | {:>8}", fast[i], djk[i]);
    }
    println!("identical distances: {}", fast == djk);
    println!("a 0-weight edge goes to the FRONT of the deque, so the popped node always");
    println!("has the smallest tentative distance → bucket-queue behaviour in O(V+E)");
    println!("(Dijkstra needs a heap: O((V+E) log V))");
}

fn zero_one_impl(g: &[Vec<(usize, i64)>], s: usize) -> Vec<i64> {
    let mut dist = vec![i64::MAX; g.len()];
    let mut dq = std::collections::VecDeque::new();
    dist[s] = 0;
    dq.push_back(s);
    while let Some(u) = dq.pop_front() {
        for &(v, w) in &g[u] {
            if dist[u] + w < dist[v] {
                dist[v] = dist[u] + w;
                if w == 0 {
                    dq.push_front(v); // same distance class — process next
                } else {
                    dq.push_back(v); // one class worse
                }
            }
        }
    }
    dist
}

// ─── Tarjan strongly connected components ───────────────────────────────────

const SCC_EDGES: &[(usize, usize)] = &[
    (0, 1),
    (1, 2),
    (2, 0), // cycle A
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 3), // cycle B
    (5, 6), // tail
];

pub fn tarjan_scc() {
    const N: usize = 7;
    let mut g = vec![Vec::new(); N];
    for &(u, v) in SCC_EDGES {
        g[u].push(v);
    }

    let mut comps = tarjan(&g);
    for c in &mut comps {
        c.sort_unstable();
    }
    comps.sort_by_key(|c| c[0]);

    // component id per node
    let mut owner = vec![usize::MAX; N];
    for (id, c) in comps.iter().enumerate() {
        for &v in c {
            owner[v] = id;
        }
    }

    // transitive closure → u and v share an SCC iff they reach each other
    let mut reach = vec![vec![false; N]; N];
    for u in 0..N {
        let mut st = vec![u];
        reach[u][u] = true;
        while let Some(x) = st.pop() {
            for &y in &g[x] {
                if !reach[u][y] {
                    reach[u][y] = true;
                    st.push(y);
                }
            }
        }
    }
    let ok = (0..N).all(|u| {
        (0..N).all(|v| (reach[u][v] && reach[v][u]) == (owner[u] == owner[v]))
    });

    println!("directed edges: {SCC_EDGES:?}");
    println!("strongly connected components:");
    for (id, c) in comps.iter().enumerate() {
        println!("  scc {id}: {c:?}");
    }
    println!("mutual-reachability ⇔ same component for every pair: {ok}");
    println!("one DFS maintaining low-link values → O(V+E);");
    println!("collapsing each SCC gives the condensation DAG (used by 2-SAT and cycle checks)");
}

struct Tarjan<'a> {
    g: &'a [Vec<usize>],
    index: Vec<usize>, // discovery order, usize::MAX = unvisited
    low: Vec<usize>,   // earliest discovery reachable through the subtree
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    next: usize,
    comps: Vec<Vec<usize>>,
}

impl Tarjan<'_> {
    fn visit(&mut self, u: usize) {
        self.index[u] = self.next;
        self.low[u] = self.next;
        self.next += 1;
        self.stack.push(u);
        self.on_stack[u] = true;

        let neigh = self.g[u].clone(); // clone: visit() needs &mut self
        for v in neigh {
            if self.index[v] == usize::MAX {
                self.visit(v);
                self.low[u] = self.low[u].min(self.low[v]);
            } else if self.on_stack[v] {
                // back edge into the current SCC
                self.low[u] = self.low[u].min(self.index[v]);
            }
        }

        // u roots an SCC: everything above u on the stack belongs to it
        if self.low[u] == self.index[u] {
            let mut comp = Vec::new();
            loop {
                let w = self.stack.pop().expect("u is on the stack");
                self.on_stack[w] = false;
                comp.push(w);
                if w == u {
                    break;
                }
            }
            self.comps.push(comp);
        }
    }
}

fn tarjan(g: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = g.len();
    let mut t = Tarjan {
        g,
        index: vec![usize::MAX; n],
        low: vec![0; n],
        on_stack: vec![false; n],
        stack: Vec::new(),
        next: 0,
        comps: Vec::new(),
    };
    for u in 0..n {
        if t.index[u] == usize::MAX {
            t.visit(u);
        }
    }
    t.comps
}
