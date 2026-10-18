//! Business logic for the three report types:
//!
//! 1. Invoice Submission List (ส่งหนี้เบิกยา)
//! 2. Receiving Summary (สรุปรับยา)
//! 3. Cover Letters (เบิกยาปะหน้า)

use crate::date::{format_thai_date, format_thai_date_full};
use crate::models::{
  CoverLetterPage, CoverLettersParams, InvoiceRow, InvoiceSubmissionParams, InvoiceSubmissionRow,
  ReceivingNumberAssignment, ReceivingSummaryParams, ReceivingSummaryRow,
};
use crate::money::round_satang;
use crate::register::{compute_reg_for_item, format_reg_no, parse_reg_no, validate_start_running};

// 1. Invoice Submission List (ส่งหนี้เบิกยา)

/// Build the Invoice Submission rows for a batch of invoices.
///
/// Each row is assigned a sequential register number (`start_reg_no` plus
/// the per-batch offset) and a running slot within the register (0–9).
///
/// # Errors
///
/// Returns a Thai error message when the register number is malformed or the
/// starting slot is outside `0..=9`.
pub fn process_invoice_submission(
  invoices: &[InvoiceRow],
  params: &InvoiceSubmissionParams,
) -> Result<Vec<InvoiceSubmissionRow>, String> {
  let (reg_prefix, reg_start_num) = parse_reg_no(&params.start_reg_no)?;
  validate_start_running(params.start_running)?;
  let mut rows: Vec<InvoiceSubmissionRow> = Vec::with_capacity(invoices.len());

  for (i, inv) in invoices.iter().enumerate() {
    let seq = (i as u32) + 1;
    let (reg_num, running) = compute_reg_for_item(i as u32, params.start_running, reg_start_num);
    let reg_no = format_reg_no(&reg_prefix, reg_num);

    let receive_date_str = format_thai_date(&inv.receive_date);
    let invoice_date_str = receive_date_str.clone();

    rows.push(InvoiceSubmissionRow {
      seq,
      receive_date: receive_date_str,
      invoice_no: inv.invoice_no.clone(),
      reg_no,
      running_in_reg: running,
      invoice_date: invoice_date_str,
      company_name: inv.company_name.clone(),
      category: inv.category.clone(),
      total_amount: round_satang(inv.total_cost),
    });
  }

  Ok(rows)
}

// 2. Receiving Summary (สรุปรับยา)

/// Build the Receiving Summary rows for a batch of invoices using a
/// deterministic per-row number assignment.
///
/// # Errors
///
/// Returns a Thai error message when the register number is malformed, the
/// starting slot is outside `0..=9`, or a start number is 0.
pub fn process_receiving_summary(
  invoices: &[InvoiceRow],
  params: &ReceivingSummaryParams,
) -> Result<Vec<ReceivingSummaryRow>, String> {
  crate::numbering::validate_receiving_start_numbers(params.start_po_no, params.start_purchase_no)?;

  let assignments: Vec<ReceivingNumberAssignment> = invoices
    .iter()
    .enumerate()
    .map(|(index, _)| ReceivingNumberAssignment {
      request_no: params.start_po_no + (index as u32 * 2),
      report_no: params.start_po_no + (index as u32 * 2) + 1,
      purchase_no: params.start_purchase_no + index as u32,
    })
    .collect();

  process_receiving_summary_with_numbers(invoices, params, &assignments)
}

/// Build the Receiving Summary rows using caller-supplied number assignments
/// (e.g. produced by [`crate::numbering::allocate_receiving_numbers`] which
/// honours locked number sets).
///
/// The approval date and the purchase-order number come from `MS_PO`
/// (`REQ_APP_DATE` and `REF_NO`) via the linked [`InvoiceRow`]; the assigned
/// numbers are only a fallback for invoices without a linked purchase order.
///
/// # Errors
///
/// Returns a Thai error message when the register number is malformed or the
/// starting slot is outside `0..=9`.
pub fn process_receiving_summary_with_numbers(
  invoices: &[InvoiceRow],
  params: &ReceivingSummaryParams,
  assignments: &[ReceivingNumberAssignment],
) -> Result<Vec<ReceivingSummaryRow>, String> {
  let (reg_prefix, reg_start_num) = parse_reg_no(&params.start_reg_no)?;
  validate_start_running(params.start_running)?;
  let mut rows: Vec<ReceivingSummaryRow> = Vec::with_capacity(invoices.len());

  // Safety net when MS_PO carries no REQ_APP_DATE: reuse the first invoice's
  // receive date instead of leaving the column blank.
  let fallback_approval_date = invoices
    .first()
    .map(|inv| format_thai_date(&inv.receive_date))
    .unwrap_or_default();

  // Purchase-order sequences already reserved by linked MS_PO references.
  // Fallback counters for unlinked invoices must skip them so no row shows
  // a purchase-order number twice.
  let linked_sequences: Vec<u32> = invoices
    .iter()
    .filter_map(|inv| inv.po_ref_no.as_deref().and_then(po_sequence_number))
    .collect();
  let mut used_fallbacks: Vec<u32> = Vec::with_capacity(invoices.len());

  for (i, (inv, assigned)) in invoices.iter().zip(assignments.iter()).enumerate() {
    let (reg_num, running) = compute_reg_for_item(i as u32, params.start_running, reg_start_num);
    let reg_no = format_reg_no(&reg_prefix, reg_num);

    let receive_date_str = format_thai_date(&inv.receive_date);
    let approval_date_str = inv
      .req_app_date
      .as_ref()
      .map_or_else(|| fallback_approval_date.clone(), format_thai_date);
    let po_no = inv
      .po_ref_no
      .as_deref()
      .and_then(po_sequence_text)
      .unwrap_or_else(|| {
        next_free_purchase_no(assigned.purchase_no, &linked_sequences, &mut used_fallbacks)
      });

    let receiving_code = (i as u32) + 1;

    rows.push(ReceivingSummaryRow {
      approval_date: approval_date_str.clone(),
      po_date: approval_date_str,
      receive_date: receive_date_str,
      company_code: inv.company_keyword.clone(),
      total_amount: round_satang(inv.total_cost),
      receiving_code,
      reg_no,
      running_in_reg: running,
      invoice_no: inv.invoice_no.clone(),
      request_no: assigned.request_no,
      report_no: assigned.report_no,
      po_no,
    });
  }

  Ok(rows)
}

