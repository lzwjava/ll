// word2vec-rs: Rust port of Mikolov's original C word2vec
// Skip-gram and CBOW with Negative Sampling
// Zero external dependencies — faithful to the original algorithm
//
// Original: https://code.google.com/archive/p/word2vec/
// Reference: Mikolov et al., "Distributed Representations of Words and Phrases
//            and their Compositionality" (NeurIPS 2013)

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::time::Instant;

// ─── Constants (matching original) ────────────────────────────────────────────

const EXP_TABLE_SIZE: usize = 1000;
const MAX_EXP: f32 = 6.0;
const TABLE_SIZE: usize = 100_000_000; // 1e8 negative sampling table
const MAX_SEN_LEN: usize = 1000;

// ─── LCG PRNG (matches original C: next_random * 25214903917 + 11) ───────────

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(25214903917).wrapping_add(11);
        self.0
    }
}

// ─── Vocabulary ──────────────────────────────────────────────────────────────

struct Vocab {
    words: Vec<String>,
    counts: Vec<u64>,
    map: HashMap<String, usize>,
    total: u64,
}

impl Vocab {
    fn from_corpus(path: &str, min_count: u32) -> io::Result<Self> {
        let mut freq: HashMap<String, u64> = HashMap::new();
        for line in BufReader::new(File::open(path)?).lines() {
            for tok in line?.split_whitespace() {
                if !tok.is_empty() {
                    *freq.entry(tok.to_lowercase()).or_default() += 1;
                }
            }
        }

        let mut pairs: Vec<(String, u64)> = freq
            .into_iter()
            .filter(|(_, c)| *c >= min_count as u64)
            .collect();
        pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let total: u64 = pairs.iter().map(|(_, c)| c).sum();
        let mut map = HashMap::with_capacity(pairs.len());
        let mut words = Vec::with_capacity(pairs.len());
        let mut counts = Vec::with_capacity(pairs.len());

        for (i, (w, c)) in pairs.into_iter().enumerate() {
            map.insert(w.clone(), i);
            words.push(w);
            counts.push(c);
        }

        eprintln!(
            "Vocab: {} words, {} total tokens (min_count={})",
            words.len(),
            total,
            min_count
        );

        Ok(Self {
            words,
            counts,
            map,
            total,
        })
    }

    fn len(&self) -> usize {
        self.words.len()
    }

    fn idx(&self, w: &str) -> Option<usize> {
        self.map.get(w).copied()
    }
}

// ─── Training Config ─────────────────────────────────────────────────────────

struct TrainCfg {
    window: usize,
    negative: usize,
    lr: f32,
    min_lr: f32,
    epochs: usize,
    cbow: bool,
    subsample_threshold: f32, // 0 = disabled
}

impl Default for TrainCfg {
    fn default() -> Self {
        Self {
            window: 5,
            negative: 5,
            lr: 0.025,
            min_lr: 0.0001,
            epochs: 5,
            cbow: false,
            subsample_threshold: 1e-4,
        }
    }
}

// ─── Model ───────────────────────────────────────────────────────────────────

struct Model {
    vocab: Vocab,
    dim: usize,
    syn0: Vec<f32>, // input embeddings  [vocab_size × dim]
    syn1: Vec<f32>, // output embeddings [vocab_size × dim] (neg sampling)
    exp_table: Vec<f32>,
    neg_table: Vec<u32>,
}

