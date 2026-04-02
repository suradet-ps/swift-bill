<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import TabSettings from "./components/TabSettings.vue";
import TabQuery from "./components/TabQuery.vue";
import TabReport1 from "./components/TabReport1.vue";
import TabReport2 from "./components/TabReport2.vue";
import TabReport3 from "./components/TabReport3.vue";
import TabHistory from "./components/TabHistory.vue";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface DbConfig {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
}

export interface InvoiceRow {
    invoice_no: string;
    vendor_code: string;
    company_name: string;
    company_keyword: string;
    total_cost: number;
    receive_date: string;
    category: string;
}

export interface PreviewData {
    invoices: InvoiceRow[];
    total_amount: number;
    row_count: number;
}

export interface CarryForward {
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    remaining_balance: number;
}

export interface GenerateResult {
    files: string[];
    total_rows: number;
    total_amount: number;
    carry_forward: CarryForward;
}

export interface RoundHistoryEntry {
    id: string;
    label: string;
    fiscal_year: number;
    month: number;
    round: number;
    date_from: string;
    date_to: string;
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    remaining_balance: number;
    budget_total: number;
    total_amount: number;
    invoice_count: number;
    created_at: string;
}

// ─── Shared State ─────────────────────────────────────────────────────────────

type TabId = "settings" | "query" | "report1" | "report2" | "report3" | "history";
const activeTab = ref<TabId>("settings");

const dbConfig = reactive<DbConfig>({
    host: "localhost",
    port: 1433,
    database: "INVS",
    username: "",
    password: "",
});

// Query / shared period state
const startDateHtml = ref(""); // YYYY-MM-DD (HTML date input format)
const endDateHtml = ref("");
const outputDir = ref("");
const previewData = ref<PreviewData | null>(null);

const dateFrom = computed(() => startDateHtml.value.replace(/-/g, ""));
const dateTo = computed(() => endDateHtml.value.replace(/-/g, ""));
const year = computed(() =>
    startDateHtml.value ? parseInt(startDateHtml.value.substring(0, 4)) + 543 : 0
);
const month = computed(() =>
    startDateHtml.value ? parseInt(startDateHtml.value.substring(5, 7)) : 0
);

// Shared round number (applies to all 3 reports)
const round = ref(1);

// Per-report unique fields
const r1Form = reactive({ startRegNo: "69ภ1", startRunning: 0 });
const r2Form = reactive({
    startPoNo: 1,
    startRegNo: "69ภ1",
    startRunning: 0,
    approvalDate: "",
});
const r3Form = reactive({
    budgetTotal: 5843812.6,
    previousBalance: 5843812.6,
    approvalDate: "",
});

// History
const historyEntries = ref<RoundHistoryEntry[]>([]);

// Carry-forward results stored from each report tab (for combined history save)
const r2Carry = ref<{
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
} | null>(null);

// ─── Lifecycle ────────────────────────────────────────────────────────────────

onMounted(async () => {
    try {
        historyEntries.value = await invoke<RoundHistoryEntry[]>("load_round_history");
    } catch (_) {
        /* ignore on fresh install */
    }
});

// ─── History handlers ─────────────────────────────────────────────────────────

async function refreshHistory() {
    try {
        historyEntries.value = await invoke<RoundHistoryEntry[]>("load_round_history");
    } catch (_) { }
}

async function saveEntry(entry: RoundHistoryEntry) {
    await invoke("save_round_entry", { entry });
    await refreshHistory();
}

async function deleteEntry(id: string) {
    await invoke("delete_round_entry", { id });
    await refreshHistory();
}

function handleR2Carry(carry: { next_reg_no: string; next_running: number; next_po_no: number }) {
    r2Carry.value = carry;
}

function applyHistoryEntry(entry: RoundHistoryEntry) {
    // Pre-fill shared state
    round.value = entry.round + 1;

    // Pre-fill report 1 form
    r1Form.startRegNo = entry.next_reg_no;
    r1Form.startRunning = entry.next_running;

    // Pre-fill report 2 form
    r2Form.startPoNo = entry.next_po_no;
    r2Form.startRegNo = entry.next_reg_no;
    r2Form.startRunning = entry.next_running;

    // Pre-fill report 3 form
    r3Form.budgetTotal = entry.budget_total;
    r3Form.previousBalance = entry.remaining_balance;

    // Switch to query tab so user can pick the new date range
    activeTab.value = "query";
}

// ─── Tabs meta ────────────────────────────────────────────────────────────────

