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
/// let (prefix, num) = parse_reg_no("69ภ12");
/// assert_eq!(prefix, "69ภ");
/// assert_eq!(num, 12);
/// ```
#[must_use]
pub fn parse_reg_no(reg: &str) -> (String, u32) {
  let last_non_digit = reg.char_indices().rev().find(|(_, c)| !c.is_ascii_digit());
  match last_non_digit {
    Some((idx, ch)) => {
      let prefix_end = idx + ch.len_utf8();
      let prefix = &reg[..prefix_end];
      let num_str = &reg[prefix_end..];
      let num = num_str.parse::<u32>().unwrap_or(0);
      (prefix.to_string(), num)
    }
    None => {
      let num = reg.parse::<u32>().unwrap_or(0);
      (String::new(), num)
    }
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
    let (prefix, num) = parse_reg_no("69ภ12");
    assert_eq!(prefix, "69ภ");
    assert_eq!(num, 12);
  }

  #[test]
  fn parse_pure_digits() {
    let (prefix, num) = parse_reg_no("42");
    assert_eq!(prefix, "");
    assert_eq!(num, 42);
  }

  #[test]
  fn parse_unparseable_returns_zero() {
    let (prefix, num) = parse_reg_no("abc");
    assert_eq!(prefix, "abc");
    assert_eq!(num, 0);
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
