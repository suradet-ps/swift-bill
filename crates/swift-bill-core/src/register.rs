//! Register-number parsing and formatting.
//!
//! A Thai hospital "ทะเบียนคุม" is a sequential ledger code such as
//! `69ภ12` (year 69, letter ภ, sequence 12). These helpers split it into
//! its prefix and numeric tail so callers can compute the next register
//! number and slot for a given batch of items.

/// Parse a register string like `"69ภ12"` into `(prefix, number)`.
///
/// # Examples
///
/// ```
/// use swift_bill_core::register::parse_reg_no;
///
/// let (prefix, num) = parse_reg_no("69ภ12").unwrap();
/// assert_eq!(prefix, "69ภ");
/// assert_eq!(num, 12);
/// ```
///
/// # Errors
///
/// Returns a Thai error message when the value is empty, has no numeric tail
/// (e.g. `"69ภ"`), or the tail does not fit in `u32` (e.g.
/// `"69ภ99999999999"`). A malformed register must never silently fall back
/// to book 0.
pub fn parse_reg_no(reg: &str) -> Result<(String, u32), String> {
  let trimmed = reg.trim();
  if trimmed.is_empty() {
    return Err("กรุณาระบุเลขทะเบียนคุม เช่น 69ภ12".to_string());
  }
  let last_non_digit = trimmed
    .char_indices()
    .rev()
    .find(|(_, c)| !c.is_ascii_digit());
  match last_non_digit {
    Some((idx, ch)) => {
      let prefix_end = idx + ch.len_utf8();
      let prefix = &trimmed[..prefix_end];
      let num_str = &trimmed[prefix_end..];
      let num = num_str.parse::<u32>().map_err(|_| {
        if num_str.is_empty() {
          missing_number_error(trimmed)
        } else {
          overflowing_number_error(trimmed)
        }
      })?;
      Ok((prefix.to_string(), num))
    }
    None => {
      let num = trimmed
        .parse::<u32>()
        .map_err(|_| overflowing_number_error(trimmed))?;
      Ok((String::new(), num))
    }
  }
}

fn missing_number_error(reg: &str) -> String {
  format!("เลขทะเบียนคุม \"{reg}\" ไม่ถูกต้อง: ต้องลงท้ายด้วยตัวเลข เช่น 69ภ12")
}

fn overflowing_number_error(reg: &str) -> String {
  format!("เลขทะเบียนคุม \"{reg}\" ไม่ถูกต้อง: ตัวเลขเกินค่าที่รองรับ")
}

/// Validate the starting slot within a register book.
///
/// # Errors
///
/// Returns a Thai error message when `start_running` is outside `0..=9`.
pub fn validate_start_running(start_running: u32) -> Result<(), String> {
  if start_running > 9 {
    Err(format!(
      "ลำดับเริ่มต้นในสมุดต้องอยู่ระหว่าง 0-9 (ระบุ {start_running})"
    ))
  } else {
    Ok(())
  }
}

/// Format a register string from a prefix and a number, e.g.
/// `("69ภ", 12) -> "69ภ12"`.
#[must_use]
pub fn format_reg_no(prefix: &str, number: u32) -> String {
  format!("{prefix}{number}")
}

/// Compute the register number and running position for one item in a batch.
///
/// Each register holds 10 items (positions 0–9). When the position reaches
/// 10, the register number increments and the position resets to 0.
///
/// * `item_index`    – zero-based index of the current item in this batch.
/// * `start_running` – starting position within the first register (0–9).
/// * `start_reg_num` – numeric portion of the first register number.
#[must_use]
pub fn compute_reg_for_item(item_index: u32, start_running: u32, start_reg_num: u32) -> (u32, u32) {
  let absolute_pos = start_running + item_index;
  let reg_offset = absolute_pos / 10;
  let running_in_reg = absolute_pos % 10;
  (start_reg_num + reg_offset, running_in_reg)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_with_thai_letter() {
    let (prefix, num) = parse_reg_no("69ภ12").unwrap();
    assert_eq!(prefix, "69ภ");
    assert_eq!(num, 12);
  }

  #[test]
  fn parse_pure_digits() {
    let (prefix, num) = parse_reg_no("42").unwrap();
    assert_eq!(prefix, "");
    assert_eq!(num, 42);
  }

  #[test]
  fn parse_trims_surrounding_whitespace() {
    let (prefix, num) = parse_reg_no(" 69ภ12 ").unwrap();
    assert_eq!(prefix, "69ภ");
    assert_eq!(num, 12);
  }

  #[test]
  fn parse_missing_number_is_an_error() {
    // A malformed register must never fall back to book 0.
    let err = parse_reg_no("69ภ").unwrap_err();
    assert!(err.contains("69ภ"), "error should quote the input: {err}");
    assert!(
      err.contains("ตัวเลข"),
      "error should explain the cause: {err}"
    );
    assert!(parse_reg_no("abc").is_err());
    assert!(parse_reg_no("").is_err());
    assert!(parse_reg_no("   ").is_err());
  }

  #[test]
  fn parse_overflowing_number_is_an_error() {
    let err = parse_reg_no("69ภ99999999999").unwrap_err();
    assert!(err.contains("เกินค่าที่รองรับ"), "got: {err}");
    assert!(parse_reg_no("99999999999").is_err());
  }

  #[test]
  fn validate_start_running_accepts_slots() {
    assert!(validate_start_running(0).is_ok());
    assert!(validate_start_running(9).is_ok());
  }

  #[test]
  fn validate_start_running_rejects_above_nine() {
    let err = validate_start_running(12).unwrap_err();
    assert!(err.contains("0-9"), "got: {err}");
    assert!(err.contains("12"), "got: {err}");
  }

  #[test]
  fn format_round_trip() {
    assert_eq!(format_reg_no("69ภ", 12), "69ภ12");
    assert_eq!(format_reg_no("69ภ", 13), "69ภ13");
  }

  #[test]
  fn compute_within_register() {
    assert_eq!(compute_reg_for_item(0, 3, 12), (12, 3));
    assert_eq!(compute_reg_for_item(6, 3, 12), (12, 9));
  }

  #[test]
  fn compute_overflows_register() {
    assert_eq!(compute_reg_for_item(7, 3, 12), (13, 0));
  }
}