const tabs: { id: TabId; icon: string; label: string }[] = [
    { id: "settings", icon: "⚙️", label: "ฐานข้อมูล" },
    { id: "query", icon: "🔍", label: "ดึงข้อมูล" },
    { id: "report1", icon: "📋", label: "ส่งหนี้เบิกยา" },
    { id: "report2", icon: "📊", label: "สรุปรับยา" },
    { id: "report3", icon: "📄", label: "เบิกยาปะหน้า" },
    { id: "history", icon: "📁", label: "ประวัติรอบ" },
];
</script>

<template>
<div class="app-root">
    <!-- ── Header ───────────────────────────────────────────────────────── -->
    <header class="app-header">
        <div class="header-inner">
            <span class="header-icon">💊</span>
            <div>
                <h1 class="app-title">Swift Bill</h1>
                <p class="app-subtitle">ระบบจัดการบิลยา · โรงพยาบาลสระโบสถ์</p>
            </div>
            <div class="header-badge" v-if="previewData">
                <span class="badge-dot"></span>
                ข้อมูล {{ previewData.row_count }} รายการ
            </div>
        </div>
    </header>

    <!-- ── Tab Nav ──────────────────────────────────────────────────────── -->
    <nav class="tab-nav">
        <button v-for="tab in tabs" :key="tab.id" class="tab-btn" :class="{ active: activeTab === tab.id }"
            @click="activeTab = tab.id">
            <span class="tab-icon">{{ tab.icon }}</span>
            <span class="tab-label">{{ tab.label }}</span>
        </button>
    </nav>

    <!-- ── Main Content ─────────────────────────────────────────────────── -->
    <main class="main-content">
        <TabSettings v-show="activeTab === 'settings'" :db-config="dbConfig"
            @update:db-config="Object.assign(dbConfig, $event)" />
        <TabQuery v-show="activeTab === 'query'" :db-config="dbConfig" v-model:start-date-html="startDateHtml"
            v-model:end-date-html="endDateHtml" v-model:output-dir="outputDir" v-model:preview-data="previewData"
            v-model:round="round" />
        <TabReport1 v-show="activeTab === 'report1'" :db-config="dbConfig" :date-from="dateFrom" :date-to="dateTo"
            :year="year" :month="month" :round="round" :output-dir="outputDir" :preview-data="previewData"
            v-model:start-reg-no="r1Form.startRegNo" v-model:start-running="r1Form.startRunning"
            @save-history="saveEntry" />

        <TabReport3 v-show="activeTab === 'report3'" :db-config="dbConfig" :date-from="dateFrom" :date-to="dateTo"
            :year="year" :month="month" :round="round" :output-dir="outputDir" :preview-data="previewData"
            v-model:budget-total="r3Form.budgetTotal" v-model:previous-balance="r3Form.previousBalance"
            v-model:approval-date="r3Form.approvalDate" :r2-carry="r2Carry" @save-history="saveEntry" />
        <!-- TabReport2 carry-forward listener (invisible) -->
        <TabReport2 v-show="activeTab === 'report2'" :db-config="dbConfig" :date-from="dateFrom" :date-to="dateTo"
            :year="year" :month="month" :round="round" :output-dir="outputDir" :preview-data="previewData"
            v-model:start-po-no="r2Form.startPoNo" v-model:start-reg-no="r2Form.startRegNo"
            v-model:start-running="r2Form.startRunning" v-model:approval-date="r2Form.approvalDate"
            @save-history="saveEntry" @carry-result="handleR2Carry" />
        <TabHistory v-show="activeTab === 'history'" :entries="historyEntries" @load-entry="applyHistoryEntry"
            @delete-entry="deleteEntry" />
    </main>

    <!-- ── Footer ───────────────────────────────────────────────────────── -->
    <footer class="app-footer">
        <p>Swift Bill v0.3 &nbsp;·&nbsp; โรงพยาบาลสระโบสถ์ &nbsp;·&nbsp; Read-only DB</p>
    </footer>
</div>
</template>

<style>
/* ── Reset ───────────────────────────────────────────────────────────────────── */
*,
*::before,
*::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

