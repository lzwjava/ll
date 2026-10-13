//! Catalog of classic algorithms, invoked as: `ll algorithm <name>`.
//!
//! `ll algorithm list` shows every entry grouped by category.
//! Names are matched ignoring case and dashes (`quicksort` == `quick-sort`).

pub mod arrays;
pub mod backtracking;
pub mod crypto;
pub mod dnc;
pub mod dp;
pub mod flow;
pub mod geometry;
pub mod graph;
pub mod greedy;
pub mod ml;
pub mod numbertheory;
pub mod searching;
pub mod sorting;
pub mod strings;
pub mod trees;

pub struct Algorithm {
    pub name: &'static str,
    pub category: &'static str,
    pub desc: &'static str,
    pub run: fn(),
}

pub const ALGORITHMS: &[Algorithm] = &[
    // ─── Searching ────────────────────────────────────────────────────────────
    Algorithm {
        name: "linear-search",
        category: "Searching",
        desc: "O(n) scan over unsorted data",
        run: searching::linear_search,
    },
    Algorithm {
        name: "binary-search",
        category: "Searching",
        desc: "O(log n) search over a sorted array",
        run: searching::binary_search,
    },
    Algorithm {
        name: "bfs",
        category: "Searching",
        desc: "Breadth-first traversal; unweighted shortest path",
        run: searching::bfs,
    },
    Algorithm {
        name: "dfs",
        category: "Searching",
        desc: "Depth-first traversal",
        run: searching::dfs,
    },
    Algorithm {
        name: "binary-search-on-answer",
        category: "Searching",
        desc: "Parametric search: min feasible value via a monotone predicate",
        run: searching::binary_search_on_answer,
    },
    Algorithm {
        name: "ida-star",
        category: "Searching",
        desc: "Iterative-deepening A* solves the 8-puzzle in O(depth) memory",
        run: searching::ida_star,
    },
    // ─── Array Techniques ───────────────────────────────────────────────────────
    Algorithm {
        name: "two-pointers",
        category: "Array Techniques",
        desc: "Two pointers & sliding window subarray scans in O(n)",
        run: arrays::two_pointers,
    },
    Algorithm {
        name: "prefix-sums",
        category: "Array Techniques",
        desc: "O(1) range sums + O(1) range updates via difference array",
        run: arrays::prefix_sums,
    },
    Algorithm {
        name: "monotonic-stack",
        category: "Array Techniques",
        desc: "Next-greater-element & largest rectangle in histogram",
        run: arrays::monotonic_stack,
    },
    // ─── Sorting ──────────────────────────────────────────────────────────────
    Algorithm {
        name: "bubble-sort",
        category: "Sorting",
        desc: "Repeated swap passes; O(n^2), stable",
        run: sorting::bubble_sort,
    },
    Algorithm {
        name: "merge-sort",
        category: "Sorting",
        desc: "Divide & conquer; O(n log n), stable",
        run: sorting::merge_sort,
    },
    Algorithm {
        name: "quick-sort",
        category: "Sorting",
        desc: "Lomuto partition benchmark (10,000 ints)",
        run: sorting::quick_sort,
    },
    Algorithm {
        name: "heap-sort",
        category: "Sorting",
        desc: "Binary heap selection; O(n log n), in-place",
        run: sorting::heap_sort,
    },
    Algorithm {
        name: "tim-sort",
        category: "Sorting",
        desc: "Natural runs + insertion sort + merging",
        run: sorting::tim_sort,
    },
    Algorithm {
        name: "radix-sort",
        category: "Sorting",
        desc: "LSD radix, base 256; O(nk), non-comparison",
        run: sorting::radix_sort,
    },
    // ─── Graph ────────────────────────────────────────────────────────────────
    Algorithm {
        name: "dijkstra",
        category: "Graph",
        desc: "Single-source shortest path (non-negative weights)",
        run: graph::dijkstra,
    },
    Algorithm {
        name: "bellman-ford",
        category: "Graph",
        desc: "Shortest path with negative edges + cycle check",
        run: graph::bellman_ford,
    },
    Algorithm {
        name: "floyd-warshall",
        category: "Graph",
        desc: "All-pairs shortest path (DP over intermediates)",
        run: graph::floyd_warshall,
    },
    Algorithm {
        name: "kruskal",
        category: "Graph",
        desc: "Minimum spanning tree via sorted edges + Union-Find",
        run: graph::kruskal,
    },
    Algorithm {
        name: "prim",
        category: "Graph",
        desc: "Minimum spanning tree via greedy edge growth",
        run: graph::prim,
    },
    Algorithm {
        name: "topological-sort",
        category: "Graph",
        desc: "Dependency ordering in a DAG (Kahn's algorithm)",
        run: graph::topological_sort,
    },
    Algorithm {
        name: "a-star",
        category: "Graph",
        desc: "Heuristic pathfinding on a grid (Manhattan)",
        run: graph::a_star,
    },
    Algorithm {
        name: "zero-one-bfs",
        category: "Graph",
        desc: "Deque shortest path for 0/1 weights in O(V+E)",
        run: graph::zero_one_bfs,
    },
    Algorithm {
        name: "tarjan-scc",
        category: "Graph",
        desc: "Strongly connected components from one DFS (low-links)",
        run: graph::tarjan_scc,
    },
    Algorithm {
        name: "euler-path",
        category: "Graph",
        desc: "Visit every edge exactly once (Hierholzer circuit / trail)",
        run: graph::euler_path,
    },
    Algorithm {
        name: "bridges",
        category: "Graph",
        desc: "Cut edges & cut vertices via disc/low links",
        run: graph::bridges,
    },
    Algorithm {
        name: "two-sat",
        category: "Graph",
        desc: "2-SAT: implication graph + SCC + topological assignment",
        run: graph::two_sat,
    },
    // ─── Network Flow ─────────────────────────────────────────────────────
    Algorithm {
        name: "max-flow",
        category: "Network Flow",
        desc: "Edmonds-Karp augmenting paths; verifies max-flow = min-cut",
        run: flow::max_flow,
    },
    Algorithm {
        name: "bipartite-matching",
        category: "Network Flow",
        desc: "Kuhn's augmenting-path matching, checked against brute force",
        run: flow::bipartite_matching,
    },
    // ─── Dynamic Programming ──────────────────────────────────────────────────
    Algorithm {
        name: "fibonacci",
        category: "Dynamic Programming",
        desc: "Memoization vs tabulation",
        run: dp::fibonacci,
    },
    Algorithm {
        name: "knapsack",
        category: "Dynamic Programming",
        desc: "0/1 knapsack — resource allocation",
        run: dp::knapsack,
    },
    Algorithm {
        name: "lcs",
        category: "Dynamic Programming",
        desc: "Longest common subsequence (diff tools)",
        run: dp::lcs,
    },
    Algorithm {
        name: "edit-distance",
        category: "Dynamic Programming",
        desc: "Levenshtein distance (spell checkers, NLP)",
        run: dp::edit_distance,
    },
    Algorithm {
        name: "matrix-chain",
        category: "Dynamic Programming",
        desc: "Optimal matrix chain multiplication order",
        run: dp::matrix_chain,
    },
    Algorithm {
        name: "kadane",
        category: "Dynamic Programming",
        desc: "Maximum subarray sum in O(n)",
        run: dp::kadane,
    },
    Algorithm {
        name: "lis",
        category: "Dynamic Programming",
        desc: "Longest increasing subsequence in O(n log n) (patience)",
        run: dp::lis,
    },
    Algorithm {
        name: "coin-change",
        category: "Dynamic Programming",
        desc: "Unbounded knapsack: min coins & combination counts",
        run: dp::coin_change,
    },
    Algorithm {
        name: "bitmask-dp",
        category: "Dynamic Programming",
        desc: "TSP over subsets — dp[mask][i] in O(2ⁿ·n²)",
        run: dp::bitmask_dp,
    },
    Algorithm {
        name: "digit-dp",
        category: "Dynamic Programming",
        desc: "Count digit strings with a mod-13 + substring constraint",
        run: dp::digit_dp,
    },
    // ─── Number Theory ────────────────────────────────────────────────────────
    Algorithm {
        name: "sieve",
        category: "Number Theory",
        desc: "Primes ≤ N by crossing out multiples, O(n log log n)",
        run: numbertheory::sieve,
    },
    Algorithm {
        name: "extended-gcd",
        category: "Number Theory",
        desc: "Bézout ax + by = gcd → modular inverse, CRT building block",
        run: numbertheory::extended_gcd,
    },
    Algorithm {
        name: "fast-exponentiation",
        category: "Number Theory",
        desc: "Binary powering a^e mod m in O(log e) multiplications",
        run: numbertheory::fast_exponentiation,
    },
    Algorithm {
        name: "chinese-remainder",
        category: "Number Theory",
        desc: "CRT: merge congruences x ≡ aᵢ (mod mᵢ) for coprime moduli",
        run: numbertheory::chinese_remainder,
    },
    Algorithm {
        name: "matrix-exponentiation",
        category: "Number Theory",
        desc: "Linear recurrences (fib(10⁶)) by matrix powering in O(log n)",
        run: numbertheory::matrix_exponentiation,
    },
    Algorithm {
        name: "discrete-log",
        category: "Number Theory",
        desc: "Baby-step giant-step: smallest x with gˣ ≡ h (mod p)",
        run: numbertheory::discrete_log,
    },
    Algorithm {
        name: "miller-rabin",
        category: "Number Theory",
        desc: "Deterministic Miller–Rabin primality for all 64-bit integers",
        run: numbertheory::miller_rabin,
    },
    // ─── Cryptographic / Hashing ──────────────────────────────────────────────
    Algorithm {
        name: "sha256",
        category: "Cryptographic / Hashing",
        desc: "SHA-256 collision-resistant hash",
        run: crypto::sha256,
    },
    Algorithm {
        name: "rsa",
        category: "Cryptographic / Hashing",
        desc: "Textbook RSA keygen, encrypt, decrypt",
        run: crypto::rsa,
    },
    Algorithm {
        name: "diffie-hellman",
        category: "Cryptographic / Hashing",
        desc: "Key exchange over an insecure channel",
        run: crypto::diffie_hellman,
    },
    Algorithm {
        name: "consistent-hashing",
        category: "Cryptographic / Hashing",
        desc: "Distributed key assignment with minimal reshuffling",
        run: crypto::consistent_hashing,
    },
    // ─── Tree & Data Structure ────────────────────────────────────────────────
    Algorithm {
        name: "tree-traversals",
        category: "Tree & Data Structure",
        desc: "Inorder / preorder / postorder (recursive)",
        run: trees::tree_traversals,
    },
    Algorithm {
        name: "avl",
        category: "Tree & Data Structure",
        desc: "Self-balancing BST insert with rotations",
        run: trees::avl,
    },
    Algorithm {
        name: "trie",
        category: "Tree & Data Structure",
        desc: "Prefix tree insert/search/autocomplete",
        run: trees::trie,
    },
    Algorithm {
        name: "union-find",
        category: "Tree & Data Structure",
        desc: "Disjoint set with path compression + union by rank",
        run: trees::union_find,
    },
    Algorithm {
        name: "segment-tree",
        category: "Tree & Data Structure",
        desc: "Lazy range-add + range-sum queries in O(log n)",
        run: trees::segment_tree,
    },
    Algorithm {
        name: "fenwick-tree",
        category: "Tree & Data Structure",
        desc: "Binary indexed tree: prefix sums & inversion counting",
        run: trees::fenwick_tree,
    },
    // ─── Divide & Conquer ─────────────────────────────────────────────────────
    Algorithm {
        name: "strassen",
        category: "Divide & Conquer",
        desc: "Matrix multiplication in O(n^2.81)",
        run: dnc::strassen,
    },
    Algorithm {
        name: "karatsuba",
        category: "Divide & Conquer",
        desc: "Fast big-integer multiplication",
        run: dnc::karatsuba,
    },
    // ─── Machine Learning ─────────────────────────────────────────────────────
    Algorithm {
        name: "gradient-descent",
        category: "Machine Learning",
        desc: "Linear regression fit by batch GD",
        run: ml::gradient_descent,
    },
    Algorithm {
        name: "backprop",
        category: "Machine Learning",
        desc: "2-2-1 neural net learns XOR",
        run: ml::backprop,
    },
    Algorithm {
        name: "k-means",
        category: "Machine Learning",
        desc: "Unsupervised clustering into k centroids",
        run: ml::k_means,
    },
    Algorithm {
        name: "knn",
        category: "Machine Learning",
        desc: "k-nearest-neighbours classification",
        run: ml::knn,
    },
    // ─── String ───────────────────────────────────────────────────────────────
    Algorithm {
        name: "kmp",
        category: "String",
        desc: "Knuth-Morris-Pratt pattern matching, O(n+m)",
        run: strings::kmp,
    },
    Algorithm {
        name: "rabin-karp",
        category: "String",
        desc: "Rolling-hash pattern matching",
        run: strings::rabin_karp,
    },
    Algorithm {
        name: "boyer-moore",
        category: "String",
        desc: "Bad-character text search (Horspool variant)",
        run: strings::boyer_moore,
    },
    Algorithm {
        name: "z-algorithm",
        category: "String",
        desc: "Z-array pattern matching — KMP's prefix trick, one array",
        run: strings::z_algorithm,
    },
    Algorithm {
        name: "manacher",
        category: "String",
        desc: "Longest palindromic substring in O(n) with mirrored radii",
        run: strings::manacher,
    },
    Algorithm {
        name: "aho-corasick",
        category: "String",
        desc: "Trie + failure links: all patterns in one text scan",
        run: strings::aho_corasick,
    },
    Algorithm {
        name: "suffix-array",
        category: "String",
        desc: "Sorted suffixes + LCP (prefix doubling, Kasai)",
        run: strings::suffix_array,
    },
    // ─── Greedy ───────────────────────────────────────────────────────────────
    Algorithm {
        name: "huffman",
        category: "Greedy",
        desc: "Optimal prefix coding (ZIP, PNG)",
        run: greedy::huffman,
    },
    Algorithm {
        name: "activity-selection",
        category: "Greedy",
        desc: "Maximize compatible activities by finish time",
        run: greedy::activity_selection,
    },
    Algorithm {
        name: "fractional-knapsack",
        category: "Greedy",
        desc: "Best value/weight ratio first, take fractions",
        run: greedy::fractional_knapsack,
    },
    // ─── Geometry ──────────────────────────────────────────────────────────────
    Algorithm {
        name: "convex-hull",
        category: "Geometry",
        desc: "Andrew's monotone chain hull of 2-D points, O(n log n)",
        run: geometry::convex_hull,
    },
    Algorithm {
        name: "segment-intersection",
        category: "Geometry",
        desc: "Do two segments touch? Orientation tests, all edge cases",
        run: geometry::segment_intersection,
    },
    Algorithm {
        name: "point-in-polygon",
        category: "Geometry",
        desc: "Ray casting (even-odd) vs winding number, boundary handling",
        run: geometry::point_in_polygon,
    },
    Algorithm {
        name: "closest-pair",
        category: "Geometry",
        desc: "Nearest 2-D points by divide & conquer in O(n log n)",
        run: geometry::closest_pair,
    },
    Algorithm {
        name: "rotating-calipers",
        category: "Geometry",
        desc: "Convex polygon diameter from antipodal pairs in O(h)",
        run: geometry::rotating_calipers,
    },
    Algorithm {
        name: "triangle-centers",
        category: "Geometry",
        desc: "Centroid, in-, circum-, orthocenter and the Euler line",
        run: geometry::triangle_centers,
    },
    // ─── Backtracking ───────────────────────────────────────────────────────
    Algorithm {
        name: "n-queens",
        category: "Backtracking",
        desc: "Bitmask placement; counts all 724 solutions for n = 10",
        run: backtracking::n_queens,
    },
    Algorithm {
        name: "sudoku",
        category: "Backtracking",
        desc: "MRV backtracking solves a classic puzzle",
        run: backtracking::sudoku,
    },
];

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn find(name: &str) -> Option<&'static Algorithm> {
    let key = normalize(name);
    ALGORITHMS.iter().find(|a| normalize(a.name) == key)
}

fn list() {
    println!("Usage: ll algorithm <name>");
    println!("       ll algorithm list\n");
    let mut category = "";
    for a in ALGORITHMS {
        if a.category != category {
            category = a.category;
            println!("{category}");
        }
        println!("  {:<24} {}", a.name, a.desc);
    }
}

pub fn main_with_args(args: &[String]) -> std::io::Result<()> {
    match args.first().map(String::as_str) {
        None | Some("list") | Some("--help") | Some("-h") => {
            list();
            Ok(())
        }
        Some(name) => match find(name) {
            Some(algo) => {
                println!("── {} ── ({}) ─────────────────", algo.name, algo.category);
                (algo.run)();
                Ok(())
            }
            None => {
                eprintln!("Unknown algorithm: {name}\n");
                list();
                Ok(())
            }
        },
    }
}