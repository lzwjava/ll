//! Backtracking: N-Queens (bitmask), Sudoku solver (MRV heuristic).

// ─── N-Queens ─────────────────────────────────────────────────────────────────

/// Solution counts for n = 0..=10 (OEIS A000170) — the demo checks against them.
const KNOWN_SOLUTIONS: [usize; 11] = [0, 1, 0, 0, 2, 10, 4, 40, 92, 352, 724];

fn n_queens_count(n: usize) -> usize {
    fn rec(row: usize, n: usize, cols: usize, diag_down: usize, diag_up: usize) -> usize {
        if row == n {
            return 1; // every row is filled → one complete solution
        }
        // Free squares: not attacked on the column or either diagonal.
        let mut free = !cols & !diag_down & !diag_up & ((1 << n) - 1);
        let mut total = 0;
        while free != 0 {
            let bit = free & free.wrapping_neg(); // lowest free square
            free ^= bit;
            total += rec(
                row + 1,
                n,
                cols | bit,
                (diag_down | bit) << 1, // attacks shift down-right
                (diag_up | bit) >> 1,   // attacks shift down-left
            );
        }
        total
    }
    rec(0, n, 0, 0, 0)
}

fn first_solution(n: usize) -> Vec<usize> {
    fn rec(row: usize, n: usize, cols: usize, d1: usize, d2: usize, placed: &mut Vec<usize>) -> bool {
        if row == n {
            return true;
        }
        let mut free = !cols & !d1 & !d2 & ((1 << n) - 1);
        while free != 0 {
            let bit = free & free.wrapping_neg();
            free ^= bit;
            placed.push(bit.trailing_zeros() as usize);
            if rec(row + 1, n, cols | bit, (d1 | bit) << 1, (d2 | bit) >> 1, placed) {
                return true;
            }
            placed.pop();
        }
        false
    }
    let mut placed = Vec::new();
    rec(0, n, 0, 0, 0, &mut placed);
    placed
}

pub fn n_queens() {
    let mut all_ok = true;
    for n in 1..=10 {
        let count = n_queens_count(n);
        let ok = count == KNOWN_SOLUTIONS[n];
        all_ok &= ok;
        println!("  n = {n:>2}: {count:>3} solutions   (known {}, match: {ok})", KNOWN_SOLUTIONS[n]);
    }
    println!("all counts match: {all_ok}");

    let n = 8;
    let pos = first_solution(n);
    println!("one solution for n = {n}: {pos:?}  (column index of the queen in each row)");
    for &c in &pos {
        let line: String = (0..n)
            .map(|j| if j == c { 'Q' } else { '.' })
            .collect();
        println!("  {line}");
    }
    println!("one queen per row; three bitmasks reject attacked squares in O(1) per branch");
    println!("→ worst case O(n!), but the pruning collapses it to milliseconds even at n = 10");
}

// ─── Sudoku ───────────────────────────────────────────────────────────────────

const PUZZLE: [[u8; 9]; 9] = [
    [5, 3, 0, 0, 7, 0, 0, 0, 0],
    [6, 0, 0, 1, 9, 5, 0, 0, 0],
    [0, 9, 8, 0, 0, 0, 0, 6, 0],
    [8, 0, 0, 0, 6, 0, 0, 0, 3],
    [4, 0, 0, 8, 0, 3, 0, 0, 1],
    [7, 0, 0, 0, 2, 0, 0, 0, 6],
    [0, 6, 0, 0, 0, 0, 2, 8, 0],
    [0, 0, 0, 4, 1, 9, 0, 0, 5],
    [0, 0, 0, 0, 8, 0, 0, 7, 9],
];

fn candidates(g: &[[u8; 9]; 9], r: usize, c: usize) -> u16 {
    let mut used = 0u16;
    for i in 0..9 {
        if g[r][i] != 0 {
            used |= 1 << (g[r][i] - 1);
        }
        if g[i][c] != 0 {
            used |= 1 << (g[i][c] - 1);
        }
    }
    let (br, bc) = (r / 3 * 3, c / 3 * 3);
    for dr in 0..3 {
        for dc in 0..3 {
            let v = g[br + dr][bc + dc];
            if v != 0 {
                used |= 1 << (v - 1);
            }
        }
    }
    !used & 0x1FF // bits 0..8 = digits 1..9 that are still free
}