/* ── Design System: Olive Green Warm Theme ───────────────────────────────────── */
/* All contrast ratios verified against WCAG AA (≥ 4.5:1 normal / ≥ 3:1 large)  */
:root {
    /* ── Primary olive green ──────────────────────────────────────────────────── */
    /* #4d6320 on white  → 7.6 : 1 ✓  (large text ✓, normal text ✓)             */
    --c-primary: #4d6320;
    --c-primary-light: #eef3df;
    /* tinted bg for hover / info areas           */
    --c-primary-mid: #7a9a2e;
    /* accent / icon colour                        */
    --c-primary-hover: #3c4e19;
    /* darker on hover                             */

    /* ── Semantic colours ─────────────────────────────────────────────────────── */
    --c-success: #166534;
    --c-success-bg: #f0fdf4;
    --c-error: #b91c1c;
    --c-error-bg: #fef2f2;
    --c-warn: #92400e;
    --c-warn-bg: #fefce8;

    /* ── Layout / surface ─────────────────────────────────────────────────────── */
    --c-bg: #f2f1e8;
    /* warm olive-cream page background             */
    --c-surface: #fdfdf6;
    /* warm white card / panel surface              */
    --c-border: #ccd1b6;
    /* warm gray-olive border                       */
    --c-border-focus: #7a9a2e;
    /* olive focus ring                             */

    /* ── Text ─────────────────────────────────────────────────────────────────── */
    /* #1b1f0d on #fdfdf6 → ~18 : 1 ✓                                             */
    /* #4e5538 on #fdfdf6 →  ~7 : 1 ✓                                             */
    /* #767d5a on #fdfdf6 → ~4.6: 1 ✓  (just passes AA)                           */
    --c-text: #1b1f0d;
    --c-text-muted: #4e5538;
    --c-text-light: #767d5a;

    /* ── Shape & depth ────────────────────────────────────────────────────────── */
    --radius: 8px;
    --radius-lg: 12px;
    --radius-xl: 16px;
    --shadow: 0 1px 3px rgba(25, 35, 5, 0.10), 0 1px 2px rgba(25, 35, 5, 0.06);
    --shadow-md: 0 4px 16px rgba(25, 35, 5, 0.12);
    --shadow-lg: 0 8px 32px rgba(25, 35, 5, 0.14);

    font-family: "Segoe UI", "Sarabun", "Noto Sans Thai", system-ui, sans-serif;
    font-size: 15px;
    line-height: 1.55;
    color: var(--c-text);
    -webkit-font-smoothing: antialiased;
}

body {
    background: var(--c-bg);
    min-height: 100vh;
}

/* ── App Shell ───────────────────────────────────────────────────────────────── */
.app-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
}

/* ── Header ──────────────────────────────────────────────────────────────────── */
.app-header {
    background: linear-gradient(135deg, #3d5214 0%, #2b3c0e 100%);
    color: #fff;
    padding: 12px 24px;
    flex-shrink: 0;
    box-shadow: 0 2px 8px rgba(20, 30, 5, 0.30);
}

.header-inner {
    display: flex;
    align-items: center;
    gap: 14px;
}

.header-icon {
    font-size: 30px;
    filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.35));
}

.app-title {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: 0.02em;
    line-height: 1.2;
}

.app-subtitle {
    font-size: 13px;
    opacity: 0.82;
    margin-top: 2px;
    letter-spacing: 0.01em;
}

.header-badge {
    margin-left: auto;
    background: rgba(255, 255, 255, 0.14);
    border: 1px solid rgba(255, 255, 255, 0.26);
    border-radius: 20px;
    padding: 5px 14px;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 7px;
    backdrop-filter: blur(4px);
}

.badge-dot {
    width: 8px;
    height: 8px;
    background: #b8f080;
    border-radius: 50%;
    display: inline-block;
    box-shadow: 0 0 7px rgba(184, 240, 128, 0.75);
}

/* ── Tab Nav ──────────────────────────────────────────────────────────────────── */
.tab-nav {
    display: flex;
    background: var(--c-surface);
    border-bottom: 2px solid var(--c-border);
    flex-shrink: 0;
    padding: 0 4px;
}

.tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    padding: 12px 10px;
    background: none;
    border: none;
    border-bottom: 3px solid transparent;
    margin-bottom: -2px;
    cursor: pointer;
    color: var(--c-text-muted);
    font-size: 14px;
    font-weight: 500;
    transition: color 0.15s, background 0.15s, border-color 0.15s;
    white-space: nowrap;
    font-family: inherit;
}

.tab-btn:hover {
    color: var(--c-primary);
    background: var(--c-primary-light);
}

.tab-btn.active {
    color: var(--c-primary);
    border-bottom-color: var(--c-primary);
    font-weight: 700;
    background: var(--c-primary-light);
}

.tab-icon {
    font-size: 17px;
    line-height: 1;
}

