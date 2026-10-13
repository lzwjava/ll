//! Catalog of classic algorithms, invoked as: `ll algorithm <name>`.
//!
//! `ll algorithm list` shows every entry grouped by category.
//! Names are matched ignoring case and dashes (`quicksort` == `quick-sort`).

pub mod crypto;
pub mod dnc;
pub mod dp;
pub mod graph;
pub mod greedy;
pub mod ml;
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