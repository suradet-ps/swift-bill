//! Excel (.xlsx) export for the Invoice Submission and Receiving Summary
//! reports. Pure Rust via `rust_xlsxwriter` — no external Office process
//! required.

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod invoice_submission;
mod receiving_summary;

pub use invoice_submission::generate_invoice_submission_excel;
pub use receiving_summary::generate_receiving_summary_excel;

/// Map a [`XlsxError`](rust_xlsxwriter::XlsxError) into a plain `String`
/// so callers can return `Result<_, String>` without dragging the
/// dependency into their own error types.
#[inline]
pub(crate) fn map_xlsx_err(err: rust_xlsxwriter::XlsxError) -> String {
  format!("Excel error: {err}")
}

/// Full Thai month name for a 1-based month number (1–12).
/// Returns `""` for out-of-range inputs.
#[must_use]
pub(crate) fn thai_month(m: u32) -> &'static str {
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
    _ => "",
  }
}