impl Model {
    fn new(vocab: Vocab, dim: usize) -> Self {
        let n = vocab.len();

        // Pre-compute sigmoid approximation table
        // exp_table[i] = sigmoid((i/size * 2 - 1) * MAX_EXP)
        let mut exp_table = vec![0.0f32; EXP_TABLE_SIZE];
        for i in 0..EXP_TABLE_SIZE {
            let x = (i as f32 / EXP_TABLE_SIZE as f32 * 2.0 - 1.0) * MAX_EXP;
            exp_table[i] = 1.0 / (1.0 + (-x).exp());
        }

        // Build negative sampling table: unigram distribution ^ 0.75
        let mut neg_table = vec![0u32; TABLE_SIZE];
        let pow_sum: f64 = vocab.counts.iter().map(|&c| (c as f64).powf(0.75)).sum();
        let mut acc = 0.0f64;
        let mut ti = 0;
        for i in 0..n {
            acc += (vocab.counts[i] as f64).powf(0.75) / pow_sum;
            while ti < TABLE_SIZE && (ti as f64 / TABLE_SIZE as f64) < acc {
                neg_table[ti] = i as u32;
                ti += 1;
            }
        }
        // Fill remainder with last word
        while ti < TABLE_SIZE {
            neg_table[ti] = (n - 1) as u32;
            ti += 1;
        }

        // Initialize syn0: uniform [-0.5/dim, 0.5/dim]
        let mut rng = Rng(1);
        let syn0: Vec<f32> = (0..n * dim)
            .map(|_| (rng.next_u64() as f32 / u64::MAX as f32 - 0.5) / dim as f32)
            .collect();

        // syn1 initialized to zero (correct for negative sampling)
        let syn1 = vec![0.0f32; n * dim];

        Self {
            vocab,
            dim,
            syn0,
            syn1,
            exp_table,
            neg_table,
        }
    }

    #[inline]
    fn sigmoid(&self, x: f32) -> f32 {
        if x >= MAX_EXP {
            1.0
        } else if x <= -MAX_EXP {
            0.0
        } else {
            let i = ((x + MAX_EXP) * (EXP_TABLE_SIZE as f32 / MAX_EXP / 2.0)) as usize;
            self.exp_table[i.min(EXP_TABLE_SIZE - 1)]
        }
    }

    fn train(&mut self, path: &str, cfg: &TrainCfg) -> io::Result<()> {
        let total_target = self.vocab.total as u64 * cfg.epochs as u64;
        let start = Instant::now();
        let mut rng = Rng(1);
        let mut word_count: u64 = 0;
        let mut alpha = cfg.lr;

        eprintln!(
            "Training: {} epochs, window={}, negative={}, lr={}",
            cfg.epochs, cfg.window, cfg.negative, cfg.lr
        );
        eprintln!(
            "Architecture: {}",
            if cfg.cbow { "CBOW" } else { "Skip-gram" }
        );

        for epoch in 0..cfg.epochs {
            let reader = BufReader::new(File::open(path)?);
            let mut sentence: Vec<usize> = Vec::with_capacity(MAX_SEN_LEN);
            let mut neu1e = vec![0.0f32; self.dim];

            for line_result in reader.lines() {
                let line = line_result?;
                for tok in line.split_whitespace() {
                    let w = tok.to_lowercase();
                    let wi = match self.vocab.idx(&w) {
                        Some(i) => i,
                        None => continue,
                    };

                    word_count += 1;

                    // Linear learning rate decay
                    if word_count % 10000 == 0 {
                        let progress = word_count as f32 / total_target as f32;
                        alpha = cfg.lr * (1.0 - progress);
                        if alpha < cfg.min_lr {
                            alpha = cfg.min_lr;
                        }
                        eprint!(
                            "\rEpoch {}/{} | {:.1}% | lr: {:.6} | {}k words",
                            epoch + 1,
                            cfg.epochs,
                            progress * 100.0,
                            alpha,
                            word_count / 1000
                        );
                    }

                    // Subsampling of frequent words (Mikolov's trick)
                    if cfg.subsample_threshold > 0.0 {
                        let f = self.vocab.counts[wi] as f32 / self.vocab.total as f32;
                        let t = cfg.subsample_threshold;
                        // Original formula: ran = (sqrt(f/t) + 1) * (t/f)
                        let ran = ((f / t).sqrt() + 1.0) * (t / f);
                        let r = (rng.next_u64() & 0xFFFF) as f32 / 65536.0;
                        if ran < r {
                            continue;
                        }
                    }

                    sentence.push(wi);

                    if sentence.len() >= MAX_SEN_LEN {
                        self.train_sentence(&sentence, cfg, alpha, &mut neu1e, &mut rng);
                        sentence.clear();
                    }
                }

                // Train remaining words in partial sentence
                if !sentence.is_empty() {
                    self.train_sentence(&sentence, cfg, alpha, &mut neu1e, &mut rng);
                    sentence.clear();
                }
            }

            eprintln!(
                "\rEpoch {}/{} complete | alpha: {:.6}                    ",
                epoch + 1,
                cfg.epochs,
                alpha
            );
        }

        let elapsed = start.elapsed().as_secs_f64();
        eprintln!(
            "Trained {} words in {:.1}s ({:.0} words/s)",
            word_count,
            elapsed,
            word_count as f64 / elapsed
        );
        Ok(())
    }

