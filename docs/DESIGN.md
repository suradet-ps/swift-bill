# Design System - Warm Hospital Theme

This document describes the UI that ships in `src/`. It replaces the
previous aspirational draft (Geist typography, Tailwind rings, blue links)
which no longer matched the product. The palette is unchanged: warm cream
canvas, off-white surfaces, hospital red for action and emphasis.

- Tokens: `src/assets/design-system.css` (`:root` and the dark-mode block)
- Shared types and formatters: `src/lib/types.ts`, `src/lib/format.ts`
- Shell and navigation: `src/App.vue`
- Overview screen: `src/components/TabHome.vue`

---

## 1. Principles

1. **Overview first.** The app opens on `ภาพรวม` (overview), not on a
   settings form. The overview answers three questions: what round am I in,
   is my data loaded, and what is the next step.
2. **Workflow, not tabs.** The sidebar mirrors the monthly job: fetch data,
   then the three statutory reports. Steps are numbered and show completion.
3. **Simple but deliberate.** One spacing scale, one radius scale, one type
   scale. No decorative color; color means status or action.
4. **Every state designed.** Loading, empty, error, and success states are
   first-class screens, not afterthoughts.
5. **Dense but calm.** This is a desktop back-office tool. Body text is
   14px, tables are 13px, page titles are 24px. Density over drama.
6. **Keyboard first.** Focus is always visible, controls are reachable, and
   labels are programmatically associated.

---

## 2. Color

### Brand

| Token | Light | Role |
|---|---|---|
| `--c-primary` | `#C8102E` | Primary actions, active nav, key numbers |
| `--c-primary-hover` | `#A50026` | Primary hover / pressed |
| `--c-primary-mid` | `#E03050` | Warm accent (rare) |
| `--c-primary-light` | `#FFF0EC` | Tinted surfaces, active nav background |
| `--c-primary-border` | `rgba(200,16,46,.14)` | Hairline around tinted surfaces |

### Semantic

| Token | Light | Use |
|---|---|---|
| `--c-success` / `--c-success-bg` | `#166534` / `#F0FDF4` | Completed steps, export success |
| `--c-error` / `--c-error-bg` | `#B91C1C` / `#FEF2F2` | Failures, destructive |
| `--c-warn` / `--c-warn-bg` | `#92400E` / `#FEFCE8` | Missing data, skipped locked numbers |

### Surfaces and text

| Token | Light | Role |
|---|---|---|
| `--c-bg` | `#FBF3EC` | Page canvas |
| `--c-surface` | `#FFFCF9` | Cards, sidebar |
| `--c-surface-raised` | `#FFFFFF` | Inputs, table body |
| `--c-surface-sunken` | `#F7ECE4` | Read-only fields, skeletons |
| `--c-border` | `#EDD5C8` | Hairlines |
| `--c-text` | `#1C0A05` | Primary text |
| `--c-text-muted` | `#5C2C1E` | Secondary text |
| `--c-text-light` | `#9C6A58` | Hints, metadata |

Dark mode is a token swap under `prefers-color-scheme: dark`. Components
must consume tokens, never hardcoded hex values, so both themes stay in sync
(the toasts previously hardcoded colors and were converted).

---

## 3. Typography

Font stack: `system-ui, -apple-system, "Segoe UI", "Noto Sans Thai",
"Sarabun", "Leelawadee UI", "Tahoma", sans-serif`. Thai rendering uses the
OS Thai font, so the app has no runtime font dependency. Numbers use
`font-variant-numeric: tabular-nums` wherever values align in columns.

| Token | Size | Weight | Use |
|---|---|---|---|
| `--fs-display` | 24px | 600 | Page titles, tracking -0.6px |
| `--fs-title` | 19px | 600 | (reserved) |
| `--fs-heading` | 15px | 600 | Card titles |
| `--fs-body` | 14px | 400/500 | Body, labels, buttons |
| `--fs-sm` | 13px | 400/500 | Descriptions, table cells |
| `--fs-xs` | 12px | 400/600 | Hints, metadata |
| `--fs-label` | 11px | 600 | Uppercase section labels |

---

## 4. Space, Radius, Depth

