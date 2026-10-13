//! Greedy algorithms: Huffman coding, activity selection, fractional knapsack.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;

// ─── Huffman coding ───────────────────────────────────────────────────────────

struct Huff {
    freq: usize,
    ch: Option<char>,
    left: Option<Rc<Huff>>,
    right: Option<Rc<Huff>>,
}

// Min-heap: reverse the frequency ordering so the smallest pops first.
impl PartialEq for Huff {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}
impl Eq for Huff {}
impl PartialOrd for Huff {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Huff {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

pub fn huffman() {
    let text = "huffman coding example";
    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in text.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let mut heap: BinaryHeap<Huff> = freq
        .iter()
        .map(|(&ch, &f)| Huff {
            freq: f,
            ch: Some(ch),
            left: None,
            right: None,
        })
        .collect();

    // Greedily merge the two rarest nodes until one tree remains.
    while heap.len() > 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        heap.push(Huff {
            freq: a.freq + b.freq,
            ch: None,
            left: Some(Rc::new(a)),
            right: Some(Rc::new(b)),
        });
    }
    let root = heap.pop().unwrap();

    let mut codes: HashMap<char, String> = HashMap::new();
    build_codes(&root, String::new(), &mut codes);

    println!("text = \"{text}\" ({} chars)", text.len());
    println!("symbol frequencies: {:?}", {
        let mut v: Vec<(char, usize)> = freq.into_iter().collect();
        v.sort_by_key(|&(c, _)| c);
        v
    });
    println!("prefix codes:");
    let mut sorted: Vec<(&char, &String)> = codes.iter().collect();
    sorted.sort_by_key(|(c, _)| **c);
    for (c, code) in &sorted {
        print!("  {c:?} → {code}   ");
    }
    println!();

    let total_bits: usize = text.chars().map(|c| codes[&c].len()).sum();
    let fixed = text.len() * 8;
    println!("encoded length = {total_bits} bits  vs  fixed 8-bit ASCII = {fixed} bits");
    println!("saving = {:.1}%  (frequent symbols get short codes — greedy is optimal here)",
        100.0 * (1.0 - total_bits as f64 / fixed as f64));
}

fn build_codes(node: &Huff, prefix: String, out: &mut HashMap<char, String>) {
    if let Some(c) = node.ch {
        out.insert(c, if prefix.is_empty() { "0".into() } else { prefix });
        return;
    }
    if let Some(l) = &node.left {
        build_codes(l, format!("{prefix}0"), out);
    }
    if let Some(r) = &node.right {
        build_codes(r, format!("{prefix}1"), out);
    }
}

// ─── Activity selection ───────────────────────────────────────────────────────

pub fn activity_selection() {
    // (name, start, finish)
    let mut acts = vec![
        ("A", 1, 4),
        ("B", 3, 5),
        ("C", 0, 6),
        ("D", 8, 9),
        ("E", 5, 7),
        ("F", 8, 11),
        ("G", 6, 10),
        ("H", 11, 12),
    ];
    // Greedy choice: always take the activity that finishes earliest.
    acts.sort_by_key(|a| a.2);

    let mut chosen = Vec::new();
    let mut last_finish = -1;
    for (name, start, finish) in &acts {
        if *start >= last_finish {
            chosen.push((name, start, finish));
            last_finish = *finish;
        }
    }

    println!("activities (start, finish):");
    for (name, s, f) in &acts {
        println!("  {name}: ({s}, {f})");
    }
    println!(
        "greedy selection (earliest finish first): {:?}",
        chosen.iter().map(|c| c.0).collect::<Vec<_>>()
    );
    println!("picked {} of {} activities — provably optimal for max count", chosen.len(), acts.len());
}

// ─── Fractional knapsack ──────────────────────────────────────────────────────

pub fn fractional_knapsack() {
    // (name, weight, value)
    let mut items: Vec<(&str, f64, f64)> = vec![
        ("A", 10.0, 60.0),
        ("B", 20.0, 100.0),
        ("C", 30.0, 120.0),
    ];
    let capacity = 50.0;

    // Greedy choice: best value/weight ratio first, taking fractions if needed.
    items.sort_by(|a, b| {
        let ra = a.2 / a.1;
        let rb = b.2 / b.1;
        rb.partial_cmp(&ra).unwrap_or(Ordering::Equal)
    });

    println!("items (weight, value, ratio):");
    for (name, w, v) in &items {
        println!("  {name}: ({w}, {v}, ratio {:.2})", v / w);
    }
    println!("capacity = {capacity}");

    let mut remaining = capacity;
    let mut total = 0.0;
    for (name, w, v) in &items {
        if remaining <= 0.0 {
            break;
        }
        let take = (*w).min(remaining);
        let got = v * take / w;
        remaining -= take;
        let frac = take / w;
        println!(
            "  take {name}: {take} of {w} units ({:.0}%) → value {got:.1}",
            frac * 100.0
        );
        total += got;
    }
    println!("total value = {total:.1}");
    println!("(0/1 knapsack would need DP — greedy is only optimal when fractions are allowed)");
}
