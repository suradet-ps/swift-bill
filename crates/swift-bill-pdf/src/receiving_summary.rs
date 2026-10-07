//! FILE 2 - สรุปรับยา (Receiving Summary) - A4 Landscape
//!
//! Renders a multi-page table with 12 columns. Every page repeats the document
//! header and the accent column band; the grand total and the signature block
//! appear on the last page only.

#![allow(clippy::too_many_arguments)]

use printpdf::{Op, PdfDocument};
use swift_bill_core::ReceivingSummaryRow;

use crate::shared::{
  A4_LAND_W, COLOR_ACCENT, COLOR_BORDER, COLOR_INK, COLOR_TINT, COLOR_WHITE, COLOR_ZEBRA, MARGIN,
  PageCtx, fmt_money, load_fonts, make_landscape_page, op_box_rect, op_doc_header, op_filled_rect,
  op_hline_colored, op_page_footer, op_set_stroke, op_signature_block, op_text_center,
  op_text_center_colored, op_text_right, op_text_right_colored, op_vline_colored, output_path,
  pt_f, thai_month,
};

const ROWS_PER_PAGE: usize = 8;
const TABLE_TOP: f64 = 42.0;
const HDR_H: f64 = 17.0;
const ROW_H: f64 = 12.0;

/// Column widths (mm), summing to the printable width of 267 mm.
const COL_W: [f64; 12] = [
  26.0, 24.0, 24.0, 18.0, 30.0, 18.0, 20.0, 12.0, 22.0, 24.0, 25.0, 24.0,
];

/// Left edge (mm) of every column, including both outer table edges.
fn col_x() -> Vec<f64> {
  let mut xs = vec![MARGIN];
  let mut x = MARGIN;
  for w in COL_W {
    x += w;
    xs.push(x);
  }
  xs
}

