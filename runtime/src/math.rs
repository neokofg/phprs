//! Math functions for PHPRS Runtime
//!
//! Optimized implementations of common PHP math functions.

// =============================================================================
// Basic Math Functions
// =============================================================================

/// Absolute value for integers (PHP: abs)
#[no_mangle]
pub extern "C" fn rt_abs(n: i64) -> i64 {
    n.abs()
}

/// Absolute value for floats
#[no_mangle]
pub extern "C" fn rt_fabs(n: f64) -> f64 {
    n.abs()
}

/// Minimum of two integers (PHP: min)
#[no_mangle]
pub extern "C" fn rt_min(a: i64, b: i64) -> i64 {
    a.min(b)
}

/// Maximum of two integers (PHP: max)
#[no_mangle]
pub extern "C" fn rt_max(a: i64, b: i64) -> i64 {
    a.max(b)
}

/// Minimum of two floats
#[no_mangle]
pub extern "C" fn rt_fmin(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Maximum of two floats
#[no_mangle]
pub extern "C" fn rt_fmax(a: f64, b: f64) -> f64 {
    a.max(b)
}

// =============================================================================
// Rounding Functions (pure Rust implementations)
// =============================================================================

/// Round to nearest integer (PHP: round)
#[no_mangle]
pub extern "C" fn rt_round(n: f64) -> f64 {
    n.round()
}

/// Round down (PHP: floor)
#[no_mangle]
pub extern "C" fn rt_floor(n: f64) -> f64 {
    n.floor()
}

/// Round up (PHP: ceil)
#[no_mangle]
pub extern "C" fn rt_ceil(n: f64) -> f64 {
    n.ceil()
}

/// Truncate towards zero (PHP: intval behavior for floats)
#[no_mangle]
pub extern "C" fn rt_trunc(n: f64) -> f64 {
    n.trunc()
}

// =============================================================================
// Trigonometric Functions
// =============================================================================

/// Sine (PHP: sin)
#[no_mangle]
pub extern "C" fn rt_sin(n: f64) -> f64 {
    n.sin()
}

/// Cosine (PHP: cos)
#[no_mangle]
pub extern "C" fn rt_cos(n: f64) -> f64 {
    n.cos()
}

/// Tangent (PHP: tan)
#[no_mangle]
pub extern "C" fn rt_tan(n: f64) -> f64 {
    n.tan()
}

/// Arc sine (PHP: asin)
#[no_mangle]
pub extern "C" fn rt_asin(n: f64) -> f64 {
    n.asin()
}

/// Arc cosine (PHP: acos)
#[no_mangle]
pub extern "C" fn rt_acos(n: f64) -> f64 {
    n.acos()
}

/// Arc tangent (PHP: atan)
#[no_mangle]
pub extern "C" fn rt_atan(n: f64) -> f64 {
    n.atan()
}

/// Arc tangent of two variables (PHP: atan2)
#[no_mangle]
pub extern "C" fn rt_atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

// =============================================================================
// Exponential and Logarithmic Functions
// =============================================================================

/// Square root (PHP: sqrt)
#[no_mangle]
pub extern "C" fn rt_sqrt(n: f64) -> f64 {
    n.sqrt()
}

/// Exponential (PHP: exp)
#[no_mangle]
pub extern "C" fn rt_exp(n: f64) -> f64 {
    n.exp()
}

/// Natural logarithm (PHP: log)
#[no_mangle]
pub extern "C" fn rt_log(n: f64) -> f64 {
    n.ln()
}

/// Base 10 logarithm (PHP: log10)
#[no_mangle]
pub extern "C" fn rt_log10(n: f64) -> f64 {
    n.log10()
}

/// Power (PHP: pow)
#[no_mangle]
pub extern "C" fn rt_pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Integer power (faster than pow for integer exponents)
#[no_mangle]
pub extern "C" fn rt_powi(base: f64, exp: i32) -> f64 {
    base.powi(exp)
}

// =============================================================================
// Random Numbers
// =============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

// Simple xorshift64* PRNG state
static RNG_STATE: AtomicU64 = AtomicU64::new(0x853c49e6748fea9b);

/// Seed the random number generator
#[no_mangle]
pub extern "C" fn rt_srand(seed: u64) {
    // Ensure seed is not zero (would cause all-zero output)
    let seed = if seed == 0 { 1 } else { seed };
    RNG_STATE.store(seed, Ordering::Relaxed);
}

/// Generate random integer (PHP: rand)
/// Uses xorshift64* algorithm for speed
#[no_mangle]
pub extern "C" fn rt_rand() -> i64 {
    let mut state = RNG_STATE.load(Ordering::Relaxed);

    // xorshift64*
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;

    RNG_STATE.store(state, Ordering::Relaxed);

    (state.wrapping_mul(0x2545f4914f6cdd1d) >> 1) as i64
}

/// Generate random integer in range [min, max] (PHP: rand with args)
#[no_mangle]
pub extern "C" fn rt_rand_range(min: i64, max: i64) -> i64 {
    if min >= max {
        return min;
    }

    let range = (max - min + 1) as u64;
    let r = rt_rand() as u64;

    min + (r % range) as i64
}

/// Generate random float between 0 and 1
#[no_mangle]
pub extern "C" fn rt_rand_float() -> f64 {
    (rt_rand() as u64 as f64) / (i64::MAX as f64)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Check if finite (not NaN or Inf)
#[no_mangle]
pub extern "C" fn rt_is_finite(n: f64) -> bool {
    n.is_finite()
}

/// Check if NaN
#[no_mangle]
pub extern "C" fn rt_is_nan(n: f64) -> bool {
    n.is_nan()
}

/// Check if infinite
#[no_mangle]
pub extern "C" fn rt_is_infinite(n: f64) -> bool {
    n.is_infinite()
}

/// Float modulo (PHP: fmod)
#[no_mangle]
pub extern "C" fn rt_fmod(x: f64, y: f64) -> f64 {
    x % y
}

/// Hypotenuse (PHP: hypot)
#[no_mangle]
pub extern "C" fn rt_hypot(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(rt_abs(-5), 5);
        assert_eq!(rt_abs(5), 5);
        assert_eq!(rt_abs(0), 0);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(rt_min(3, 5), 3);
        assert_eq!(rt_max(3, 5), 5);
        assert_eq!(rt_min(-1, 1), -1);
        assert_eq!(rt_max(-1, 1), 1);
    }

    #[test]
    fn test_rounding() {
        assert_eq!(rt_round(3.5), 4.0);
        assert_eq!(rt_round(3.4), 3.0);
        assert_eq!(rt_floor(3.9), 3.0);
        assert_eq!(rt_ceil(3.1), 4.0);
    }

    #[test]
    fn test_trig() {
        let pi = std::f64::consts::PI;
        assert!((rt_sin(0.0) - 0.0).abs() < 1e-10);
        assert!((rt_cos(0.0) - 1.0).abs() < 1e-10);
        assert!((rt_sin(pi / 2.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sqrt_pow() {
        assert_eq!(rt_sqrt(4.0), 2.0);
        assert_eq!(rt_sqrt(9.0), 3.0);
        assert_eq!(rt_pow(2.0, 3.0), 8.0);
        assert_eq!(rt_powi(2.0, 10), 1024.0);
    }

    #[test]
    fn test_log_exp() {
        assert!((rt_exp(0.0) - 1.0).abs() < 1e-10);
        assert!((rt_log(std::f64::consts::E) - 1.0).abs() < 1e-10);
        assert!((rt_log10(100.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_rand() {
        // Test that rand produces different values
        let r1 = rt_rand();
        let r2 = rt_rand();
        let r3 = rt_rand();

        // Very unlikely all three are the same
        assert!(!(r1 == r2 && r2 == r3));

        // All should be non-negative (we shift right by 1)
        assert!(r1 >= 0);
        assert!(r2 >= 0);
        assert!(r3 >= 0);
    }

    #[test]
    fn test_rand_range() {
        for _ in 0..100 {
            let r = rt_rand_range(1, 10);
            assert!(r >= 1 && r <= 10);
        }
    }

    #[test]
    fn test_is_finite() {
        assert!(rt_is_finite(1.0));
        assert!(!rt_is_finite(f64::NAN));
        assert!(!rt_is_finite(f64::INFINITY));
        assert!(rt_is_nan(f64::NAN));
        assert!(rt_is_infinite(f64::INFINITY));
    }
}