/// First counter value not reserved by a linked PO reference or used by an
/// earlier fallback row.
fn next_free_purchase_no(candidate: u32, linked: &[u32], used: &mut Vec<u32>) -> String {
  let mut candidate = candidate;
  while linked.contains(&candidate) || used.contains(&candidate) {
    candidate += 1;
  }
  used.push(candidate);
  candidate.to_string()
}

/// Sequence part of a `MS_PO.REF_NO` reference.
///
/// INVS stores references as `"001/2570"` (sequence before the slash) or
/// `"2569-0001"` (sequence after the dash). The year is supplied by the
/// report header, so only the sequence is returned, e.g. `"001"`.
#[must_use]
pub fn po_sequence_text(ref_no: &str) -> Option<String> {
  let trimmed = ref_no.trim();
  if trimmed.is_empty() {
    return None;
  }
  let sequence = if let Some((before_slash, _)) = trimmed.split_once('/') {
    before_slash.trim()
  } else if let Some((_, after_dash)) = trimmed.rsplit_once('-') {
    after_dash.trim()
  } else {
    trimmed
  };
  if sequence.is_empty() {
    None
  } else {
    Some(sequence.to_string())
  }
}

/// Numeric form of [`po_sequence_text`], used to continue the purchase-order
/// counter from the numbers `MS_PO` has already issued.
#[must_use]
pub fn po_sequence_number(ref_no: &str) -> Option<u32> {
  po_sequence_text(ref_no).and_then(|text| text.parse::<u32>().ok())
}

/// Purchase-order number to save for the next round.
///
/// One past the highest sequence already issued in `rows` (linked `MS_PO`
/// references and fallback counters alike), and never behind `allocated_next`.
#[must_use]
pub fn next_purchase_no(rows: &[ReceivingSummaryRow], allocated_next: u32) -> u32 {
  rows
    .iter()
    .filter_map(|row| po_sequence_number(&row.po_no))
    .max()
    .map_or(allocated_next, |highest| {
      highest.saturating_add(1).max(allocated_next)
    })
}

// 3. Cover Letters (เบิกยาปะหน้า)

/// Build one [`CoverLetterPage`] per invoice, carrying the running budget
/// balance and cumulative spent amount.
///
/// Every step of the balance chain is rounded to satang, so the printed
/// figures always agree exactly with the value carried into the next round
/// and a balance that should be 0.00 is exactly 0.00.
#[must_use]
pub fn process_cover_letters(
  invoices: &[InvoiceRow],
  params: &CoverLettersParams,
) -> Vec<CoverLetterPage> {
  // The fiscal_year is already a Buddhist year supplied by the frontend.
  let fiscal_year = format!("{}", params.year);

  let date_text = if let Some(ref dt) = params.approval_date {
    dt.clone()
  } else {
    invoices
      .first()
      .map(|inv| format_thai_date_full(&inv.receive_date))
      .unwrap_or_default()
  };

  let mut pages: Vec<CoverLetterPage> = Vec::with_capacity(invoices.len());

  let total_budget = round_satang(params.budget_total);
  let mut running_balance = round_satang(params.previous_balance);
  let mut cumulative_spent = round_satang(total_budget - running_balance);

  for inv in invoices.iter() {
    let previous_balance = running_balance;
    let current_amount = round_satang(inv.total_cost);
    let remaining_balance = round_satang(previous_balance - current_amount);

    pages.push(CoverLetterPage {
      company_name: inv.company_name.clone(),
      category: inv.category.clone(),
      budget_total: total_budget,
      previous_spent: cumulative_spent,
      previous_balance,
      current_amount,
      remaining_balance,
      fiscal_year: fiscal_year.clone(),
      date_text: date_text.clone(),
    });

    running_balance = remaining_balance;
    cumulative_spent = round_satang(cumulative_spent + current_amount);
  }

  pages
}

// Carry-forward

