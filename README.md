# ll

A Rust-powered TUI dashboard. Default command shows a centered "Hello" in a bordered box; `ll info` displays live CPU, memory, and disk stats using [ratatui](https://github.com/ratatui/ratatui) and [sysinfo](https://github.com/GuillaumeGomez/sysinfo).

## Usage

```bash
cargo run          # Hello TUI (press 'q' to quit)
cargo run -- info  # System info dashboard (press 'q' to quit)
cargo run -- algorithm list  # List all algorithm demos
cargo run -- algorithm quicksort  # Run one (58 classic algorithms)
```

## Algorithms

`ll algorithm <name>` runs a self-contained demo of a classic algorithm, grouped by category:

- **Searching**: linear-search, binary-search, bfs, dfs, binary-search-on-answer
- **Array Techniques**: two-pointers, prefix-sums, monotonic-stack
- **Sorting**: bubble-sort, merge-sort, quick-sort, heap-sort, tim-sort, radix-sort
- **Graph**: dijkstra, bellman-ford, floyd-warshall, kruskal, prim, topological-sort, a-star, zero-one-bfs, tarjan-scc
- **Dynamic Programming**: fibonacci, knapsack, lcs, edit-distance, matrix-chain, kadane, lis, coin-change, bitmask-dp
- **Number Theory**: sieve, extended-gcd
- **Cryptographic / Hashing**: sha256, rsa, diffie-hellman, consistent-hashing
- **Tree & Data Structure**: tree-traversals, avl, trie, union-find, segment-tree, fenwick-tree
- **Divide & Conquer**: strassen, karatsuba
- **Machine Learning**: gradient-descent, backprop, k-means, knn
- **String**: kmp, rabin-karp, boyer-moore, z-algorithm
- **Greedy**: huffman, activity-selection, fractional-knapsack
- **Geometry**: convex-hull

Names are matched ignoring case and dashes (`quicksort` == `quick-sort`).

Most demos verify their result against a brute-force implementation and print a
complexity footnote (e.g. `segment-tree` runs 500 random updates against a plain
array, `bitmask-dp` checks its tour against all 8! orderings).

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) 0.29 — terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) 0.28 — cross-platform terminal handling
- [sysinfo](https://crates.io/crates/sysinfo) 0.35 — system information

## License

MIT
