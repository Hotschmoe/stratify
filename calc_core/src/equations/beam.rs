//! # Simply-Supported Beam Formulas
//!
//! Fundamental equations for simply-supported beams under various loading conditions.
//! All formulas assume a beam with pin support at left (x=0) and roller at right (x=L).
//!
//! ## Notation
//!
//! - `L` = Span length
//! - `x` = Position along beam from left support
//! - `a` = Load position from left support
//! - `P` = Point load magnitude
//! - `w` = Uniform load intensity (force per unit length)
//! - `M` = Bending moment
//! - `V` = Shear force
//! - `δ` = Deflection
//! - `E` = Modulus of elasticity
//! - `I` = Moment of inertia
//! - `R1` = Left reaction, `R2` = Right reaction
//!
//! ## Sign Conventions
//!
//! - Loads: Positive downward
//! - Moment: Positive causes tension on bottom (sagging)
//! - Shear: Positive when left side up relative to right
//! - Deflection: Positive downward
//! - Reactions: Positive upward
//!
//! ## References
//!
//! - Roark's Formulas for Stress and Strain, 8th Edition, Table 8.1
//! - Structural Analysis by R.C. Hibbeler

// =============================================================================
// POINT LOAD FORMULAS
// Simply-supported beam with concentrated load P at distance 'a' from left
// =============================================================================

/// Calculate reactions for point load P at position a on span L
///
/// ```text
///        P
///        ↓
///    ────┬────────────
///    △   a            △
///   R1  ←───────L────→ R2
/// ```
///
/// # Formulas (Roark's Table 8.1, Case 1a)
/// - R1 = P(L-a)/L
/// - R2 = Pa/L
///
/// # Arguments
/// * `p` - Point load magnitude (positive downward)
/// * `a` - Distance from left support to load
/// * `l` - Span length
///
/// # Returns
/// (R1, R2) - Left and right reactions (positive upward)
#[inline]
pub fn point_load_reactions(p: f64, a: f64, l: f64) -> (f64, f64) {
    let r1 = p * (l - a) / l;
    let r2 = p * a / l;
    (r1, r2)
}

/// Calculate shear at position x for point load P at position a
///
/// # Formulas
/// - V(x) = R1           for x < a
/// - V(x) = R1 - P       for x ≥ a
///
/// where R1 = P(L-a)/L
#[inline]
pub fn point_load_shear(p: f64, a: f64, l: f64, x: f64) -> f64 {
    let (r1, _) = point_load_reactions(p, a, l);
    if x < a {
        r1
    } else {
        r1 - p
    }
}

/// Calculate moment at position x for point load P at position a
///
/// # Formulas (Roark's Table 8.1, Case 1a)
/// - M(x) = R1·x           for x ≤ a
/// - M(x) = R1·x - P(x-a)  for x > a
///
/// Maximum moment occurs at the load point:
/// - M_max = Pa(L-a)/L = R1·a = R2·(L-a)
#[inline]
pub fn point_load_moment(p: f64, a: f64, l: f64, x: f64) -> f64 {
    let (r1, _) = point_load_reactions(p, a, l);
    if x <= a {
        r1 * x
    } else {
        r1 * x - p * (x - a)
    }
}

/// Calculate deflection at position x for point load P at position a
///
/// # Formulas (Roark's Table 8.1, Case 1a)
///
/// For x ≤ a:
/// ```text
/// δ(x) = Pbx(L² - b² - x²) / (6EIL)
/// ```
///
/// For x > a:
/// ```text
/// δ(x) = Pa(L-x)(2Lx - x² - a²) / (6EIL)
/// ```
///
/// where b = L - a
///
/// # Arguments
/// * `p` - Point load (positive downward)
/// * `a` - Distance from left support to load
/// * `l` - Span length
/// * `x` - Position to calculate deflection
/// * `e` - Modulus of elasticity
/// * `i` - Moment of inertia
///
/// # Returns
/// Deflection (positive downward)
#[inline]
pub fn point_load_deflection(p: f64, a: f64, l: f64, x: f64, e: f64, i: f64) -> f64 {
    let b = l - a;
    let ei = e * i;

    if x <= a {
        // δ = Pbx(L² - b² - x²) / (6EIL)
        p * b * x * (l * l - b * b - x * x) / (6.0 * ei * l)
    } else {
        // δ = Pa(L-x)(2Lx - x² - a²) / (6EIL)
        p * a * (l - x) * (2.0 * l * x - x * x - a * a) / (6.0 * ei * l)
    }
}

/// Maximum deflection for point load at any position
///
/// For load at midspan (a = L/2):
/// ```text
/// δ_max = PL³ / (48EI)   at x = L/2
/// ```
///
/// For load not at midspan, max deflection is NOT at the load point.
/// Location of max deflection (Roark's):
/// ```text
/// x_max = √((L² - a²)/3)   for a < L/2
/// ```
#[inline]
pub fn point_load_max_deflection_midspan(p: f64, l: f64, e: f64, i: f64) -> f64 {
    p * l.powi(3) / (48.0 * e * i)
}

