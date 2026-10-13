# ll

A Rust-powered TUI dashboard. Default command shows a centered "Hello" in a bordered box; `ll info` displays live CPU, memory, and disk stats using [ratatui](https://github.com/ratatui/ratatui) and [sysinfo](https://github.com/GuillaumeGomez/sysinfo).

## Usage

```bash
cargo run          # Hello TUI (press 'q' to quit)
cargo run -- info  # System info dashboard (press 'q' to quit)
cargo run -- algorithm list  # List all algorithm demos
cargo run -- algorithm quicksort  # Run one (42 classic algorithms)
```

## Algorithms

`ll algorithm <name>` runs a self-contained demo of a classic algorithm, grouped by category:

- **Searching**: linear-search, binary-search, bfs, dfs
- **Sorting**: bubble-sort, merge-sort, quick-sort, heap-sort, tim-sort, radix-sort
- **Graph**: dijkstra, bellman-ford, floyd-warshall, kruskal, prim, topological-sort, a-star
- **Dynamic Programming**: fibonacci, knapsack, lcs, edit-distance, matrix-chain
- **Cryptographic / Hashing**: sha256, rsa, diffie-hellman, consistent-hashing
- **Tree & Data Structure**: tree-traversals, avl, trie, union-find
- **Divide & Conquer**: strassen, karatsuba
- **Machine Learning**: gradient-descent, backprop, k-means, knn
- **String**: kmp, rabin-karp, boyer-moore
- **Greedy**: huffman, activity-selection, fractional-knapsack

Names are matched ignoring case and dashes (`quicksort` == `quick-sort`).

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) 0.29 — terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) 0.28 — cross-platform terminal handling
- [sysinfo](https://crates.io/crates/sysinfo) 0.35 — system information

## License

MIT
