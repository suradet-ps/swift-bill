use std::fs;

use chrono::Utc;
use tauri::Manager;

use crate::models::{NumberLockBatchParams, NumberLockEntry, NumberLockStore};

pub fn load_number_locks(app: &tauri::AppHandle) -> Result<Vec<NumberLockEntry>, String> {
  let mut store = load_number_lock_store(app)?;
  sort_entries(&mut store.entries);
  Ok(store.entries)
}

pub fn create_number_locks(
  app: &tauri::AppHandle,
  params: NumberLockBatchParams,
) -> Result<Vec<NumberLockEntry>, String> {
  validate_batch_params(&params)?;

  let path = get_number_lock_path(app)?;
  let mut store = load_number_lock_store(app)?;
  let created_at = Utc::now().to_rfc3339();
  let mut created: Vec<NumberLockEntry> = Vec::with_capacity(params.count as usize);

  for offset in 0..params.count {
    let request_no = params.start_request_no + offset * 2;
    let report_no = request_no + 1;
    let purchase_no = params.start_purchase_no + offset;

    if let Some(existing) = find_overlapping_entry(
      &store.entries,
      params.fiscal_year,
      request_no,
      report_no,
      purchase_no,
    ) {
      return Err(format!(
        "เลขชุด {request_no}/{report_no}/{purchase_no} ถูกล็อกไว้แล้ว ({})",
        existing.reason
      ));
    }

    created.push(NumberLockEntry {
      id: format!("{}-{}-{}", params.fiscal_year, request_no, purchase_no),
      fiscal_year: params.fiscal_year,
      request_no,
      report_no,
      purchase_no,
      reason: params.reason.trim().to_string(),
      note: params.note.trim().to_string(),
      created_at: created_at.clone(),
    });
  }

  store.entries.extend(created.iter().cloned());
  sort_entries(&mut store.entries);
  write_number_lock_store(path, &store)?;
  Ok(created)
}

pub fn delete_number_lock(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
  let path = get_number_lock_path(app)?;
  let mut store = load_number_lock_store(app)?;
  store.entries.retain(|entry| entry.id != id);
  sort_entries(&mut store.entries);
  write_number_lock_store(path, &store)
}

fn validate_batch_params(params: &NumberLockBatchParams) -> Result<(), String> {
  if params.fiscal_year <= 0 {
    return Err("ปีงบประมาณไม่ถูกต้อง".to_string());
  }
  if params.start_request_no == 0 {
    return Err("เลขขอซื้อเริ่มต้นต้องมากกว่า 0".to_string());
  }
  if params.start_purchase_no == 0 {
    return Err("เลขใบสั่งซื้อเริ่มต้นต้องมากกว่า 0".to_string());
  }
  if params.count == 0 {
    return Err("จำนวนชุดที่ต้องการล็อกต้องมากกว่า 0".to_string());
  }
  if params.reason.trim().is_empty() {
    return Err("กรุณาระบุเหตุผลในการล็อกเลข".to_string());
  }
  Ok(())
}

fn overlaps(
  entry: &NumberLockEntry,
  fiscal_year: i32,
  request_no: u32,
  report_no: u32,
  purchase_no: u32,
) -> bool {
  entry.fiscal_year == fiscal_year
    && (entry.request_no == request_no
      || entry.report_no == report_no
      || entry.purchase_no == purchase_no)
}

fn find_overlapping_entry(
  entries: &[NumberLockEntry],
  fiscal_year: i32,
  request_no: u32,
  report_no: u32,
  purchase_no: u32,
) -> Option<&NumberLockEntry> {
  entries
    .iter()
    .find(|entry| overlaps(entry, fiscal_year, request_no, report_no, purchase_no))
}

fn get_number_lock_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
  let data_dir = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;
  fs::create_dir_all(&data_dir).map_err(|e| format!("Cannot create data dir: {e}"))?;
  Ok(data_dir.join("number_locks.json"))
}

fn load_number_lock_store(app: &tauri::AppHandle) -> Result<NumberLockStore, String> {
  let path = get_number_lock_path(app)?;
  if !path.exists() {
    return Ok(NumberLockStore::default());
  }
  let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read number locks: {e}"))?;
  serde_json::from_str(&content).map_err(|e| format!("Cannot parse number locks: {e}"))
}

fn write_number_lock_store(
  path: std::path::PathBuf,
  store: &NumberLockStore,
) -> Result<(), String> {
  let content = serde_json::to_string_pretty(store)
    .map_err(|e| format!("Cannot serialize number locks: {e}"))?;
  fs::write(&path, content).map_err(|e| format!("Cannot write number locks: {e}"))
}

fn sort_entries(entries: &mut [NumberLockEntry]) {
  entries.sort_by(|a, b| {
    b.fiscal_year
      .cmp(&a.fiscal_year)
      .then(a.request_no.cmp(&b.request_no))
      .then(a.purchase_no.cmp(&b.purchase_no))
  });
}