/// Returns `(next_reg_no_string, next_running_slot)` - the register number
/// and position that the *next* batch should start from.
///
/// # Errors
///
/// Returns a Thai error message when the register number is malformed or the
/// starting slot is outside `0..=9`.
pub fn compute_next_reg(
  start_reg_no: &str,
  start_running: u32,
  count: u32,
) -> Result<(String, u32), String> {
  validate_start_running(start_running)?;
  let (prefix, start_num) = parse_reg_no(start_reg_no)?;
  let next_absolute = start_running + count;
  let reg_offset = next_absolute / 10;
  let next_running = next_absolute % 10;
  let next_reg_no = format_reg_no(&prefix, start_num + reg_offset);
  Ok((next_reg_no, next_running))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::{DbConfig, InvoiceRow};
  use chrono::NaiveDate;

  fn sample_invoices() -> Vec<InvoiceRow> {
    vec![
      InvoiceRow {
        invoice_no: "INV001".to_string(),
        vendor_code: "V001".to_string(),
        company_name: "บริษัท ก".to_string(),
        company_keyword: "ABC".to_string(),
        total_cost: 10_000.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        category: "ยา".to_string(),
        po_ref_no: None,
        req_app_date: None,
      },
      InvoiceRow {
        invoice_no: "INV002".to_string(),
        vendor_code: "V002".to_string(),
        company_name: "บริษัท ข".to_string(),
        company_keyword: "DEF".to_string(),
        total_cost: 20_000.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
        category: "ยา".to_string(),
        po_ref_no: None,
        req_app_date: None,
      },
      InvoiceRow {
        invoice_no: "INV003".to_string(),
        vendor_code: "PS001".to_string(),
        company_name: "พลาสติก สโตร์".to_string(),
        company_keyword: "PS001".to_string(),
        total_cost: 5_000.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 9).unwrap(),
        category: "วัสดุเภสัชกรรม".to_string(),
        po_ref_no: None,
        req_app_date: None,
      },
    ]
  }

  fn sample_submission_params() -> InvoiceSubmissionParams {
    InvoiceSubmissionParams {
      db_config: DbConfig {
        host: "localhost".into(),
        port: 1433,
        database: "test".into(),
        username: "sa".into(),
        password: "pass".into(),
      },
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      start_reg_no: "69ภ12".into(),
      start_running: 3,
      output_dir: "/tmp".into(),
    }
  }

  fn sample_summary_params() -> ReceivingSummaryParams {
    ReceivingSummaryParams {
      db_config: DbConfig {
        host: "localhost".into(),
        port: 1433,
        database: "test".into(),
        username: "sa".into(),
        password: "pass".into(),
      },
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      start_po_no: 253,
      start_purchase_no: 253,
      start_reg_no: "69ภ12".into(),
      start_running: 3,
      output_dir: "/tmp".into(),
    }
  }

  fn sample_cover_params() -> CoverLettersParams {
    CoverLettersParams {
      db_config: DbConfig {
        host: "localhost".into(),
        port: 1433,
        database: "test".into(),
        username: "sa".into(),
        password: "pass".into(),
      },
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      budget_total: 5_843_812.60,
      previous_balance: 1_000_000.0,
      approval_date: Some("15 ม.ค. 69".into()),
      output_dir: "/tmp".into(),
    }
  }

  #[test]
  fn invoice_submission_assigns_register_and_running() {
    let invoices = sample_invoices();
    let params = sample_submission_params();
    let rows = process_invoice_submission(&invoices, &params).unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].seq, 1);
    assert_eq!(rows[0].reg_no, "69ภ12");
    assert_eq!(rows[0].running_in_reg, 3);
    assert_eq!(rows[1].seq, 2);
    assert_eq!(rows[1].running_in_reg, 4);
    assert_eq!(rows[2].seq, 3);
    assert_eq!(rows[2].running_in_reg, 5);
  }

  #[test]
  fn receiving_summary_counters() {
    let invoices = sample_invoices();
    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows.len(), 3);
    // request_no: 253, 255, 257
    assert_eq!(rows[0].request_no, 253);
    assert_eq!(rows[1].request_no, 255);
    assert_eq!(rows[2].request_no, 257);
    // report_no: 254, 256, 258
    assert_eq!(rows[0].report_no, 254);
    assert_eq!(rows[1].report_no, 256);
    assert_eq!(rows[2].report_no, 258);
    // po_no: 253, 254, 255 (fallback counter - no MS_PO reference on these rows)
    assert_eq!(rows[0].po_no, "253");
    assert_eq!(rows[1].po_no, "254");
    assert_eq!(rows[2].po_no, "255");
  }

  #[test]
  fn cover_letters_running_balance() {
    let invoices = sample_invoices();
    let params = sample_cover_params();
    let pages = process_cover_letters(&invoices, &params);

    assert_eq!(pages.len(), 3);

    // Page 1: balance 1,000,000 - 10,000 = 990,000
    assert!((pages[0].previous_balance - 1_000_000.0).abs() < 0.01);
    assert!((pages[0].current_amount - 10_000.0).abs() < 0.01);
    assert!((pages[0].remaining_balance - 990_000.0).abs() < 0.01);

    // Page 2: balance 990,000 - 20,000 = 970,000
    assert!((pages[1].previous_balance - 990_000.0).abs() < 0.01);
    assert!((pages[1].current_amount - 20_000.0).abs() < 0.01);
    assert!((pages[1].remaining_balance - 970_000.0).abs() < 0.01);

    // Page 3: balance 970,000 - 5,000 = 965,000
    assert!((pages[2].previous_balance - 970_000.0).abs() < 0.01);
    assert!((pages[2].current_amount - 5_000.0).abs() < 0.01);
    assert!((pages[2].remaining_balance - 965_000.0).abs() < 0.01);
  }

  #[test]
  fn cover_letters_cumulative_spent() {
    let invoices = sample_invoices();
    let params = sample_cover_params();
    let pages = process_cover_letters(&invoices, &params);

    let initial_spent = params.budget_total - params.previous_balance;
    // 5,843,812.60 - 1,000,000 = 4,843,812.60
    assert!((pages[0].previous_spent - initial_spent).abs() < 0.01);
    assert!((pages[1].previous_spent - (initial_spent + 10_000.0)).abs() < 0.01);
    assert!((pages[2].previous_spent - (initial_spent + 30_000.0)).abs() < 0.01);
  }

  #[test]
  fn cover_letters_fiscal_year() {
    let invoices = sample_invoices();
    let params = sample_cover_params();
    let pages = process_cover_letters(&invoices, &params);
    // year = 2569 (Buddhist, passed directly)
    assert_eq!(pages[0].fiscal_year, "2569");
  }

  fn invoice_with_amount(amount: f64) -> InvoiceRow {
    InvoiceRow {
      invoice_no: "INV".to_string(),
      vendor_code: "V".to_string(),
      company_name: "บริษัท ทดสอบ".to_string(),
      company_keyword: "T".to_string(),
      total_cost: amount,
      receive_date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
      category: "ยา".to_string(),
      po_ref_no: None,
      req_app_date: None,
    }
  }

  /// A round that spends the balance down to zero must land on exactly 0.00,
  /// not on binary float noise, so the next round's carry-forward is exact.
  #[test]
  // Exact float equality is the point: the chain must land on the same f64.
  #[allow(clippy::float_cmp)]
  fn cover_letter_chain_is_exact_satang() {
    let invoices = vec![
      invoice_with_amount(3.33),
      invoice_with_amount(3.33),
      invoice_with_amount(3.34),
    ];
    let mut params = sample_cover_params();
    params.budget_total = 10.0;
    params.previous_balance = 10.0;

    let pages = process_cover_letters(&invoices, &params);

    assert_eq!(pages[0].previous_balance, 10.0);
    assert_eq!(pages[0].current_amount, 3.33);
    assert_eq!(pages[0].remaining_balance, 6.67);
    assert_eq!(pages[1].remaining_balance, 3.34);
    assert_eq!(pages[2].remaining_balance, 0.0);
    // Spent before the last page is 3.33 + 3.33; the last page spends the
    // final 3.34 and lands on exactly 0.00.
    assert_eq!(pages[2].previous_spent, 6.66);
    assert_eq!(pages[2].previous_balance, 3.34);
  }

  /// Raw database amounts with more than two decimals are rounded to satang
  /// at every boundary.
  #[test]
  // Exact float equality is the point: the amounts must land on exact satang.
  #[allow(clippy::float_cmp)]
  fn cover_letter_rounds_raw_amounts_to_satang() {
    let invoices = vec![invoice_with_amount(100.014), invoice_with_amount(0.004)];
    let mut params = sample_cover_params();
    params.budget_total = 100.01;
    params.previous_balance = 100.01;

    let pages = process_cover_letters(&invoices, &params);

    assert_eq!(pages[0].current_amount, 100.01);
    assert_eq!(pages[0].remaining_balance, 0.0);
    assert_eq!(pages[1].current_amount, 0.0);
    assert_eq!(pages[1].remaining_balance, 0.0);
    assert_eq!(pages[1].previous_spent, 100.01);
  }

  #[test]
  fn register_overflow() {
    let mut invoices = Vec::new();
    for i in 0..5 {
      invoices.push(InvoiceRow {
        invoice_no: format!("INV{:03}", i + 1),
        vendor_code: "V001".to_string(),
        company_name: "Test Co".to_string(),
        company_keyword: "TC".to_string(),
        total_cost: 1_000.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        category: "ยา".to_string(),
        po_ref_no: None,
        req_app_date: None,
      });
    }

    let mut params = sample_submission_params();
    params.start_running = 8;
    params.start_reg_no = "69ภ12".to_string();

    let rows = process_invoice_submission(&invoices, &params).unwrap();
    // Item 0: reg 12 pos 8
    assert_eq!(rows[0].reg_no, "69ภ12");
    assert_eq!(rows[0].running_in_reg, 8);
    // Item 1: reg 12 pos 9
    assert_eq!(rows[1].reg_no, "69ภ12");
    assert_eq!(rows[1].running_in_reg, 9);
    // Item 2: reg 13 pos 0 (overflow!)
    assert_eq!(rows[2].reg_no, "69ภ13");
    assert_eq!(rows[2].running_in_reg, 0);
    // Item 3: reg 13 pos 1
    assert_eq!(rows[3].reg_no, "69ภ13");
    assert_eq!(rows[3].running_in_reg, 1);
    // Item 4: reg 13 pos 2
    assert_eq!(rows[4].reg_no, "69ภ13");
    assert_eq!(rows[4].running_in_reg, 2);
  }

  #[test]
  fn round2_po_numbering() {
    let invoices = sample_invoices();
    let mut params = sample_summary_params();
    params.round = 2;
    params.start_po_no = 256;
    params.start_purchase_no = 256;
    params.start_running = 3;

    let rows = process_receiving_summary(&invoices, &params).unwrap();
    assert_eq!(rows[0].po_no, "256");
    assert_eq!(rows[0].request_no, 256);
    assert_eq!(rows[0].report_no, 257);
    assert_eq!(rows[1].po_no, "257");
    assert_eq!(rows[2].po_no, "258");
  }

  #[test]
  fn compute_next_reg_no_overflow() {
    // 3 items starting at pos 3 in reg 12 → next is pos 6 in reg 12
    let (next_reg, next_run) = compute_next_reg("69ภ12", 3, 3).unwrap();
    assert_eq!(next_reg, "69ภ12");
    assert_eq!(next_run, 6);
  }

  #[test]
  fn compute_next_reg_with_overflow() {
    // 5 items starting at pos 8 in reg 12 → last at pos 2 in reg 13, next is pos 3
    let (next_reg, next_run) = compute_next_reg("69ภ12", 8, 5).unwrap();
    assert_eq!(next_reg, "69ภ13");
    assert_eq!(next_run, 3);
  }

  #[test]
  fn compute_next_reg_exact_boundary() {
    // 2 items starting at pos 8 → fills pos 8,9 → next is pos 0 in reg 13
    let (next_reg, next_run) = compute_next_reg("69ภ12", 8, 2).unwrap();
    assert_eq!(next_reg, "69ภ13");
    assert_eq!(next_run, 0);
  }

  // Input validation: malformed registers, out-of-range slots, and zero
  // start numbers must be rejected before any report is produced.

  #[test]
  fn invoice_submission_rejects_malformed_register() {
    let invoices = sample_invoices();
    let mut params = sample_submission_params();
    params.start_reg_no = "69ภ".to_string();

    let err = process_invoice_submission(&invoices, &params).unwrap_err();
    assert!(err.contains("69ภ"), "got: {err}");
  }

  #[test]
  fn invoice_submission_rejects_running_above_nine() {
    let invoices = sample_invoices();
    let mut params = sample_submission_params();
    params.start_running = 12;

    let err = process_invoice_submission(&invoices, &params).unwrap_err();
    assert!(err.contains("0-9"), "got: {err}");
  }

  #[test]
  fn receiving_summary_rejects_bad_numbering_inputs() {
    let invoices = sample_invoices();

    let mut params = sample_summary_params();
    params.start_reg_no = "69ภ".to_string();
    assert!(process_receiving_summary(&invoices, &params).is_err());

    let mut params = sample_summary_params();
    params.start_running = 10;
    assert!(process_receiving_summary(&invoices, &params).is_err());

    let mut params = sample_summary_params();
    params.start_po_no = 0;
    let err = process_receiving_summary(&invoices, &params).unwrap_err();
    assert!(err.contains("ขอซื้อ"), "got: {err}");

    let mut params = sample_summary_params();
    params.start_purchase_no = 0;
    let err = process_receiving_summary(&invoices, &params).unwrap_err();
    assert!(err.contains("ใบสั่งซื้อ"), "got: {err}");
  }

  #[test]
  fn compute_next_reg_rejects_bad_inputs() {
    assert!(compute_next_reg("69ภ", 0, 1).is_err());
    assert!(compute_next_reg("", 0, 1).is_err());
    assert!(compute_next_reg("69ภ12", 10, 1).is_err());
  }

  // Carry-forward tests

  /// After processing n=3 invoices starting at start_po_no=253,
  /// the last request_no used is 253 + (3-1)*2 = 257.
  /// Therefore the next batch must start at 253 + 3*2 = 259.
  #[test]
  fn next_po_no_carry_forward_correct() {
    let invoices = sample_invoices(); // 3 invoices
    let params = sample_summary_params(); // start_po_no = 253
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows.len(), 3);

    // Last request_no in this batch is 257 (253 + 2*2)
    assert_eq!(rows[2].request_no, 257);

    // The next batch must start at 259 (253 + 3*2), NOT 256 (253 + 3)
    let n = rows.len() as u32;
    let expected_next_po_no = params.start_po_no + n * 2;
    assert_eq!(
      expected_next_po_no, 259,
      "next_po_no must be start_po_no + n*2 so the next batch starts after the last used pair"
    );

    // Confirm the wrong value (n instead of n*2) would collide with existing numbers
    let wrong_next_po_no = params.start_po_no + n; // 256 - already used as report_no
    assert_eq!(rows[1].report_no, 256);
    assert_ne!(
      wrong_next_po_no, expected_next_po_no,
      "start_po_no + n would collide with an already-issued report_no"
    );
  }

  /// Round-trip: second batch starts at 259 and its numbers do not overlap
  /// with the first batch's numbers (253–258).
  #[test]
  fn next_po_no_no_overlap_between_rounds() {
    let invoices = sample_invoices(); // 3 invoices
    let mut params = sample_summary_params(); // start_po_no = 253
    let n = invoices.len() as u32;
    let next_po_no = params.start_po_no + n * 2; // 259

    // Simulate round 2 starting where round 1 left off
    params.start_po_no = next_po_no;
    params.start_purchase_no = params.start_purchase_no + n; // 256
    params.round = 2;

    let rows2 = process_receiving_summary(&invoices, &params).unwrap();

    // Round 2 request numbers: 259, 261, 263
    assert_eq!(rows2[0].request_no, 259);
    assert_eq!(rows2[1].request_no, 261);
    assert_eq!(rows2[2].request_no, 263);

    // Round 2 report numbers: 260, 262, 264
    assert_eq!(rows2[0].report_no, 260);
    assert_eq!(rows2[1].report_no, 262);
    assert_eq!(rows2[2].report_no, 264);

    // Round 2 po_no (purchase): 256, 257, 258
    assert_eq!(rows2[0].po_no, "256");
    assert_eq!(rows2[1].po_no, "257");
    assert_eq!(rows2[2].po_no, "258");
  }

  // Approval date from MS_PO.REQ_APP_DATE

  /// Every row must carry the Thai-formatted `REQ_APP_DATE` of its own
  /// linked purchase order.
  #[test]
  fn approval_date_comes_from_req_app_date() {
    let mut invoices = sample_invoices();
    invoices[0].req_app_date = NaiveDate::from_ymd_opt(2026, 1, 12);
    invoices[1].req_app_date = NaiveDate::from_ymd_opt(2026, 1, 15);
    invoices[2].req_app_date = NaiveDate::from_ymd_opt(2026, 1, 20);

    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows[0].approval_date, "12 ม.ค. 69");
    assert_eq!(rows[1].approval_date, "15 ม.ค. 69");
    assert_eq!(rows[2].approval_date, "20 ม.ค. 69");
    // วันที่สั่งซื้อ mirrors วันที่ขออนุมัติ until the user edits the row.
    assert_eq!(rows[0].po_date, "12 ม.ค. 69");
  }

  /// Rows without a linked `MS_PO` fall back to the first invoice's receive
  /// date instead of leaving the column blank.
  #[test]
  fn approval_date_falls_back_to_first_receive_date() {
    let invoices = sample_invoices();
    let params = sample_summary_params();

    let rows = process_receiving_summary(&invoices, &params).unwrap();

    // First invoice receive_date is 2026-01-05 -> Thai "5 ม.ค. 69"
    let expected = "5 ม.ค. 69";
    for row in &rows {
      assert_eq!(
        row.approval_date, expected,
        "rows without REQ_APP_DATE must fall back to the first receive date"
      );
    }
  }

  // Purchase-order number from MS_PO.REF_NO

  /// `001/2570` yields `001` - the year is supplied by the report header.
  #[test]
  fn po_no_comes_from_ref_no_before_slash() {
    let mut invoices = sample_invoices();
    invoices[0].po_ref_no = Some("001/2570".to_string());
    invoices[1].po_ref_no = Some("002/2570".to_string());
    invoices[2].po_ref_no = Some("004/2570".to_string());

    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows[0].po_no, "001");
    assert_eq!(rows[1].po_no, "002");
    assert_eq!(rows[2].po_no, "004");
  }

  /// `2569-0001` yields `0001` - the sequence follows the dash.
  #[test]
  fn po_no_comes_from_ref_no_after_dash() {
    let mut invoices = sample_invoices();
    invoices[0].po_ref_no = Some("2569-0001".to_string());
    invoices[1].po_ref_no = Some("2569-0002".to_string());
    invoices[2].po_ref_no = Some("2569-0003".to_string());

    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows[0].po_no, "0001");
    assert_eq!(rows[1].po_no, "0002");
    assert_eq!(rows[2].po_no, "0003");
  }

  /// Without a `REF_NO` the assigned counter is used as a safety net.
  #[test]
  fn po_no_falls_back_to_assigned_counter() {
    let invoices = sample_invoices();
    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows[0].po_no, "253");
    assert_eq!(rows[1].po_no, "254");
    assert_eq!(rows[2].po_no, "255");
  }

  #[test]
  fn po_sequence_text_formats() {
    assert_eq!(po_sequence_text("001/2570"), Some("001".to_string()));
    assert_eq!(po_sequence_text("2569-0001"), Some("0001".to_string()));
    assert_eq!(po_sequence_text(" 042 "), Some("042".to_string()));
    assert_eq!(po_sequence_text(""), None);
    assert_eq!(po_sequence_text("   "), None);
    assert_eq!(po_sequence_text("/2570"), None);
    assert_eq!(po_sequence_text("2569-"), None);
  }

  #[test]
  fn po_sequence_number_parses_text() {
    assert_eq!(po_sequence_number("001/2570"), Some(1));
    assert_eq!(po_sequence_number("2569-0001"), Some(1));
    assert_eq!(po_sequence_number("253"), Some(253));
    assert_eq!(po_sequence_number("A/2570"), None);
    assert_eq!(po_sequence_number(""), None);
  }

  /// A fallback counter for an unlinked invoice must skip sequences that a
  /// linked PO reference already occupies, so no number repeats.
  #[test]
  fn po_no_fallback_skips_linked_sequences() {
    let mut invoices = sample_invoices();
    invoices[0].po_ref_no = Some("1/2570".to_string());
    invoices[1].po_ref_no = None;
    invoices[2].po_ref_no = Some("2/2570".to_string());

    let mut params = sample_summary_params();
    params.start_purchase_no = 1;
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows[0].po_no, "1");
    assert_eq!(rows[1].po_no, "3", "fallback must skip the linked 1 and 2");
    assert_eq!(rows[2].po_no, "2");
    // All purchase numbers in the batch are distinct.
    let mut numbers: Vec<&str> = rows.iter().map(|r| r.po_no.as_str()).collect();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(numbers, ["1", "2", "3"]);
  }

  /// The carry-forward must sit past every issued number, not just past the
  /// allocated counter.
  #[test]
  fn next_purchase_no_helper_takes_highest_issued() {
    let mut invoices = sample_invoices();
    invoices.truncate(2);
    invoices[0].po_ref_no = Some("5/2570".to_string());
    invoices[1].po_ref_no = Some("10/2570".to_string());

    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    // The allocated counter says 7, but sequence 10 was already issued.
    assert_eq!(next_purchase_no(&rows, 7), 11);
    // An allocation already ahead of the issued numbers wins.
    assert_eq!(next_purchase_no(&rows, 100), 100);
  }

  #[test]
  fn next_purchase_no_helper_falls_back_to_allocation() {
    let invoices = sample_invoices();
    let params = sample_summary_params();
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    // Fallback counters 253..255 are the highest issued numbers.
    assert_eq!(next_purchase_no(&rows, 256), 256);
    assert_eq!(next_purchase_no(&[], 42), 42);
  }
}

