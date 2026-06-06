//! Invoice Submission (ส่งหนี้เบิกยา) Excel export.

use std::path::Path;

use rust_xlsxwriter::{Workbook, Worksheet};
use swift_bill_core::InvoiceSubmissionRow;

use crate::{map_xlsx_err, thai_month};

/// Generate an `.xlsx` file for **ส่งหนี้เบิกยา** (Invoice Submission List).
///
/// Columns:
///   ลำดับ | วันที่รับของ | เลขที่เอกสาร | เลขทะเบียนคุม | ลำดับ |
///   วัน/เดือน/ปีใบส่งของ | รหัสบริษัท | ค่าใช้จ่ายเรื่อง | จำนวนเงินรวม
pub fn generate_invoice_submission_excel(
  rows: &[InvoiceSubmissionRow],
  year: i32,
  month: u32,
  round: u32,
  output_dir: &str,
) -> Result<String, String> {
  let mut workbook = Workbook::new();
  let ws: &mut Worksheet = workbook.add_worksheet();

  let month_name = thai_month(month);

  // Row 0: title
  ws.write(
    0,
    0,
    format!(
      "ส่งรายการหนี้สินและเอกสารเบิกเงิน  เดือน{}  รอบ {}  ปีงบประมาณ {}",
      month_name, round, year
    ),
  )
  .map_err(map_xlsx_err)?;

  // Row 1: column headers
  let headers: &[&str] = &[
    "ลำดับ",
    "วันที่รับของ",
    "เลขที่เอกสาร",
    "เลขทะเบียนคุม",
    "ลำดับ",
    "วัน/เดือน/ปีใบส่งของ",
    "รหัสบริษัท",
    "ค่าใช้จ่ายเรื่อง",
    "จำนวนเงินรวม",
  ];
  for (col, h) in headers.iter().enumerate() {
    ws.write(1, col as u16, *h).map_err(map_xlsx_err)?;
  }

  // Rows 2+: data
  let mut grand_total = 0.0_f64;
  for (i, row) in rows.iter().enumerate() {
    let r = (i as u32) + 2;
    ws.write(r, 0, row.seq as f64).map_err(map_xlsx_err)?;
    ws.write(r, 1, row.receive_date.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 2, row.invoice_no.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 3, row.reg_no.as_str()).map_err(map_xlsx_err)?;
    ws.write(r, 4, row.running_in_reg as f64)
      .map_err(map_xlsx_err)?;
    ws.write(r, 5, row.invoice_date.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 6, row.company_name.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 7, row.category.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 8, row.total_amount).map_err(map_xlsx_err)?;
    grand_total += row.total_amount;
  }

  // Total row
  let total_r = (rows.len() as u32) + 2;
  ws.write(total_r, 7, "รวมทั้งสิ้น").map_err(map_xlsx_err)?;
  ws.write(total_r, 8, grand_total).map_err(map_xlsx_err)?;

  // Save
  let filename = format!("ส่งหนี้เบิกยา_{year}_เดือน{month}_รอบ{round}.xlsx");
  let path = Path::new(output_dir).join(&filename);
  let path_str = path.to_string_lossy().to_string();
  workbook
    .save(&path_str)
    .map_err(|e| format!("บันทึก Excel ไม่สำเร็จ: {e}"))?;

  Ok(path_str)
}

#[cfg(test)]
mod tests {
  use super::*;
  use swift_bill_core::InvoiceSubmissionRow;

  fn sample_rows() -> Vec<InvoiceSubmissionRow> {
    vec![
      InvoiceSubmissionRow {
        seq: 1,
        receive_date: "5 ม.ค. 69".to_string(),
        invoice_no: "INV001".to_string(),
        reg_no: "69ภ12".to_string(),
        running_in_reg: 3,
        invoice_date: "5 ม.ค. 69".to_string(),
        company_name: "บริษัท ก".to_string(),
        category: "ยา".to_string(),
        total_amount: 10_000.0,
      },
      InvoiceSubmissionRow {
        seq: 2,
        receive_date: "7 ม.ค. 69".to_string(),
        invoice_no: "INV002".to_string(),
        reg_no: "69ภ12".to_string(),
        running_in_reg: 4,
        invoice_date: "7 ม.ค. 69".to_string(),
        company_name: "บริษัท ข".to_string(),
        category: "ยา".to_string(),
        total_amount: 20_000.0,
      },
    ]
  }

  fn tempdir(suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
      "swift_bill_excel_{suffix}_{}",
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn generate_writes_file() {
    let tmp = tempdir("sub");
    let result =
      generate_invoice_submission_excel(&sample_rows(), 2569, 1, 1, tmp.to_str().unwrap());
    assert!(result.is_ok(), "generate failed: {:?}", result.err());

    let path = result.unwrap();
    let meta = std::fs::metadata(&path).expect("output file should exist");
    assert!(
      meta.len() > 1000,
      "xlsx output too small: {} bytes",
      meta.len()
    );

    let _ = std::fs::remove_dir_all(&tmp);
  }
}
