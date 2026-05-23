# ll

A Rust-powered TUI dashboard. Default command shows a centered "Hello" in a bordered box; `ll info` displays live CPU, memory, and disk stats using [ratatui](https://github.com/ratatui/ratatui) and [sysinfo](https://github.com/GuillaumeGomez/sysinfo).

Also includes quicksort benchmarks in Rust, Python, and Java — sorting 10,000 numbers across languages.

## Usage

```bash
cargo run          # Hello TUI (press 'q' to quit)
cargo run -- info  # System info dashboard (press 'q' to quit)
```

## Quick Sort Benchmarks

```bash
# Rust (via the TUI project's main binary — see src/main.rs)
# Python
python3 quicksort.py
# Java
javac Quicksort.java && java Quicksort
```

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) 0.29 — terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) 0.28 — cross-platform terminal handling
- [sysinfo](https://crates.io/crates/sysinfo) 0.35 — system information

## License

MIT