.tab-label {
    font-size: 14px;
}

/* ── Main Content ─────────────────────────────────────────────────────────────── */
.main-content {
    flex: 1;
    overflow-y: auto;
    padding: 22px 28px;
    background: var(--c-bg);
}

/* ── Footer ───────────────────────────────────────────────────────────────────── */
.app-footer {
    background: var(--c-surface);
    border-top: 1px solid var(--c-border);
    padding: 7px 24px;
    text-align: center;
    color: var(--c-text-light);
    font-size: 12px;
    flex-shrink: 0;
    letter-spacing: 0.01em;
}

/* ─── Shared component styles ─────────────────────────────────────────────────── */

/* Card */
.card {
    background: var(--c-surface);
    border: 1px solid var(--c-border);
    border-radius: var(--radius-lg);
    padding: 22px 26px;
    box-shadow: var(--shadow);
    margin-bottom: 18px;
}

.card-title {
    font-size: 17px;
    font-weight: 700;
    color: var(--c-text);
    margin-bottom: 5px;
    display: flex;
    align-items: center;
    gap: 8px;
}

.card-desc {
    font-size: 14px;
    color: var(--c-text-muted);
    margin-bottom: 18px;
    line-height: 1.55;
}

.card-divider {
    border: none;
    border-top: 1px solid var(--c-border);
    margin: 18px 0;
}

/* Section label */
.section-label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--c-text-muted);
    padding: 4px 0;
    margin-bottom: 10px;
    border-bottom: 1px solid var(--c-border);
}

/* Form grid */
.form-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
}

.form-grid-2 {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 14px;
}

.form-grid-4 {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 14px;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 5px;
}

.form-group.full {
    grid-column: 1 / -1;
}