// =============================================================================
// UNIFORM LOAD FORMULAS
// Simply-supported beam with uniform load w over entire span
// =============================================================================

/// Calculate reactions for uniform load w over full span L
///
/// ```text
///    ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓ w
///    ═════════════════
///    △                △
///   R1  ←─────L─────→ R2
/// ```
///
/// # Formula
/// R1 = R2 = wL/2
#[inline]
pub fn uniform_load_reactions(w: f64, l: f64) -> (f64, f64) {
    let r = w * l / 2.0;
    (r, r)
}

/// Calculate shear at position x for uniform load w over full span
///
/// # Formula
/// V(x) = wL/2 - wx = w(L/2 - x)
///
/// - At x=0: V = +wL/2 (positive, left side up)
/// - At x=L/2: V = 0
/// - At x=L: V = -wL/2 (negative, right side up)
#[inline]
pub fn uniform_load_shear(w: f64, l: f64, x: f64) -> f64 {
    w * (l / 2.0 - x)
}

/// Calculate moment at position x for uniform load w over full span
///
/// # Formula (Roark's Table 8.1, Case 2a)
/// M(x) = wx(L-x)/2
///
/// Maximum moment at midspan:
/// M_max = wL²/8  at x = L/2
#[inline]
pub fn uniform_load_moment(w: f64, l: f64, x: f64) -> f64 {
    w * x * (l - x) / 2.0
}

/// Maximum moment for uniform load
///
/// # Formula
/// M_max = wL²/8
#[inline]
pub fn uniform_load_max_moment(w: f64, l: f64) -> f64 {
    w * l * l / 8.0
}

/// Calculate deflection at position x for uniform load w
///
/// # Formula (Roark's Table 8.1, Case 2a)
/// δ(x) = wx(L³ - 2Lx² + x³) / (24EI)
///
/// # Note
/// Maximum deflection at midspan:
/// δ_max = 5wL⁴ / (384EI)
#[inline]
pub fn uniform_load_deflection(w: f64, l: f64, x: f64, e: f64, i: f64) -> f64 {
    w * x * (l.powi(3) - 2.0 * l * x * x + x.powi(3)) / (24.0 * e * i)
}

/// Maximum deflection for uniform load (at midspan)
///
/// # Formula
/// δ_max = 5wL⁴ / (384EI)
#[inline]
pub fn uniform_load_max_deflection(w: f64, l: f64, e: f64, i: f64) -> f64 {
    5.0 * w * l.powi(4) / (384.0 * e * i)
}

// =============================================================================
// PARTIAL UNIFORM LOAD FORMULAS
// Simply-supported beam with uniform load w from position a to b
// =============================================================================

/// Calculate reactions for partial uniform load w from a to b
///
/// ```text
///          ↓↓↓↓↓↓↓↓↓ w
///    ══════════════════
///    △     a     b     △
///   R1  ←─────L─────→ R2
/// ```
///
/// # Formulas
/// Total load W = w(b-a)
/// Centroid at c = (a+b)/2
/// - R1 = W(L-c)/L
/// - R2 = Wc/L
#[inline]
pub fn partial_uniform_reactions(w: f64, a: f64, b: f64, l: f64) -> (f64, f64) {
    let total_load = w * (b - a);
    let centroid = (a + b) / 2.0;
    let r1 = total_load * (l - centroid) / l;
    let r2 = total_load * centroid / l;
    (r1, r2)
}

/// Calculate shear at position x for partial uniform load
///
/// # Formulas
/// - V(x) = R1                     for x ≤ a
/// - V(x) = R1 - w(x-a)            for a < x < b
/// - V(x) = R1 - w(b-a) = -R2      for x ≥ b
#[inline]
pub fn partial_uniform_shear(w: f64, a: f64, b: f64, l: f64, x: f64) -> f64 {
    let (r1, _) = partial_uniform_reactions(w, a, b, l);

    if x <= a {
        r1
    } else if x >= b {
        r1 - w * (b - a)
    } else {
        r1 - w * (x - a)
    }
}

/// Calculate moment at position x for partial uniform load
///
/// # Formulas
/// - M(x) = R1·x                           for x ≤ a
/// - M(x) = R1·x - w(x-a)²/2               for a < x < b
/// - M(x) = R1·x - W(x-c)                  for x ≥ b
///   where W = w(b-a), c = (a+b)/2
#[inline]
pub fn partial_uniform_moment(w: f64, a: f64, b: f64, l: f64, x: f64) -> f64 {
    let (r1, _) = partial_uniform_reactions(w, a, b, l);

    if x <= a {
        r1 * x
    } else if x >= b {
        let total_load = w * (b - a);
        let centroid = (a + b) / 2.0;
        r1 * x - total_load * (x - centroid)
    } else {
        r1 * x - w * (x - a).powi(2) / 2.0
    }
}

// =============================================================================
// APPLIED MOMENT FORMULAS
// Simply-supported beam with applied moment M0 at position a
// =============================================================================

