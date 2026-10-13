//! Geometry: convex hull (Andrew's monotone chain), segment intersection,
//! point-in-polygon, closest pair (divide & conquer), rotating calipers,
//! triangle centers.

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

type Pt = (i64, i64);

/// Twice the signed area of triangle abc (positive when counter-clockwise).
fn orient(a: Pt, b: Pt, c: Pt) -> i64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

fn on_segment(a: Pt, b: Pt, p: Pt) -> bool {
    p.0 >= a.0.min(b.0)
        && p.0 <= a.0.max(b.0)
        && p.1 >= a.1.min(b.1)
        && p.1 <= a.1.max(b.1)
}

fn dist2(a: Pt, b: Pt) -> i64 {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    dx * dx + dy * dy
}

/// Small deterministic LCG so demos are reproducible without dependencies.
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed >> 33
}

// ─── Segment intersection ─────────────────────────────────────────────────────

/// True iff segments `ab` and `cd` share at least one point.
fn segments_intersect(a: Pt, b: Pt, c: Pt, d: Pt) -> bool {
    let (o1, o2, o3, o4) = (
        orient(a, b, c),
        orient(a, b, d),
        orient(c, d, a),
        orient(c, d, b),
    );
    // Proper crossing: the endpoints of each segment straddle the other one.
    if o1 * o2 < 0 && o3 * o4 < 0 {
        return true;
    }
    // Degenerate cases: an endpoint lies on the other segment
    // (this also covers collinear overlap).
    (o1 == 0 && on_segment(a, b, c))
        || (o2 == 0 && on_segment(a, b, d))
        || (o3 == 0 && on_segment(c, d, a))
        || (o4 == 0 && on_segment(c, d, b))
}

fn point_seg_dist(p: (f64, f64), a: Pt, b: Pt) -> f64 {
    let (ax, ay) = (a.0 as f64, a.1 as f64);
    let (dx, dy) = (b.0 as f64 - ax, b.1 as f64 - ay);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.0 - ax) * dx + (p.1 - ay) * dy) / len2).clamp(0.0, 1.0)
    };
    ((p.0 - (ax + t * dx)).powi(2) + (p.1 - (ay + t * dy)).powi(2)).sqrt()
}

/// Independent estimate of the smallest distance between two segments, obtained
/// by sampling 2001 points along each of them (intersection ⇒ gap ≈ 0).
fn sampled_gap(a: Pt, b: Pt, c: Pt, d: Pt) -> f64 {
    const K: usize = 2000;
    let mut best = f64::MAX;
    for i in 0..=K {
        let t = i as f64 / K as f64;
        let p = (
            a.0 as f64 + (b.0 - a.0) as f64 * t,
            a.1 as f64 + (b.1 - a.1) as f64 * t,
        );
        best = best.min(point_seg_dist(p, c, d));
        let q = (
            c.0 as f64 + (d.0 - c.0) as f64 * t,
            c.1 as f64 + (d.1 - c.1) as f64 * t,
        );
        best = best.min(point_seg_dist(q, a, b));
    }
    best
}

pub fn segment_intersection() {
    let cases: &[(&str, Pt, Pt, Pt, Pt)] = &[
        ("proper crossing", (0, 0), (4, 3), (0, 3), (4, 0)),
        ("shared endpoint", (0, 0), (2, 2), (2, 2), (4, 0)),
        ("collinear overlap", (0, 0), (4, 0), (2, 0), (6, 0)),
        ("collinear but apart", (0, 0), (1, 0), (2, 0), (3, 0)),
        ("T junction", (0, 0), (4, 0), (2, 0), (2, 3)),
        ("passes above", (0, 0), (3, 0), (1, 1), (1, 2)),
        ("integer crossing", (0, 0), (5, 5), (0, 5), (5, 0)),
        ("endpoint on interior", (0, 0), (4, 4), (1, 1), (3, 0)),
    ];

    println!("  {:<20} {:>9} {:>14}", "case", "intersect", "sampled gap");
    let mut all_ok = true;
    for &(name, a, b, c, d) in cases {
        let hit = segments_intersect(a, b, c, d);
        let gap = sampled_gap(a, b, c, d);
        let ok = hit == (gap < 0.01);
        all_ok &= ok;
        println!("  {name:<20} {hit:>9} {gap:>14.6}");
    }
    println!("orientation test agrees with the sampled distance on all {} cases: {all_ok}", cases.len());
    println!("4 orientation products + 4 on-segment checks → O(1), integer arithmetic only;");
    println!("the sampler (2001 points per segment, both directions) is just an independent cross-check");
}

