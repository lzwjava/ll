//! Geometry: convex hull (Andrew's monotone chain).

// ─── Convex hull ─────────────────────────────────────────────────────────────

pub fn convex_hull() {
    // Includes interior, collinear and duplicate-edge cases on purpose.
    let mut pts: Vec<(i64, i64)> = vec![
        (0, 0),
        (1, 1),
        (2, 0),
        (3, 1),
        (3, 3),
        (2, 4),
        (1, 4),
        (0, 3),
        (1, 2),
        (2, 2),
        (1, 3),
        (3, 0),
    ];
    pts.sort();
    pts.dedup();

    let hull = andrew_hull(&pts);

    // Verification: every point inside/on the hull + all hull turns counter-clockwise.
    let cross = |o: (i64, i64), a: (i64, i64), b: (i64, i64)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };
    let inside = pts.iter().all(|&p| {
        (0..hull.len()).all(|i| cross(hull[i], hull[(i + 1) % hull.len()], p) >= 0)
    });
    let convex = (0..hull.len()).all(|i| {
        cross(hull[i], hull[(i + 1) % hull.len()], hull[(i + 2) % hull.len()]) > 0
    });

    let area = {
        let mut s = 0i64;
        for i in 0..hull.len() {
            let (x1, y1) = hull[i];
            let (x2, y2) = hull[(i + 1) % hull.len()];
            s += x1 * y2 - x2 * y1;
        }
        s.abs()
    };

    println!("points = {pts:?}");
    println!("hull (counter-clockwise):");
    for p in &hull {
        println!("  {p:?}");
    }
    println!("all input points inside or on hull: {inside}");
    println!("every hull turn is counter-clockwise (strictly convex): {convex}");
    println!("shoelace area = {} (× ½ → {})", area, area as f64 / 2.0);
    println!("sort by x + two monotone chains → O(n log n) from sorting, O(n) hull build");
}

/// Andrew's monotone chain: lower then upper chain, popping while the turn
/// is not counter-clockwise (drops collinear points on the hull edges).
fn andrew_hull(pts: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut p = pts.to_vec();
    p.sort();
    p.dedup();
    if p.len() <= 2 {
        return p;
    }
    let cross = |o: (i64, i64), a: (i64, i64), b: (i64, i64)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };

    let mut chain: Vec<(i64, i64)> = Vec::new();
    for &pt in &p {
        while chain.len() >= 2 && cross(chain[chain.len() - 2], chain[chain.len() - 1], pt) <= 0 {
            chain.pop();
        }
        chain.push(pt);
    }
    let lower_len = chain.len();
    for &pt in p.iter().rev().skip(1) {
        while chain.len() > lower_len
            && cross(chain[chain.len() - 2], chain[chain.len() - 1], pt) <= 0
        {
            chain.pop();
        }
        chain.push(pt);
    }
    chain.pop(); // last point repeats the first one
    chain
}