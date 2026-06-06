//! Thai date formatting helpers.

use chrono::{Datelike, NaiveDate};

/// Format a [`NaiveDate`] as a Thai short date string, e.g. `5 ม.ค. 69`.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use swift_bill_core::date::format_thai_date;
///
/// let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
/// assert_eq!(format_thai_date(&d), "5 ม.ค. 69");
/// ```
#[must_use]
pub fn format_thai_date(date: &NaiveDate) -> String {
  const THAI_MONTHS: &[&str] = &[
    "ม.ค.",
    "ก.พ.",
    "มี.ค.",
    "เม.ย.",
    "พ.ค.",
    "มิ.ย.",
    "ก.ค.",
    "ส.ค.",
    "ก.ย.",
    "ต.ค.",
    "พ.ย.",
    "ธ.ค.",
  ];
  let buddhist_year = date.year() + 543;
  let short_year = buddhist_year % 100;
  format!(
    "{} {} {}",
    date.day(),
    THAI_MONTHS[date.month0() as usize],
    short_year
  )
}

/// Format a [`NaiveDate`] as a Thai full date string, e.g. `5 มกราคม 2569`.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use swift_bill_core::date::format_thai_date_full;
///
/// let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
/// assert_eq!(format_thai_date_full(&d), "5 มกราคม 2569");
/// ```
#[must_use]
pub fn format_thai_date_full(date: &NaiveDate) -> String {
  const THAI_MONTHS: &[&str] = &[
    "มกราคม",
    "กุมภาพันธ์",
    "มีนาคม",
    "เมษายน",
    "พฤษภาคม",
    "มิถุนายน",
    "กรกฎาคม",
    "สิงหาคม",
    "กันยายน",
    "ตุลาคม",
    "พฤศจิกายน",
    "ธันวาคม",
  ];
  let buddhist_year = date.year() + 543;
  format!(
    "{} {} {}",
    date.day(),
    THAI_MONTHS[date.month0() as usize],
    buddhist_year
  )
}

/// Full Thai month name for a 1-based month number (1–12).
/// Returns `"ไม่ทราบ"` for any out-of-range input.
#[must_use]
pub fn thai_month(m: u32) -> &'static str {
  match m {
    1 => "มกราคม",
    2 => "กุมภาพันธ์",
    3 => "มีนาคม",
    4 => "เมษายน",
    5 => "พฤษภาคม",
    6 => "มิถุนายน",
    7 => "กรกฎาคม",
    8 => "สิงหาคม",
    9 => "กันยายน",
    10 => "ตุลาคม",
    11 => "พฤศจิกายน",
    12 => "ธันวาคม",
    _ => "ไม่ทราบ",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn short_format() {
    let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    assert_eq!(format_thai_date(&d), "5 ม.ค. 69");
  }

  #[test]
  fn short_format_year_end() {
    let d = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
    assert_eq!(format_thai_date(&d), "31 ธ.ค. 69");
  }

  #[test]
  fn full_format() {
    let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    assert_eq!(format_thai_date_full(&d), "5 มกราคม 2569");
  }

  #[test]
  fn thai_month_in_range() {
    assert_eq!(thai_month(1), "มกราคม");
    assert_eq!(thai_month(12), "ธันวาคม");
  }

  #[test]
  fn thai_month_out_of_range() {
    assert_eq!(thai_month(0), "ไม่ทราบ");
    assert_eq!(thai_month(13), "ไม่ทราบ");
  }
}