// ─── Point in polygon ──────────────────────────────────────────────────────────

fn point_on_boundary(poly: &[Pt], p: Pt) -> bool {
    poly.iter().enumerate().any(|(i, &a)| {
        let b = poly[(i + 1) % poly.len()];
        orient(a, b, p) == 0 && on_segment(a, b, p)
    })
}

/// Even-odd rule: does the +x ray from `p` cross an odd number of edges?
fn even_odd_inside(poly: &[Pt], p: Pt) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.1 > p.1) != (b.1 > p.1) {
            let x = a.0 as f64 + (b.0 - a.0) as f64 * (p.1 - a.1) as f64 / (b.1 - a.1) as f64;
            if x > p.0 as f64 {
                inside = !inside;
            }
        }
    }
    inside
}

/// Non-zero winding rule: signed crossings wrapped around `p`.
fn winding_number(poly: &[Pt], p: Pt) -> i32 {
    let mut w = 0;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if a.1 <= p.1 {
            if b.1 > p.1 && orient(a, b, p) > 0 {
                w += 1;
            }
        } else if b.1 <= p.1 && orient(a, b, p) < 0 {
            w -= 1;
        }
    }
    w
}

pub fn point_in_polygon() {
    // L-shaped polygon (4×4 square with the top-right 2×2 corner removed).
    let poly: &[Pt] = &[(0, 0), (4, 0), (4, 2), (2, 2), (2, 4), (0, 4)];
    // (probe, expected) — `None` means the probe sits exactly on the boundary.
    let probes: &[(Pt, Option<bool>)] = &[
        ((1, 1), Some(true)),
        ((3, 1), Some(true)),
        ((1, 3), Some(true)),
        ((3, 3), Some(false)), // inside the missing corner
        ((5, 5), Some(false)),
        ((-1, 2), Some(false)),
        ((4, 4), Some(false)),
        ((2, 2), None), // vertex
        ((2, 0), None), // edge midpoint
        ((0, 2), None), // left edge
    ];

    println!("polygon counter-clockwise: {poly:?}");
    println!("  {:>3} {:>3}  {:<9} {:>8} {:>8}", "x", "y", "class", "even-odd", "winding");
    let mut ok = true;
    for &(p, expect) in probes {
        let boundary = point_on_boundary(poly, p);
        let eo = even_odd_inside(poly, p);
        let w = winding_number(poly, p);
        let class = if boundary {
            "boundary"
        } else if eo {
            "inside"
        } else {
            "outside"
        };
        let agree = match expect {
            Some(e) => !boundary && eo == e && eo == (w != 0),
            None => boundary,
        };
        ok &= agree;
        println!("  {:>3} {:>3}  {class:<9} {eo:>8} {w:>8}", p.0, p.1);
    }
    println!("matches the hand-checked answer, and even-odd ⇔ nonzero winding: {ok}");
    println!("ray casting walks the edges once → O(n) per point (the boundary pre-check is O(n) too);");
    println!("the two rules only disagree for self-intersecting polygons");
}

// ─── Closest pair of points ───────────────────────────────────────────────────

fn brute_closest(pts: &[Pt]) -> ((Pt, Pt), i64) {
    let mut best = (pts[0], pts[1]);
    let mut bd = dist2(pts[0], pts[1]);
    for i in 0..pts.len() {
        for j in i + 1..pts.len() {
            let d = dist2(pts[i], pts[j]);
            if d < bd {
                bd = d;
                best = (pts[i], pts[j]);
            }
        }
    }
    (best, bd)
}

/// `v` must be sorted by x. Returns the closest pair and its squared distance.
fn closest_rec(v: &[Pt]) -> ((Pt, Pt), i64) {
    if v.len() <= 3 {
        return brute_closest(v);
    }
    let m = v.len() / 2;
    let mid_x = v[m].0;
    let (left, right) = v.split_at(m);
    let (p1, d1) = closest_rec(left);
    let (p2, d2) = closest_rec(right);
    let (mut best, mut bd) = if d1 <= d2 { (p1, d1) } else { (p2, d2) };

    // Only pairs straddling the seam can beat `bd`, so a √bd-wide strip suffices.
    let mut strip: Vec<Pt> = v
        .iter()
        .copied()
        .filter(|&(x, _)| (x - mid_x) * (x - mid_x) < bd)
        .collect();
    strip.sort_by_key(|p| p.1);
    for i in 0..strip.len() {
        for j in i + 1..strip.len() {
            let dy = strip[j].1 - strip[i].1;
            if dy * dy >= bd {
                break; // the rest of the y-sorted strip is even farther
            }
            let d = dist2(strip[i], strip[j]);
            if d < bd {
                bd = d;
                best = (strip[i], strip[j]);
            }
        }
    }
    (best, bd)
}