/// Backtrack with MRV: always branch on the empty cell with fewest candidates.
fn solve(g: &mut [[u8; 9]; 9], nodes: &mut usize) -> bool {
    *nodes += 1;
    let (mut best_r, mut best_c, mut best_mask) = (0usize, 0usize, 0u16);
    let mut best_count = 10u32;
    let mut empty = false;
    for r in 0..9 {
        for c in 0..9 {
            if g[r][c] != 0 {
                continue;
            }
            empty = true;
            let mask = candidates(g, r, c);
            let count = mask.count_ones();
            if count < best_count {
                best_count = count;
                (best_r, best_c, best_mask) = (r, c, mask);
                if count == 0 {
                    break; // dead end — no need to look further
                }
            }
        }
        if best_count == 0 {
            break;
        }
    }
    if !empty {
        return true; // board full
    }
    if best_count == 0 {
        return false; // contradiction up the stack
    }
    let mut m = best_mask;
    while m != 0 {
        let bit = m & m.wrapping_neg();
        m ^= bit;
        g[best_r][best_c] = bit.trailing_zeros() as u8 + 1;
        if solve(g, nodes) {
            return true;
        }
    }
    g[best_r][best_c] = 0; // undo this level's choice
    false
}

fn print_grid(g: &[[u8; 9]; 9]) {
    for (i, row) in g.iter().enumerate() {
        if i % 3 == 0 && i != 0 {
            println!("      ------+-------+------");
        }
        let mut line = String::from("      ");
        for (j, &v) in row.iter().enumerate() {
            if j % 3 == 0 && j != 0 {
                line.push_str("| ");
            }
            line.push(if v == 0 { '.' } else { (b'0' + v) as char });
            line.push(' ');
        }
        println!("{line}");
    }
}

fn solved(g: &[[u8; 9]; 9]) -> bool {
    let is_one_to_nine = |line: [u8; 9]| {
        let mut s = line;
        s.sort_unstable();
        s == [1, 2, 3, 4, 5, 6, 7, 8, 9]
    };
    let rows_ok = (0..9).all(|r| is_one_to_nine(g[r]));
    let cols_ok = (0..9).all(|c| {
        let mut col = [0u8; 9];
        for r in 0..9 {
            col[r] = g[r][c];
        }
        is_one_to_nine(col)
    });
    let boxes_ok = (0..9).all(|b| {
        let mut sq = [0u8; 9];
        let (br, bc) = (b / 3 * 3, b % 3 * 3);
        for k in 0..9 {
            sq[k] = g[br + k / 3][bc + k % 3];
        }
        is_one_to_nine(sq)
    });
    let givens_ok = (0..9).all(|r| (0..9).all(|c| PUZZLE[r][c] == 0 || PUZZLE[r][c] == g[r][c]));
    rows_ok && cols_ok && boxes_ok && givens_ok
}

pub fn sudoku() {
    println!("puzzle (0 = empty):");
    print_grid(&PUZZLE);

    let mut grid = PUZZLE;
    let mut nodes = 0usize;
    if !solve(&mut grid, &mut nodes) {
        println!("no solution");
        return;
    }
    println!("\nsolution ({nodes} recursion nodes):");
    print_grid(&grid);

    println!("\nrows each contain 1..9, same for columns and 3×3 boxes: {ok}", ok = solved(&grid));
    println!("every given digit stayed in place: {ok2}", ok2 = (0..9).all(|r|
        (0..9).all(|c| PUZZLE[r][c] == 0 || PUZZLE[r][c] == grid[r][c])));
    println!("backtracking tries one digit at a time and undoes on failure → O(9^empties) worst case;");
    println!("MRV (fewest candidates first) makes the constraint propagation visible: {nodes} nodes only");
}
