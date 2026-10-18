//! Money rounding helpers.
//!
//! Amounts travel through the app as `f64` (database, JSON history, frontend),
//! but every calculation boundary rounds to satang (2 decimal places).
//! Chains of additions and subtractions then stay exact: a balance that
//! should be 0.00 is exactly 0.00, and the printed chain always agrees with
//! the value carried into the next round.

/// Convert baht to satang, rounding half away from zero.
///
/// The input is a raw amount from the database or the UI; binary float noise
/// a few orders of magnitude below one satang is absorbed by the rounding.
/// Non-finite input is treated as zero.
// Saturation on absurdly large values is intended; realistic amounts round exactly.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn to_satang(value: f64) -> i64 {
  if value.is_finite() {
    (value * 100.0).round() as i64
  } else {
    0
  }
}

/// Convert satang back to baht.
// Amounts stay far below 2^52 satang (about 90 trillion baht), so this is exact.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn from_satang(satang: i64) -> f64 {
  satang as f64 / 100.0
}

/// Round `value` to the nearest satang, half away from zero.
#[must_use]
pub fn round_satang(value: f64) -> f64 {
  from_satang(to_satang(value))
}

/// `true` when two amounts are equal at satang precision.
///
/// Used instead of a tolerance so float noise stored by older rounds does not
/// trigger a false mismatch, while a genuine one-satang difference still
/// does.
#[must_use]
pub fn satang_eq(a: f64, b: f64) -> bool {
  to_satang(a) == to_satang(b)
}

#[cfg(test)]
// Exact float equality is the contract these tests verify.
#[allow(clippy::float_cmp)]
mod tests {
  use super::*;

  #[test]
  fn rounds_float_noise_to_satang() {
    assert_eq!(round_satang(0.1 + 0.2), 0.3);
    assert_eq!(round_satang(10.0 - 3.33), 6.67);
    assert_eq!(round_satang(1.004), 1.0);
    assert_eq!(round_satang(1.006), 1.01);
  }

  #[test]
  fn rounds_signed_values_away_from_zero() {
    assert_eq!(round_satang(-1.006), -1.01);
    assert_eq!(round_satang(-0.004), -0.0);
  }

  #[test]
  fn satang_round_trip() {
    assert_eq!(to_satang(1234.56), 123_456);
    assert_eq!(from_satang(123_456), 1234.56);
    assert_eq!(round_satang(0.0), 0.0);
  }

  #[test]
  fn non_finite_input_is_zero() {
    assert_eq!(to_satang(f64::NAN), 0);
    assert_eq!(to_satang(f64::INFINITY), 0);
    assert_eq!(to_satang(f64::NEG_INFINITY), 0);
  }

  #[test]
  fn satang_eq_ignores_sub_satang_noise() {
    assert!(satang_eq(3_850_000.004, 3_850_000.0));
    assert!(!satang_eq(3_850_000.01, 3_850_000.0));
  }
}
