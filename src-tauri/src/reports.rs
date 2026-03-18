use chrono::{Datelike, NaiveDate};

use crate::models::{
    CoverLetterPage, GenerateParams, InvoiceRow, InvoiceSubmissionRow, ReceivingSummaryRow,
};

/// Format a NaiveDate as a Thai short date string, e.g. "5 ม.ค. 69"
fn format_thai_date(date: &NaiveDate) -> String {
    let thai_months = [
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
        thai_months[date.month0() as usize],
        short_year
    )
}

/// Parse a register number string like "69ภ12" into (prefix, number).
fn parse_reg_no(reg: &str) -> (String, u32) {
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

/// Format a register string from prefix + number, e.g. ("69ภ", 12) → "69ภ12"
fn format_reg_no(prefix: &str, number: u32) -> String {
    format!("{}{}", prefix, number)
}

/// Compute register number and running position within the register.
///
/// Each register holds 10 items (positions 0–9). When the position reaches 10,
/// the register number increments and the position resets to 0.
///
/// `item_index`   – zero-based index of the current item in this batch.
/// `start_running`– starting position within the first register (0–9).
/// `start_reg_num`– numeric portion of the first register number.
fn compute_reg_for_item(item_index: u32, start_running: u32, start_reg_num: u32) -> (u32, u32) {
    let absolute_pos = start_running + item_index;
    let reg_offset = absolute_pos / 10;
    let running_in_reg = absolute_pos % 10;
    (start_reg_num + reg_offset, running_in_reg)
}

// ---------------------------------------------------------------------------
// 1. Invoice Submission List  (ส่งหนี้เบิกยา)
// ---------------------------------------------------------------------------

pub fn process_invoice_submission(
    invoices: &[InvoiceRow],
    params: &GenerateParams,
) -> Vec<InvoiceSubmissionRow> {
    let (reg_prefix, reg_start_num) = parse_reg_no(&params.start_reg_no);
    let mut rows: Vec<InvoiceSubmissionRow> = Vec::with_capacity(invoices.len());

    for (i, inv) in invoices.iter().enumerate() {
        // seq is purely within this batch (1-based); the round label on the
        // printed document already disambiguates it across rounds.
        let seq = (i as u32) + 1;
        let (reg_num, running) =
            compute_reg_for_item(i as u32, params.start_running, reg_start_num);
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
            total_amount: inv.total_cost,
        });
    }

    rows
}

// ---------------------------------------------------------------------------
// 2. Receiving Summary  (สรุปรับยา)
// ---------------------------------------------------------------------------

pub fn process_receiving_summary(
    invoices: &[InvoiceRow],
    params: &GenerateParams,
) -> Vec<ReceivingSummaryRow> {
    let (reg_prefix, reg_start_num) = parse_reg_no(&params.start_reg_no);
    let mut rows: Vec<ReceivingSummaryRow> = Vec::with_capacity(invoices.len());

    // PO-related counters continue across rounds.
    // start_po_no already holds the correct starting value for this round.
    // request_no and report_no increment by 2 per row.
    // po_no increments by 1 per row.
    let mut request_no = params.start_po_no;
    let mut report_no = params.start_po_no + 1;
    let mut po_no = params.start_po_no;

    let approval_date_str = params.approval_date.clone().unwrap_or_default();

    for (i, inv) in invoices.iter().enumerate() {
        let (reg_num, running) =
            compute_reg_for_item(i as u32, params.start_running, reg_start_num);
        let reg_no = format_reg_no(&reg_prefix, reg_num);

        let receive_date_str = format_thai_date(&inv.receive_date);
        let po_date_str = receive_date_str.clone();

        // Receiving code: sequential within this batch (1-based).
        // Across rounds the counter is continuous because start_running is
        // already set to the next available slot by the user.
        let receiving_code = (i as u32) + 1;

        rows.push(ReceivingSummaryRow {
            approval_date: approval_date_str.clone(),
            po_date: po_date_str,
            receive_date: receive_date_str,
            company_code: inv.vendor_code.clone(),
            total_amount: inv.total_cost,
            receiving_code,
            reg_no,
            running_in_reg: running,
            invoice_no: inv.invoice_no.clone(),
            request_no,
            report_no,
            po_no,
        });

        request_no += 2;
        report_no += 2;
        po_no += 1;
    }

    rows
}

