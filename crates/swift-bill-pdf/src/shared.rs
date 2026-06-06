//! Shared PDF helpers: page-dimension constants, Thai month names, money
//! formatting, op-stream primitives, and font loading.

use printpdf::{
  Color, FontId, Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfPage,
  PdfSaveOptions, PdfWarnMsg, Point, Polygon, PolygonRing, Rgb, WindingOrder,
};
use std::fs;
use std::path::Path;

use printpdf::Pt;

// Embedded fonts (compiled into the binary)
const FONT_REGULAR: &[u8] = include_bytes!("../assets/THSarabun.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../assets/THSarabunBold.ttf");

// Page dimensions in mm
pub const A4_LAND_W: f64 = 297.0;
pub const A4_LAND_H: f64 = 210.0;
pub const A4_PORT_W: f64 = 210.0;
pub const A4_PORT_H: f64 = 297.0;

// Margin (mm)
pub const MARGIN: f64 = 15.0;

// f32 versions used directly by printpdf Mm/Pt structs
pub const A4_LAND_W_F: f32 = 297.0;
pub const A4_LAND_H_F: f32 = 210.0;
pub const A4_PORT_W_F: f32 = 210.0;
pub const A4_PORT_H_F: f32 = 297.0;

// Unit helpers
#[inline]
#[must_use]
pub fn mm(v: f64) -> Pt {
  Mm(v as f32).into()
}

#[inline]
#[must_use]
pub fn pt_f(v: f64) -> Pt {
  Pt(v as f32)
}

/// Number formatter: thousands-separated, 2 decimal places.
#[must_use]
pub fn fmt_money(v: f64) -> String {
  let s = format!("{:.2}", v);
  let (int_s, dec_s) = s.split_once('.').unwrap_or((&s, "00"));
  let negative = int_s.starts_with('-');
  let digits = if negative { &int_s[1..] } else { int_s };
  let mut rev = String::new();
  for (i, ch) in digits.chars().rev().enumerate() {
    if i > 0 && i % 3 == 0 {
      rev.push(',');
    }
    rev.push(ch);
  }
  let formatted: String = rev.chars().rev().collect();
  if negative {
    format!("-{}.{}", formatted, dec_s)
  } else {
    format!("{}.{}", formatted, dec_s)
  }
}

/// Full Thai month name for a 1-based month number (1–12).
/// Returns `"ไม่ทราบ"` for out-of-range inputs.
#[must_use]
pub fn thai_month(m: u32) -> &'static str {
  match m {
    1 => "มกราคม",
    2 => "กุมภาพันธ์",
    3 => "มีนาคม",
    4 => "เมษายน",
    5 => "พฤษภาคม",
    6 => "มิถุนายน",
    7 => "กรกฎาคม",
    8 => "สิงหาคม",
    9 => "กันยายน",
    10 => "ตุลาคม",
    11 => "พฤศจิกายน",
    12 => "ธันวาคม",
    _ => "ไม่ทราบ",
  }
}

// Public accessors for constants used by sibling modules.

/// Page-rendering context. Carries font handles and page dimensions so
/// op-stream helpers can flip top-down y to PDF bottom-up coordinates.
pub struct PageCtx {
  #[allow(dead_code)]
  pub font_id: FontId,
  #[allow(dead_code)]
  pub font_bold_id: FontId,
  #[allow(dead_code)]
  pub page_w: f64, // mm
  #[allow(dead_code)]
  pub page_h: f64, // mm
}

impl PageCtx {
  /// Convert a top-down y coordinate (distance from top) to PDF bottom-up pt.
  #[must_use]
  pub fn y(&self, top_y_mm: f64) -> Pt {
    mm(self.page_h - top_y_mm)
  }
}

/// Append ops to place a text string at (x_mm, top_y_mm).
pub fn op_text(
  ops: &mut Vec<Op>,
  ctx: &PageCtx,
  font_id: &FontId,
  size: f64,
  x_mm: f64,
  top_y_mm: f64,
  s: &str,
) {
  ops.push(Op::StartTextSection);
  ops.push(Op::SetFont {
    font: printpdf::PdfFontHandle::External(font_id.clone()),
    size: pt_f(size),
  });
  ops.push(Op::SetTextCursor {
    pos: Point {
      x: mm(x_mm),
      y: ctx.y(top_y_mm),
    },
  });
  ops.push(Op::ShowText {
    items: vec![printpdf::TextItem::Text(s.to_string())],
  });
  ops.push(Op::EndTextSection);
}