pub fn closest_pair() {
    let mut seed = 0x00C0_FFEE_u64;
    let mut pts: Vec<Pt> = (0..400)
        .map(|_| {
            (
                (next_rand(&mut seed) % 1000) as i64,
                (next_rand(&mut seed) % 1000) as i64,
            )
        })
        .collect();
    pts.sort();
    pts.dedup();

    let ((a, b), bd) = closest_rec(&pts);
    let ((c, d), bbd) = brute_closest(&pts);

    println!("{} random points in [0, 1000)²", pts.len());
    println!("divide & conquer = {a:?} … {b:?}, distance = {:.6}", (bd as f64).sqrt());
    println!("brute force      = {c:?} … {d:?}, distance = {:.6}", (bbd as f64).sqrt());
    println!("identical squared distance: {}", bd == bbd);
    println!("halve by x, then scan a √d-wide strip sorted by y (≤ 7 neighbours each) → O(n log n);");
    println!("brute force compares all n(n−1)/2 = {} pairs", pts.len() * (pts.len() - 1) / 2);
}

// ─── Rotating calipers (convex polygon diameter) ──────────────────────────────

/// Antipodal-pair sweep: the farthest point rotates around the hull in one pass.
fn caliper_diameter(h: &[Pt]) -> ((Pt, Pt), i64) {
    let n = h.len();
    if n <= 2 {
        return ((h[0], h[1]), dist2(h[0], h[1]));
    }
    let mut best = 0i64;
    let mut pair = (h[0], h[1]);
    let mut j = 1usize;
    for i in 0..n {
        let i2 = (i + 1) % n;
        // Advance the antipode while the triangle (i, i+1, ·) keeps growing.
        loop {
            let j2 = (j + 1) % n;
            if orient(h[i], h[i2], h[j2]) > orient(h[i], h[i2], h[j]) {
                j = j2;
            } else {
                break;
            }
        }
        for &k in &[j, (j + 1) % n] {
            let d = dist2(h[i], h[k]);
            if d > best {
                best = d;
                pair = (h[i], h[k]);
            }
        }
    }
    (pair, best)
}

pub fn rotating_calipers() {
    let mut seed = 0xDEAD_BEEF_u64;
    let mut pts: Vec<Pt> = (0..80)
        .map(|_| {
            (
                (next_rand(&mut seed) % 1000) as i64,
                (next_rand(&mut seed) % 1000) as i64,
            )
        })
        .collect();
    pts.sort();
    pts.dedup();
    let hull = andrew_hull(&pts);

    let ((a, b), d2) = caliper_diameter(&hull);

    // Brute force over every pair of input points — the true set diameter.
    let mut brute = 0i64;
    let mut bp = (pts[0], pts[1]);
    for i in 0..pts.len() {
        for j in i + 1..pts.len() {
            let d = dist2(pts[i], pts[j]);
            if d > brute {
                brute = d;
                bp = (pts[i], pts[j]);
            }
        }
    }

    println!("{} random points → hull of {} vertices", pts.len(), hull.len());
    println!("diameter = {:.3} between {a:?} and {b:?}", (d2 as f64).sqrt());
    println!("brute force = {bp:?} (distance {:.3})", (brute as f64).sqrt());
    println!("same squared distance: {}", d2 == brute);
    println!("two antipodal pointers walk the hull once → O(h) after the O(n log n) hull, h = {} ≤ n", hull.len());
    println!("the same sweep gives the minimum width, and rotating calipers also solves");
    println!("farthest pair on two hulls, minimum/maximum distance between polygons, …");
}

// ─── Triangle centers & the Euler line ────────────────────────────────────────

/// Solve `a·x + b·y = e`, `c·x + d·y = f` (Cramer's rule).
fn solve2(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Option<(f64, f64)> {
    let det = a * d - b * c;
    if det.abs() < 1e-12 {
        return None;
    }
    Some(((e * d - b * f) / det, (a * f - e * c) / det))
}

fn dist_f(p: (f64, f64), q: (f64, f64)) -> f64 {
    ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2)).sqrt()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