// ---------------------------------------------------------------------------
// 3. Cover Letters  (เบิกยาปะหน้า)
// ---------------------------------------------------------------------------

pub fn process_cover_letters(
    invoices: &[InvoiceRow],
    params: &GenerateParams,
) -> Vec<CoverLetterPage> {
    // fiscal_year shown on cover letter (Buddhist year)
    let year: i32 = params.start_date[0..4].parse().unwrap_or(2569);
    let fiscal_year = format!("{}", year + 543);

    let date_text = if let Some(ref dt) = params.approval_date {
        dt.clone()
    } else {
        invoices
            .first()
            .map(|inv| format_thai_date(&inv.receive_date))
            .unwrap_or_default()
    };

    let mut pages: Vec<CoverLetterPage> = Vec::with_capacity(invoices.len());

    // Running balance starts from previous_balance provided by user.
    // This value already accounts for all previous rounds.
    let mut running_balance = params.previous_balance;

    // Cumulative spent before this batch = budget_total - previous_balance
    let total_budget = params.budget_total;
    let mut cumulative_spent = total_budget - params.previous_balance;

    for inv in invoices.iter() {
        let previous_balance = running_balance;
        let current_amount = inv.total_cost;
        let remaining_balance = previous_balance - current_amount;

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

        // Advance running totals for the next page
        running_balance = remaining_balance;
        cumulative_spent += current_amount;
    }

    pages
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_invoices() -> Vec<InvoiceRow> {
        vec![
            InvoiceRow {
                invoice_no: "INV001".to_string(),
                vendor_code: "V001".to_string(),
                company_name: "บริษัท ก".to_string(),
                company_keyword: "ABC".to_string(),
                total_cost: 10000.0,
                receive_date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                category: "ยา".to_string(),
            },
            InvoiceRow {
                invoice_no: "INV002".to_string(),
                vendor_code: "V002".to_string(),
                company_name: "บริษัท ข".to_string(),
                company_keyword: "DEF".to_string(),
                total_cost: 20000.0,
                receive_date: NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
                category: "ยา".to_string(),
            },
            InvoiceRow {
                invoice_no: "INV003".to_string(),
                vendor_code: "PS001".to_string(),
                company_name: "พลาสติก สโตร์".to_string(),
                company_keyword: "PS001".to_string(),
                total_cost: 5000.0,
                receive_date: NaiveDate::from_ymd_opt(2026, 1, 9).unwrap(),
                category: "วัสดุเภสัชกรรม".to_string(),
            },
        ]
    }

    fn sample_params() -> GenerateParams {
        GenerateParams {
            db_config: crate::models::DbConfig {
                host: "localhost".to_string(),
                port: 1433,
                database: "test".to_string(),
                username: "sa".to_string(),
                password: "pass".to_string(),
            },
            year: 2569,
            month: 1,
            half: 1,
            round: 1,
            budget_total: 5_843_812.60,
            previous_balance: 1_000_000.0,
            start_po_no: 253,
            start_reg_no: "69ภ12".to_string(),
            start_running: 3,
            output_dir: "/tmp".to_string(),
            approval_date: Some("15 ม.ค. 69".to_string()),
        }
    }

    #[test]
    fn test_parse_reg_no() {
        let (prefix, num) = parse_reg_no("69ภ12");
        assert_eq!(prefix, "69ภ");
        assert_eq!(num, 12);
    }

    #[test]
    fn test_format_reg_no() {
        assert_eq!(format_reg_no("69ภ", 12), "69ภ12");
        assert_eq!(format_reg_no("69ภ", 13), "69ภ13");
    }

    #[test]
    fn test_compute_reg_for_item() {
        // Starting at position 3 in register 12
        assert_eq!(compute_reg_for_item(0, 3, 12), (12, 3));
        assert_eq!(compute_reg_for_item(6, 3, 12), (12, 9));
        // Overflow into next register
        assert_eq!(compute_reg_for_item(7, 3, 12), (13, 0));
    }

    #[test]
    fn test_format_thai_date() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        assert_eq!(format_thai_date(&d), "5 ม.ค. 69");

        let d2 = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert_eq!(format_thai_date(&d2), "31 ธ.ค. 69");
    }

    #[test]
    fn test_invoice_submission() {
        let invoices = sample_invoices();
        let params = sample_params();
        let rows = process_invoice_submission(&invoices, &params);

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
    fn test_receiving_summary_counters() {
        let invoices = sample_invoices();
        let params = sample_params();
        let rows = process_receiving_summary(&invoices, &params);

        assert_eq!(rows.len(), 3);
        // request_no: 253, 255, 257
        assert_eq!(rows[0].request_no, 253);
        assert_eq!(rows[1].request_no, 255);
        assert_eq!(rows[2].request_no, 257);
        // report_no: 254, 256, 258
        assert_eq!(rows[0].report_no, 254);
        assert_eq!(rows[1].report_no, 256);
        assert_eq!(rows[2].report_no, 258);
        // po_no: 253, 254, 255
        assert_eq!(rows[0].po_no, 253);
        assert_eq!(rows[1].po_no, 254);
        assert_eq!(rows[2].po_no, 255);
    }

    #[test]
    fn test_cover_letters_running_balance() {
        let invoices = sample_invoices();
        let params = sample_params();
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
    fn test_cover_letters_cumulative_spent() {
        let invoices = sample_invoices();
        let params = sample_params();
        let pages = process_cover_letters(&invoices, &params);

        let initial_spent = params.budget_total - params.previous_balance;
        // 5,843,812.60 - 1,000,000 = 4,843,812.60
        assert!((pages[0].previous_spent - initial_spent).abs() < 0.01);
        assert!((pages[1].previous_spent - (initial_spent + 10_000.0)).abs() < 0.01);
        assert!((pages[2].previous_spent - (initial_spent + 30_000.0)).abs() < 0.01);
    }

    #[test]
    fn test_register_overflow() {
        let mut invoices = Vec::new();
        for i in 0..5 {
            invoices.push(InvoiceRow {
                invoice_no: format!("INV{:03}", i + 1),
                vendor_code: "V001".to_string(),
                company_name: "Test Co".to_string(),
                company_keyword: "TC".to_string(),
                total_cost: 1000.0,
                receive_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                category: "ยา".to_string(),
            });
        }

        let mut params = sample_params();
        params.start_running = 8;
        params.start_reg_no = "69ภ12".to_string();

        let rows = process_invoice_submission(&invoices, &params);
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
    fn test_round2_po_numbering() {
        // Round 2 scenario: previous round used 3 items with start_po_no=253
        // so round 2 starts at 253 + 3 = 256 for po_no,
        // and request_no = 256, report_no = 257 for the first item.
        let invoices = sample_invoices();
        let mut params = sample_params();
        params.round = 2;
        params.start_po_no = 256; // user sets this based on previous round's last po_no + 1
        params.start_running = 3; // first available slot in register

        let rows = process_receiving_summary(&invoices, &params);
        assert_eq!(rows[0].po_no, 256);
        assert_eq!(rows[0].request_no, 256);
        assert_eq!(rows[0].report_no, 257);
        assert_eq!(rows[1].po_no, 257);
        assert_eq!(rows[2].po_no, 258);
    }
}