/// Calculate reactions for applied moment M0 at position a
///
/// # Formulas
/// - R1 = -M0/L (downward if M0 is counterclockwise)
/// - R2 = +M0/L (upward if M0 is counterclockwise)
///
/// Note: Moment creates a couple, no net vertical force
#[inline]
pub fn applied_moment_reactions(m0: f64, l: f64) -> (f64, f64) {
    let r1 = -m0 / l;
    let r2 = m0 / l;
    (r1, r2)
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON || (a - b).abs() / b.abs().max(1.0) < 0.001
    }

    // Point load tests
    #[test]
    fn test_point_load_midspan_reactions() {
        // 10 ft beam, 1000 lb at midspan
        let (r1, r2) = point_load_reactions(1000.0, 5.0, 10.0);
        assert!(approx_eq(r1, 500.0), "R1 = {}", r1);
        assert!(approx_eq(r2, 500.0), "R2 = {}", r2);
    }

    #[test]
    fn test_point_load_asymmetric_reactions() {
        // 10 ft beam, 1000 lb at 3 ft from left
        let (r1, r2) = point_load_reactions(1000.0, 3.0, 10.0);
        assert!(approx_eq(r1, 700.0), "R1 = {} (expected 700)", r1);
        assert!(approx_eq(r2, 300.0), "R2 = {} (expected 300)", r2);
    }

    #[test]
    fn test_point_load_moment_max() {
        // 10 ft beam, 1000 lb at midspan
        // M_max = PL/4 = 1000 * 10 / 4 = 2500 ft-lb
        let m = point_load_moment(1000.0, 5.0, 10.0, 5.0);
        assert!(approx_eq(m, 2500.0), "M = {} (expected 2500)", m);
    }

    #[test]
    fn test_point_load_moment_zero_at_supports() {
        let m0 = point_load_moment(1000.0, 5.0, 10.0, 0.0);
        let ml = point_load_moment(1000.0, 5.0, 10.0, 10.0);
        assert!(approx_eq(m0, 0.0), "M(0) = {} (expected 0)", m0);
        assert!(approx_eq(ml, 0.0), "M(L) = {} (expected 0)", ml);
    }

    // Uniform load tests
    #[test]
    fn test_uniform_load_reactions() {
        // 10 ft beam, 100 plf
        let (r1, r2) = uniform_load_reactions(100.0, 10.0);
        assert!(approx_eq(r1, 500.0), "R1 = {}", r1);
        assert!(approx_eq(r2, 500.0), "R2 = {}", r2);
    }

    #[test]
    fn test_uniform_load_max_moment() {
        // 10 ft beam, 100 plf
        // M_max = wL²/8 = 100 * 100 / 8 = 1250 ft-lb
        let m = uniform_load_max_moment(100.0, 10.0);
        assert!(approx_eq(m, 1250.0), "M_max = {} (expected 1250)", m);
    }

    #[test]
    fn test_uniform_load_shear_at_supports() {
        // V at x=0 should be +wL/2
        // V at x=L should be -wL/2
        let v0 = uniform_load_shear(100.0, 10.0, 0.0);
        let vl = uniform_load_shear(100.0, 10.0, 10.0);
        assert!(approx_eq(v0, 500.0), "V(0) = {}", v0);
        assert!(approx_eq(vl, -500.0), "V(L) = {}", vl);
    }

    #[test]
    fn test_uniform_load_shear_at_midspan() {
        // V at midspan should be 0
        let v = uniform_load_shear(100.0, 10.0, 5.0);
        assert!(approx_eq(v, 0.0), "V(L/2) = {}", v);
    }

    // Partial uniform load tests
    #[test]
    fn test_partial_uniform_symmetric() {
        // 10 ft beam, 100 plf from 2 to 8 ft (symmetric)
        // Total load = 100 * 6 = 600 lb, centroid at 5 ft
        // R1 = R2 = 300 lb
        let (r1, r2) = partial_uniform_reactions(100.0, 2.0, 8.0, 10.0);
        assert!(approx_eq(r1, 300.0), "R1 = {} (expected 300)", r1);
        assert!(approx_eq(r2, 300.0), "R2 = {} (expected 300)", r2);
    }

    // Superposition principle test
    #[test]
    fn test_superposition() {
        // Two point loads should superimpose
        let l = 10.0;
        let p1 = 500.0;
        let a1 = 3.0;
        let p2 = 500.0;
        let a2 = 7.0;

        let (r1_1, r2_1) = point_load_reactions(p1, a1, l);
        let (r1_2, r2_2) = point_load_reactions(p2, a2, l);

        // Combined reaction
        let r1_total = r1_1 + r1_2;
        let r2_total = r2_1 + r2_2;

        // Should equal sum of individual reactions
        // P1 at 3ft: R1 = 500*7/10 = 350, R2 = 150
        // P2 at 7ft: R1 = 500*3/10 = 150, R2 = 350
        // Total: R1 = R2 = 500
        assert!(approx_eq(r1_total, 500.0), "R1_total = {}", r1_total);
        assert!(approx_eq(r2_total, 500.0), "R2_total = {}", r2_total);
    }
}
