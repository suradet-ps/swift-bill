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
/* ── Reset ──────────────────────────────────────────────────────────────────── */
*,
*::before,
*::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

/* ── CSS Variables ──────────────────────────────────────────────────────────── */
:root {
    --c-primary: #1a56db;
    --c-primary-light: #eff6ff;
    --c-primary-hover: #1648c0;
    --c-secondary: #6b7280;
    --c-success: #059669;
    --c-success-bg: #ecfdf5;
    --c-error: #dc2626;
    --c-error-bg: #fef2f2;
    --c-warn: #d97706;
    --c-warn-bg: #fffbeb;
    --c-bg: #f9fafb;
    --c-surface: #ffffff;
    --c-border: #e5e7eb;
    --c-border-focus: #93c5fd;
    --c-text: #111827;
    --c-text-muted: #6b7280;
    --c-text-light: #9ca3af;
    --radius: 8px;
    --radius-lg: 12px;
    --shadow: 0 1px 3px rgba(0, 0, 0, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.08);
    font-family: "Segoe UI", "Sarabun", sans-serif;
    font-size: 14px;
    color: var(--c-text);
}

body {
    background: var(--c-bg);
    min-height: 100vh;
}

/* ── Layout ──────────────────────────────────────────────────────────────────── */
.app-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
}

/* ── Header ──────────────────────────────────────────────────────────────────── */
.app-header {
    background: linear-gradient(135deg, #1a56db 0%, #1e40af 100%);
    color: #fff;
    padding: 10px 20px;
    flex-shrink: 0;
}

.header-inner {
    display: flex;
    align-items: center;
    gap: 12px;
}

.header-icon {
    font-size: 28px;
}

.app-title {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: 0.02em;
}

.app-subtitle {
    font-size: 12px;
    opacity: 0.8;
    margin-top: 1px;
}

.header-badge {
    margin-left: auto;
    background: rgba(255, 255, 255, 0.15);
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 20px;
    padding: 4px 12px;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
}

.badge-dot {
    width: 7px;
    height: 7px;
    background: #4ade80;
    border-radius: 50%;
    display: inline-block;
}

/* ── Tab Nav ──────────────────────────────────────────────────────────────────── */
.tab-nav {
    display: flex;
    background: var(--c-surface);
    border-bottom: 2px solid var(--c-border);
    flex-shrink: 0;
}

.tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 8px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    cursor: pointer;
    color: var(--c-text-muted);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
}

.tab-btn:hover {
    color: var(--c-primary);
    background: var(--c-primary-light);
}

.tab-btn.active {
    color: var(--c-primary);
    border-bottom-color: var(--c-primary);
    font-weight: 600;
}

.tab-icon {
    font-size: 16px;
}

.tab-label {
    font-size: 13px;
}

/* ── Main Content ─────────────────────────────────────────────────────────────── */
.main-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    background: var(--c-bg);
}

/* ── Footer ───────────────────────────────────────────────────────────────────── */
.app-footer {
    background: var(--c-surface);
    border-top: 1px solid var(--c-border);
    padding: 6px 20px;
    text-align: center;
    color: var(--c-text-light);
    font-size: 11px;
    flex-shrink: 0;
}

/* ─── Shared component styles (used by all tabs) ──────────────────────────────── */

/* Card */
.card {
    background: var(--c-surface);
    border: 1px solid var(--c-border);
    border-radius: var(--radius-lg);
    padding: 20px;
    box-shadow: var(--shadow);
    margin-bottom: 16px;
}

.card-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--c-text);
    margin-bottom: 4px;
    display: flex;
    align-items: center;
    gap: 8px;
}

.card-desc {
    font-size: 13px;
    color: var(--c-text-muted);
    margin-bottom: 16px;
}

.card-divider {
    border: none;
    border-top: 1px solid var(--c-border);
    margin: 16px 0;
}

/* Section label */
.section-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--c-text-muted);
    padding: 4px 0;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--c-border);
}

/* Form */
.form-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
}

.form-grid-2 {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
}

.form-grid-4 {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 5px;
}

.form-group.full {
    grid-column: 1 / -1;
}