/// Generate the Receiving Summary PDF and write it to `output_dir`.
///
/// Returns the absolute path of the saved file.
pub fn generate_receiving_summary_pdf(
  rows: &[ReceivingSummaryRow],
  year: i32,
  month: u32,
  round: u32,
  output_dir: &str,
) -> Result<String, String> {
  let month_name = thai_month(month);
  let title = "สรุปรับยา";
  let subtitle = format!("เดือน{} รอบ {} ปีงบประมาณ {}", month_name, round, year);

  let mut doc = PdfDocument::new(title);
  let (font_id, font_bold_id) = load_fonts(&mut doc)?;

  let ctx = PageCtx {
    font_id: font_id.clone(),
    font_bold_id: font_bold_id.clone(),
    page_w: A4_LAND_W,
    page_h: crate::shared::A4_LAND_H,
  };

  let xs = col_x();
  let table_right = xs[12];
  let table_w = table_right - xs[0];

  let grand_total: f64 = rows.iter().map(|r| r.total_amount).sum();
  let chunks: Vec<&[ReceivingSummaryRow]> = rows.chunks(ROWS_PER_PAGE).collect();
  let total_pages = chunks.len().max(1);

  let mut pdf_pages = Vec::new();

  if rows.is_empty() {
    let mut ops: Vec<Op> = Vec::new();
    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });
    op_doc_header(&mut ops, &ctx, &font_id, title, &subtitle);
    op_page_footer(&mut ops, &ctx, 0, 1);
    op_set_stroke(&mut ops, COLOR_BORDER);
    op_box_rect(&mut ops, &ctx, MARGIN, 70.0, table_w, 24.0);
    op_text_center(
      &mut ops,
      &ctx,
      &font_id,
      14.0,
      MARGIN,
      table_w,
      84.0,
      "ไม่มีข้อมูลสำหรับช่วงเวลานี้",
    );
    pdf_pages.push(make_landscape_page(ops));
  }

  for (page_idx, chunk) in chunks.iter().enumerate() {
    let is_last = page_idx + 1 == total_pages;
    let mut ops: Vec<Op> = Vec::new();

    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });
    op_doc_header(&mut ops, &ctx, &font_id, title, &subtitle);
    op_page_footer(&mut ops, &ctx, page_idx, total_pages);

    // Accent header band
    op_filled_rect(
      &mut ops,
      &ctx,
      xs[0],
      TABLE_TOP,
      table_w,
      HDR_H,
      COLOR_ACCENT.0,
      COLOR_ACCENT.1,
      COLOR_ACCENT.2,
    );

    let y1 = TABLE_TOP + 6.5;
    let y2 = TABLE_TOP + 13.0;
    let y3 = TABLE_TOP + 15.5;

    let two_line: &[(&str, &str, usize)] = &[
      ("วันที่", "ขออนุมัติ", 0),
      ("วันที่", "สั่งซื้อ", 1),
      ("วันที่", "รับของ", 2),
    ];
    for &(top_lbl, bot_lbl, ci) in two_line {
      op_text_center_colored(
        &mut ops,
        &ctx,
        &font_bold_id,
        12.0,
        xs[ci],
        COL_W[ci],
        y1,
        top_lbl,
        COLOR_WHITE,
      );
      op_text_center_colored(
        &mut ops,
        &ctx,
        &font_bold_id,
        12.0,
        xs[ci],
        COL_W[ci],
        y2,
        bot_lbl,
        COLOR_WHITE,
      );
    }

    let single_hdrs: &[(&str, usize)] = &[
      ("รหัสบริษัท", 3),
      ("จำนวนเงินรวม", 4),
      ("รหัสลงรับยา", 5),
      ("เลขที่ลงรับ", 8),
    ];
    for &(lbl, ci) in single_hdrs {
      op_text_center_colored(
        &mut ops,
        &ctx,
        &font_bold_id,
        12.0,
        xs[ci],
        COL_W[ci],
        y1 + 3.0,
        lbl,
        COLOR_WHITE,
      );
    }

    // เลขทะเบียนคุม spans columns 6-7
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[6],
      COL_W[6] + COL_W[7],
      y1,
      "เลขทะเบียนคุม",
      COLOR_WHITE,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      10.5,
      xs[6],
      COL_W[6],
      y2,
      "เลขทะเบียน",
      COLOR_WHITE,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[7],
      COL_W[7],
      y2,
      "ลำดับ",
      COLOR_WHITE,
    );

    // ขอซื้อ (ลบ0033.302/)
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[9],
      COL_W[9],
      y1,
      "ขอซื้อ",
      COLOR_WHITE,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      10.0,
      xs[9],
      COL_W[9],
      y3,
      "(ลบ0033.302/)",
      COLOR_WHITE,
    );

    // รายงาน/อนุมัติ (ลบ0033.302/)
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[10],
      COL_W[10],
      y1 - 1.5,
      "รายงาน/",
      COLOR_WHITE,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[10],
      COL_W[10],
      y2 - 1.5,
      "อนุมัติ",
      COLOR_WHITE,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      10.0,
      xs[10],
      COL_W[10],
      y3,
      "(ลบ0033.302/)",
      COLOR_WHITE,
    );

    // ใบสั่งซื้อ …/{year}
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[11],
      COL_W[11],
      y1,
      "ใบสั่งซื้อ",
      COLOR_WHITE,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      11.0,
      xs[11],
      COL_W[11],
      y2,
      &format!("…/{year}"),
      COLOR_WHITE,
    );

    // Data rows with warm zebra striping
    let mut cur_y = TABLE_TOP + HDR_H;

    for (i, row) in chunk.iter().enumerate() {
      if i % 2 == 1 {
        op_filled_rect(
          &mut ops,
          &ctx,
          xs[0],
          cur_y,
          table_w,
          ROW_H,
          COLOR_ZEBRA.0,
          COLOR_ZEBRA.1,
          COLOR_ZEBRA.2,
        );
      }
      let ty = cur_y + ROW_H - 3.4;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[0],
        COL_W[0],
        ty,
        &row.approval_date,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[1],
        COL_W[1],
        ty,
        &row.po_date,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[2],
        COL_W[2],
        ty,
        &row.receive_date,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[3],
        COL_W[3],
        ty,
        &row.company_code,
      );
      op_text_right(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[4],
        COL_W[4],
        ty,
        1.5,
        &fmt_money(row.total_amount),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[5],
        COL_W[5],
        ty,
        &row.receiving_code.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[6],
        COL_W[6],
        ty,
        &row.reg_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[7],
        COL_W[7],
        ty,
        &row.running_in_reg.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[8],
        COL_W[8],
        ty,
        &row.invoice_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[9],
        COL_W[9],
        ty,
        &row.request_no.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[10],
        COL_W[10],
        ty,
        &row.report_no.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        xs[11],
        COL_W[11],
        ty,
        &row.po_no.to_string(),
      );
      op_hline_colored(
        &mut ops,
        &ctx,
        xs[0],
        table_right,
        cur_y + ROW_H,
        0.3,
        COLOR_BORDER,
      );
      cur_y += ROW_H;
    }

    if is_last {
      // Totals row
      op_filled_rect(
        &mut ops,
        &ctx,
        xs[0],
        cur_y,
        table_w,
        ROW_H,
        COLOR_TINT.0,
        COLOR_TINT.1,
        COLOR_TINT.2,
      );
      op_hline_colored(&mut ops, &ctx, xs[0], table_right, cur_y, 0.8, COLOR_ACCENT);
      let ty = cur_y + ROW_H - 3.2;
      op_text_center_colored(
        &mut ops,
        &ctx,
        &font_bold_id,
        13.5,
        xs[0],
        xs[4] - xs[0],
        ty,
        "รวมทั้งสิ้น",
        COLOR_INK,
      );
      op_text_right_colored(
        &mut ops,
        &ctx,
        &font_bold_id,
        13.5,
        xs[4],
        COL_W[4],
        ty,
        1.5,
        &fmt_money(grand_total),
        COLOR_ACCENT,
      );
      op_hline_colored(
        &mut ops,
        &ctx,
        xs[0],
        table_right,
        cur_y + ROW_H,
        0.8,
        COLOR_ACCENT,
      );
      cur_y += ROW_H;

      op_signature_block(
        &mut ops,
        &ctx,
        &font_id,
        cur_y + 10.0,
        "ผู้จัดทำ",
        "เจ้าหน้าที่พัสดุ",
        "ผู้ตรวจสอบ",
        "หัวหน้ากลุ่มงานเภสัชกรรมฯ",
      );
    }

    for &cx in &xs[1..12usize] {
      op_vline_colored(
        &mut ops,
        &ctx,
        cx,
        TABLE_TOP + HDR_H,
        cur_y,
        0.3,
        COLOR_BORDER,
      );
    }

    pdf_pages.push(make_landscape_page(ops));
  }

  doc.with_pages(pdf_pages);

  let filename = format!("สรุปรับยา_{year}_เดือน{month}_รอบ{round}.pdf");
  let path_str = output_path(output_dir, filename);
  crate::shared::save_pdf(&doc, &path_str)?;
  Ok(path_str)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_rows() -> Vec<ReceivingSummaryRow> {
    vec![
      ReceivingSummaryRow {
        approval_date: "1/7/2569".into(),
        po_date: "2/7/2569".into(),
        receive_date: "3/7/2569".into(),
        company_code: "C001".into(),
        total_amount: 5432.1,
        receiving_code: 1,
        reg_no: "69ภ12".into(),
        running_in_reg: 0,
        invoice_no: "INV-0001".into(),
        request_no: 100,
        report_no: 101,
        po_no: 100,
      },
      ReceivingSummaryRow {
        approval_date: "2/7/2569".into(),
        po_date: "3/7/2569".into(),
        receive_date: "4/7/2569".into(),
        company_code: "C002".into(),
        total_amount: 1234.56,
        receiving_code: 2,
        reg_no: "69ภ12".into(),
        running_in_reg: 1,
        invoice_no: "INV-0002".into(),
        request_no: 102,
        report_no: 103,
        po_no: 101,
      },
    ]
  }

  fn test_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().to_string()
  }

  #[test]
  fn generates_pdf_file_with_rows() {
    let path =
      generate_receiving_summary_pdf(&sample_rows(), 2569, 7, 1, &test_dir("sb-pdf-rec")).unwrap();
    assert!(path.ends_with(".pdf"));
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(size > 1000, "pdf should not be empty, got {size} bytes");
  }

  #[test]
  fn generates_pdf_file_without_rows() {
    let path =
      generate_receiving_summary_pdf(&[], 2569, 7, 1, &test_dir("sb-pdf-rec-empty")).unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() > 1000);
  }
}
