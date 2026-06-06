//! Receiving Summary (สรุปรับยา) Excel export.

use std::path::Path;

use rust_xlsxwriter::{Workbook, Worksheet};
use swift_bill_core::ReceivingSummaryRow;

use crate::{map_xlsx_err, thai_month};

/// Generate an `.xlsx` file for **สรุปรับยา** (Receiving Summary).
///
/// Columns:
///   วันที่ขออนุมัติ | วันที่สั่งซื้อ | วันที่รับของ | รหัสบริษัท |
///   จำนวนเงินรวม | รหัสลงรับยา | เลขทะเบียนคุม | ลำดับ | เลขที่ลงรับ |
///   ขอซื้อ (ลบ0033.302/) | รายงาน/อนุมัติ (ลบ0033.302/) | ใบสั่งซื้อ…/{year}
pub fn generate_receiving_summary_excel(
  rows: &[ReceivingSummaryRow],
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
      "สรุปรับยาเดือน{}  รอบ {}  ปีงบประมาณ {}",
      month_name, round, year
    ),
  )
  .map_err(map_xlsx_err)?;

  // Row 1: column headers — static (cols 0–10)
  let static_headers: &[&str] = &[
    "วันที่ขออนุมัติ",
    "วันที่สั่งซื้อ",
    "วันที่รับของ",
    "รหัสบริษัท",
    "จำนวนเงินรวม",
    "รหัสลงรับยา",
    "เลขทะเบียนคุม",
    "ลำดับ",
    "เลขที่ลงรับ",
    "ขอซื้อ (ลบ0033.302/)",
    "รายงาน/อนุมัติ (ลบ0033.302/)",
  ];
  for (col, h) in static_headers.iter().enumerate() {
    ws.write(1, col as u16, *h).map_err(map_xlsx_err)?;
  }
  // Col 11: dynamic header that includes the year
  let year_header = format!("ใบสั่งซื้อ…/{}", year);
  ws.write(1, 11_u16, year_header.as_str())
    .map_err(map_xlsx_err)?;

  // Rows 2+: data
  let mut grand_total = 0.0_f64;
  for (i, row) in rows.iter().enumerate() {
    let r = (i as u32) + 2;
    ws.write(r, 0, row.approval_date.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 1, row.po_date.as_str()).map_err(map_xlsx_err)?;
    ws.write(r, 2, row.receive_date.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 3, row.company_code.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 4, row.total_amount).map_err(map_xlsx_err)?;
    ws.write(r, 5, row.receiving_code as f64)
      .map_err(map_xlsx_err)?;
    ws.write(r, 6, row.reg_no.as_str()).map_err(map_xlsx_err)?;
    ws.write(r, 7, row.running_in_reg as f64)
      .map_err(map_xlsx_err)?;
    ws.write(r, 8, row.invoice_no.as_str())
      .map_err(map_xlsx_err)?;
    ws.write(r, 9, row.request_no as f64)
      .map_err(map_xlsx_err)?;
    ws.write(r, 10, row.report_no as f64)
      .map_err(map_xlsx_err)?;
    ws.write(r, 11, row.po_no as f64).map_err(map_xlsx_err)?;
    grand_total += row.total_amount;
  }

  // Total row
  let total_r = (rows.len() as u32) + 2;
  ws.write(total_r, 3, "รวมทั้งสิ้น").map_err(map_xlsx_err)?;
  ws.write(total_r, 4, grand_total).map_err(map_xlsx_err)?;

  // Save
  let filename = format!("สรุปรับยา_{year}_เดือน{month}_รอบ{round}.xlsx");
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
  use swift_bill_core::ReceivingSummaryRow;

  fn sample_rows() -> Vec<ReceivingSummaryRow> {
    vec![ReceivingSummaryRow {
      approval_date: "5 ม.ค. 69".to_string(),
      po_date: "5 ม.ค. 69".to_string(),
      receive_date: "5 ม.ค. 69".to_string(),
      company_code: "ABC".to_string(),
      total_amount: 5_000.0,
      receiving_code: 1,
      reg_no: "69ภ12".to_string(),
      running_in_reg: 3,
      invoice_no: "INV001".to_string(),
      request_no: 253,
      report_no: 254,
      po_no: 253,
    }]
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
    let tmp = tempdir("rec");
    let result =
      generate_receiving_summary_excel(&sample_rows(), 2569, 1, 1, tmp.to_str().unwrap());
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
