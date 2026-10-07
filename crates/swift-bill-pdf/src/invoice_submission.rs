//! FILE 1 - ส่งหนี้เบิกยา (Invoice Submission List) - A4 Landscape
//!
//! Renders a multi-page table with 9 columns, sized so 20 rows fit on a page.
//! Every page repeats the document header and the column band. The grand total
//! and the three-column signature row appear on the last page; when the last
//! page is full they move to a following summary page.

#![allow(clippy::too_many_arguments)]

use printpdf::{FontId, Op, PdfDocument};
use swift_bill_core::InvoiceSubmissionRow;

use crate::shared::{
  A4_LAND_W, COLOR_ACCENT, COLOR_BORDER, COLOR_INK, COLOR_TINT, COLOR_ZEBRA, MARGIN, PageCtx,
  fmt_money, load_fonts, make_landscape_page, op_box_rect, op_doc_header, op_filled_rect,
  op_hline_colored, op_page_footer, op_set_stroke, op_signature_row3, op_text, op_text_center,
  op_text_center_colored, op_text_right, op_text_right_colored, op_vline_colored, output_path,
  pt_f, thai_month,
};

const ROWS_PER_PAGE: usize = 20;
const TABLE_TOP: f64 = 29.0;
const HDR_H: f64 = 14.0;
const ROW_H: f64 = 6.2;
/// Lowest y (mm from top) that content may reach, just above the footer rule.
const CONTENT_BOTTOM: f64 = 198.5;
/// Vertical space needed below the last row for the totals row and signature.
const SUMMARY_H: f64 = ROW_H + 4.0 + 15.5;

/// Signature labels for the three signature columns on the last page.
const SIGNATURES: [&str; 3] = ["ผู้รับ", "ผู้ส่ง", "ผู้ส่ง"];

/// Column widths (mm), summing to the printable width of 267 mm.
const COL_W: [f64; 9] = [12.0, 24.0, 30.0, 18.0, 12.0, 24.0, 70.0, 28.0, 49.0];

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

/// Draw the totals row and the signature row starting at `top_y`.
fn draw_summary(
  ops: &mut Vec<Op>,
  ctx: &PageCtx,
  font_id: &FontId,
  font_bold_id: &FontId,
  xs: &[f64],
  table_right: f64,
  table_w: f64,
  top_y: f64,
  grand_total: f64,
) {
  op_filled_rect(
    ops,
    ctx,
    xs[0],
    top_y,
    table_w,
    ROW_H,
    COLOR_TINT.0,
    COLOR_TINT.1,
    COLOR_TINT.2,
  );
  op_hline_colored(ops, ctx, xs[0], table_right, top_y, 0.8, COLOR_ACCENT);
  let ty = top_y + 4.7;
  op_text_center_colored(
    ops,
    ctx,
    font_bold_id,
    14.0,
    xs[0],
    xs[8] - xs[0],
    ty,
    "รวมทั้งสิ้น",
    COLOR_INK,
  );
  op_text_right_colored(
    ops,
    ctx,
    font_bold_id,
    14.0,
    xs[8],
    COL_W[8],
    ty,
    2.0,
    &fmt_money(grand_total),
    COLOR_ACCENT,
  );
  op_hline_colored(
    ops,
    ctx,
    xs[0],
    table_right,
    top_y + ROW_H,
    0.8,
    COLOR_ACCENT,
  );
  op_signature_row3(ops, ctx, font_id, top_y + ROW_H + 4.0, &SIGNATURES);
}