- Spacing: `--sp-1` 4px through `--sp-8` 40px. Card padding is `--sp-6`
  (24px); field gaps are `--sp-4` (16px); page rhythm is `--sp-5` (20px)
  between cards.
- Radius: `--radius-xs` 4px (code), `--radius-sm` 6px (chips), `--radius-md`
  8px (inputs, buttons, tables), `--radius-lg` 12px (cards, toasts).
- Depth uses the shadow-as-border technique: `--shadow-xs` for controls and
  tables, `--shadow-sm` for cards, `--shadow-md` for hover lift,
  `--shadow-pop` for toasts and overlays. There are no raw `border` lines on
  surfaces; hairlines inside tables use `--c-border-soft`.

---

## 5. Layout and Navigation

- Fixed 248px sidebar, fluid content capped at 1200px.
- Sidebar order: brand, live data context chip, `ภาพรวม`, `งานรายเดือน`
  (numbered steps 1 to 4), `เครื่องมือ` (`ล็อกเลข`, `ประวัติรอบ`), then the
  footer with `ตั้งค่าฐานข้อมูล`, connection status, and the real app version.
- Step badges show a check when the step is complete. Completion resets
  whenever the fetched dataset changes.
- Report screens show the next action when blocked: an empty state with a
  button back to `ดึงข้อมูล`, never a dead end.

---

## 6. Components

All shared classes live in `design-system.css`, including `.card`,
`.page-header`, `.context-bar`, `.stat-grid`, `.flow-list`, `.callout`,
`.badge`, `.empty-state`, `.table-wrap` / `.data-table`, `.cell-input`,
`.carry-box`, `.result-card`, `.confirm-strip`, and `.btn` variants.

- **Buttons:** `btn-primary` (one per screen), `btn-secondary`,
  `btn-ghost`, `btn-danger`, `btn-success`, sizes `btn-sm` / `btn-lg`,
  `btn-icon`. Destructive actions always confirm inline.
- **Forms:** `.form-group` with an explicit `<label for>` and `.field-hint`.
  Errors use `.field-error` or a `.status-msg` below the form.
- **Tables:** `.data-table` for read-only data, `.edit-table` with
  `.cell-input` / `.cell-select` for editable grids. Calculated cells use
  `.readonly-cell` and are never editable.
- **Callouts:** `callout-info`, `callout-success`, `callout-warn`,
  `callout-danger` with an icon, title, and optional action.
- **Empty states:** icon, value-led title, one or two lines of guidance, and
  exactly one primary action.

---

## 7. State Rules

| State | Treatment |
|---|---|
| Loading | Button spinner for actions; `.skeleton` where content occupies space |
| Data ready | Green `badge-success` and, on overview, a check on the step |
| No data | `.empty-state` explaining what will appear and the next action |
| Error | `.status-msg.status-error` with the raw message plus a retry path |
| Success | `.result-card` with file names, totals, and the save-round action |

Errors are never disguised as empty states, and empty states never appear
where an error actually occurred.

---

## 8. Accessibility

- Global `:focus-visible` outline using `--c-primary`; no `outline: none`
  without a visible replacement.
- Skip link to `#main-content`; `<nav aria-label="เมนูหลัก">` with
  `aria-current="page"` on the active item.
- Toasts use `role="alert"` inside an `aria-live="polite"` region.
- Editable grid inputs carry Thai `aria-label`s that include the row number.
- `prefers-reduced-motion` disables animation and transition durations.
- Body text contrast target is WCAG AA (4.5:1); semantic colors are paired
  with icons and text, never color alone.

---

## 9. Do and Don't

Do:

- Lead each screen with the decision: period, data readiness, next action.
- Keep one primary button per card.
- Use `formatMoney`, `formatBuddhistDate`, and `formatPeriodLabel` from
  `src/lib/format.ts` for consistent Thai output.
- Use shared types from `src/lib/types.ts`; they mirror the Rust structs.

Don't:

- Don't reintroduce a settings-first or tab-list-first shell.
- Don't hardcode colors outside the token blocks.
- Don't use pill radius on primary actions; pills are for badges only.
- Don't add a second competing primary button in the same card.
- Don't duplicate formatters, Thai month arrays, or DTO interfaces in
  components.