/// Golden snapshots: a fixed, hand-computed baseline for all three reports so a
/// behavioural regression in numbering, totals, or budget math fails loudly and
/// deterministically. These double as the reference for the report outputs.
#[cfg(test)]
mod golden_tests {
  use super::*;
  use crate::models::{
    CoverLettersParams, DbConfig, InvoiceRow, InvoiceSubmissionParams, ReceivingSummaryParams,
  };
  use crate::numbering::allocate_receiving_numbers;
  use chrono::NaiveDate;

  fn db() -> DbConfig {
    DbConfig {
      host: "localhost".into(),
      port: 1433,
      database: "test".into(),
      username: "sa".into(),
      password: "pass".into(),
    }
  }

  fn golden_invoices() -> Vec<InvoiceRow> {
    vec![
      InvoiceRow {
        invoice_no: "INV001".into(),
        vendor_code: "V001".into(),
        company_name: "บริษัท ก".into(),
        company_keyword: "G".into(),
        total_cost: 1_000.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
        category: "ยา".into(),
        po_ref_no: Some("005/2570".into()),
        req_app_date: NaiveDate::from_ymd_opt(2026, 1, 2),
      },
      InvoiceRow {
        invoice_no: "INV002".into(),
        vendor_code: "V002".into(),
        company_name: "บริษัท ข".into(),
        company_keyword: "K".into(),
        total_cost: 2_500.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        category: "ยา".into(),
        po_ref_no: Some("006/2570".into()),
        req_app_date: NaiveDate::from_ymd_opt(2026, 1, 4),
      },
      InvoiceRow {
        invoice_no: "INV003".into(),
        vendor_code: "V003".into(),
        company_name: "บริษัท ค".into(),
        company_keyword: "C".into(),
        total_cost: 500.0,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
        category: "วัสดุเภสัชกรรม".into(),
        po_ref_no: Some("007/2570".into()),
        req_app_date: NaiveDate::from_ymd_opt(2026, 1, 6),
      },
    ]
  }