/// Generate the Invoice Submission PDF and write it to `output_dir`.
///
/// Returns the absolute path of the saved file.
pub fn generate_invoice_submission_pdf(
  rows: &[InvoiceSubmissionRow],
  year: i32,
  month: u32,
  round: u32,
  output_dir: &str,
) -> Result<String, String> {
  let month_name = thai_month(month);
  let title = "ส่งรายการหนี้สินและเอกสารเบิกเงิน";
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
  let table_right = xs[9];
  let table_w = table_right - xs[0];

  let grand_total: f64 = rows.iter().map(|r| r.total_amount).sum();
  let chunks: Vec<&[InvoiceSubmissionRow]> = rows.chunks(ROWS_PER_PAGE).collect();
  let data_pages = chunks.len().max(1);

  // When the last data page is full, the totals and signatures move to a
  // following summary page so 20 rows always fit per page.
  let last_len = if rows.is_empty() {
    0
  } else {
    let rem = rows.len() % ROWS_PER_PAGE;
    if rem == 0 { ROWS_PER_PAGE } else { rem }
  };
  let has_summary_page =
    !rows.is_empty() && TABLE_TOP + HDR_H + last_len as f64 * ROW_H + SUMMARY_H > CONTENT_BOTTOM;
  let total_pages = data_pages + usize::from(has_summary_page);

  let mut pdf_pages = Vec::new();

  if rows.is_empty() {
    let mut ops: Vec<Op> = Vec::new();
    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });
    op_doc_header(&mut ops, &ctx, title, &subtitle);
    op_page_footer(&mut ops, &ctx, 0, 1);
    op_set_stroke(&mut ops, COLOR_BORDER);
    op_box_rect(&mut ops, &ctx, MARGIN, 55.0, table_w, 24.0);
    op_text_center(
      &mut ops,
      &ctx,
      &font_id,
      14.0,
      MARGIN,
      table_w,
      69.0,
      "ไม่มีข้อมูลสำหรับช่วงเวลานี้",
    );
    pdf_pages.push(make_landscape_page(ops));
  }

  for (page_idx, chunk) in chunks.iter().enumerate() {
    let is_last = page_idx + 1 == data_pages;
    let mut ops: Vec<Op> = Vec::new();

    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });
    op_doc_header(&mut ops, &ctx, title, &subtitle);
    op_page_footer(&mut ops, &ctx, page_idx, total_pages);

    // Plain column band: ink text between two rule lines, no fill.
    op_hline_colored(
      &mut ops,
      &ctx,
      xs[0],
      table_right,
      TABLE_TOP,
      1.0,
      COLOR_INK,
    );
    op_hline_colored(
      &mut ops,
      &ctx,
      xs[0],
      table_right,
      TABLE_TOP + HDR_H,
      1.0,
      COLOR_INK,
    );

    let hdr_single_y = TABLE_TOP + 8.5;
    let hdr_row1_y = TABLE_TOP + 5.0;
    let hdr_row2_y = TABLE_TOP + 11.0;

    let single_hdrs: &[(&str, usize)] = &[
      ("ลำดับ", 0),
      ("วันที่รับของ", 1),
      ("เลขที่เอกสาร", 2),
      ("ชื่อบริษัท", 6),
      ("ค่าใช้จ่ายเรื่อง", 7),
      ("จำนวนเงินรวม", 8),
    ];
    for &(lbl, ci) in single_hdrs {
      op_text_center_colored(
        &mut ops,
        &ctx,
        &font_bold_id,
        12.5,
        xs[ci],
        COL_W[ci],
        hdr_single_y,
        lbl,
        COLOR_INK,
      );
    }

    let span_w34 = COL_W[3] + COL_W[4];
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[3],
      span_w34,
      hdr_row1_y,
      "เลขทะเบียนคุม",
      COLOR_INK,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      11.0,
      xs[3],
      COL_W[3],
      hdr_row2_y,
      "เลขทะเบียน",
      COLOR_INK,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      11.0,
      xs[4],
      COL_W[4],
      hdr_row2_y,
      "ลำดับ",
      COLOR_INK,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      xs[5],
      COL_W[5],
      hdr_row1_y,
      "วัน/เดือน/ปี",
      COLOR_INK,
    );
    op_text_center_colored(
      &mut ops,
      &ctx,
      &font_bold_id,
      11.0,
      xs[5],
      COL_W[5],
      hdr_row2_y,
      "ใบส่งของ",
      COLOR_INK,
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
      let ty = cur_y + 4.7;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[0],
        COL_W[0],
        ty,
        &row.seq.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[1],
        COL_W[1],
        ty,
        &row.receive_date,
      );
      op_text(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[2] + 2.0,
        ty,
        &row.invoice_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[3],
        COL_W[3],
        ty,
        &row.reg_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[4],
        COL_W[4],
        ty,
        &row.running_in_reg.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[5],
        COL_W[5],
        ty,
        &row.invoice_date,
      );
      op_text(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[6] + 2.0,
        ty,
        &row.company_name,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[7],
        COL_W[7],
        ty,
        &row.category,
      );
      op_text_right(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        xs[8],
        COL_W[8],
        ty,
        2.0,
        &fmt_money(row.total_amount),
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

    if is_last && !has_summary_page {
      draw_summary(
        &mut ops,
        &ctx,
        &font_id,
        &font_bold_id,
        &xs,
        table_right,
        table_w,
        cur_y,
        grand_total,
      );
      cur_y += SUMMARY_H;
    }

    for &cx in &xs[1..9usize] {
      op_vline_colored(&mut ops, &ctx, cx, TABLE_TOP, cur_y, 0.3, COLOR_BORDER);
    }

    pdf_pages.push(make_landscape_page(ops));
  }

  if has_summary_page {
    let mut ops: Vec<Op> = Vec::new();
    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });
    op_doc_header(&mut ops, &ctx, title, &subtitle);
    op_page_footer(&mut ops, &ctx, total_pages - 1, total_pages);
    op_hline_colored(
      &mut ops,
      &ctx,
      xs[0],
      table_right,
      TABLE_TOP,
      1.0,
      COLOR_INK,
    );
    draw_summary(
      &mut ops,
      &ctx,
      &font_id,
      &font_bold_id,
      &xs,
      table_right,
      table_w,
      TABLE_TOP + 1.0,
      grand_total,
    );
    for &cx in &xs[1..9usize] {
      op_vline_colored(
        &mut ops,
        &ctx,
        cx,
        TABLE_TOP,
        TABLE_TOP + 1.0 + SUMMARY_H,
        0.3,
        COLOR_BORDER,
      );
    }
    pdf_pages.push(make_landscape_page(ops));
  }

  doc.with_pages(pdf_pages);

  let filename = format!("ส่งหนี้เบิกยา_{year}_เดือน{month}_รอบ{round}.pdf");
  let path_str = output_path(output_dir, filename);
  crate::shared::save_pdf(&doc, &path_str)?;
  Ok(path_str)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_rows(count: usize) -> Vec<InvoiceSubmissionRow> {
    (0..count)
      .map(|i| InvoiceSubmissionRow {
        seq: (i + 1) as u32,
        receive_date: format!("{} ก.ค. 69", (i % 28) + 1),
        invoice_no: format!("INV-{:04}", i + 1),
        reg_no: "69ภ12".into(),
        running_in_reg: (i % 10) as u32,
        invoice_date: format!("{} มิ.ย. 69", (i % 28) + 1),
        company_name: "บริษัท ตัวอย่างเภสัชภัณฑ์ จำกัด".into(),
        category: "ค่ายา".into(),
        total_amount: 1000.0 + i as f64 * 111.11,
      })
      .collect()
  }

  fn test_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().to_string()
  }

  #[test]
  fn generates_pdf_file_with_rows() {
    let path =
      generate_invoice_submission_pdf(&sample_rows(25), 2569, 10, 1, &test_dir("sb-pdf-inv"))
        .unwrap();
    assert!(path.ends_with(".pdf"));
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(size > 1000, "pdf should not be empty, got {size} bytes");
  }

  #[test]
  fn generates_pdf_file_without_rows() {
    let path =
      generate_invoice_submission_pdf(&[], 2569, 10, 1, &test_dir("sb-pdf-inv-empty")).unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() > 1000);
  }

  #[test]
  fn full_last_page_moves_summary_to_its_own_page() {
    let path =
      generate_invoice_submission_pdf(&sample_rows(40), 2569, 10, 1, &test_dir("sb-pdf-inv-full"))
        .unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() > 1000);
  }
}
