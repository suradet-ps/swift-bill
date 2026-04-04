use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ── Shared ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRow {
    pub invoice_no: String,
    pub vendor_code: String,
    pub company_name: String,
    pub company_keyword: String,
    pub total_cost: f64,
    pub receive_date: NaiveDate,
    pub category: String, // "ยา" or "วัสดุเภสัชกรรม"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewData {
    pub invoices: Vec<InvoiceRow>,
    pub total_amount: f64,
    pub row_count: usize,
}

/// Carry-forward values for the next round (returned with every GenerateResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarryForward {
    pub next_reg_no: String,    // e.g. "69ภ13"
    pub next_running: u32,      // next available slot (0–9) within register
    pub next_po_no: u32,        // next ขอซื้อ/รายงาน start (ReceivingSummary)
    pub next_purchase_no: u32,  // next ใบสั่งซื้อ start (ReceivingSummary)
    pub remaining_balance: f64, // (meaningful only for CoverLetters)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub files: Vec<String>,
    pub total_rows: usize,
    pub total_amount: f64,
    pub carry_forward: CarryForward,
}

// ── Intermediate row types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSubmissionRow {
    pub seq: u32,
    pub receive_date: String,
    pub invoice_no: String,
    pub reg_no: String,
    pub running_in_reg: u32,
    pub invoice_date: String,
    pub company_name: String,
    pub category: String,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingSummaryRow {
    pub approval_date: String,
    pub po_date: String,
    pub receive_date: String,
    pub company_code: String,
    pub total_amount: f64,
    pub receiving_code: u32,
    pub reg_no: String,
    pub running_in_reg: u32,
    pub invoice_no: String,
    pub request_no: u32,
    pub report_no: u32,
    pub po_no: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverLetterPage {
    pub company_name: String,
    pub category: String,
    pub budget_total: f64,
    pub previous_spent: f64,
    pub previous_balance: f64,
    pub current_amount: f64,
    pub remaining_balance: f64,
    pub fiscal_year: String,
    pub date_text: String,
}

// ── Per-report params ─────────────────────────────────────────────────────

/// Params for ส่งหนี้เบิกยา
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSubmissionParams {
    pub db_config: DbConfig,
    pub date_from: String, // YYYYMMDD Gregorian
    pub date_to: String,   // YYYYMMDD Gregorian
    pub year: i32,         // Buddhist year for display/filename e.g. 2568
    pub month: u32,        // 1–12
    pub round: u32,
    pub start_reg_no: String, // e.g. "69ภ12"
    pub start_running: u32,   // 0–9
    pub output_dir: String,
}

/// Params for สรุปรับยา
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingSummaryParams {
    pub db_config: DbConfig,
    pub date_from: String,
    pub date_to: String,
    pub year: i32,
    pub month: u32,
    pub round: u32,
    pub start_po_no: u32,       // starting number for ขอซื้อ and รายงาน/อนุมัติ
    pub start_purchase_no: u32, // starting number for ใบสั่งซื้อ (independent counter)
    pub start_reg_no: String,
    pub start_running: u32,
    pub approval_date: Option<String>,
    pub output_dir: String,
}

/// Params for เบิกยาปะหน้า
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverLettersParams {
    pub db_config: DbConfig,
    pub date_from: String,
    pub date_to: String,
    pub year: i32,
    pub month: u32,
    pub round: u32,
    pub budget_total: f64,
    pub previous_balance: f64,
    pub approval_date: Option<String>,
    pub output_dir: String,
}

// ── Round history ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundHistoryEntry {
    pub id: String,       // unique id (timestamp-based)
    pub label: String,    // e.g. "ม.ค. 2568 รอบ 1"
    pub fiscal_year: i32, // Buddhist year
    pub month: u32,
    pub round: u32,
    pub date_from: String, // YYYYMMDD
    pub date_to: String,   // YYYYMMDD
    // Carry-forward for NEXT round
    pub next_reg_no: String,
    pub next_running: u32,
    pub next_po_no: u32,
    #[serde(default)]
    pub next_purchase_no: u32, // ใบสั่งซื้อ counter (added later; default 0 for old entries)
    pub remaining_balance: f64,
    // Summary
    pub budget_total: f64,
    pub total_amount: f64,
    pub invoice_count: u32,
    pub created_at: String, // ISO datetime
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoundHistory {
    pub entries: Vec<RoundHistoryEntry>,
}

// ── Preview results ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSubmissionPreview {
    pub rows: Vec<InvoiceSubmissionRow>,
    pub carry_forward: CarryForward,
    pub total_rows: usize,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingSummaryPreview {
    pub rows: Vec<ReceivingSummaryRow>,
    pub carry_forward: CarryForward,
    pub total_rows: usize,
    pub total_amount: f64,
}

// ── Excel export params ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSubmissionExcelParams {
    pub rows: Vec<InvoiceSubmissionRow>,
    pub year: i32,
    pub month: u32,
    pub round: u32,
    pub start_reg_no: String,
    pub start_running: u32,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingSummaryExcelParams {
    pub rows: Vec<ReceivingSummaryRow>,
    pub year: i32,
    pub month: u32,
    pub round: u32,
    pub start_po_no: u32,
    pub start_purchase_no: u32,
    pub start_reg_no: String,
    pub start_running: u32,
    pub output_dir: String,
}
