//! Machine learning algorithms: gradient descent, backprop (XOR), k-means, KNN.

use std::time::Instant;

// ─── Gradient descent: linear regression ──────────────────────────────────────

pub fn gradient_descent() {
    // Underlying model: y = 3x + 7, with a little deterministic noise.
    let noise = [0.3, -0.2, 0.1, -0.1];
    let data: Vec<(f64, f64)> = (0..20)
        .map(|i| {
            let x = i as f64;
            let y = 3.0 * x + 7.0 + noise[i % 4];
            (x, y)
        })
        .collect();

    let (mut w, mut b) = (0.0f64, 0.0f64); // start far from the answer
    let lr = 0.001;
    let n = data.len() as f64;

    println!("fitting y = w·x + b to 20 noisy points (true: w=3, b=7)");
    for epoch in 0..=10_000 {
        // Batch gradient of mean squared error
        let (mut gw, mut gb) = (0.0, 0.0);
        for &(x, y) in &data {
            let err = w * x + b - y;
            gw += err * x;
            gb += err;
        }
        w -= lr * (2.0 * gw / n);
        b -= lr * (2.0 * gb / n);

        if epoch % 2000 == 0 {
            let loss: f64 = data
                .iter()
                .map(|&(x, y)| (w * x + b - y).powi(2))
                .sum::<f64>()
                / n;
            println!("  epoch {epoch:>3}: loss (MSE) = {loss:.4}   w = {w:.4}  b = {b:.4}");
        }
    }
    println!("final model: y = {w:.4}·x + {b:.4}  (converged toward 3 and 7)");
    println!("each step moves opposite ∇MSE — learning rate trades speed vs stability");
}

// ─── Backpropagation: 2-2-1 network learns XOR ────────────────────────────────

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

pub fn backprop() {
    let data = [
        (0.0, 0.0, 0.0),
        (0.0, 1.0, 1.0),
        (1.0, 0.0, 1.0),
        (1.0, 1.0, 0.0),
    ];

    // 2 inputs → 2 hidden (sigmoid) → 1 output (sigmoid)
    let mut w1 = [[0.5, -0.4], [0.6, 0.3]]; // w1[hidden][input]
    let mut b1 = [0.1, -0.1];
    let mut w2 = [0.7, -0.8]; // w2[hidden]
    let mut b2 = 0.0;
    let lr = 0.5;

    println!("training a 2-2-1 sigmoid net on XOR (lr = {lr}):");
    let start = Instant::now();
    for epoch in 0..=20_000 {
        let mut loss = 0.0;
        for &(x1, x2, target) in &data {
            // forward
            let h1 = sigmoid(w1[0][0] * x1 + w1[0][1] * x2 + b1[0]);
            let h2 = sigmoid(w1[1][0] * x1 + w1[1][1] * x2 + b1[1]);
            let o = sigmoid(w2[0] * h1 + w2[1] * h2 + b2);
            let err = o - target;
            loss += -target * o.ln() - (1.0 - target) * (1.0 - o).ln();

            // backward (chain rule)
            let d_o = err * o * (1.0 - o);
            let d_h1 = d_o * w2[0] * h1 * (1.0 - h1);
            let d_h2 = d_o * w2[1] * h2 * (1.0 - h2);

            w2[0] -= lr * d_o * h1;
            w2[1] -= lr * d_o * h2;
            b2 -= lr * d_o;
            w1[0][0] -= lr * d_h1 * x1;
            w1[0][1] -= lr * d_h1 * x2;
            b1[0] -= lr * d_h1;
            w1[1][0] -= lr * d_h2 * x1;
            w1[1][1] -= lr * d_h2 * x2;
            b1[1] -= lr * d_h2;
        }
        if epoch % 4000 == 0 {
            println!("  epoch {epoch:>5}: cross-entropy loss = {loss:.6}");
        }
    }
    let elapsed = start.elapsed();

    print!("  final outputs: ");
    for &(x1, x2, target) in &data {
        let h1 = sigmoid(w1[0][0] * x1 + w1[0][1] * x2 + b1[0]);
        let h2 = sigmoid(w1[1][0] * x1 + w1[1][1] * x2 + b1[1]);
        let o = sigmoid(w2[0] * h1 + w2[1] * h2 + b2);
        print!("({x1:.0},{x2:.0})→{o:.3} (t={target:.0})  ");
    }
    println!("\n  trained in {elapsed:.1?}");
    println!("gradients flow output→hidden via the chain rule; XOR is not linearly separable,");
    println!("so a hidden layer (non-linearity) is required.");
}