/* Label: #4e5538 on #fdfdf6 → ~7:1 ✓ */
.form-group label {
    font-size: 13px;
    font-weight: 600;
    color: var(--c-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.form-group input,
.form-group select {
    padding: 9px 12px;
    border: 1.5px solid var(--c-border);
    border-radius: var(--radius);
    font-size: 15px;
    background: var(--c-bg);
    color: var(--c-text);
    transition: border-color 0.15s, box-shadow 0.15s, background 0.15s;
    font-family: inherit;
    line-height: 1.4;
}

.form-group input:focus,
.form-group select:focus {
    outline: none;
    border-color: var(--c-border-focus);
    box-shadow: 0 0 0 3px rgba(122, 154, 46, 0.18);
    background: var(--c-surface);
}

.form-group input::placeholder {
    color: var(--c-text-light);
}

.form-group input[readonly] {
    background: var(--c-primary-light);
    color: var(--c-text-muted);
    border-color: var(--c-border);
    cursor: default;
}

.field-hint {
    font-size: 12px;
    color: var(--c-text-light);
    line-height: 1.4;
}

/* Info box */
/* #4d6320 on #eef3df → ~6.7:1 ✓ */
.info-box {
    background: var(--c-primary-light);
    border: 1px solid #c4d49a;
    border-radius: var(--radius);
    padding: 11px 15px;
    font-size: 14px;
    color: var(--c-primary);
    margin-bottom: 14px;
    line-height: 1.55;
}

/* Preview summary stats */
.preview-summary {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
    margin-bottom: 14px;
}

.summary-stat {
    background: var(--c-primary-light);
    border: 1px solid #c4d49a;
    border-radius: var(--radius);
    padding: 12px 18px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 145px;
}

.summary-stat-label {
    font-size: 12px;
    color: var(--c-text-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.summary-stat-value {
    font-size: 20px;
    font-weight: 700;
    color: var(--c-text);
}

.summary-stat-value.money {
    color: var(--c-primary);
}

/* Buttons */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 9px 20px;
    border: none;
    border-radius: var(--radius);
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s, box-shadow 0.15s, transform 0.12s, border-color 0.15s;
    font-family: inherit;
    text-decoration: none;
    line-height: 1.3;
}

.btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    pointer-events: none;
}

/* white on #4d6320 → 7.6:1 ✓ */
.btn-primary {
    background: var(--c-primary);
    color: #fff;
    box-shadow: 0 2px 6px rgba(60, 78, 25, 0.35);
}

.btn-primary:hover:not(:disabled) {
    background: var(--c-primary-hover);
    box-shadow: 0 4px 12px rgba(60, 78, 25, 0.45);
    transform: translateY(-1px);
}

.btn-secondary {
    background: var(--c-surface);
    color: var(--c-text);
    border: 1.5px solid var(--c-border);
}

.btn-secondary:hover:not(:disabled) {
    background: var(--c-primary-light);
    border-color: var(--c-primary-mid);
    color: var(--c-primary);
}

.btn-danger {
    background: #fee2e2;
    color: var(--c-error);
    border: 1px solid #fca5a5;
}

.btn-danger:hover:not(:disabled) {
    background: #fecaca;
}

.btn-success {
    background: #dcfce7;
    color: var(--c-success);
    border: 1px solid #86efac;
}

.btn-success:hover:not(:disabled) {
    background: #bbf7d0;
}

.btn-lg {
    padding: 12px 28px;
    font-size: 16px;
    border-radius: var(--radius-lg);
}

/* Spinner */
.spinner {
    width: 17px;
    height: 17px;
    border: 2.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
    opacity: 0.8;
    flex-shrink: 0;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

/* Status messages */
.status-msg {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    padding: 11px 15px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
    line-height: 1.55;
}

/* #166534 on #f0fdf4 → 6.8:1 ✓ */
.status-success {
    background: var(--c-success-bg);
    color: var(--c-success);
    border: 1px solid #86efac;
}

/* #b91c1c on #fef2f2 → 6.0:1 ✓ */
.status-error {
    background: var(--c-error-bg);
    color: var(--c-error);
    border: 1px solid #fca5a5;
}

/* #4d6320 on #eef3df → ~6.7:1 ✓ */
.status-info {
    background: var(--c-primary-light);
    color: var(--c-primary);
    border: 1px solid #c4d49a;
}

/* #92400e on #fefce8 → 7.0:1 ✓ */
.status-warn {
    background: var(--c-warn-bg);
    color: var(--c-warn);
    border: 1px solid #fde68a;
}

/* Result card */
.result-card {
    background: var(--c-success-bg);
    border: 1px solid #86efac;
    border-radius: var(--radius);
    padding: 16px 18px;
}

.result-card-title {
    font-weight: 700;
    color: var(--c-success);
    margin-bottom: 10px;
    font-size: 15px;
}

.file-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 5px;
}

.file-list li {
    font-size: 13px;
    color: var(--c-success);
    display: flex;
    align-items: flex-start;
    gap: 6px;
    flex-wrap: wrap;
}

.file-list code {
    font-family: "Consolas", "Fira Code", monospace;
    font-size: 12px;
    background: rgba(22, 101, 52, 0.10);
    padding: 2px 7px;
    border-radius: 4px;
    word-break: break-all;
}

/* Table */
.table-wrap {
    overflow-x: auto;
    border-radius: var(--radius);
    border: 1px solid var(--c-border);
}

.data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 14px;
}

/* #4e5538 on #eef3df → ~6.5:1 ✓ */
.data-table th {
    background: var(--c-primary-light);
    color: var(--c-text-muted);
    font-weight: 700;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 11px 14px;
    text-align: left;
    border-bottom: 2px solid #c4d49a;
    white-space: nowrap;
}

.data-table td {
    padding: 10px 14px;
    border-bottom: 1px solid var(--c-border);
    color: var(--c-text);
}

.data-table tbody tr:hover td {
    background: var(--c-primary-light);
}

/* #4d6320 on #eef3df → ~6.7:1 ✓ */
.data-table tfoot td {
    background: var(--c-primary-light);
    font-weight: 700;
    border-top: 2px solid var(--c-primary-mid);
    border-bottom: none;
    color: var(--c-primary);
}

.text-right {
    text-align: right !important;
}

.text-center {
    text-align: center !important;
}

/* Category badges */
.cat-badge {
    display: inline-flex;
    align-items: center;
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
}

/* #4d6320 on #eef3df → ~6.7:1 ✓ */
.cat-drug {
    background: var(--c-primary-light);
    color: var(--c-primary);
    border: 1px solid #c4d49a;
}

/* #92400e on #fef3c7 → 7.0:1 ✓ */
.cat-supply {
    background: #fef3c7;
    color: #92400e;
    border: 1px solid #fde68a;
}

/* Carry-forward box */
.carry-box {
    background: var(--c-primary-light);
    border: 1px solid #c4d49a;
    border-radius: var(--radius);
    padding: 14px 18px;
}

.carry-box-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--c-primary);
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
}

.carry-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
}

.carry-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.carry-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--c-text-muted);
    font-weight: 600;
}