/// Rough character-width estimator (mm) for centering/right-align.
#[must_use]
pub fn char_width_mm(size: f64) -> f64 {
  // 1 pt ≈ 0.353 mm; Thai glyphs in Cordia/Sarabun are roughly 0.55× em wide.
  size * 0.353 * 0.55
}

/// Total width (mm) of a string at a given font size.
#[must_use]
pub fn text_width_mm(s: &str, size: f64) -> f64 {
  s.chars().count() as f64 * char_width_mm(size)
}

/// Place text centred within `[col_x .. col_x+col_w]`.
pub fn op_text_center(
  ops: &mut Vec<Op>,
  ctx: &PageCtx,
  font_id: &FontId,
  size: f64,
  col_x: f64,
  col_w: f64,
  top_y: f64,
  s: &str,
) {
  let tw = text_width_mm(s, size);
  let x = col_x + (col_w - tw) / 2.0;
  op_text(ops, ctx, font_id, size, x.max(col_x + 0.5), top_y, s);
}

/// Place text right-aligned within `[col_x .. col_x+col_w]`, with `padding_right` mm
/// from the right edge.
pub fn op_text_right(
  ops: &mut Vec<Op>,
  ctx: &PageCtx,
  font_id: &FontId,
  size: f64,
  col_x: f64,
  col_w: f64,
  top_y: f64,
  padding_right: f64,
  s: &str,
) {
  let tw = text_width_mm(s, size);
  let x = col_x + col_w - tw - padding_right;
  op_text(ops, ctx, font_id, size, x.max(col_x + 0.5), top_y, s);
}

/// Draw a horizontal line.
pub fn op_hline(ops: &mut Vec<Op>, ctx: &PageCtx, x1: f64, x2: f64, top_y: f64) {
  let y = ctx.y(top_y);
  ops.push(Op::DrawLine {
    line: Line {
      points: vec![
        LinePoint {
          p: Point { x: mm(x1), y },
          bezier: false,
        },
        LinePoint {
          p: Point { x: mm(x2), y },
          bezier: false,
        },
      ],
      is_closed: false,
    },
  });
}

/// Draw a vertical line.
pub fn op_vline(ops: &mut Vec<Op>, ctx: &PageCtx, x: f64, top_y1: f64, top_y2: f64) {
  ops.push(Op::DrawLine {
    line: Line {
      points: vec![
        LinePoint {
          p: Point {
            x: mm(x),
            y: ctx.y(top_y1),
          },
          bezier: false,
        },
        LinePoint {
          p: Point {
            x: mm(x),
            y: ctx.y(top_y2),
          },
          bezier: false,
        },
      ],
      is_closed: false,
    },
  });
}

/// Draw a filled rectangle. `top_y` is the distance from the page top.
#[allow(clippy::too_many_arguments)]
pub fn op_filled_rect(
  ops: &mut Vec<Op>,
  ctx: &PageCtx,
  x: f64,
  top_y: f64,
  w: f64,
  h: f64,
  r: f64,
  g: f64,
  b: f64,
) {
  ops.push(Op::SetFillColor {
    col: Color::Rgb(Rgb {
      r: r as f32,
      g: g as f32,
      b: b as f32,
      icc_profile: None,
    }),
  });
  let y_bottom = ctx.y(top_y + h); // bottom-left in PDF coords
  ops.push(Op::DrawPolygon {
    polygon: Polygon {
      rings: vec![PolygonRing {
        points: vec![
          LinePoint {
            p: Point {
              x: mm(x),
              y: y_bottom,
            },
            bezier: false,
          },
          LinePoint {
            p: Point {
              x: mm(x + w),
              y: y_bottom,
            },
            bezier: false,
          },
          LinePoint {
            p: Point {
              x: mm(x + w),
              y: ctx.y(top_y),
            },
            bezier: false,
          },
          LinePoint {
            p: Point {
              x: mm(x),
              y: ctx.y(top_y),
            },
            bezier: false,
          },
        ],
      }],
      mode: PaintMode::Fill,
      winding_order: WindingOrder::NonZero,
    },
  });
  // Reset to black fill
  ops.push(Op::SetFillColor {
    col: Color::Rgb(Rgb {
      r: 0.0_f32,
      g: 0.0_f32,
      b: 0.0_f32,
      icc_profile: None,
    }),
  });
}