    fn train_sentence(
        &mut self,
        sent: &[usize],
        cfg: &TrainCfg,
        alpha: f32,
        neu1e: &mut [f32],
        rng: &mut Rng,
    ) {
        if cfg.cbow {
            self.train_cbow(sent, cfg, alpha, neu1e, rng);
        } else {
            self.train_sg(sent, cfg, alpha, neu1e, rng);
        }
    }

    /// Skip-gram with Negative Sampling
    /// Faithful to original word2vec.c: context word → syn0, predict center word
    fn train_sg(
        &mut self,
        sent: &[usize],
        cfg: &TrainCfg,
        alpha: f32,
        neu1e: &mut [f32],
        rng: &mut Rng,
    ) {
        let d = self.dim;
        let w = cfg.window;

        for pos in 0..sent.len() {
            let center = sent[pos];

            // Random window reduction (original: b = next_random % window)
            let b = (rng.next_u64() % w as u64) as usize;

            for a in b..(2 * w + 1 - b) {
                if a == w {
                    continue;
                }
                let cp = pos as isize - w as isize + a as isize;
                if cp < 0 || cp as usize >= sent.len() {
                    continue;
                }
                let ctx = sent[cp as usize];
                let l1 = ctx * d; // input: context word embedding

                // Zero error accumulator
                for v in neu1e.iter_mut() {
                    *v = 0.0;
                }

                // Negative sampling
                for nd in 0..=cfg.negative {
                    let (tgt, label) = if nd == 0 {
                        (center, 1.0f32)
                    } else {
                        let idx = (rng.next_u64() >> 16) as usize % TABLE_SIZE;
                        let t = self.neg_table[idx] as usize;
                        if t == center {
                            continue;
                        }
                        (t, 0.0f32)
                    };

                    let l2 = tgt * d;

                    // Dot product: syn0[ctx] · syn1[center/neg]
                    let f: f32 = (0..d).map(|c| self.syn0[l1 + c] * self.syn1[l2 + c]).sum();

                    // Gradient: g = (label - sigmoid(f)) * alpha
                    let g = (label - self.sigmoid(f)) * alpha;

                    // Accumulate error for input vector
                    for c in 0..d {
                        neu1e[c] += g * self.syn1[l2 + c];
                    }
                    // Update output vector
                    for c in 0..d {
                        self.syn1[l2 + c] += g * self.syn0[l1 + c];
                    }
                }

                // Apply accumulated error to input embedding
                for c in 0..d {
                    self.syn0[l1 + c] += neu1e[c];
                }
            }
        }
    }

    /// CBOW with Negative Sampling: average context → predict center
    fn train_cbow(
        &mut self,
        sent: &[usize],
        cfg: &TrainCfg,
        alpha: f32,
        neu1e: &mut [f32],
        rng: &mut Rng,
    ) {
        let d = self.dim;
        let w = cfg.window;

        for pos in 0..sent.len() {
            let center = sent[pos];
            let mut neu1 = vec![0.0f32; d];
            let mut cw = 0u32;

            let b = (rng.next_u64() % w as u64) as usize;

            // Average context word vectors
            for a in b..(2 * w + 1 - b) {
                if a == w {
                    continue;
                }
                let cp = pos as isize - w as isize + a as isize;
                if cp < 0 || cp as usize >= sent.len() {
                    continue;
                }
                let ctx = sent[cp as usize];
                let l1 = ctx * d;
                for c in 0..d {
                    neu1[c] += self.syn0[l1 + c];
                }
                cw += 1;
            }

            if cw == 0 {
                continue;
            }

            // Average
            let inv_cw = 1.0 / cw as f32;
            for c in 0..d {
                neu1[c] *= inv_cw;
            }

            // Zero error accumulator
            for v in neu1e.iter_mut() {
                *v = 0.0;
            }

            // Negative sampling
            for nd in 0..=cfg.negative {
                let (tgt, label) = if nd == 0 {
                    (center, 1.0f32)
                } else {
                    let idx = (rng.next_u64() >> 16) as usize % TABLE_SIZE;
                    let t = self.neg_table[idx] as usize;
                    if t == center {
                        continue;
                    }
                    (t, 0.0f32)
                };

                let l2 = tgt * d;

                let f: f32 = (0..d).map(|c| neu1[c] * self.syn1[l2 + c]).sum();

                let g = (label - self.sigmoid(f)) * alpha;

                for c in 0..d {
                    neu1e[c] += g * self.syn1[l2 + c];
                }
                for c in 0..d {
                    self.syn1[l2 + c] += g * neu1[c];
                }
            }

            // Apply error to all context word embeddings
            for a in b..(2 * w + 1 - b) {
                if a == w {
                    continue;
                }
                let cp = pos as isize - w as isize + a as isize;
                if cp < 0 || cp as usize >= sent.len() {
                    continue;
                }
                let ctx = sent[cp as usize];
                let l1 = ctx * d;
                for c in 0..d {
                    self.syn0[l1 + c] += neu1e[c];
                }
            }
        }
    }

