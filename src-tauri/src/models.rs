use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
pub struct GenerateParams {
    pub db_config: DbConfig,
    pub start_date: String,            // Start date in YYYYMMDD format
    pub end_date: String,              // End date in YYYYMMDD format
    pub round: u32,                    // Batch/round number (รอบ) within the period e.g. 1, 2, ...
    pub budget_total: f64,             // Total allocated budget e.g. 5843812.60
    pub previous_balance: f64,         // Remaining budget from previous period
    pub start_po_no: u32,              // Starting PO number e.g. 253
    pub start_reg_no: String,          // Starting register number e.g. "69ภ12"
    pub start_running: u32,            // Starting running receipt number
    pub output_dir: String,            // Directory to save generated files
    pub approval_date: Option<String>, // Date of approval
}

// For invoice submission list (ส่งหนี้เบิกยา)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSubmissionRow {
    pub seq: u32,
    pub receive_date: String, // Formatted Thai date
    pub invoice_no: String,
    pub reg_no: String,       // e.g. "69ภ12"
    pub running_in_reg: u32,  // running number within register
    pub invoice_date: String, // Same as receive date
    pub company_name: String,
    pub category: String,
    pub total_amount: f64,
}

// For receiving summary (สรุปรับยา)
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
    pub request_no: u32, // ขอซื้อ (PO request)
    pub report_no: u32,  // รายงาน/อนุมัติ
    pub po_no: u32,      // ใบสั่งซื้อ
}

// For cover letter (เบิกยาปะหน้า)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverLetterPage {
    pub company_name: String,
    pub category: String, // "ยา" or "วัสดุเภสัชกรรม"
    pub budget_total: f64,
    pub previous_spent: f64,
    pub previous_balance: f64,
    pub current_amount: f64,
    pub remaining_balance: f64,
    pub fiscal_year: String,
    pub date_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub files: Vec<String>, // paths to generated PDF files
    pub total_rows: usize,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewData {
    pub invoices: Vec<InvoiceRow>,
    pub total_amount: f64,
    pub row_count: usize,
}
