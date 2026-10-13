//! Tree & data-structure algorithms: traversals, AVL insert, Trie, Union-Find.

// ─── Binary tree traversals ───────────────────────────────────────────────────

#[derive(Clone)]
struct Node {
    val: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn leaf(val: i32) -> Option<Box<Node>> {
    Some(Box::new(Node {
        val,
        left: None,
        right: None,
    }))
}

/// ```text
///         1
///       /   \
///      2     3
///     / \     \
///    4   5     6
/// ```
fn sample_tree() -> Option<Box<Node>> {
    let left = Some(Box::new(Node {
        val: 2,
        left: leaf(4),
        right: leaf(5),
    }));
    let right = Some(Box::new(Node {
        val: 3,
        left: None,
        right: leaf(6),
    }));
    Some(Box::new(Node {
        val: 1,
        left,
        right,
    }))
}

pub fn tree_traversals() {
    let t = sample_tree();

    let mut pre = Vec::new();
    preorder(&t, &mut pre);
    let mut ino = Vec::new();
    inorder(&t, &mut ino);
    let mut post = Vec::new();
    postorder(&t, &mut post);

    println!("tree:        1\n            / \\\n           2   3\n          / \\   \\\n         4   5   6");
    println!("preorder  (root-left-right)  = {pre:?}");
    println!("inorder   (left-root-right)  = {ino:?}  ← sorted for a BST");
    println!("postorder (left-right-root)  = {post:?}");
    println!("all O(n) time, O(h) stack space (h = tree height)");
}

fn preorder(n: &Option<Box<Node>>, out: &mut Vec<i32>) {
    if let Some(node) = n {
        out.push(node.val);
        preorder(&node.left, out);
        preorder(&node.right, out);
    }
}

fn inorder(n: &Option<Box<Node>>, out: &mut Vec<i32>) {
    if let Some(node) = n {
        inorder(&node.left, out);
        out.push(node.val);
        inorder(&node.right, out);
    }
}

fn postorder(n: &Option<Box<Node>>, out: &mut Vec<i32>) {
    if let Some(node) = n {
        postorder(&node.left, out);
        postorder(&node.right, out);
        out.push(node.val);
    }
}

// ─── AVL tree ─────────────────────────────────────────────────────────────────

struct AvlNode {
    val: i32,
    height: i32,
    left: Option<Box<AvlNode>>,
    right: Option<Box<AvlNode>>,
}

impl AvlNode {
    fn new(val: i32) -> Box<AvlNode> {
        Box::new(AvlNode {
            val,
            height: 1,
            left: None,
            right: None,
        })
    }
}

fn height(n: &Option<Box<AvlNode>>) -> i32 {
    n.as_ref().map(|n| n.height).unwrap_or(0)
}

fn update(n: &mut AvlNode) {
    n.height = 1 + height(&n.left).max(height(&n.right));
}

fn balance_factor(n: &AvlNode) -> i32 {
    height(&n.left) - height(&n.right)
}

fn rotate_right(mut y: Box<AvlNode>) -> Box<AvlNode> {
    let mut x = y.left.take().expect("right rotation needs a left child");
    y.left = x.right.take();
    update(&mut y);
    x.right = Some(y);
    update(&mut x);
    x
}

fn rotate_left(mut x: Box<AvlNode>) -> Box<AvlNode> {
    let mut y = x.right.take().expect("left rotation needs a right child");
    x.right = y.left.take();
    update(&mut x);
    y.left = Some(x);
    update(&mut y);
    y
}

fn avl_insert(node: Option<Box<AvlNode>>, val: i32) -> Box<AvlNode> {
    let mut n = match node {
        None => return AvlNode::new(val),
        Some(n) => n,
    };
    if val < n.val {
        n.left = Some(avl_insert(n.left.take(), val));
    } else {
        n.right = Some(avl_insert(n.right.take(), val));
    }
    update(&mut n);

    // Rebalance if this node became skewed
    let bf = balance_factor(&n);
    if bf > 1 {
        if val < n.left.as_ref().map(|l| l.val).unwrap_or(i32::MAX) {
            // left-left → single right rotation
            rotate_right(n)
        } else {
            // left-right → left child first, then right
            n.left = n.left.map(rotate_left);
            rotate_right(n)
        }
    } else if bf < -1 {
        if val > n.right.as_ref().map(|r| r.val).unwrap_or(i32::MIN) {
            rotate_left(n)
        } else {
            n.right = n.right.map(rotate_right);
            rotate_left(n)
        }
    } else {
        n
    }
}

pub fn avl() {
    // Sorted input is the worst case for a plain BST (would become a linked list).
    let mut root: Option<Box<AvlNode>> = None;
    for v in 1..=15 {
        root = Some(avl_insert(root, v));
    }
    let h = height(&root);
    let mut ino = Vec::new();
    inorder_avl(&root, &mut ino);
    println!("inserted 1..=15 in ascending order (worst case for a plain BST)");
    println!("inorder = {ino:?}  (still sorted → BST property held)");
    println!("tree height = {h}  (⌈log₂(15)⌉+1 = 4; a plain BST would be 15)");
    println!("rotations keep |left − right| ≤ 1 at every node → O(log n) ops");
}

fn inorder_avl(n: &Option<Box<AvlNode>>, out: &mut Vec<i32>) {
    if let Some(node) = n {
        inorder_avl(&node.left, out);
        out.push(node.val);
        inorder_avl(&node.right, out);
    }
}

// ─── Trie (prefix tree) ───────────────────────────────────────────────────────

struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    end: bool,
}