.form-group label {
    font-size: 12px;
    font-weight: 600;
    color: var(--c-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.form-group input,
.form-group select {
    padding: 8px 10px;
    border: 1px solid var(--c-border);
    border-radius: var(--radius);
    font-size: 14px;
    background: var(--c-bg);
    color: var(--c-text);
    transition: border-color 0.15s, box-shadow 0.15s;
    font-family: inherit;
}

.form-group input:focus,
.form-group select:focus {
    outline: none;
    border-color: var(--c-border-focus);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.12);
    background: var(--c-surface);
}

.form-group input::placeholder {
    color: var(--c-text-light);
}

.form-group input[readonly] {
    background: #f3f4f6;
    color: var(--c-text-muted);
    cursor: default;
}

.field-hint {
    font-size: 11px;
    color: var(--c-text-light);
}

/* Info box */
.info-box {
    background: var(--c-primary-light);
    border: 1px solid #bfdbfe;
    border-radius: var(--radius);
    padding: 10px 14px;
    font-size: 13px;
    color: #1e40af;
    margin-bottom: 12px;
}

/* Preview summary */
.preview-summary {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 12px;
}

.summary-stat {
    background: var(--c-bg);
    border: 1px solid var(--c-border);
    border-radius: var(--radius);
    padding: 10px 16px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 130px;
}

.summary-stat-label {
    font-size: 11px;
    color: var(--c-text-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.summary-stat-value {
    font-size: 18px;
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
    padding: 9px 18px;
    border: none;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    font-family: inherit;
    text-decoration: none;
}

.btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    pointer-events: none;
}

.btn-primary {
    background: var(--c-primary);
    color: #fff;
}

.btn-primary:hover:not(:disabled) {
    background: var(--c-primary-hover);
}

.btn-secondary {
    background: var(--c-surface);
    color: var(--c-text);
    border: 1px solid var(--c-border);
}

.btn-secondary:hover:not(:disabled) {
    background: var(--c-bg);
    border-color: #9ca3af;
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
    background: #d1fae5;
    color: var(--c-success);
    border: 1px solid #6ee7b7;
}

.btn-success:hover:not(:disabled) {
    background: #a7f3d0;
}

.btn-lg {
    padding: 11px 24px;
    font-size: 15px;
}

/* Spinner */
.spinner {
    width: 16px;
    height: 16px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
    opacity: 0.7;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

/* Status messages */
.status-msg {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: var(--radius);
    font-size: 13px;
    font-weight: 500;
}

.status-success {
    background: var(--c-success-bg);
    color: var(--c-success);
    border: 1px solid #6ee7b7;
}

.status-error {
    background: var(--c-error-bg);
    color: var(--c-error);
    border: 1px solid #fca5a5;
}

.status-info {
    background: var(--c-primary-light);
    color: var(--c-primary);
    border: 1px solid #bfdbfe;
}

.status-warn {
    background: var(--c-warn-bg);
    color: var(--c-warn);
    border: 1px solid #fcd34d;
}

/* Result card */
.result-card {
    background: var(--c-success-bg);
    border: 1px solid #6ee7b7;
    border-radius: var(--radius);
    padding: 14px 16px;
}

.result-card-title {
    font-weight: 700;
    color: var(--c-success);
    margin-bottom: 8px;
    font-size: 14px;
}

.file-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.file-list li {
    font-size: 12px;
    color: var(--c-success);
    display: flex;
    align-items: center;
    gap: 6px;
}

.file-list code {
    font-family: monospace;
    font-size: 11px;
    background: rgba(5, 150, 105, 0.1);
    padding: 2px 6px;
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
    font-size: 13px;
}

.data-table th {
    background: #f3f4f6;
    color: var(--c-text-muted);
    font-weight: 700;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 10px 12px;
    text-align: left;
    border-bottom: 1px solid var(--c-border);
    white-space: nowrap;
}

.data-table td {
    padding: 9px 12px;
    border-bottom: 1px solid var(--c-border);
    color: var(--c-text);
}

.data-table tbody tr:hover td {
    background: #f9fafb;
}

.data-table tfoot td {
    background: #f3f4f6;
    font-weight: 700;
    border-top: 2px solid var(--c-border);
    border-bottom: none;
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
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 600;
}

.cat-drug {
    background: #dbeafe;
    color: #1d4ed8;
}

.cat-supply {
    background: #fef3c7;
    color: #92400e;
}

/* Carry-forward box */
.carry-box {
    background: #f0fdf4;
    border: 1px solid #bbf7d0;
    border-radius: var(--radius);
    padding: 12px 16px;
}

.carry-box-title {
    font-size: 12px;
    font-weight: 700;
    color: var(--c-success);
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 6px;
}

.carry-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
}

.carry-item {
    display: flex;
    flex-direction: column;
    gap: 1px;
}

.carry-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--c-text-muted);
    font-weight: 600;
}

.carry-val {
    font-size: 14px;
    font-weight: 700;
    color: var(--c-text);
}

/* History table */
.history-empty {
    text-align: center;
    padding: 40px;
    color: var(--c-text-light);
    font-size: 14px;
}

/* Period badge */
.period-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--c-primary-light);
    color: var(--c-primary);
    border: 1px solid #bfdbfe;
    border-radius: 20px;
    padding: 5px 14px;
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 12px;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
    :root {
        --c-bg: #111827;
        --c-surface: #1f2937;
        --c-border: #374151;
        --c-border-focus: #3b82f6;
        --c-text: #f9fafb;
        --c-text-muted: #9ca3af;
        --c-text-light: #6b7280;
        --c-primary-light: #1e3a5f;
        --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }

    .data-table th {
        background: #374151;
    }

    .data-table tbody tr:hover td {
        background: #374151;
    }

    .data-table tfoot td {
        background: #374151;
    }

    .tab-nav {
        background: #1f2937;
        border-bottom-color: #374151;
    }

    .app-footer {
        background: #1f2937;
        border-top-color: #374151;
    }

    .form-group input,
    .form-group select {
        background: #374151;
        border-color: #4b5563;
        color: #f9fafb;
    }

    .form-group input:focus,
    .form-group select:focus {
        background: #1f2937;
    }

    .form-group input[readonly] {
        background: #374151;
        color: #9ca3af;
    }

    .info-box {
        background: #1e3a5f;
        border-color: #2563eb;
        color: #93c5fd;
    }

    .summary-stat {
        background: #374151;
        border-color: #4b5563;
    }

    .carry-box {
        background: #064e3b;
        border-color: #059669;
    }
}
</style>