  #[test]
  fn golden_invoice_submission() {
    let invoices = golden_invoices();
    let params = InvoiceSubmissionParams {
      db_config: db(),
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      start_reg_no: "69ภ12".into(),
      start_running: 0,
      output_dir: "/tmp".into(),
    };
    let rows = process_invoice_submission(&invoices, &params).unwrap();

    assert_eq!(rows.len(), 3);
    let total: f64 = rows.iter().map(|r| r.total_amount).sum();
    assert!((total - 4_000.0).abs() < 1e-9);
    assert_eq!(rows[0].seq, 1);
    assert_eq!(rows[0].reg_no, "69ภ12");
    assert_eq!(rows[0].running_in_reg, 0);
    assert_eq!(rows[1].reg_no, "69ภ12");
    assert_eq!(rows[1].running_in_reg, 1);
    assert_eq!(rows[2].reg_no, "69ภ12");
    assert_eq!(rows[2].running_in_reg, 2);
    assert_eq!(rows[0].company_name, "บริษัท ก");
    assert_eq!(rows[2].category, "วัสดุเภสัชกรรม");
  }

  #[test]
  fn golden_receiving_summary() {
    let invoices = golden_invoices();
    let params = ReceivingSummaryParams {
      db_config: db(),
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      start_po_no: 10,
      start_purchase_no: 5,
      start_reg_no: "69ภ12".into(),
      start_running: 0,
      output_dir: "/tmp".into(),
    };
    let rows = process_receiving_summary(&invoices, &params).unwrap();

    assert_eq!(rows.len(), 3);
    let total: f64 = rows.iter().map(|r| r.total_amount).sum();
    assert!((total - 4_000.0).abs() < 1e-9);
    assert_eq!(rows[0].request_no, 10);
    assert_eq!(rows[0].report_no, 11);
    assert_eq!(rows[0].po_no, "005");
    assert_eq!(rows[1].request_no, 12);
    assert_eq!(rows[1].report_no, 13);
    assert_eq!(rows[1].po_no, "006");
    assert_eq!(rows[2].request_no, 14);
    assert_eq!(rows[2].report_no, 15);
    assert_eq!(rows[2].po_no, "007");
    assert_eq!(rows[0].reg_no, "69ภ12");
    assert_eq!(rows[2].running_in_reg, 2);
    // วันที่ขออนุมัติ per row comes from the linked MS_PO.REQ_APP_DATE.
    assert_eq!(rows[0].approval_date, "2 ม.ค. 69");
    assert_eq!(rows[1].approval_date, "4 ม.ค. 69");
    assert_eq!(rows[2].approval_date, "6 ม.ค. 69");
  }