impl TrieNode {
    fn new() -> Box<TrieNode> {
        Box::new(TrieNode {
            children: std::array::from_fn(|_| None),
            end: false,
        })
    }
}

fn insert(trie: &mut Option<Box<TrieNode>>, word: &str) {
    if trie.is_none() {
        *trie = Some(TrieNode::new());
    }
    let mut node = trie.as_mut().unwrap();
    for b in word.bytes() {
        let i = (b - b'a') as usize;
        if node.children[i].is_none() {
            node.children[i] = Some(TrieNode::new());
        }
        node = node.children[i].as_mut().unwrap();
    }
    node.end = true;
}

fn search(trie: &Option<Box<TrieNode>>, word: &str) -> bool {
    let mut node = match trie.as_ref() {
        None => return false,
        Some(n) => n,
    };
    for b in word.bytes() {
        let i = (b - b'a') as usize;
        node = match node.children[i].as_ref() {
            None => return false,
            Some(n) => n,
        };
    }
    node.end
}

fn starts_with(trie: &Option<Box<TrieNode>>, prefix: &str) -> bool {
    let mut node = match trie.as_ref() {
        None => return false,
        Some(n) => n,
    };
    for b in prefix.bytes() {
        let i = (b - b'a') as usize;
        match node.children[i].as_ref() {
            None => return false,
            Some(n) => node = n,
        }
    }
    true
}

pub fn trie() {
    let words = ["car", "card", "care", "dog", "do"];
    let mut trie: Option<Box<TrieNode>> = None;
    for w in words {
        insert(&mut trie, w);
    }
    println!("inserted: {words:?}");
    for w in ["car", "cat"] {
        println!("  search(\"{w}\")  = {}", search(&trie, w));
    }
    for p in ["ca", "d", "z"] {
        println!("  prefix(\"{p}\") = {}", starts_with(&trie, p));
    }
    println!("lookup O(L) for a word of length L — autocomplete collects the subtree");
}

// ─── Union-Find (Disjoint Set) ────────────────────────────────────────────────

pub struct Dsu {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl Dsu {
    pub fn new(n: usize) -> Dsu {
        Dsu {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path halving
            x = self.parent[x];
        }
        x
    }

    /// Returns true if the sets were separate and are now merged.
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        // union by rank
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
        true
    }
}

pub fn union_find() {
    let mut dsu = Dsu::new(7);
    println!("union operations on 7 elements:");
    for (a, b) in [(0, 1), (2, 3), (1, 3), (4, 5), (5, 6)] {
        let merged = dsu.union(a, b);
        println!("  union({a}, {b}) → merged: {merged}");
    }
    println!("connectivity queries:");
    for (a, b) in [(0, 3), (0, 2), (4, 6), (0, 4)] {
        let (ra, rb) = (dsu.find(a), dsu.find(b));
        println!("  find({a}) == find({b}) ? {}", ra == rb);
    }
    println!("amortized ~O(α(n)) per op — near-constant (α = inverse Ackermann)");
}