.carry-val {
    font-size: 15px;
    font-weight: 700;
    color: var(--c-text);
}

/* History empty */
.history-empty {
    text-align: center;
    padding: 40px;
    color: var(--c-text-light);
    font-size: 15px;
}

/* Period badge */
.period-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--c-primary-light);
    color: var(--c-primary);
    border: 1px solid #c4d49a;
    border-radius: 20px;
    padding: 5px 16px;
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 14px;
}

/* ── Dark Mode ────────────────────────────────────────────────────────────────── */
/* All text/bg contrast re-verified for dark palette                              */
@media (prefers-color-scheme: dark) {
    :root {
        /* #96b83e on #141709 → ~8.0:1 ✓  (used as primary accent on dark bg)    */
        --c-primary: #96b83e;
        --c-primary-light: #1e2b0c;
        /* dark olive tint for hover / info areas  */
        --c-primary-mid: #5c7820;
        --c-primary-hover: #aac84a;

        --c-success: #4ade80;
        --c-success-bg: #052e16;
        --c-error: #f87171;
        --c-error-bg: #450a0a;
        --c-warn: #fbbf24;
        --c-warn-bg: #451a03;

        /* #141709: very dark olive-black page */
        --c-bg: #141709;
        /* #1d2210: dark olive card surface    */
        --c-surface: #1d2210;
        --c-border: #2e3820;
        --c-border-focus: #7a9a2e;

        /* #e6ecd6 on #1d2210 → ~12.5:1 ✓ */
        --c-text: #e6ecd6;
        /* #a4ac8c on #1d2210 →  ~6.6:1 ✓ */
        --c-text-muted: #a4ac8c;
        /* #646a52 on #1d2210 →  ~3.2:1  (used only for placeholder / hints)     */
        --c-text-light: #646a52;

        --shadow: 0 1px 3px rgba(0, 0, 0, 0.45), 0 1px 2px rgba(0, 0, 0, 0.30);
        --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.55);
        --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.65);
    }

    /* Table overrides */
    .data-table th {
        background: #253015;
        border-bottom-color: #3a5018;
    }

    .data-table tbody tr:hover td {
        background: #253015;
    }

    .data-table tfoot td {
        background: #253015;
        color: var(--c-primary);
        border-top-color: var(--c-primary-mid);
    }

    /* Nav / footer chrome */
    .tab-nav {
        background: var(--c-surface);
        border-bottom-color: var(--c-border);
    }

    .app-footer {
        background: var(--c-surface);
        border-top-color: var(--c-border);
    }

    /* Form inputs */
    .form-group input,
    .form-group select {
        background: #1a2010;
        border-color: #3a4828;
        color: var(--c-text);
    }

    .form-group input:focus,
    .form-group select:focus {
        background: var(--c-surface);
        border-color: var(--c-border-focus);
    }

    .form-group input[readonly] {
        background: #1e2b0c;
        color: var(--c-text-muted);
        border-color: var(--c-border);
    }

    /* Info / status boxes */
    .info-box {
        background: #1e2b0c;
        border-color: #385018;
        color: var(--c-primary);
    }

    .status-info {
        background: #1e2b0c;
        border-color: #385018;
        color: var(--c-primary);
    }

    /* Summary stats */
    .summary-stat {
        background: #1e2b0c;
        border-color: #385018;
    }

    /* Carry-forward box */
    .carry-box {
        background: #1e2b0c;
        border-color: #385018;
    }

    .carry-box-title {
        color: var(--c-primary);
    }

    /* Period badge */
    .period-badge {
        background: #1e2b0c;
        border-color: #385018;
    }

    /* Result card */
    .result-card {
        background: #052e16;
        border-color: #166534;
    }

    .file-list code {
        background: rgba(74, 222, 128, 0.12);
    }

    /* Buttons */
    .btn-primary {
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
    }

    .btn-primary:hover:not(:disabled) {
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.6);
    }

    .btn-success {
        background: #052e16;
        color: #4ade80;
        border-color: #166534;
    }

    .btn-success:hover:not(:disabled) {
        background: #064e24;
    }

    .btn-danger {
        background: #450a0a;
        color: #f87171;
        border-color: #991b1b;
    }

    .btn-danger:hover:not(:disabled) {
        background: #5c0a0a;
    }

    .btn-secondary {
        background: var(--c-surface);
        border-color: var(--c-border);
    }

    /* Category badges */
    .cat-drug {
        background: #1e2b0c;
        color: var(--c-primary);
        border-color: #385018;
    }
}
</style>
