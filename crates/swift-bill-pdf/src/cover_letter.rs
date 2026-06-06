//! FILE 3 – เบิกยาปะหน้า (Disbursement Cover Letters) – A4 Portrait
//!
//! Pure-printpdf implementation that builds the cover letter from scratch
//! on each page. The `cover_letter_template` module is the actively used
//! path (a pre-built PDF template + lopdf overlay); this module is kept
//! for parity with the original codebase.

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use printpdf::{FontId, Op, PdfDocument};
use swift_bill_core::{CoverLetterPage, CoverLettersParams};

use crate::shared::{
  A4_PORT_H, A4_PORT_W, MARGIN, PageCtx, fmt_money, load_fonts, make_portrait_page, op_box_rect,
  op_filled_rect, op_hline, op_text, op_text_center, op_text_right, op_vline, output_path, pt_f,
};

/// Public entry point — builds a multi-page A4 portrait PDF where each
/// [`CoverLetterPage`] becomes one page.
pub fn generate_cover_letters_pdf(
  pages: &[CoverLetterPage],
  params: &CoverLettersParams,
) -> Result<String, String> {
  generate_combined_cover(pages, params)
}

fn build_cover_letter_ops(
  page: &CoverLetterPage,
  _params: &CoverLettersParams,
  font_id: &FontId,
  font_bold_id: &FontId,
) -> Vec<Op> {
  let ctx = PageCtx {
    font_id: font_id.clone(),
    font_bold_id: font_bold_id.clone(),
    page_w: A4_PORT_W,
    page_h: A4_PORT_H,
  };

  let mut ops: Vec<Op> = Vec::new();
  ops.push(Op::SetOutlineThickness { pt: pt_f(0.3) });

  let lm = 20.0_f64;
  let rm = A4_PORT_W - 15.0;
  let pw = A4_PORT_W;

  let title_size = 16.0;
  let body_size = 16.0;

  let mut y = MARGIN + 8.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_bold_id,
    title_size,
    0.0,
    pw,
    y,
    "บันทึกข้อความ",
  );

  y += 12.0;
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm,
    y,
    "ส่วนราชการ.....กลุ่มงาน/ฝ่าย..กลุ่มงานเภสัชกรรมและคุ้มครองผู้บริโภค",
  );
  y += 8.0;
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm,
    y,
    "โรงพยาบาลสระโบสถ์....อำเภอสระโบสถ์.....จังหวัดลพบุรี",
  );

  y += 9.0;
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm,
    y,
    "ที่     ลบ 0033/พิเศษ",
  );
  op_text(&mut ops, &ctx, font_id, body_size, lm + 80.0, y, "วันที่");
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 93.0,
    y,
    &page.date_text,
  );
  op_hline(&mut ops, &ctx, lm + 93.0, rm, y + 2.0);

  y += 9.0;
  op_text(&mut ops, &ctx, font_bold_id, body_size, lm, y, "เรื่อง");
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 15.0,
    y,
    "ขออนุมัติเบิกเงิน ตามโครงการ/กิจกรรม พัฒนาระบบบริหารคลัง/พัสดุ/ครุภัณฑ์",
  );

  y += 9.0;
  op_text(&mut ops, &ctx, font_bold_id, body_size, lm, y, "เรียน");
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 15.0,
    y,
    "ปลัดกระทรวงสาธารณสุข",
  );

  y += 10.0;
  op_text(&mut ops, &ctx, font_id, body_size, lm + 10.0, y, "ด้วย");
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 24.0,
    y,
    "กลุ่มงานเภสัชกรรมและคุ้มครองผู้บริโภค",
  );
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 112.0,
    y,
    "มีความประสงค์จะขอเบิกเงิน",
  );

  y += 9.0;
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm,
    y,
    &format!("ค่า.....{}.....", page.category),
  );
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 58.0,
    y,
    &format!("โดยขอใช้งบประมาณประจำปี..........{}...", page.fiscal_year),
  );

  y += 9.0;
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 10.0,
    y,
    "แผนงาน/โครงการ",
  );
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 49.0,
    y,
    &format!("ซื้อยาจาก{}............", page.company_name),
  );

  // Budget table
  y += 12.0;
  let tbl_x = lm;
  let tbl_w = rm - lm;
  let tbl_col_w = tbl_w / 5.0;
  let tbl_h = 10.0;

  op_filled_rect(&mut ops, &ctx, tbl_x, y, tbl_w, tbl_h, 0.82, 0.88, 0.96);

  let budget_hdrs = [
    "ยอดเงินจัดสรร",
    "ยอดเบิกจ่ายแล้ว",
    "ยอดคงเหลือ",
    "เบิกจ่ายครั้งนี้",
    "ยอดเงินคงเหลือ",
  ];
  for (i, lbl) in budget_hdrs.iter().enumerate() {
    let cx = tbl_x + i as f64 * tbl_col_w;
    op_text_center(
      &mut ops,
      &ctx,
      font_bold_id,
      13.0,
      cx,
      tbl_col_w,
      y + tbl_h * 0.62,
      lbl,
    );
  }

  let val_y = y + tbl_h;
  let budget_vals = [
    fmt_money(page.budget_total),
    fmt_money(page.previous_spent),
    fmt_money(page.previous_balance),
    fmt_money(page.current_amount),
    fmt_money(page.remaining_balance),
  ];
  for (i, val) in budget_vals.iter().enumerate() {
    let cx = tbl_x + i as f64 * tbl_col_w;
    op_text_right(
      &mut ops,
      &ctx,
      font_bold_id,
      14.0,
      cx,
      tbl_col_w,
      val_y + tbl_h - 2.5,
      3.0,
      val,
    );
  }

  op_box_rect(&mut ops, &ctx, tbl_x, y, tbl_w, tbl_h * 2.0);
  op_hline(&mut ops, &ctx, tbl_x, tbl_x + tbl_w, y + tbl_h);
  for i in 1..5usize {
    op_vline(
      &mut ops,
      &ctx,
      tbl_x + i as f64 * tbl_col_w,
      y,
      y + tbl_h * 2.0,
    );
  }

  y += tbl_h * 2.0 + 12.0;
  op_text(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    lm + 10.0,
    y,
    "จึงเรียนมาเพื่อโปรดพิจารณาอนุมัติ",
  );

  let sig_x = lm + 90.0;
  let sig_w = rm - sig_x;

  y += 20.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "ลงชื่อ..............................ผู้ขออนุมัติ",
  );
  y += 9.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "(.......................................)",
  );
  y += 8.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "เจ้าหน้าที่พัสดุ",
  );

  y += 16.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "ลงชื่อ..............................ผู้เห็นชอบ",
  );
  y += 9.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "(.......................................)",
  );
  y += 8.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "หัวหน้ากลุ่มงานเภสัชกรรมฯ",
  );

  y += 16.0;
  op_text(&mut ops, &ctx, font_bold_id, body_size, lm, y, "อนุมัติ");

  y += 16.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "ลงชื่อ..............................ผู้อนุมัติ",
  );
  y += 9.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "(.......................................)",
  );
  y += 8.0;
  op_text_center(
    &mut ops,
    &ctx,
    font_id,
    body_size,
    sig_x,
    sig_w,
    y,
    "ผู้อำนวยการโรงพยาบาลสระโบสถ์",
  );

  ops
}

fn generate_combined_cover(
  pages: &[CoverLetterPage],
  params: &CoverLettersParams,
) -> Result<String, String> {
  let mut doc = PdfDocument::new(&format!("เบิกยาปะหน้า - รอบ {}", params.round));
  let (font_id, font_bold_id) = load_fonts(&mut doc)?;

  let mut pdf_pages = Vec::new();

  for page in pages.iter() {
    let ops = build_cover_letter_ops(page, params, &font_id, &font_bold_id);
    pdf_pages.push(make_portrait_page(ops));
  }

  doc.with_pages(pdf_pages);

  let filename = format!(
    "เบิกยาปะหน้า_{}_เดือน{}_รอบ{}.pdf",
    params.year, params.month, params.round
  );
  let path_str = output_path(&params.output_dir, filename);
  crate::shared::save_pdf(&doc, &path_str)?;
  Ok(path_str)
}
