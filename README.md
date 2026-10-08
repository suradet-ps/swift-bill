# Swift Bill

[![Test Build](https://github.com/suradet-ps/swift-bill/actions/workflows/test-build.yml/badge.svg)](https://github.com/suradet-ps/swift-bill/actions/workflows/test-build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust: stable](https://img.shields.io/badge/rust-stable-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-24c8db.svg?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue v3](https://img.shields.io/badge/Vue-v3-4FC08D.svg?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![TypeScript v6](https://img.shields.io/badge/TypeScript-v6-3178C6.svg?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/suradet-ps/swift-bill/issues)

---

## ◆ PULSE

A disbursement report assembled by hand is a report that arrives late,
and a late report is a missing claim. Swift Bill automates the
pharmaceutical disbursement reports for the hospital that still lives in
Excel: connect to INVS over native TDS, fetch the round's (รอบ) invoices,
allocate register numbers (เลขทะเบียนคุม) and request / purchase numbers
with lock awareness, carry the budget balance forward, and print the
three statutory forms as A4 PDFs with CordiaNew Thai typography embedded -
the same page on every workstation, every time. The Excel era's
arithmetic is done; the document's continuity is the machine's job now.

| Report 1 ส่งหนี้เบิกยา | Report 2 สรุปรับยา | Report 3 เบิกยาปะหน้า | Numbers | Fonts | Secrets |
| --- | --- | --- | --- | --- | --- |
| PDF + Excel | PDF + Excel | one page per invoice | lock-aware | embedded | OS keychain |

*The continuity core - number locks, carry-forward, golden tests - and the
encrypted credential store are in place; the CI quality gate, report-3
preview and Excel, and the in-app print preview stand open.*

> Built with Tauri 2 + Vue 3, read from INVS by native TDS, drawn by
> `printpdf` - the report that used to take an afternoon now takes a
> click.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One runtime, two commands.

```
⟫ git clone https://github.com/suradet-ps/swift-bill.git
⟫ cd swift-bill
⟫ bun install
⟫ bun run tauri dev
```

The release artifact: `⟫ bun run tauri build` - native executables in
`src-tauri/target/release/bundle`.

<details>
<summary>Prerequisites</summary>

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Bun](https://bun.sh/) (v1.4+, pinned to 1.4.2)
- Tauri CLI dependencies for your OS
- An INVS MS SQL Server database reachable over TCP/IP

</details>

---

## ◆ ANATOMY

Four crates, three reports, a vault for the credentials.

- **Connects** - `swift-bill-db` reads the legacy MS SQL Server over
  native TDS (`tiberius`, no ODBC drivers, no middleware): the `MS_IVO`
  invoices joined to the `COMPANY` vendor master, read-only.
- **Computes** - `swift-bill-core` holds the business logic: report
  processing for all three forms, register-number math, receiving-number
  allocation aware of locked ranges, budget carry-forward validation,
  and งวด / date helpers - unit-tested and golden-tested, the arithmetic
  Excel used to guess at.
- **Draws** - `swift-bill-pdf` renders reports 1 and 2 through
  `printpdf` and overlays report 3 onto a pre-built template with
  `lopdf`; CordiaNew regular and bold are embedded from the bundled TTF
  files, so the Thai page renders identically on any workstation.
- **Exports** - `swift-bill-excel` writes the report 1 and 2 spreadsheets
  for the workflows that still need them - the Excel door stays open
  while the PDF door opens.
- **Seals** - connection settings are encrypted at rest with
  `encryptman` (AES-256-GCM) under a master key held in the OS keychain
  (Windows Credential Manager / macOS Keychain / Secret Service) - the
  INVS password is not a plaintext line in a config file anymore.

---

## ◆ RITUALS

**The core ceremony** - the monthly disbursement report:

1. Configure the connection (ตั้งค่าฐานข้อมูล) once. The settings are
   sealed in the OS keychain and loaded again on every launch.
2. Fetch the round (ดึงข้อมูล): pick a งวด preset (1-10, 11-20, 21-end)
   or a manual date range, set the round (รอบ) and the output folder,
   and pull the invoices - count, total, and the ยา / วัสดุเภสัชกรรม
   split.
3. Preview the reports. Reports 1 and 2 (ส่งหนี้เบิกยา, สรุปรับยา) show
   editable rows before export - PDF or Excel; report 3 (เบิกยาปะหน้า)
   shows the budget math and writes one PDF, one page per invoice.
4. Save the round to ประวัติรอบ. The next round is loaded from history -
   register numbers, PO numbers, and the remaining balance are
   pre-filled, never retyped.

**The ceremony of the lock** - ล็อกเลข reserves register slots
(เลขทะเบียนคุม) and request / report / purchase number ranges;
allocation skips them, so two rounds cannot silently reuse the same
numbers.

**The ceremony of the same page** - CordiaNew ships inside the binary.
Whatever workstation prints the report prints the same letters - the
Thai page cannot quietly change fonts between rooms.

---

## ◆ ECHOES

**Where this artifact is heading**

```
P1 ▸ correctness & continuity: locks, carry-forward ─────────────────── ▸ forging
P2 ▸ CI quality gate ────────────────────────────────────────────────── ▸ open
P3 ▸ report-3 preview & Excel, in-app print preview ─────────────────── ▸ open
P4 ▸ round wizard, งวด UX ───────────────────────────────────────────── ▸ partly
P5 ▸ secure settings: encrypted credentials ─────────────────────────── ▸ sealed
P6-P9 ▸ frontend tests, reconciliation reports, performance, v1.0 ───── ▸ open
```

**Raising the artifact** - the honest plan lives in `docs/ROADMAP.md`;
the design language in `docs/DESIGN.md`; the contribution rules in
`docs/CONTRIBUTING.md`; the security posture in `docs/security.md`.
Open an issue first to discuss a change.

**Status** - releases build from tags; the CI quality gate is on the
roadmap (P2). [Watch the workflows](.github/workflows).

---

```
  ─────────────────────────────────────────
   An Excel report is a promise
   that someone will not slip.
  ─────────────────────────────────────────
```

Licensed under the [MIT License](LICENSE).
