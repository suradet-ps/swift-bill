//! FILE 1 – ส่งหนี้เบิกยา (Invoice Submission List) – A4 Landscape
//!
//! Renders a multi-page table with 9 columns. Multi-page documents get
//! a page footer (e.g. "หน้า 1/3") and the grand total appears on the
//! last page only, followed by a signature block (ผู้จัดทำ / ผู้ตรวจสอบ).

#![allow(clippy::too_many_arguments)]

use printpdf::{Op, PdfDocument};
use swift_bill_core::{InvoiceSubmissionParams, InvoiceSubmissionRow};

use crate::shared::{
  fmt_money, load_fonts, make_landscape_page, op_box_rect, op_filled_rect, op_hline, op_text,
  op_text_center, op_text_right, op_vline, output_path, pt_f, PageCtx, A4_LAND_H, A4_LAND_W,
  MARGIN,
};

const ROWS_PER_PAGE: usize = 7;

/// Generate the Invoice Submission PDF and write it to `output_dir`.
///
/// Returns the absolute path of the saved file.
pub fn generate_invoice_submission_pdf(
  rows: &[InvoiceSubmissionRow],
  params: &InvoiceSubmissionParams,
) -> Result<String, String> {
  let year = params.year;
  let month = params.month;
  let month_name = crate::shared::thai_month(month);
  let title = "ส่งรายการหนี้สินและเอกสารเบิกเงิน";
  let subtitle = format!(
    "เดือน{} รอบ {}/ ปีงบประมาณ {}",
    month_name, params.round, year
  );

  let mut doc = PdfDocument::new(title);
  let (font_id, font_bold_id) = load_fonts(&mut doc)?;

  let ctx = PageCtx {
    font_id: font_id.clone(),
    font_bold_id: font_bold_id.clone(),
    page_w: A4_LAND_W,
    page_h: A4_LAND_H,
  };

  // Column X positions (mm from left)
  let col_x: &[f64] = &[
    MARGIN,
    MARGIN + 12.0,
    MARGIN + 34.0,
    MARGIN + 62.0,
    MARGIN + 80.0,
    MARGIN + 92.0,
    MARGIN + 116.0,
    MARGIN + 178.0,
    MARGIN + 212.0,
    A4_LAND_W - MARGIN,
  ];
  let cw: Vec<f64> = col_x.windows(2).map(|w| w[1] - w[0]).collect();

  let table_top = MARGIN + 19.0;
  let hdr_h = 16.0;
  let row_h = 12.5;

  // Pre-compute grand total across ALL rows for the last-page totals row
  let grand_total: f64 = rows.iter().map(|r| r.total_amount).sum();

  let chunks: Vec<&[InvoiceSubmissionRow]> = rows.chunks(ROWS_PER_PAGE).collect();
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
      20.0,
      0.0,
      A4_LAND_W,
      MARGIN + 6.0,
      title,
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      18.0,
      0.0,
      A4_LAND_W,
      MARGIN + 13.0,
      &subtitle,
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
      col_x[9] - col_x[0],
      hdr_h,
      0.82,
      0.88,
      0.96,
    );

    // Header labels
    let hdr_y_single = table_top + hdr_h * 0.58;
    let hdr_y1 = table_top + hdr_h * 0.34;
    let hdr_y2 = table_top + hdr_h * 0.76;

    // Single-row column headers
    let single_hdrs: &[(&str, usize)] = &[
      ("ลำดับ", 0),
      ("วันที่รับของ", 1),
      ("เลขที่เอกสาร", 2),
      ("ชื่อบริษัท", 6),
      ("ค่าใช้จ่ายเรื่อง", 7),
      ("จำนวนเงินรวม", 8),
    ];
    for &(lbl, ci) in single_hdrs {
      op_text_center(
        &mut ops,
        &ctx,
        &font_bold_id,
        13.0,
        col_x[ci],
        cw[ci],
        hdr_y_single,
        lbl,
      );
    }

    // Two-row span: เลขทะเบียนคุม (cols 3–4)
    let span_w34 = cw[3] + cw[4];
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      13.0,
      col_x[3],
      span_w34,
      hdr_y1,
      "เลขทะเบียนคุม",
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      col_x[3],
      cw[3],
      hdr_y2,
      "เลขทะเบียน",
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      col_x[4],
      cw[4],
      hdr_y2,
      "ลำดับ",
    );

    // Two-row: วัน/เดือน/ปีใบส่งของ (col 5)
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      13.0,
      col_x[5],
      cw[5],
      hdr_y1,
      "วัน/เดือน/ปี",
    );
    op_text_center(
      &mut ops,
      &ctx,
      &font_bold_id,
      12.0,
      col_x[5],
      cw[5],
      hdr_y2,
      "ใบส่งของ",
    );

    // Data rows
    let mut cur_y = table_top + hdr_h;

    for row in *chunk {
      let ty = cur_y + row_h - 1.8;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.5,
        col_x[0],
        cw[0],
        ty,
        &row.seq.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        col_x[1],
        cw[1],
        ty,
        &row.receive_date,
      );
      op_text(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[2] + 1.0,
        ty,
        &row.invoice_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        col_x[3],
        cw[3],
        ty,
        &row.reg_no,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        col_x[4],
        cw[4],
        ty,
        &row.running_in_reg.to_string(),
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        14.0,
        col_x[5],
        cw[5],
        ty,
        &row.invoice_date,
      );
      op_text(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[6] + 1.0,
        ty,
        &row.company_name,
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        13.5,
        col_x[7],
        cw[7],
        ty,
        &row.category,
      );
      op_text_right(
        &mut ops,
        &ctx,
        &font_id,
        14.5,
        col_x[8],
        cw[8],
        ty,
        2.0,
        &fmt_money(row.total_amount),
      );
      op_hline(&mut ops, &ctx, col_x[0], col_x[9], cur_y + row_h);
      cur_y += row_h;
    }

    // Last page: totals row + signature block
    if is_last {
      op_filled_rect(
        &mut ops,
        &ctx,
        col_x[0],
        cur_y,
        col_x[9] - col_x[0],
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
        col_x[8] - col_x[0],
        ty,
        "รวมทั้งสิ้น",
      );
      op_text_right(
        &mut ops,
        &ctx,
        &font_bold_id,
        15.0,
        col_x[8],
        cw[8],
        ty,
        2.0,
        &fmt_money(grand_total),
      );
      op_hline(&mut ops, &ctx, col_x[0], col_x[9], cur_y + row_h);
      cur_y += row_h;

      // Signature block
      let sig_y = cur_y + 12.0;
      op_text_center(&mut ops, &ctx, &font_id, 16.0, MARGIN, 60.0, sig_y, "ผู้จัดทำ");
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        A4_LAND_W - MARGIN - 65.0,
        65.0,
        sig_y,
        "ผู้ตรวจสอบ",
      );
      let sig2 = sig_y + 8.0;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        MARGIN,
        60.0,
        sig2,
        "ลงชื่อ................................",
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        A4_LAND_W - MARGIN - 65.0,
        65.0,
        sig2,
        "ลงชื่อ................................",
      );
      let sig3 = sig2 + 7.0;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        MARGIN,
        60.0,
        sig3,
        "(............................................)",
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        A4_LAND_W - MARGIN - 65.0,
        65.0,
        sig3,
        "(............................................)",
      );
      let sig4 = sig3 + 6.0;
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        MARGIN,
        60.0,
        sig4,
        "เจ้าหน้าที่พัสดุ",
      );
      op_text_center(
        &mut ops,
        &ctx,
        &font_id,
        16.0,
        A4_LAND_W - MARGIN - 65.0,
        65.0,
        sig4,
        "หัวหน้ากลุ่มงานเภสัชกรรมฯ",
      );
    }

    // Table box + vertical lines
    op_box_rect(
      &mut ops,
      &ctx,
      col_x[0],
      table_top,
      col_x[9] - col_x[0],
      cur_y - table_top,
    );
    for &cx in &col_x[1..9usize] {
      op_vline(&mut ops, &ctx, cx, table_top, cur_y);
    }
    op_hline(&mut ops, &ctx, col_x[0], col_x[9], table_top + hdr_h);

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
      20.0,
      0.0,
      A4_LAND_W,
      MARGIN + 6.0,
      title,
    );
    pdf_pages.push(make_landscape_page(ops));
  }

  doc.with_pages(pdf_pages);

  let filename = format!("ส่งหนี้เบิกยา_{year}_เดือน{month}_รอบ{}.pdf", params.round);
  let path_str = output_path(&params.output_dir, filename);
  crate::shared::save_pdf(&doc, &path_str)?;
  Ok(path_str)
}
