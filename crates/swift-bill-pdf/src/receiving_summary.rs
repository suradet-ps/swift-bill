//! FILE 2 – สรุปรับยา (Receiving Summary) – A4 Landscape
//!
//! Renders a multi-page table with 12 columns. Multi-page documents get
//! a page footer (e.g. "หน้า 1/2") and the grand total appears on the
//! last page only.

#![allow(clippy::too_many_arguments)]

use printpdf::{Op, PdfDocument};
use swift_bill_core::{ReceivingSummaryParams, ReceivingSummaryRow};

use crate::shared::{
  fmt_money, load_fonts, make_landscape_page, op_box_rect, op_filled_rect, op_hline, op_text,
  op_text_center, op_text_right, op_vline, output_path, pt_f, PageCtx, A4_LAND_H, A4_LAND_W,
  MARGIN,
};

const ROWS_PER_PAGE: usize = 10;

/// Generate the Receiving Summary PDF and write it to `output_dir`.
///
/// Returns the absolute path of the saved file.
pub fn generate_receiving_summary_pdf(
  rows: &[ReceivingSummaryRow],
  params: &ReceivingSummaryParams,
) -> Result<String, String> {
  let year = params.year;
  let month = params.month;
  let month_name = crate::shared::thai_month(month);
  let title = format!(
    "สรุปยอดยาเดือน{} รอบ {} ปีงบประมาณ {}",
    month_name, params.round, year
  );

  let mut doc = PdfDocument::new(&title);
  let (font_id, font_bold_id) = load_fonts(&mut doc)?;

  let ctx = PageCtx {
    font_id: font_id.clone(),
    font_bold_id: font_bold_id.clone(),
    page_w: A4_LAND_W,
    page_h: A4_LAND_H,
  };

  // Column definitions (12 data columns)
  let col_x: &[f64] = &[
    MARGIN,
    MARGIN + 24.0,
    MARGIN + 48.0,
    MARGIN + 72.0,
    MARGIN + 88.0,
    MARGIN + 114.0,
    MARGIN + 130.0,
    MARGIN + 148.0,
    MARGIN + 160.0,
    MARGIN + 186.0,
    MARGIN + 206.0,
    MARGIN + 228.0,
    A4_LAND_W - MARGIN,
  ];
  let cw: Vec<f64> = col_x.windows(2).map(|w| w[1] - w[0]).collect();

  let table_top = MARGIN + 15.0;
  let hdr_h = 18.0;
  let row_h = 12.5;

  // Pre-compute grand total across ALL rows for the last-page totals row
  let grand_total: f64 = rows.iter().map(|r| r.total_amount).sum();

  let chunks: Vec<&[ReceivingSummaryRow]> = rows.chunks(ROWS_PER_PAGE).collect();
  let total_pages = chunks.len().max(1);

  let mut pdf_pages = Vec::new();

  for (page_idx, chunk) in chunks.iter().enumerate() {
    let is_last = page_idx + 1 == total_pages;
    let mut ops: Vec<Op> = Vec::new();

    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });

    // Title
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      19.0,
      0.0,
      A4_LAND_W,
      MARGIN + 6.0,
      &title,
    );

    // Page number footer for multi-page documents
    if total_pages > 1 {
      op_text_right(
        &mut ops,
        &ctx,
        &font_id,
        12.0,
        0.0,
        A4_LAND_W - MARGIN,
        MARGIN + 4.0,
        3.0,
        &format!("หน้า {}/{}", page_idx + 1, total_pages),
      );
    }

    // Header background
    op_filled_rect(
      &mut ops,
      &ctx,
      col_x[0],
      table_top,
      col_x[12] - col_x[0],
      hdr_h,
      0.82,
      0.88,
      0.96,
    );

    // Single-row headers: vertically centred
    let hdr_y_single = table_top + hdr_h * 0.58;
    // Two-row headers
    let hdr_y1 = table_top + hdr_h * 0.32;
    let hdr_y2 = table_top + hdr_h * 0.72;

    // Single-row column headers
    let single_hdrs2: &[(&str, usize)] = &[
      ("วันที่ขออนุมัติ", 0),
      ("วันที่สั่งซื้อ", 1),
      ("วันที่รับของ", 2),
      ("รหัสบริษัท", 3),
      ("จำนวนเงินรวม", 4),
      ("รหัสลงรับยา", 5),
      ("เลขที่ลงรับ", 8),
    ];
    for &(lbl, ci) in single_hdrs2 {
      op_text_center(
        &mut ops,
        &ctx,
        &font_bold_id,
        12.5,
        col_x[ci],
        cw[ci],
        hdr_y_single,
        lbl,
      );
    }

    // Two-row span: เลขทะเบียนคุม (cols 6–7)
    let span_w67 = col_x[8] - col_x[6];
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.5,
      col_x[6],
      span_w67,
      hdr_y1,
      "เลขทะเบียนคุม",
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      col_x[6],
      cw[6],
      hdr_y2,
      "เลขทะเบียน",
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      col_x[7],
      cw[7],
      hdr_y2,
      "ลำดับ",
    );

    // Two-row: ขอซื้อ / รายงาน / ใบสั่งซื้อ with sub-labels
    let two_row_cols: &[(&str, &str, usize)] = &[
      ("ขอซื้อ", "ลบ0033.302/", 9),
      ("รายงาน/อนุมัติ", "ลบ0033.302/", 10),
    ];
    for &(top_lbl, bot_lbl, ci) in two_row_cols {
      op_text_center(
        &mut ops,
        &ctx,
        &font_bold_id,
        12.5,
        col_x[ci],
        cw[ci],
        hdr_y1,
        top_lbl,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        11.5,
        col_x[ci],
        cw[ci],
        hdr_y2,
        bot_lbl,
      );
    }
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.5,
      col_x[11],
      cw[11],
      hdr_y1,
      "ใบสั่งซื้อ",
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_id,
      11.5,
      col_x[11],
      cw[11],
      hdr_y2,
      &format!("…/{}", year),
    );

    // Data rows
    let mut cur_y = table_top + hdr_h;

    for row in *chunk {
      let ty = cur_y + row_h - 1.8;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[0],
        cw[0],
        ty,
        &row.approval_date,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[1],
        cw[1],
        ty,
        &row.po_date,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[2],
        cw[2],
        ty,
        &row.receive_date,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[3],
        cw[3],
        ty,
        &row.company_code,
      );
      op_text_right(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[4],
        cw[4],
        ty,
        1.5,
        &fmt_money(row.total_amount),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[5],
        cw[5],
        ty,
        &row.receiving_code.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[6],
        cw[6],
        ty,
        &row.reg_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[7],
        cw[7],
        ty,
        &row.running_in_reg.to_string(),
      );
      op_text(
        &mut ops,
        &ctx,
        &font_id,
        13.0,
        col_x[8] + 1.0,
        ty,
        &row.invoice_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[9],
        cw[9],
        ty,
        &row.request_no.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[10],
        cw[10],
        ty,
        &row.report_no.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[11],
        cw[11],
        ty,
        &row.po_no.to_string(),
      );
      op_hline(&mut ops, &ctx, col_x[0], col_x[12], cur_y + row_h);
      cur_y += row_h;
    }

    // Last page: totals row
    if is_last {
      op_filled_rect(
        &mut ops,
        &ctx,
        col_x[0],
        cur_y,
        col_x[12] - col_x[0],
        row_h,
        0.93,
        0.93,
        0.93,
      );
      let ty = cur_y + row_h - 1.8;
      op_text_center(
        &mut ops,
        &ctx,
        &font_bold_id,
        15.0,
        col_x[0],
        col_x[4] - col_x[0],
        ty,
        "รวมทั้งสิ้น",
      );
      op_text_right(
        &mut ops,
        &ctx,
        &font_bold_id,
        15.0,
        col_x[4],
        cw[4],
        ty,
        1.5,
        &fmt_money(grand_total),
      );
      op_hline(&mut ops, &ctx, col_x[0], col_x[12], cur_y + row_h);
      cur_y += row_h;
    }

    // Table borders + verticals
    op_box_rect(
      &mut ops,
      &ctx,
      col_x[0],
      table_top,
      col_x[12] - col_x[0],
      cur_y - table_top,
    );
    for &cx in &col_x[1..12usize] {
      op_vline(&mut ops, &ctx, cx, table_top, cur_y);
    }
    op_hline(&mut ops, &ctx, col_x[0], col_x[12], table_top + hdr_h);

    pdf_pages.push(make_landscape_page(ops));
  }

  // Handle empty rows case
  if pdf_pages.is_empty() {
    let mut ops = Vec::new();
    ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      19.0,
      0.0,
      A4_LAND_W,
      MARGIN + 6.0,
      &title,
    );
    pdf_pages.push(make_landscape_page(ops));
  }

  doc.with_pages(pdf_pages);

  let filename = format!("สรุปรับยา_{year}_เดือน{month}_รอบ{}.pdf", params.round);
  let path_str = output_path(&params.output_dir, filename);
  crate::shared::save_pdf(&doc, &path_str)?;
  Ok(path_str)
}