    // ─── Save ─────────────────────────────────────────────────────────────

    fn save(&self, path: &str, binary: bool) -> io::Result<()> {
        let f = File::create(path)?;
        let mut w = BufWriter::with_capacity(1 << 20, f);
        writeln!(w, "{} {}", self.vocab.len(), self.dim)?;

        for i in 0..self.vocab.len() {
            if binary {
                // Binary: word\0<float32 * dim>
                w.write_all(self.vocab.words[i].as_bytes())?;
                w.write_all(&[0u8])?;
                for d in 0..self.dim {
                    w.write_all(&self.syn0[i * self.dim + d].to_le_bytes())?;
                }
            } else {
                // Text: word f0 f1 f2 ...
                write!(w, "{}", self.vocab.words[i])?;
                for d in 0..self.dim {
                    write!(w, " {:.6}", self.syn0[i * self.dim + d])?;
                }
                writeln!(w)?;
            }
        }

        eprintln!(
            "Vectors saved to {} ({})",
            path,
            if binary { "binary" } else { "text" }
        );
        Ok(())
    }
}

// ─── Vectors (for distance / analogy queries) ────────────────────────────────

struct Vectors {
    words: Vec<String>,
    data: Vec<f32>, // [vocab_size × dim], normalized
    dim: usize,
    map: HashMap<String, usize>,
}