  #[test]
  fn golden_cover_letters_budget_math() {
    let invoices = golden_invoices();
    let params = CoverLettersParams {
      db_config: db(),
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      budget_total: 100_000.0,
      previous_balance: 20_000.0,
      approval_date: Some("15 ม.ค. 69".into()),
      output_dir: "/tmp".into(),
    };
    let pages = process_cover_letters(&invoices, &params);

    assert_eq!(pages.len(), 3);
    // Page 1: 20,000 - 1,000 = 19,000 remaining.
    assert!((pages[0].previous_balance - 20_000.0).abs() < 1e-9);
    assert!((pages[0].previous_spent - 80_000.0).abs() < 1e-9);
    assert!((pages[0].current_amount - 1_000.0).abs() < 1e-9);
    assert!((pages[0].remaining_balance - 19_000.0).abs() < 1e-9);
    // Page 2: 19,000 - 2,500 = 16,500 remaining.
    assert!((pages[1].previous_balance - 19_000.0).abs() < 1e-9);
    assert!((pages[1].previous_spent - 81_000.0).abs() < 1e-9);
    assert!((pages[1].remaining_balance - 16_500.0).abs() < 1e-9);
    // Page 3: 16,500 - 500 = 16,000 remaining.
    assert!((pages[2].previous_balance - 16_500.0).abs() < 1e-9);
    assert!((pages[2].remaining_balance - 16_000.0).abs() < 1e-9);
    // Carry-forward (last remaining balance) for the next round.
    assert!((pages.last().unwrap().remaining_balance - 16_000.0).abs() < 1e-9);
  }

