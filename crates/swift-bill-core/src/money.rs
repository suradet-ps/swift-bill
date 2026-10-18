//! Money rounding helpers.
//!
//! Amounts travel through the app as `f64` (database, JSON history, frontend),
//! but every calculation boundary rounds to satang (2 decimal places).
//! Chains of additions and subtractions then stay exact: a balance that
//! should be 0.00 is exactly 0.00, and the printed chain always agrees with
//! the value carried into the next round.

/// Convert baht to satang, rounding half away from zero.
///
/// The conversion runs on the shortest decimal representation of the value
/// (`1.005` stays `1.005`), so decimal inputs round exactly as written
/// instead of through their binary approximation (`1.005 * 100.0` is
/// `100.4999...`, which would round down). Non-finite input is treated as
/// zero, and absurdly large values saturate.
#[must_use]
pub fn to_satang(value: f64) -> i64 {
  if !value.is_finite() {
    return 0;
  }
  let text = format!("{value}");
  let (sign, digits) = text
    .strip_prefix('-')
    .map_or((1_i128, text.as_str()), |rest| (-1_i128, rest));
  let (int_text, frac_text) = digits.split_once('.').unwrap_or((digits, ""));

  let Ok(int_part) = int_text.parse::<i128>() else {
    return if sign < 0 { i64::MIN } else { i64::MAX };
  };
  let mut satang = int_part.saturating_mul(100);

  let frac = frac_text.as_bytes();
  let tenths = frac.first().map_or(0, |b| i128::from(b - b'0'));
  let hundredths = frac.get(1).map_or(0, |b| i128::from(b - b'0'));
  satang = satang.saturating_add(tenths * 10 + hundredths);

  // Half away from zero: the first dropped decimal digit decides, since any
  // value from x.xx5 up is at least half a satang and anything below is less.
  if frac.get(2).is_some_and(|digit| *digit >= b'5') {
    satang = satang.saturating_add(1);
  }

  let signed = satang.saturating_mul(sign);
  i64::try_from(signed).unwrap_or(if sign < 0 { i64::MIN } else { i64::MAX })
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

  /// Decimal ties round half away from zero even though the binary value is
  /// slightly below the exact half (1.005 * 100.0 = 100.4999...).
  #[test]
  fn decimal_ties_round_half_away_from_zero() {
    assert_eq!(to_satang(1.005), 101);
    assert_eq!(to_satang(-1.005), -101);
    assert_eq!(to_satang(2.675), 268);
    assert_eq!(round_satang(1.005), 1.01);
    assert_eq!(round_satang(-1.005), -1.01);
    assert_eq!(round_satang(0.005), 0.01);
  }

  #[test]
  fn decimals_below_and_above_the_tie_round_to_nearest() {
    assert_eq!(to_satang(1.004), 100);
    assert_eq!(to_satang(1.006), 101);
    assert_eq!(to_satang(0.004), 0);
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
  fn extreme_values_saturate_or_zero() {
    assert_eq!(to_satang(1e300), i64::MAX);
    assert_eq!(to_satang(-1e300), i64::MIN);
    assert_eq!(to_satang(1e-7), 0);
    assert_eq!(to_satang(-0.0), 0);
  }

  #[test]
  fn satang_eq_ignores_sub_satang_noise() {
    assert!(satang_eq(3_850_000.004, 3_850_000.0));
    assert!(!satang_eq(3_850_000.01, 3_850_000.0));
  }
}
