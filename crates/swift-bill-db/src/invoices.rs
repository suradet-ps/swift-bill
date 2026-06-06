//! Invoice fetch and row-normalisation helpers.

use chrono::NaiveDate;
use swift_bill_core::{DbConfig, InvoiceRow};
use tiberius::Row;

use crate::connect::{self, DbError};

/// Fetch every invoice in `MS_IVO` whose `RECEIVE_DATE` falls within
/// `date_from..=date_to` (inclusive on both ends, `YYYYMMDD` strings).
///
/// Joins `COMPANY` for vendor metadata. Rows whose `RECEIVE_DATE` cannot
/// be parsed are silently skipped (they cannot participate in the reports).
///
/// # Errors
///
/// Returns [`DbError`] if the connection or query fails.
pub async fn fetch_invoices(
  config: &DbConfig,
  date_from: &str,
  date_to: &str,
) -> Result<Vec<InvoiceRow>, DbError> {
  let mut client = connect::connect(config).await?;

  let query = r#"
        SELECT
            i.INVOICE_NO,
            i.VENDOR_CODE,
            c.COMPANY_NAME,
            c.KEY_WORD,
            i.TOTAL_COST,
            i.RECEIVE_DATE
        FROM MS_IVO i
        LEFT JOIN COMPANY c ON i.VENDOR_CODE = c.COMPANY_CODE
        WHERE i.RECEIVE_DATE >= @P1
          AND i.RECEIVE_DATE <= @P2
        ORDER BY i.PO_NO ASC, i.RECEIVE_DATE, i.VENDOR_CODE
    "#;

  let result = client
    .query(query, &[&date_from, &date_to])
    .await
    .map_err(|e| DbError::Tds(e.to_string()))?;

  let rows = result
    .into_results()
    .await
    .map_err(|e| DbError::Tds(e.to_string()))?;

  let mut invoices: Vec<InvoiceRow> = Vec::new();

  for result_set in &rows {
    for row in result_set {
      let invoice_no = get_string(row, "INVOICE_NO");
      let vendor_code = get_string(row, "VENDOR_CODE");
      let company_name = get_string(row, "COMPANY_NAME");
      let company_keyword = get_string(row, "KEY_WORD");
      let total_cost = get_f64(row, "TOTAL_COST").unwrap_or(0.0);
      let receive_date_raw = get_string(row, "RECEIVE_DATE");

      let Some(receive_date) = parse_receive_date(&receive_date_raw) else {
        // Skip rows with unparseable dates
        continue;
      };

      let category = determine_category(&company_keyword, &vendor_code);

      invoices.push(InvoiceRow {
        invoice_no,
        vendor_code,
        company_name,
        company_keyword,
        total_cost,
        receive_date,
        category,
      });
    }
  }

  Ok(invoices)
}

fn get_f64(row: &Row, col: &str) -> Option<f64> {
  if let Ok(Some(v)) = row.try_get::<f64, _>(col) {
    return Some(v);
  }
  if let Ok(Some(v)) = row.try_get::<f32, _>(col) {
    return Some(f64::from(v));
  }
  if let Ok(Some(v)) = row.try_get::<tiberius::numeric::Numeric, _>(col) {
    let s = format!("{v}");
    if let Ok(f) = s.parse::<f64>() {
      return Some(f);
    }
  }
  None
}

fn get_string(row: &Row, col: &str) -> String {
  row
    .try_get::<&str, _>(col)
    .ok()
    .flatten()
    .unwrap_or("")
    .to_string()
}

fn parse_receive_date(raw: &str) -> Option<NaiveDate> {
  let trimmed = raw.trim();
  if trimmed.len() == 8 {
    NaiveDate::parse_from_str(trimmed, "%Y%m%d").ok()
  } else if trimmed.len() == 10 {
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()
  } else {
    None
  }
}

fn determine_category(keyword: &str, vendor_code: &str) -> String {
  let kw_upper = keyword.to_uppercase();
  let vc_upper = vendor_code.to_uppercase();
  if kw_upper.starts_with("PS") || vc_upper.starts_with("PS") {
    "วัสดุเภสัชกรรม".to_string()
  } else {
    "ยา".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_receive_date_yyyymmdd() {
    assert_eq!(
      parse_receive_date("20260105"),
      NaiveDate::from_ymd_opt(2026, 1, 5)
    );
  }

  #[test]
  fn parse_receive_date_iso() {
    assert_eq!(
      parse_receive_date("2026-01-05"),
      NaiveDate::from_ymd_opt(2026, 1, 5)
    );
  }

  #[test]
  fn parse_receive_date_invalid() {
    assert!(parse_receive_date("garbage").is_none());
    assert!(parse_receive_date("").is_none());
  }

  #[test]
  fn determine_category_uses_ps_prefix() {
    assert_eq!(determine_category("PS001", "V001"), "วัสดุเภสัชกรรม");
    assert_eq!(determine_category("ABC", "PS001"), "วัสดุเภสัชกรรม");
    assert_eq!(determine_category("ps01", "v01"), "วัสดุเภสัชกรรม");
  }

  #[test]
  fn determine_category_defaults_to_drug() {
    assert_eq!(determine_category("ABC", "V001"), "ยา");
  }
}
