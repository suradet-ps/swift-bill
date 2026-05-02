use crate::models::{
  NumberLockEntry, ReceivingNumberAssignment, ReceivingNumberingInfo, SkippedLockedNumberSet,
};

pub struct ReceivingNumberAllocation {
  pub assignments: Vec<ReceivingNumberAssignment>,
  pub numbering_info: ReceivingNumberingInfo,
  pub next_po_no: u32,
  pub next_purchase_no: u32,
}

pub fn normalize_receiving_start_numbers(
  fiscal_year: i32,
  start_po_no: u32,
  start_purchase_no: u32,
  locks: &[NumberLockEntry],
) -> ReceivingNumberingInfo {
  let mut current_po_no = start_po_no;
  let mut current_purchase_no = start_purchase_no;
  let mut skipped_locked_sets = Vec::new();

  while let Some(lock) = find_matching_lock(fiscal_year, current_po_no, current_purchase_no, locks)
  {
    skipped_locked_sets.push(to_skipped_locked_set(lock));
    current_po_no += 2;
    current_purchase_no += 1;
  }

  ReceivingNumberingInfo {
    start_po_no: current_po_no,
    start_purchase_no: current_purchase_no,
    skipped_locked_sets,
  }
}

pub fn allocate_receiving_numbers(
  fiscal_year: i32,
  start_po_no: u32,
  start_purchase_no: u32,
  count: u32,
  locks: &[NumberLockEntry],
) -> ReceivingNumberAllocation {
  let normalized =
    normalize_receiving_start_numbers(fiscal_year, start_po_no, start_purchase_no, locks);
  let mut current_po_no = normalized.start_po_no;
  let mut current_purchase_no = normalized.start_purchase_no;
  let mut skipped_locked_sets = normalized.skipped_locked_sets;
  let mut assignments = Vec::with_capacity(count as usize);

  while assignments.len() < count as usize {
    if let Some(lock) = find_matching_lock(fiscal_year, current_po_no, current_purchase_no, locks) {
      skipped_locked_sets.push(to_skipped_locked_set(lock));
      current_po_no += 2;
      current_purchase_no += 1;
      continue;
    }

    assignments.push(ReceivingNumberAssignment {
      request_no: current_po_no,
      report_no: current_po_no + 1,
      purchase_no: current_purchase_no,
    });
    current_po_no += 2;
    current_purchase_no += 1;
  }

  ReceivingNumberAllocation {
    assignments,
    numbering_info: ReceivingNumberingInfo {
      start_po_no: normalized.start_po_no,
      start_purchase_no: normalized.start_purchase_no,
      skipped_locked_sets,
    },
    next_po_no: current_po_no,
    next_purchase_no: current_purchase_no,
  }
}

fn find_matching_lock(
  fiscal_year: i32,
  request_no: u32,
  purchase_no: u32,
  locks: &[NumberLockEntry],
) -> Option<&NumberLockEntry> {
  let report_no = request_no + 1;
  locks.iter().find(|lock| {
    lock.fiscal_year == fiscal_year
      && (lock.request_no == request_no
        || lock.report_no == report_no
        || lock.purchase_no == purchase_no)
  })
}

fn to_skipped_locked_set(lock: &NumberLockEntry) -> SkippedLockedNumberSet {
  SkippedLockedNumberSet {
    request_no: lock.request_no,
    report_no: lock.report_no,
    purchase_no: lock.purchase_no,
    reason: lock.reason.clone(),
    note: lock.note.clone(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::NumberLockEntry;

  fn sample_lock(
    fiscal_year: i32,
    request_no: u32,
    purchase_no: u32,
    reason: &str,
  ) -> NumberLockEntry {
    NumberLockEntry {
      id: format!("{fiscal_year}-{request_no}-{purchase_no}"),
      fiscal_year,
      request_no,
      report_no: request_no + 1,
      purchase_no,
      reason: reason.to_string(),
      note: String::new(),
      created_at: "2026-01-01T00:00:00Z".to_string(),
    }
  }

  #[test]
  fn allocate_without_locks_matches_legacy_sequence() {
    let allocation = allocate_receiving_numbers(2569, 253, 253, 3, &[]);
    assert_eq!(allocation.assignments.len(), 3);
    assert_eq!(allocation.assignments[0].request_no, 253);
    assert_eq!(allocation.assignments[1].request_no, 255);
    assert_eq!(allocation.assignments[2].request_no, 257);
    assert_eq!(allocation.assignments[0].purchase_no, 253);
    assert_eq!(allocation.assignments[2].purchase_no, 255);
    assert_eq!(allocation.next_po_no, 259);
    assert_eq!(allocation.next_purchase_no, 256);
    assert!(allocation.numbering_info.skipped_locked_sets.is_empty());
  }

  #[test]
  fn normalize_skips_locked_start_set() {
    let locks = vec![sample_lock(2569, 253, 253, "ใช้ไปแล้ว")];
    let normalized = normalize_receiving_start_numbers(2569, 253, 253, &locks);
    assert_eq!(normalized.start_po_no, 255);
    assert_eq!(normalized.start_purchase_no, 254);
    assert_eq!(normalized.skipped_locked_sets.len(), 1);
    assert_eq!(normalized.skipped_locked_sets[0].request_no, 253);
  }

  #[test]
  fn allocate_skips_locked_set_in_middle() {
    let locks = vec![sample_lock(2569, 255, 254, "ล็อกกลางชุด")];
    let allocation = allocate_receiving_numbers(2569, 253, 253, 3, &locks);
    assert_eq!(allocation.assignments[0].request_no, 253);
    assert_eq!(allocation.assignments[1].request_no, 257);
    assert_eq!(allocation.assignments[2].request_no, 259);
    assert_eq!(allocation.assignments[0].purchase_no, 253);
    assert_eq!(allocation.assignments[1].purchase_no, 255);
    assert_eq!(allocation.assignments[2].purchase_no, 256);
    assert_eq!(allocation.next_po_no, 261);
    assert_eq!(allocation.next_purchase_no, 257);
    assert_eq!(allocation.numbering_info.skipped_locked_sets.len(), 1);
    assert_eq!(
      allocation.numbering_info.skipped_locked_sets[0].purchase_no,
      254
    );
  }

  #[test]
  fn normalize_ignores_other_fiscal_years() {
    let locks = vec![sample_lock(2568, 253, 253, "ปีก่อน")];
    let normalized = normalize_receiving_start_numbers(2569, 253, 253, &locks);
    assert_eq!(normalized.start_po_no, 253);
    assert_eq!(normalized.start_purchase_no, 253);
    assert!(normalized.skipped_locked_sets.is_empty());
  }

  #[test]
  fn carry_forward_advances_past_skipped_sets() {
    let locks = vec![
      sample_lock(2569, 255, 254, "ชุดที่ 2"),
      sample_lock(2569, 259, 256, "ชุดที่ 4"),
    ];
    let allocation = allocate_receiving_numbers(2569, 253, 253, 3, &locks);
    assert_eq!(allocation.assignments[0].request_no, 253);
    assert_eq!(allocation.assignments[1].request_no, 257);
    assert_eq!(allocation.assignments[2].request_no, 261);
    assert_eq!(allocation.next_po_no, 263);
    assert_eq!(allocation.next_purchase_no, 258);
    assert_eq!(allocation.numbering_info.skipped_locked_sets.len(), 2);
  }
}