// ─── K-Means clustering ───────────────────────────────────────────────────────

pub fn k_means() {
    let points = [
        (1.0, 1.0),
        (1.2, 0.8),
        (0.8, 1.1),
        (5.0, 5.0),
        (5.2, 4.8),
        (4.9, 5.1),
    ];
    let k = 2;

    // Initial centroids: first k points
    let mut centroids: Vec<(f64, f64)> = points[..k].to_vec();
    let mut assign = vec![0usize; points.len()];

    println!("k-means (k = {k}) on 6 points in two obvious clusters:");
    for it in 0..10 {
        // Assignment step
        for (i, &p) in points.iter().enumerate() {
            let mut best = 0;
            let mut best_d = f64::MAX;
            for (c, &cen) in centroids.iter().enumerate() {
                let d = (p.0 - cen.0).powi(2) + (p.1 - cen.1).powi(2);
                if d < best_d {
                    best_d = d;
                    best = c;
                }
            }
            assign[i] = best;
        }
        // Update step: mean of each cluster
        let mut sums = vec![(0.0, 0.0); k];
        let mut counts = vec![0usize; k];
        for (i, &p) in points.iter().enumerate() {
            let c = assign[i];
            sums[c].0 += p.0;
            sums[c].1 += p.1;
            counts[c] += 1;
        }
        let old = centroids.clone();
        for c in 0..k {
            if counts[c] > 0 {
                centroids[c] = (sums[c].0 / counts[c] as f64, sums[c].1 / counts[c] as f64);
            }
        }
        let delta: f64 = centroids
            .iter()
            .zip(&old)
            .map(|(a, b)| (a.0 - b.0).abs() + (a.1 - b.1).abs())
            .sum();
        if it == 0 || delta < 1e-9 {
            println!("  iteration {}: centroids moved by {delta:.4}", it + 1);
        }
        if delta < 1e-9 {
            break;
        }
    }

    println!("final centroids: {:?}", centroids);
    for (c, _) in centroids.iter().enumerate() {
        let members: Vec<usize> = (0..points.len()).filter(|&i| assign[i] == c).collect();
        println!("  cluster {c} = {members:?}");
    }
    println!("O(n·k·i) — converges to a local optimum (init-sensitive)");
}

// ─── K-Nearest Neighbours ─────────────────────────────────────────────────────

pub fn knn() {
    // Training set: label A near origin, label B near (5,5)
    let train: [((f64, f64), char); 6] = [
        ((0.5, 0.6), 'A'),
        ((0.8, 1.1), 'A'),
        ((1.0, 0.4), 'A'),
        ((4.5, 4.7), 'B'),
        ((5.1, 5.0), 'B'),
        ((4.8, 5.3), 'B'),
    ];
    let test: (f64, f64) = (4.0, 4.2);
    let k = 3;

    let mut dists: Vec<(f64, char)> = train
        .iter()
        .map(|&(p, label)| {
            let d = ((p.0 - test.0).powi(2) + (p.1 - test.1).powi(2)).sqrt();
            (d, label)
        })
        .collect();
    dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    println!("training points: A near (0.5,0.5), B near (5,5)");
    println!("test point = {test:?}, k = {k}");
    println!("nearest neighbours (euclidean):");
    for (d, label) in dists.iter().take(k) {
        println!("  {label} at distance {d:.3}");
    }

    let mut votes = [0usize; 2];
    for &(_, label) in dists.iter().take(k) {
        votes[if label == 'A' { 0 } else { 1 }] += 1;
    }
    let winner = if votes[0] > votes[1] { 'A' } else { 'B' };
    println!("votes: A = {}, B = {} → predicted {winner}", votes[0], votes[1]);
    println!("no training phase — prediction cost is O(n) per query (use a KD-tree for speed)");
}