/// Draw a stroked (outline only) rectangle.
pub fn op_box_rect(ops: &mut Vec<Op>, ctx: &PageCtx, x: f64, top_y: f64, w: f64, h: f64) {
  let y_bottom = ctx.y(top_y + h);
  ops.push(Op::DrawPolygon {
    polygon: Polygon {
      rings: vec![PolygonRing {
        points: vec![
          LinePoint {
            p: Point {
              x: mm(x),
              y: y_bottom,
            },
            bezier: false,
          },
          LinePoint {
            p: Point {
              x: mm(x + w),
              y: y_bottom,
            },
            bezier: false,
          },
          LinePoint {
            p: Point {
              x: mm(x + w),
              y: ctx.y(top_y),
            },
            bezier: false,
          },
          LinePoint {
            p: Point {
              x: mm(x),
              y: ctx.y(top_y),
            },
            bezier: false,
          },
        ],
      }],
      mode: PaintMode::Stroke,
      winding_order: WindingOrder::NonZero,
    },
  });
}

/// Load the embedded Thai fonts and return their [`FontId`]s.
pub fn load_fonts(doc: &mut PdfDocument) -> Result<(FontId, FontId), String> {
  let font_reg = ParsedFont::from_bytes(FONT_REGULAR, 0, &mut Vec::new())
    .ok_or_else(|| "Failed to parse regular font".to_string())?;
  let font_bld = ParsedFont::from_bytes(FONT_BOLD, 0, &mut Vec::new())
    .ok_or_else(|| "Failed to parse bold font".to_string())?;
  let id_reg = doc.add_font(&font_reg);
  let id_bld = doc.add_font(&font_bld);
  Ok((id_reg, id_bld))
}

/// Save the document to disk.
pub fn save_pdf(doc: &PdfDocument, path: &str) -> Result<(), String> {
  let mut warnings: Vec<PdfWarnMsg> = Vec::new();
  let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
  fs::write(path, &bytes).map_err(|e| format!("Cannot write PDF to {path}: {e}"))
}

/// Write a PDF page to disk using the standard A4 landscape dimensions.
pub fn make_landscape_page(ops: Vec<Op>) -> PdfPage {
  PdfPage::new(Mm(A4_LAND_W_F), Mm(A4_LAND_H_F), ops)
}

/// Write a PDF page to disk using the standard A4 portrait dimensions.
pub fn make_portrait_page(ops: Vec<Op>) -> PdfPage {
  PdfPage::new(Mm(A4_PORT_W_F), Mm(A4_PORT_H_F), ops)
}

/// Build the output path `<output_dir>/<filename>` and return its string form.
pub fn output_path(output_dir: &str, filename: String) -> String {
  Path::new(output_dir).join(&filename).to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fmt_money_zero() {
    assert_eq!(fmt_money(0.0), "0.00");
  }

  #[test]
  fn fmt_money_thousands() {
    assert_eq!(fmt_money(1_000.0), "1,000.00");
    assert_eq!(fmt_money(1_234_567.89), "1,234,567.89");
    assert_eq!(fmt_money(10_000_000.0), "10,000,000.00");
  }

  #[test]
  fn fmt_money_negative() {
    assert_eq!(fmt_money(-500.50), "-500.50");
  }

  #[test]
  fn fmt_money_two_decimals() {
    assert_eq!(fmt_money(99.99), "99.99");
    assert_eq!(fmt_money(0.01), "0.01");
  }

  #[test]
  fn thai_month_in_range() {
    assert_eq!(thai_month(1), "มกราคม");
    assert_eq!(thai_month(12), "ธันวาคม");
  }

  #[test]
  fn thai_month_out_of_range() {
    assert_eq!(thai_month(0), "ไม่ทราบ");
    assert_eq!(thai_month(13), "ไม่ทราบ");
  }
}
