// =========================================================================
// Logicodex v1.30 — Math Utilities
// Stage 1 Quickfix: clamp, lerp, remap
//
// These are pure Rust functions (no FFI) used by Logicodex code and
// also available for the FFI layer. They mirror common game math
// operations found in Raylib's raymath.h.
// =========================================================================

/// Clamp a value between min and max (inclusive).
/// Returns min if v < min, max if v > max, otherwise v.
///
/// # Examples
/// ```
/// use logicodex::ffi::math::clamp;
/// assert_eq!(clamp(5, 0, 10), 5);
/// assert_eq!(clamp(-3, 0, 10), 0);
/// assert_eq!(clamp(15, 0, 10), 10);
/// ```
pub fn clamp<T: PartialOrd>(v: T, min: T, max: T) -> T {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// Linear interpolation between two values.
/// t=0.0 returns a, t=1.0 returns b.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Remap a value from one range to another.
/// value in [low1, high1] → mapped to [low2, high2]
pub fn remap(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    if high1 == low1 {
        return low2; // avoid division by zero
    }
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
}

/// Check if two float values are approximately equal.
/// Default epsilon = 1e-6.
pub fn float_equals(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-6
}

/// Check if a float value is approximately zero.
pub fn float_zero(v: f32) -> bool {
    v.abs() < 1e-6
}

/// Normalize a value to 0.0..1.0 range.
/// If high == low, returns 0.0 (avoid NaN).
pub fn normalize(value: f32, low: f32, high: f32) -> f32 {
    if high == low {
        0.0
    } else {
        clamp((value - low) / (high - low), 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_in_range() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(5.5, 0.0, 10.0), 5.5);
    }

    #[test]
    fn clamp_below_min() {
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(-1.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn clamp_above_max() {
        assert_eq!(clamp(15, 0, 10), 10);
        assert_eq!(clamp(11.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn lerp_basic() {
        assert!(float_equals(lerp(0.0, 10.0, 0.0), 0.0));
        assert!(float_equals(lerp(0.0, 10.0, 0.5), 5.0));
        assert!(float_equals(lerp(0.0, 10.0, 1.0), 10.0));
    }

    #[test]
    fn remap_basic() {
        assert!(float_equals(remap(5.0, 0.0, 10.0, 0.0, 100.0), 50.0));
        assert!(float_equals(remap(0.0, 0.0, 10.0, 0.0, 100.0), 0.0));
        assert!(float_equals(remap(10.0, 0.0, 10.0, 0.0, 100.0), 100.0));
    }

    #[test]
    fn remap_divide_by_zero() {
        // When low1 == high1, return low2 (avoid NaN)
        assert!(float_equals(remap(5.0, 10.0, 10.0, 0.0, 100.0), 0.0));
    }

    #[test]
    fn normalize_basic() {
        assert!(float_equals(normalize(5.0, 0.0, 10.0), 0.5));
        assert!(float_equals(normalize(0.0, 0.0, 10.0), 0.0));
        assert!(float_equals(normalize(10.0, 0.0, 10.0), 1.0));
    }

    #[test]
    fn normalize_clamps() {
        assert!(float_equals(normalize(-5.0, 0.0, 10.0), 0.0));
        assert!(float_equals(normalize(15.0, 0.0, 10.0), 1.0));
    }

    #[test]
    fn float_equals_precision() {
        assert!(float_equals(0.1 + 0.2, 0.3)); // classic floating point
        assert!(!float_equals(0.1, 0.2));
    }
}
