# ll

A Rust-powered TUI dashboard. Default command shows a centered "Hello" in a bordered box; `ll info` displays live CPU, memory, and disk stats using [ratatui](https://github.com/ratatui/ratatui) and [sysinfo](https://github.com/GuillaumeGomez/sysinfo).

## Usage

```bash
cargo run          # Hello TUI (press 'q' to quit)
cargo run -- info  # System info dashboard (press 'q' to quit)
cargo run -- algorithm list  # List all algorithm demos
cargo run -- algorithm quicksort  # Run one (80 classic algorithms)
cargo run -- image list       # List every image command
cargo run -- video list       # List every video command
ll image crop photo.png out.png -g 400x300+50+50
ll video trim clip.mp4 part.mp4 -s 10 -e 25
```

## Algorithms

`ll algorithm <name>` runs a self-contained demo of a classic algorithm, grouped by
category. Besides the everyday interview/production staples, the graph, geometry,
string and math entries follow the contest classics from Liu Rujia's books
(*Algorithm Contest Entry Guide* / 《算法竞赛入门经典》《训练指南》).

- **Searching**: linear-search, binary-search, bfs, dfs, binary-search-on-answer, ida-star
- **Array Techniques**: two-pointers, prefix-sums, monotonic-stack
- **Sorting**: bubble-sort, merge-sort, quick-sort, heap-sort, tim-sort, radix-sort
- **Graph**: dijkstra, bellman-ford, floyd-warshall, kruskal, prim, topological-sort, a-star, zero-one-bfs, tarjan-scc, euler-path, bridges, two-sat
- **Network Flow**: max-flow, bipartite-matching
- **Dynamic Programming**: fibonacci, knapsack, lcs, edit-distance, matrix-chain, kadane, lis, coin-change, bitmask-dp, digit-dp
- **Number Theory**: sieve, extended-gcd, fast-exponentiation, chinese-remainder, matrix-exponentiation, discrete-log, miller-rabin
- **Cryptographic / Hashing**: sha256, rsa, diffie-hellman, consistent-hashing
- **Tree & Data Structure**: tree-traversals, avl, trie, union-find, segment-tree, fenwick-tree
- **Divide & Conquer**: strassen, karatsuba
- **Machine Learning**: gradient-descent, backprop, k-means, knn
- **String**: kmp, rabin-karp, boyer-moore, z-algorithm, manacher, aho-corasick, suffix-array
- **Greedy**: huffman, activity-selection, fractional-knapsack
- **Geometry**: convex-hull, segment-intersection, point-in-polygon, closest-pair, rotating-calipers, triangle-centers
- **Backtracking**: n-queens, sudoku

Names are matched ignoring case and dashes (`quicksort` == `quick-sort`).

Most demos verify their result against a brute-force implementation and print a
complexity footnote (e.g. `segment-tree` runs 500 random updates against a plain
array, `bitmask-dp` checks its tour against all 8! orderings, `max-flow` re-derives
the min-cut by enumerating all 16 s/t cuts, `bridges` re-counts components after
deleting each edge, `n-queens` matches OEIS A000170, and `ida-star` replays its
moves and compares against BFS-optimal depth).

## Image editor

`ll image <command>` is a mini Photoshop/ImageMagick-style toolkit implemented in
pure Rust with the [`image`](https://crates.io/crates/image) crate — no external
`convert` binary needed. Run `ll image list` for the full catalog and
`ll image <command> --help` for the options of a single command.

```bash
ll image info photo.png --json                 # format, size, color, file size
ll image crop photo.png out.png -g 400x300+50+50   # px, %, -X/-Y from the far edge
ll image resize photo.png out.png -w 800 --mode fill  # or -p 50%, --filter lanczos3
ll image thumbnail photo.png thumb.png -s 256   # fast, never enlarges
ll image rotate photo.png out.png -a 45 --background '#202020'
ll image flip photo.png out.png --horizontal
ll image slice photo.png tiles/part --cols 3 --rows 2   # grid tiling
ll image slice photo.png tiles/chunk --tile 512x512      # fixed-size tiles
ll image montage photos/ --out sheet.png --cols 4 --thumb 320x320
ll image convert photo.png photo.webp -q 85     # png/jpg/webp/gif/bmp/tiff/...
ll image view photo.png                         # truecolor terminal preview
ll image apply photo.png out.webp crop:80%x80% resize:1200x_ grayscale sharpen:2
```

**Geometry**: `crop`, `resize`, `thumbnail`, `rotate` (any angle, fast paths for
90/180/270), `flip`, `pad` (canvas + gravity), `border`, `trim` (auto-crop
uniform edges).

**Color**: `grayscale`, `invert`, `sepia`, `brightness`, `contrast`, `saturate`,
`hue`, `opacity` (alpha fade), `posterize`.

**Filters**: `blur` (+ `--fast`), `sharpen` (unsharp mask), `pixelate`, `grain`,
`edge` (Sobel), `emboss`.

**Files & tools**: `info`, `slice`, `montage`, `convert`, `view`, `apply`
(chains any of the operators above in one pass, e.g.
`apply in.png out.jpg blur:2 saturate:150 border:10:black -q 90`).

Notes:

- Geometry follows ImageMagick: `WxH+X+Y`, percentages (`50%x50%`), and
  negative offsets (`-10-10` = 10 px from the right/bottom edge).
- Colors accept `#rgb` / `#rrggbb` / `#rrggbbaa`, `r,g,b[,a]`, names (`red`,
  `gold`, `transparent`, …).
- Missing output directories are created automatically; formats without alpha
  (jpeg, bmp, pnm) are flattened onto `--background` (white by default).
- Every operator also accepts its argument positionally:
  `ll image brightness in.png out.png -30`.

## Video tools

`ll video <command>` handles the common video chores by driving the system
`ffmpeg` / `ffprobe` binaries (they must be on `PATH` — transcoding in pure Rust
is not realistic). Every command echoes the exact ffmpeg command line it runs,
and `--dry-run` prints it without executing. Run `ll video list` for the full
catalog.

```bash
ll video info clip.mp4 --json                 # container, streams, codecs, fps
ll video screenshot clip.mp4 shot.png -t 1:30 # one frame as png/jpg/webp
ll video frames clip.mp4 shots/f --fps 1      # image sequence (--every N too)
ll video trim clip.mp4 part.mp4 -s 10 -e 25   # stream copy, --reencode for accuracy
ll video concat movie.mp4 a.mp4 b.mp4 c.mp4   # join clips
ll video audio clip.mp4 song.mp3              # mp3/m4a/wav/flac/opus by extension
ll video mute clip.mp4 silent.mp4             # drop the audio track
ll video volume clip.mp4 quiet.mp4 -v -6dB    # or -v 0.5
ll video resize clip.mp4 small.mp4 -w 1280    # -s WxH, --percent, --keep-aspect
ll video crop clip.mp4 out.mp4 -g 640x480+10+20
ll video rotate clip.mp4 out.mp4 -a 90        # flip --horizontal/--vertical
ll video speed clip.mp4 fast.mp4 -v 2         # 0.5x slower, audio follows
ll video gif clip.mp4 loop.gif -s 12 -w 480   # palette-optimised gif
ll video gif2video loop.gif clip.mp4
ll video sprite clip.mp4 sheet.png --cols 4 --rows 3   # contact sheet
ll video convert clip.mov clip.mp4 --codec h264 --crf 20 --preset slow
```

Notes:

- Times accept `90`, `1:30` or `01:02:03.5`.
- `trim`/`concat` stream-copy by default (instant, keyframe-aligned); add
  `--reencode` for frame-accurate cuts.
- Transform commands re-encode with sensible container defaults (h264+aac for
  mp4/mov, vp9+opus for webm) and honour `--crf` / `--preset`.
- Output directories are created automatically, and missing ffmpeg produces a
  friendly install hint instead of a crash.

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) 0.29 — terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) 0.28 — cross-platform terminal handling
- [sysinfo](https://crates.io/crates/sysinfo) 0.35 — system information
- [image](https://crates.io/crates/image) 0.25 — decoding/encoding and pixel ops for `ll image`

## License

MIT