impl Vectors {
    fn load(path: &str) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        // Parse header: "vocab_size dim\n"
        let hdr_end = buf.iter().position(|&b| b == b'\n').unwrap_or(buf.len());
        let hdr = std::str::from_utf8(&buf[..hdr_end])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let parts: Vec<&str> = hdr.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad header"));
        }
        let vocab_size: usize = parts[0].parse().unwrap();
        let dim: usize = parts[1].parse().unwrap();
        eprintln!("Loading {} vectors, dim={}...", vocab_size, dim);

        let data_start = hdr_end + 1;

        // Detect format: binary has \0 bytes before first \n
        let first_nl = buf[data_start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(buf.len() - data_start);
        let is_binary = buf[data_start..data_start + first_nl].contains(&b'\0');

        let mut words = Vec::with_capacity(vocab_size);
        let mut data = Vec::with_capacity(vocab_size * dim);
        let mut map = HashMap::with_capacity(vocab_size);

        if is_binary {
            let mut pos = data_start;
            for i in 0..vocab_size {
                // Read null-terminated word
                let word_end = buf[pos..].iter().position(|&b| b == 0).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing null byte")
                })?;
                let word = String::from_utf8_lossy(&buf[pos..pos + word_end]).into_owned();
                pos += word_end + 1;

                // Read dim f32 values (little-endian)
                if pos + dim * 4 > buf.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Truncated vector data",
                    ));
                }
                for d in 0..dim {
                    let off = pos + d * 4;
                    data.push(f32::from_le_bytes([
                        buf[off],
                        buf[off + 1],
                        buf[off + 2],
                        buf[off + 3],
                    ]));
                }
                pos += dim * 4;

                map.insert(word.clone(), i);
                words.push(word);
            }
        } else {
            let text = &buf[data_start..];
            for (i, line) in text.split(|&b| b == b'\n').enumerate() {
                if i >= vocab_size {
                    break;
                }
                let line_str = std::str::from_utf8(line).unwrap_or("");
                let mut parts = line_str.split_whitespace();
                let word = match parts.next() {
                    Some(w) => w.to_string(),
                    None => continue,
                };
                let vals: Vec<f32> = parts.filter_map(|s| s.parse().ok()).collect();
                if vals.len() != dim {
                    continue;
                }
                map.insert(word.clone(), i);
                words.push(word);
                data.extend_from_slice(&vals);
            }
        }

        eprintln!("Loaded {} vectors", words.len());

        let mut v = Self {
            words,
            data,
            dim,
            map,
        };
        v.normalize();
        Ok(v)
    }

    /// L2-normalize all vectors in-place
    fn normalize(&mut self) {
        for i in 0..self.words.len() {
            let base = i * self.dim;
            let norm: f32 = self.data[base..base + self.dim]
                .iter()
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt();
            if norm > 0.0 {
                for d in 0..self.dim {
                    self.data[base + d] /= norm;
                }
            }
        }
    }

    fn cosine(&self, i: usize, j: usize) -> f32 {
        let (si, sj) = (i * self.dim, j * self.dim);
        (0..self.dim)
            .map(|d| self.data[si + d] * self.data[sj + d])
            .sum()
    }

    fn most_similar(&self, word: &str, topn: usize) -> Vec<(&str, f32)> {
        let idx = match self.map.get(word) {
            Some(&i) => i,
            None => return vec![],
        };

        let mut sims: Vec<(usize, f32)> = (0..self.words.len())
            .filter(|&i| i != idx)
            .map(|i| (i, self.cosine(idx, i)))
            .collect();

        sims.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sims.iter()
            .take(topn)
            .map(|&(i, s)| (self.words[i].as_str(), s))
            .collect()
    }

    fn analogy(&self, a: &str, b: &str, c: &str, topn: usize) -> Vec<(&str, f32)> {
        // a - b + c = ?  (e.g., king - man + woman = queen)
        let ia = match self.map.get(a) {
            Some(&i) => i,
            None => return vec![],
        };
        let ib = match self.map.get(b) {
            Some(&i) => i,
            None => return vec![],
        };
        let ic = match self.map.get(c) {
            Some(&i) => i,
            None => return vec![],
        };

        let d = self.dim;
        // target = vec(a) - vec(b) + vec(c)
        let mut target = vec![0.0f32; d];
        for k in 0..d {
            target[k] = self.data[ia * d + k] - self.data[ib * d + k] + self.data[ic * d + k];
        }

        // Normalize target
        let norm: f32 = target.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in target.iter_mut() {
                *v /= norm;
            }
        }

        let exclude = [ia, ib, ic];
        let mut sims: Vec<(usize, f32)> = (0..self.words.len())
            .filter(|i| !exclude.contains(i))
            .map(|i| {
                let base = i * d;
                let sim: f32 = (0..d).map(|k| target[k] * self.data[base + k]).sum();
                (i, sim)
            })
            .collect();

        sims.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sims.iter()
            .take(topn)
            .map(|&(i, s)| (self.words[i].as_str(), s))
            .collect()
    }
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

pub fn main(args: &[String]) -> io::Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("train") => cmd_train(&args[1..]),
        Some("distance") | Some("dist") => cmd_distance(&args[1..]),
        Some("analogy") => cmd_analogy(&args[1..]),
        Some("accuracy") => cmd_accuracy(&args[1..]),
        _ => {
            usage();
            Ok(())
        }
    }
}

fn usage() {
    eprintln!("word2vec-rs — Rust port of Mikolov's word2vec");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  word2vec-rs train -input FILE -output FILE [options]");
    eprintln!("  word2vec-rs distance VECTORS_FILE");
    eprintln!("  word2vec-rs analogy  VECTORS_FILE");
    eprintln!();
    eprintln!("Train options:");
    eprintln!("  -input    FILE    Training corpus (one sentence per line)");
    eprintln!("  -output   FILE    Output vector file");
    eprintln!("  -size     N       Embedding dimension      [200]");
    eprintln!("  -window   N       Context window size       [5]");
    eprintln!("  -negative N       Negative samples          [5]");
    eprintln!("  -epochs   N       Training epochs           [5]");
    eprintln!("  -lr       F       Initial learning rate     [0.025]");
    eprintln!("  -min-count N      Min word frequency        [5]");
    eprintln!("  -cbow     0/1     Use CBOW (1) or Skip-gram (0) [0]");
    eprintln!("  -binary   0/1     Save as binary (1) or text (0) [1]");
    eprintln!("  -sample   F       Subsampling threshold     [1e-4]");
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag {
            return args.get(i + 1).cloned();
        }
    }
    None
}

