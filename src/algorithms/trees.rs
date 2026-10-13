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