// ─── Segment tree (lazy range-add / range-sum) ──────────────────────────────

pub struct SegTree {
    n: usize,
    sum: Vec<i64>,
    lazy: Vec<i64>, // pending range additions
}

impl SegTree {
    pub fn new(data: &[i64]) -> SegTree {
        let n = data.len();
        let mut st = SegTree {
            n,
            sum: vec![0; 4 * n],
            lazy: vec![0; 4 * n],
        };
        st.build(1, 0, n, data);
        st
    }

    fn build(&mut self, node: usize, lo: usize, hi: usize, data: &[i64]) {
        if lo + 1 == hi {
            self.sum[node] = data[lo];
            return;
        }
        let mid = (lo + hi) / 2;
        self.build(2 * node, lo, mid, data);
        self.build(2 * node + 1, mid, hi, data);
        self.sum[node] = self.sum[2 * node] + self.sum[2 * node + 1];
    }

    fn apply(&mut self, node: usize, len: usize, v: i64) {
        self.sum[node] += v * len as i64;
        self.lazy[node] += v;
    }

    /// Hand a pending update down to the children (only non-leaves have them).
    fn push(&mut self, node: usize, len: usize) {
        if self.lazy[node] == 0 {
            return;
        }
        let v = self.lazy[node];
        if len > 1 {
            self.apply(2 * node, len / 2, v);
            self.apply(2 * node + 1, len / 2, v);
        }
        self.lazy[node] = 0;
    }

    /// Add `v` to every index in `[ql, qr)`.
    fn add(&mut self, node: usize, lo: usize, hi: usize, ql: usize, qr: usize, v: i64) {
        if ql >= qr || lo >= hi {
            return;
        }
        if ql <= lo && hi <= qr {
            self.apply(node, hi - lo, v); // fully covered → stop here
            return;
        }
        if hi - lo == 1 {
            return; // uncovered leaf — it lies outside [ql, qr)
        }
        self.push(node, hi - lo);
        let mid = (lo + hi) / 2;
        self.add(2 * node, lo, mid, ql, qr, v);
        self.add(2 * node + 1, mid, hi, ql, qr, v);
        self.sum[node] = self.sum[2 * node] + self.sum[2 * node + 1];
    }

    /// Sum of `[ql, qr)`.
    fn ask(&mut self, node: usize, lo: usize, hi: usize, ql: usize, qr: usize) -> i64 {
        if ql >= qr || lo >= hi {
            return 0;
        }
        if ql <= lo && hi <= qr {
            return self.sum[node];
        }
        if hi - lo == 1 {
            return 0; // uncovered leaf — it lies outside [ql, qr)
        }
        self.push(node, hi - lo);
        let mid = (lo + hi) / 2;
        self.ask(2 * node, lo, mid, ql, qr) + self.ask(2 * node + 1, mid, hi, ql, qr)
    }

    pub fn range_add(&mut self, l: usize, r: usize, v: i64) {
        self.add(1, 0, self.n, l, r, v);
    }

    pub fn query(&mut self, l: usize, r: usize) -> i64 {
        self.ask(1, 0, self.n, l, r)
    }
}

pub fn segment_tree() {
    let data = [5i64, 2, 7, 1, 3, 6, 4, 0];
    let mut seg = SegTree::new(&data);
    let mut brute = data.to_vec();
    let mut ok = true;

    println!("initial array = {data:?}");
    let scripted: [(&str, usize, usize, i64); 4] = [
        ("add", 2, 6, 3),
        ("sum", 0, 8, 0),
        ("add", 0, 4, -2),
        ("sum", 3, 7, 0),
    ];
    for (op, l, r, v) in scripted {
        if op == "add" {
            seg.range_add(l, r, v);
            for x in &mut brute[l..r] {
                *x += v;
            }
            println!("  range_add [{l}, {r}) += {v}");
        } else {
            let got = seg.query(l, r);
            let want: i64 = brute[l..r].iter().sum();
            ok &= got == want;
            println!("  query [{l}, {r}) = {got:>3}   (brute force = {want:>3})");
        }
    }

    // Randomized soak: 500 interleaved updates/queries against a plain array.
    let mut seed = 0xfeed_u64;
    for _ in 0..500 {
        let l = (next_rand(&mut seed) as usize) % data.len();
        let r = l + 1 + (next_rand(&mut seed) as usize) % (data.len() - l);
        if next_rand(&mut seed) % 2 == 0 {
            let v = next_rand(&mut seed) as i64 % 21 - 10;
            seg.range_add(l, r, v);
            for x in &mut brute[l..r] {
                *x += v;
            }
        } else {
            ok &= seg.query(l, r) == brute[l..r].iter().sum::<i64>();
        }
    }
    println!("500 random range-add / range-sum ops match a plain array: {ok}");
    println!("O(n) build, O(log n) per update/query, O(n) space — lazy tags keep");
    println!("whole covered segments from touching their children");
}

fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed >> 33
}

// ─── Fenwick tree (binary indexed tree) ─────────────────────────────────────

pub struct Fenwick {
    bit: Vec<i64>, // 1-based; bit[i] covers a run of lowbit(i) elements
}

impl Fenwick {
    pub fn new(n: usize) -> Fenwick {
        Fenwick { bit: vec![0; n + 1] }
    }

    pub fn from(data: &[i64]) -> Fenwick {
        let mut fw = Fenwick::new(data.len());
        for (i, &v) in data.iter().enumerate() {
            fw.add(i, v);
        }
        fw
    }

    /// Add `delta` at 0-based index `i`.
    pub fn add(&mut self, mut i: usize, delta: i64) {
        i += 1;
        while i < self.bit.len() {
            self.bit[i] += delta;
            i += i & i.wrapping_neg(); // jump to the parent covering this run
        }
    }

    /// Sum of `[0, i]` (0-based, inclusive).
    pub fn prefix(&self, mut i: usize) -> i64 {
        i += 1;
        let mut s = 0i64;
        while i > 0 {
            s += self.bit[i];
            i -= i & i.wrapping_neg();
        }
        s
    }

    /// Sum of `[l, r)` (0-based).
    pub fn range_sum(&self, l: usize, r: usize) -> i64 {
        if l == 0 {
            self.prefix(r - 1)
        } else {
            self.prefix(r - 1) - self.prefix(l - 1)
        }
    }
}

pub fn fenwick_tree() {
    let data = [5i64, 3, 8, 1, 6, 2, 7, 4];
    let mut fw = Fenwick::from(&data);
    let mut naive = data.to_vec();
    let mut ok = true;

    println!("array = {data:?}");
    for (l, r) in [(0, 5), (2, 8), (3, 4), (1, 7)] {
        let got = fw.range_sum(l, r);
        let want: i64 = data[l..r].iter().sum();
        ok &= got == want;
        println!("  sum[{l}..{r}) = {got:>2}   (slice sum = {want:>2})");
    }
    fw.add(3, 10); // point update: data[3] 1 → 11
    naive[3] += 10;
    println!("after add(3, +10): sum[1..6) = {} (brute force = {})", fw.range_sum(1, 6), naive[1..6].iter().sum::<i64>());
    ok &= fw.range_sum(1, 6) == naive[1..6].iter().sum::<i64>();
    println!("all queries match brute force: {ok}\n");

    // Killer app: count inversions in O(n log n) over the value domain.
    let arr = [6i64, 3, 9, 1, 8, 2, 5, 4];
    let mut ft = Fenwick::new(*arr.iter().max().unwrap() as usize + 1);
    let mut inv = 0i64;
    for (i, &v) in arr.iter().enumerate() {
        inv += i as i64 - ft.prefix(v as usize); // how many seen values exceed v
        ft.add(v as usize, 1);
    }
    let brute_inv: i64 = (0..arr.len())
        .map(|i| (0..i).filter(|&j| arr[j] > arr[i]).count() as i64)
        .sum();
    println!("array = {arr:?}");
    println!("inversions = {inv}  (brute force = {brute_inv}, match: {})", inv == brute_inv);
    println!("O(log n) per update/query vs the segment tree's O(log n) with less code;");
    println!("bit tricks i += i & −i / i −= i & −i walk the index DAG up and down");
}