fn parse_flag_f32(args: &[String], flag: &str, default: f32) -> f32 {
    parse_flag(args, flag)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_flag_usize(args: &[String], flag: &str, default: usize) -> usize {
    parse_flag(args, flag)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn cmd_train(args: &[String]) -> io::Result<()> {
    let input = match parse_flag(args, "-input") {
        Some(f) => f,
        None => {
            eprintln!("Error: -input FILE required");
            return Ok(());
        }
    };
    let output = match parse_flag(args, "-output") {
        Some(f) => f,
        None => {
            eprintln!("Error: -output FILE required");
            return Ok(());
        }
    };

    let dim = parse_flag_usize(args, "-size", 200);
    let window = parse_flag_usize(args, "-window", 5);
    let negative = parse_flag_usize(args, "-negative", 5);
    let epochs = parse_flag_usize(args, "-epochs", 5);
    let lr = parse_flag_f32(args, "-lr", 0.025);
    let min_count = parse_flag_usize(args, "-min-count", 5) as u32;
    let cbow = parse_flag_usize(args, "-cbow", 0) == 1;
    let binary = parse_flag_usize(args, "-binary", 1) == 1;
    let sample = parse_flag_f32(args, "-sample", 1e-4);

    eprintln!("Building vocabulary from {}...", input);
    let vocab = Vocab::from_corpus(&input, min_count)?;
    if vocab.len() == 0 {
        eprintln!("Error: empty vocabulary (check min_count or input file)");
        return Ok(());
    }

    let mut model = Model::new(vocab, dim);
    let cfg = TrainCfg {
        window,
        negative,
        lr,
        min_lr: lr * 0.0001,
        epochs,
        cbow,
        subsample_threshold: sample,
    };

    model.train(&input, &cfg)?;
    model.save(&output, binary)?;

    Ok(())
}

fn cmd_distance(args: &[String]) -> io::Result<()> {
    let path = match args.first() {
        Some(f) => f.as_str(),
        None => {
            eprintln!("Usage: word2vec-rs distance <vectors-file>");
            return Ok(());
        }
    };

    let vecs = Vectors::load(path)?;
    let stdin = io::stdin();

    loop {
        eprint!("\nEnter word or sentence (EXIT to break): ");
        io::stderr().flush().ok();

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let word = line.trim().to_lowercase();
        if word == "exit" || word.is_empty() {
            break;
        }

        match vecs.map.get(&word) {
            Some(&idx) => {
                eprintln!("Word: {}  Position in vocabulary: {}", word, idx);
                eprintln!("{:>45}       Cosine distance", "Word");
                eprintln!("{}", "-".repeat(70));

                let results = vecs.most_similar(&word, 40);
                for (w, sim) in &results {
                    eprintln!("{:>45}       {:.6}", w, sim);
                }
            }
            None => {
                eprintln!("Word '{}' not in vocabulary", word);
            }
        }
    }

    Ok(())
}

fn cmd_analogy(args: &[String]) -> io::Result<()> {
    let path = match args.first() {
        Some(f) => f.as_str(),
        None => {
            eprintln!("Usage: word2vec-rs analogy <vectors-file>");
            return Ok(());
        }
    };

    let vecs = Vectors::load(path)?;
    let stdin = io::stdin();

    loop {
        eprint!("\nEnter three words, a b c (a is to b as c is to ?) (EXIT to break): ");
        io::stderr().flush().ok();

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.first() == Some(&"exit") || parts.is_empty() {
            break;
        }
        if parts.len() < 3 {
            eprintln!("Please enter three words separated by spaces");
            continue;
        }

        let (a, b, c) = (parts[0], parts[1], parts[2]);

        for w in &[a, b, c] {
            if !vecs.map.contains_key(*w) {
                eprintln!("Word '{}' not in vocabulary", w);
            }
        }

        eprintln!("{} is to {} as {} is to ?", a, b, c);
        eprintln!("{:>45}       Cosine distance", "Word");
        eprintln!("{}", "-".repeat(70));

        let results = vecs.analogy(a, b, c, 40);
        for (w, sim) in &results {
            eprintln!("{:>45}       {:.6}", w, sim);
        }
    }

    Ok(())
}

/// Evaluate word analogy accuracy (e.g., Google analogy test set)
/// Format: ": category\nword1 word2 word3 word4\n..." (word1 - word2 + word3 = word4)
fn cmd_accuracy(args: &[String]) -> io::Result<()> {
    let vectors_path = match args.first() {
        Some(f) => f.as_str(),
        None => {
            eprintln!("Usage: word2vec-rs accuracy <vectors-file> < <questions-file>");
            return Ok(());
        }
    };

    let vecs = Vectors::load(vectors_path)?;
    let stdin = io::stdin();
    let mut total = 0u64;
    let mut correct = 0u64;
    let mut categories: HashMap<String, (u64, u64)> = HashMap::new();
    let mut current_cat = String::from("unknown");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.starts_with(':') {
            current_cat = line[2..].to_string();
            continue;
        }

        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() < 4 {
            continue;
        }

        total += 1;
        let results = vecs.analogy(words[0], words[1], words[2], 1);
        let entry = categories.entry(current_cat.clone()).or_insert((0, 0));
        entry.0 += 1;

        if let Some((predicted, _)) = results.first() {
            if *predicted == words[3] {
                correct += 1;
                entry.1 += 1;
            }
        }
    }

    eprintln!(
        "\nAccuracy: {:.2}% ({}/{})",
        correct as f64 / total as f64 * 100.0,
        correct,
        total
    );
    for (cat, (t, c)) in &categories {
        eprintln!(
            "  {}: {:.2}% ({}/{})",
            cat,
            *c as f64 / *t as f64 * 100.0,
            c,
            t
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab() {
        // Write a tiny corpus
        let path = "/tmp/w2v_test_corpus.txt";
        std::fs::write(
            path,
            "the king is good\nthe queen is good\na man and a woman\n",
        )
        .unwrap();

        let vocab = Vocab::from_corpus(path, 1).unwrap();
        assert!(vocab.len() > 0);
        assert!(vocab.idx("king").is_some());
    }

    #[test]
    fn test_sigmoid_table() {
        let vocab = Vocab {
            words: vec!["a".into(), "b".into()],
            counts: vec![100, 50],
            map: HashMap::new(),
            total: 150,
        };
        let model = Model::new(vocab, 10);

        // sigmoid(0) ≈ 0.5
        let s0 = model.sigmoid(0.0);
        assert!((s0 - 0.5).abs() < 0.01, "sigmoid(0) = {}", s0);

        // sigmoid(6) ≈ 1.0
        assert!((model.sigmoid(6.0) - 1.0).abs() < 0.001);

        // sigmoid(-6) ≈ 0.0
        assert!(model.sigmoid(-6.0) < 0.001);
    }

    #[test]
    fn test_neg_table() {
        let vocab = Vocab {
            words: vec!["frequent".into(), "rare".into()],
            counts: vec![1000, 10],
            map: HashMap::new(),
            total: 1010,
        };
        let model = Model::new(vocab, 10);

        // Word 0 (frequent) should appear much more in the table
        let count0 = model.neg_table.iter().filter(|&&x| x == 0).count();
        let count1 = model.neg_table.iter().filter(|&&x| x == 1).count();
        assert!(count0 > count1 * 10, "count0={}, count1={}", count0, count1);
    }

    #[test]
    fn test_roundtrip_save_load() {
        use std::io::Read;

        let vocab = Vocab {
            words: vec!["hello".into(), "world".into()],
            counts: vec![100, 50],
            map: HashMap::new(),
            total: 150,
        };
        let mut model = Model::new(vocab, 3);
        model.syn0[0] = 0.1;
        model.syn0[1] = 0.2;
        model.syn0[2] = 0.3;

        // Save as text
        model.save("/tmp/w2v_test.txt", false).unwrap();
        let mut text = String::new();
        File::open("/tmp/w2v_test.txt")
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert!(text.contains("hello"));
        assert!(text.contains("0.100000"));

        // Save as binary
        model.save("/tmp/w2v_test.bin", true).unwrap();
        let vecs = Vectors::load("/tmp/w2v_test.bin").unwrap();
        assert_eq!(vecs.words.len(), 2);
        assert_eq!(vecs.words[0], "hello");
    }
}