pub fn triangle_centers() {
    let a = (0.0f64, 0.0);
    let b = (6.0f64, 0.0);
    let c = (2.0f64, 5.0);

    // Centroid: average of the vertices, sits 2/3 along every median.
    let g = (
        (a.0 + b.0 + c.0) / 3.0,
        (a.1 + b.1 + c.1) / 3.0,
    );
    let m_a = ((b.0 + c.0) / 2.0, (b.1 + c.1) / 2.0); // midpoint of BC
    let m_b = ((a.0 + c.0) / 2.0, (a.1 + c.1) / 2.0);
    let m_c = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let on_median = |v: (f64, f64), m: (f64, f64)| {
        let cross = (g.0 - v.0) * (m.1 - v.1) - (g.1 - v.1) * (m.0 - v.0);
        cross.abs() < 1e-9 && near(dist_f(g, v), 2.0 / 3.0 * dist_f(m, v))
    };
    let median_ok = on_median(a, m_a) && on_median(b, m_b) && on_median(c, m_c);

    // Side lengths (a opposite A, …) and the incenter (angle-bisector weights).
    let (la, lb, lc) = (dist_f(b, c), dist_f(c, a), dist_f(a, b));
    let i_pt = (
        (la * a.0 + lb * b.0 + lc * c.0) / (la + lb + lc),
        (la * a.1 + lb * b.1 + lc * c.1) / (la + lb + lc),
    );
    let area2 = ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs(); // twice the area
    let inradius = area2 / (la + lb + lc);
    let line_dist = |p: (f64, f64), u: (f64, f64), v: (f64, f64)| {
        ((v.0 - u.0) * (p.1 - u.1) - (v.1 - u.1) * (p.0 - u.0)).abs() / dist_f(u, v)
    };
    let incenter_ok = near(line_dist(i_pt, b, c), inradius)
        && near(line_dist(i_pt, c, a), inradius)
        && near(line_dist(i_pt, a, b), inradius);

    // Circumcenter: intersection of the perpendicular bisectors of AB and AC.
    let m_ab = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let m_ac = ((a.0 + c.0) / 2.0, (a.1 + c.1) / 2.0);
    let o = solve2(
        b.0 - a.0,
        b.1 - a.1,
        c.0 - a.0,
        c.1 - a.1,
        (b.0 - a.0) * m_ab.0 + (b.1 - a.1) * m_ab.1,
        (c.0 - a.0) * m_ac.0 + (c.1 - a.1) * m_ac.1,
    )
    .expect("non-degenerate triangle");
    let circum_ok = near(dist_f(o, a), dist_f(o, b)) && near(dist_f(o, b), dist_f(o, c));

    // Orthocenter: intersection of the altitudes from A and B.
    let h = solve2(
        b.0 - c.0,
        b.1 - c.1,
        a.0 - c.0,
        a.1 - c.1,
        (b.0 - c.0) * a.0 + (b.1 - c.1) * a.1,
        (a.0 - c.0) * b.0 + (a.1 - c.1) * b.1,
    )
    .expect("non-degenerate triangle");
    let dot = |p: (f64, f64), q: (f64, f64)| p.0 * q.0 + p.1 * q.1;
    let ortho_ok = dot((h.0 - a.0, h.1 - a.1), (b.0 - c.0, b.1 - c.1)).abs() < 1e-9
        && dot((h.0 - b.0, h.1 - b.1), (a.0 - c.0, a.1 - c.1)).abs() < 1e-9
        && dot((h.0 - c.0, h.1 - c.1), (a.0 - b.0, a.1 - b.1)).abs() < 1e-9;

    // Euler line: O, G, H are collinear with OG : GH = 1 : 2.
    let euler_ok = ((g.0 - o.0) * (h.1 - o.1) - (g.1 - o.1) * (h.0 - o.0)).abs() < 1e-9
        && near(dist_f(o, g) * 2.0, dist_f(g, h))
        && near(h.0, a.0 + b.0 + c.0 - 2.0 * o.0)
        && near(h.1, a.1 + b.1 + c.1 - 2.0 * o.1);

    let fmt = |p: (f64, f64)| format!("({:.4}, {:.4})", p.0, p.1);
    println!("triangle A(0,0) B(6,0) C(2,5)");
    println!("  centroid     G = {}  lies on all 3 medians at 2/3: {median_ok}", fmt(g));
    println!("  incenter     I = {}  equal distance {} to all sides: {incenter_ok}", fmt(i_pt), format!("{inradius:.4}"));
    println!("  circumcenter O = {}  |OA| = |OB| = |OC| = {:.4}: {circum_ok}", fmt(o), dist_f(o, a));
    println!("  orthocenter  H = {}  AH⟂BC, BH⟂AC, CH⟂AB: {ortho_ok}", fmt(h));
    println!("  Euler line O–G–H with OG:GH = 1:2 and H = A+B+C−2O: {euler_ok}");
    println!("all centers solve a 2×2 linear system (Cramer's rule) → O(1) with exact or float math");
}