  // Preview/generate parity: the carry-forward values the preview screen
  // reports must be exactly what the generate path persists for the next round,
  // otherwise continuity breaks silently between the two flows.

  #[test]
  fn parity_register_carry_forward() {
    let invoices = golden_invoices();
    let n = invoices.len() as u32;
    let params = InvoiceSubmissionParams {
      db_config: db(),
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      start_reg_no: "69ภ12".into(),
      start_running: 0,
      output_dir: "/tmp".into(),
    };
    // Preview renders these rows; generate persists the same rows.
    let preview_rows = process_invoice_submission(&invoices, &params).unwrap();
    // Generate computes its carry-forward via compute_next_reg (preview does too).
    let (next_reg_no, next_running) =
      compute_next_reg(&params.start_reg_no, params.start_running, n).unwrap();

    let last = preview_rows.last().unwrap();
    let (last_prefix, last_num) = parse_reg_no(&last.reg_no).unwrap();
    let expected_num = last_num + (last.running_in_reg + 1) / 10;
    let expected_running = (last.running_in_reg + 1) % 10;
    assert_eq!(next_reg_no, format_reg_no(&last_prefix, expected_num));
    assert_eq!(next_running, expected_running);
  }

  #[test]
  fn parity_receiving_carry_forward() {
    let invoices = golden_invoices();
    let n = invoices.len() as u32;
    let params = ReceivingSummaryParams {
      db_config: db(),
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      start_po_no: 10,
      start_purchase_no: 5,
      start_reg_no: "69ภ12".into(),
      start_running: 0,
      output_dir: "/tmp".into(),
    };
    let allocation =
      allocate_receiving_numbers(2569, params.start_po_no, params.start_purchase_no, n, &[])
        .unwrap();
    let rows =
      process_receiving_summary_with_numbers(&invoices, &params, &allocation.assignments).unwrap();

    // Preview and generate both derive these next values identically.
    assert_eq!(allocation.next_po_no, params.start_po_no + n * 2);
    assert_eq!(allocation.next_purchase_no, params.start_purchase_no + n);
    // The next request number must be exactly one past the last report number.
    assert_eq!(allocation.next_po_no, rows.last().unwrap().report_no + 1);
  }

  #[test]
  fn parity_cover_letter_budget_continuity() {
    // The remaining balance of one batch must feed the next batch's previous_balance.
    let batch1 = golden_invoices();
    let params1 = CoverLettersParams {
      db_config: db(),
      date_from: "20260101".into(),
      date_to: "20260110".into(),
      year: 2569,
      month: 1,
      round: 1,
      budget_total: 100_000.0,
      previous_balance: 20_000.0,
      approval_date: Some("15 ม.ค. 69".into()),
      output_dir: "/tmp".into(),
    };
    let pages1 = process_cover_letters(&batch1, &params1);
    let carry = pages1.last().unwrap().remaining_balance;

    let batch2 = golden_invoices();
    let params2 = CoverLettersParams {
      previous_balance: carry,
      ..params1
    };
    let pages2 = process_cover_letters(&batch2, &params2);
    // Second batch opens with the prior batch's closing balance.
    assert!((pages2[0].previous_balance - carry).abs() < 1e-9);
    // And the second batch's first cumulative spent equals budget - carry.
    assert!((pages2[0].previous_spent - (100_000.0 - carry)).abs() < 1e-9);
  }
}
