<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { BarChart3, AlertTriangle, Hash, CalendarDays, Eye, XCircle, Pencil, FileSpreadsheet, CheckCircle, ArrowRight, Save, Package, Banknote, X } from 'lucide-vue-next'

interface DbConfig {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
}

interface PreviewData {
    invoices: unknown[];
    total_amount: number;
    row_count: number;
}

interface CarryForward {
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    next_purchase_no: number;
    remaining_balance: number;
}

interface SkippedLockedNumberSet {
    request_no: number;
    report_no: number;
    purchase_no: number;
    reason: string;
    note: string;
}

interface ReceivingNumberingInfo {
    start_po_no: number;
    start_purchase_no: number;
    skipped_locked_sets: SkippedLockedNumberSet[];
}

interface ReceivingSummaryRow {
    approval_date: string;
    po_date: string;
    receive_date: string;
    company_code: string;
    total_amount: number;
    receiving_code: number;
    reg_no: string;
    running_in_reg: number;
    invoice_no: string;
    request_no: number;
    report_no: number;
    po_no: number;
}

interface ReceivingSummaryPreview {
    rows: ReceivingSummaryRow[];
    carry_forward: CarryForward;
    total_rows: number;
    total_amount: number;
    numbering_info: ReceivingNumberingInfo;
}

interface ReceivingSummaryGenerateResult {
    files: string[];
    total_rows: number;
    total_amount: number;
    carry_forward: CarryForward;
    numbering_info: ReceivingNumberingInfo;
}

interface RoundHistoryEntry {
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
    next_purchase_no?: number;
    remaining_balance: number;
    budget_total: number;
    total_amount: number;
    invoice_count: number;
    created_at: string;
    source_tab?: string;
}

const props = defineProps<{
    dbConfig: DbConfig;
    dateFrom: string;
    dateTo: string;
    year: number;
    month: number;
    round: number;
    outputDir: string;
    previewData: PreviewData | null;
    startPoNo: number;
    startPurchaseNo: number;
    startRegNo: string;
    startRunning: number;
    approvalDate: string;
}>();

const emit = defineEmits<{
    (e: "update:startPoNo", v: number): void;
    (e: "update:startPurchaseNo", v: number): void;
    (e: "update:startRegNo", v: string): void;
    (e: "update:startRunning", v: number): void;
    (e: "update:approvalDate", v: string): void;
    (e: "saveHistory", entry: RoundHistoryEntry): void;
    (e: "carryResult", carry: { next_reg_no: string; next_running: number; next_po_no: number; next_purchase_no: number }): void;
}>();

const toast = useToast();

const THAI_MONTHS = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];
const THAI_MONTHS_SHORT = [
    "ม.ค.", "ก.พ.", "มี.ค.", "เม.ย.", "พ.ค.", "มิ.ย.",
    "ก.ค.", "ส.ค.", "ก.ย.", "ต.ค.", "พ.ย.", "ธ.ค.",
];

const previewLoading = ref(false);
const exportLoading = ref(false);
const previewError = ref("");
const exportError = ref("");
const editableRows = ref<ReceivingSummaryRow[]>([]);
const carryForward = ref<CarryForward | null>(null);
const numberingInfo = ref<ReceivingNumberingInfo | null>(null);
const exportedFile = ref<string | null>(null);

// Thai date picker for Approval Date
// Holds the native <input type="date"> value (YYYY-MM-DD).
// When changed, converts to Thai short format and emits to parent.
const approvalDatePicker = ref("");

function toThaiShortDate(htmlDate: string): string {
    if (!htmlDate) return "";
    const parts = htmlDate.split("-");
    if (parts.length !== 3) return "";
    const year = parseInt(parts[0], 10);
    const month = parseInt(parts[1], 10);
    const day = parseInt(parts[2], 10);
    if (isNaN(year) || isNaN(month) || isNaN(day)) return "";
    const thaiYear = (year + 543) % 100;
    return `${day} ${THAI_MONTHS_SHORT[month - 1] ?? ""} ${thaiYear}`;
}

function onApprovalDatePick() {
    emit("update:approvalDate", toThaiShortDate(approvalDatePicker.value));
}

const periodText = computed(() => {
    if (!props.year || !props.month) return "ยังไม่ได้เลือกช่วงวันที่";
    return `${THAI_MONTHS[props.month - 1]} ${props.year} รอบ ${props.round}`;
});

const canPreview = computed(
    () =>
        props.previewData !== null &&
        props.previewData.row_count > 0 &&
        props.dateFrom !== "" &&
        props.dateTo !== "" &&
        props.startRegNo.trim() !== "" &&
        props.startPoNo > 0 &&
        props.startPurchaseNo > 0
);

const canExport = computed(() => editableRows.value.length > 0 && !previewLoading.value);

const exportedTotal = computed(() =>
    editableRows.value.reduce((s, r) => s + r.total_amount, 0)
);

function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function fileName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
}

async function previewReport() {
    if (!canPreview.value) return;
    previewLoading.value = true;
    previewError.value = "";
    editableRows.value = [];
    exportedFile.value = null;
    exportError.value = "";
    carryForward.value = null;
    numberingInfo.value = null;

    try {
        const preview = await invoke<ReceivingSummaryPreview>("preview_receiving_summary", {
            params: {
                db_config: { ...props.dbConfig },
                date_from: props.dateFrom,
                date_to: props.dateTo,
                year: props.year,
                month: props.month,
                round: props.round,
                start_po_no: props.startPoNo,
                start_purchase_no: props.startPurchaseNo,
                start_reg_no: props.startRegNo,
                start_running: props.startRunning,
                approval_date: props.approvalDate.trim() || null,
                output_dir: props.outputDir,
            },
        });
        editableRows.value = preview.rows.map((r) => ({ ...r }));
        carryForward.value = preview.carry_forward;
        numberingInfo.value = preview.numbering_info;
        toast.success("โหลดตัวอย่างสำเร็จ", `พบ ${preview.rows.length} รายการ`);
    } catch (e) {
        previewError.value = String(e);
        toast.error("โหลดตัวอย่างล้มเหลว", String(e));
    } finally {
        previewLoading.value = false;
    }
}

async function exportExcel() {
    if (!canExport.value) return;
    exportLoading.value = true;
    exportError.value = "";
    exportedFile.value = null;

    try {
        const res = await invoke<ReceivingSummaryGenerateResult>("export_receiving_summary_excel", {
            params: {
                rows: editableRows.value,
                year: props.year,
                month: props.month,
                round: props.round,
                start_po_no: props.startPoNo,
                start_purchase_no: props.startPurchaseNo,
                start_reg_no: props.startRegNo,
                start_running: props.startRunning,
                output_dir: props.outputDir,
            },
        });
        exportedFile.value = res.files[0];
        carryForward.value = res.carry_forward;
        numberingInfo.value = res.numbering_info;
        emit("carryResult", {
            next_reg_no: res.carry_forward.next_reg_no,
            next_running: res.carry_forward.next_running,
            next_po_no: res.carry_forward.next_po_no,
            next_purchase_no: res.carry_forward.next_purchase_no,
        });
        toast.success("ส่งออก Excel สำเร็จ", `บันทึกไฟล์เรียบร้อยแล้ว`);
    } catch (e) {
        exportError.value = String(e);
        toast.error("ส่งออก Excel ล้มเหลว", String(e));
    } finally {
        exportLoading.value = false;
    }
}

function saveToHistory() {
    if (!carryForward.value || !exportedFile.value) return;
    const now = new Date().toISOString();
    const monthShort = THAI_MONTHS_SHORT[props.month - 1] ?? "";
    const entry: RoundHistoryEntry = {
        id: now,
        label: `${monthShort} ${props.year} รอบ ${props.round}`,
        fiscal_year: props.year,
        month: props.month,
        round: props.round,
        date_from: props.dateFrom,
        date_to: props.dateTo,
        next_reg_no: carryForward.value.next_reg_no,
        next_running: carryForward.value.next_running,
        next_po_no: carryForward.value.next_po_no,
        next_purchase_no: carryForward.value.next_purchase_no,
        remaining_balance: carryForward.value.remaining_balance,
        budget_total: 0,
        total_amount: exportedTotal.value,
        invoice_count: editableRows.value.length,
        source_tab: "📊 สรุปรับยา",
        created_at: now,
    };
    emit("saveHistory", entry);
}
</script>

<template>
<div class="report-wrap">
    <div class="page-header">
        <h2 class="page-title">สรุปรับยา</h2>
        <p class="page-desc">Receiving Summary — สร้างสรุปยอดรับยาประจำเดือน</p>
    </div>

    <!-- Data summary -->
    <div class="card">
        <div class="card-title">
            <BarChart3 :size="17" /> ข้อมูลที่จะใช้สร้างรายงาน
        </div>
        <div v-if="!previewData" class="no-data">
            <AlertTriangle :size="14" /> ยังไม่มีข้อมูล — กรุณาไปที่แท็บ ดึงข้อมูล ก่อน
        </div>
        <div v-else>
            <div class="preview-summary">
                <div class="summary-stat">
                    <span class="summary-stat-label">ช่วงเวลา</span>
                    <span class="summary-stat-value period">{{ periodText }}</span>
                </div>
                <div class="summary-stat">
                    <span class="summary-stat-label">จำนวนบิล</span>
                    <span class="summary-stat-value">{{ previewData.row_count }} รายการ</span>
                </div>
                <div class="summary-stat">
                    <span class="summary-stat-label">ยอดรวม</span>
                    <span class="summary-stat-value money">{{ formatMoney(previewData.total_amount) }} บาท</span>
                </div>
            </div>
        </div>
    </div>

    <!-- Report params -->
    <div class="card">
        <div class="card-title">
            <Hash :size="17" /> ตั้งค่าเลขที่เอกสาร
        </div>
        <div class="card-desc">ค่าเหล่านี้ต่อเนื่องจากรอบก่อน — สามารถโหลดจากประวัติรอบได้</div>

        <div class="section-label">
            <CalendarDays :size="14" /> ข้อมูลงวด
        </div>
        <div class="form-grid">
            <div class="form-group">
                <label>ปีงบประมาณ</label>
                <input type="text" :value="year > 0 ? String(year) : '—'" readonly />
            </div>
            <div class="form-group">
                <label>เดือน</label>
                <input type="text" :value="month > 0 ? THAI_MONTHS[month - 1] : '—'" readonly />
            </div>
            <div class="form-group">
                <label>รอบที่</label>
                <input type="text" :value="round" readonly />
            </div>
        </div>

        <div class="section-label section-spaced">
            <Hash :size="14" /> เลขที่เอกสาร (ต่อเนื่องจากรอบก่อน)
        </div>
        <div class="form-grid">
            <div class="form-group">
                <label>เลขขอซื้อ / รายงาน เริ่มต้น</label>
                <input type="number" min="1" :value="startPoNo"
                    @input="emit('update:startPoNo', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                <span class="field-hint">ขอซื้อ (ลบ0033.302/) และ รายงาน/อนุมัติ</span>
            </div>
            <div class="form-group">
                <label>เลขใบสั่งซื้อ เริ่มต้น</label>
                <input type="number" min="1" :value="startPurchaseNo"
                    @input="emit('update:startPurchaseNo', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                <span class="field-hint">ใบสั่งซื้อ…/{year} — นับอิสระจากเลขขอซื้อ</span>
            </div>
            <div class="form-group">
                <label>เลขทะเบียนคุมเริ่มต้น</label>
                <input type="text" :value="startRegNo"
                    @input="emit('update:startRegNo', ($event.target as HTMLInputElement).value)"
                    placeholder="เช่น 69ภ12" />
            </div>
            <div class="form-group">
                <label>ลำดับเริ่มต้นในสมุด (0–9)</label>
                <input type="number" min="0" max="9" :value="startRunning"
                    @input="emit('update:startRunning', parseInt(($event.target as HTMLInputElement).value) || 0)" />
            </div>
            <div class="form-group">
                <label>วันที่ขออนุมัติ (แสดงบนเอกสาร)</label>
                <div class="date-picker-row">
                    <input type="date" v-model="approvalDatePicker" @change="onApprovalDatePick"
                        class="date-input-cal" />
                    <button v-if="approvalDate || approvalDatePicker" type="button" class="date-clear-btn"
                        @click="approvalDatePicker = ''; emit('update:approvalDate', '')" title="ล้างวันที่">
                        <X :size="14" />
                    </button>
                </div>
                <span v-if="approvalDate" class="field-hint date-thai-preview">
                    <CalendarDays :size="13" /> {{ approvalDate }}
                </span>
                <span v-else class="field-hint">ปล่อยว่าง = ใช้วันที่รับของจากบิลแรก</span>
            </div>
        </div>

        <!-- Preview button -->
        <div class="actions">
            <button class="btn btn-primary btn-lg" :disabled="!canPreview || previewLoading" @click="previewReport">
                <span v-if="previewLoading" class="spinner"></span>
                <Eye v-if="!previewLoading" :size="16" />
                {{ previewLoading ? "กำลังโหลดตัวอย่าง..." : "แสดงตัวอย่าง" }}
            </button>
        </div>

        <div v-if="previewError" class="status-msg status-error status-stack">
            <XCircle :size="14" /> {{ previewError }}
        </div>
    </div>

    <div v-if="numberingInfo && numberingInfo.skipped_locked_sets.length > 0" class="card">
        <div class="card-title">
            <AlertTriangle :size="17" /> พบเลขล็อกที่ระบบข้ามให้อัตโนมัติ
        </div>
        <div class="status-msg status-warn status-stack">
            <AlertTriangle :size="14" />
            เริ่มใช้เลขจริงที่ ขอซื้อ/รายงาน {{ numberingInfo.start_po_no }} และ ใบสั่งซื้อ {{ numberingInfo.start_purchase_no }}
            โดยข้ามเลขล็อก {{ numberingInfo.skipped_locked_sets.length }} ชุด
        </div>

        <div class="table-wrap locked-table-wrap">
            <table class="data-table">
                <thead>
                    <tr>
                        <th class="text-center">เลขขอซื้อ</th>
                        <th class="text-center">รายงาน</th>
                        <th class="text-center">ใบสั่งซื้อ</th>
                        <th>เหตุผล</th>
                        <th>หมายเหตุ</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="locked in numberingInfo.skipped_locked_sets"
                        :key="`${locked.request_no}-${locked.purchase_no}`">
                        <td class="text-center">{{ locked.request_no }}</td>
                        <td class="text-center">{{ locked.report_no }}</td>
                        <td class="text-center">{{ locked.purchase_no }}</td>
                        <td>{{ locked.reason }}</td>
                        <td>{{ locked.note || "—" }}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>

    <!-- Editable preview table -->
    <div v-if="editableRows.length > 0" class="card">
        <div class="card-title">
            <Pencil :size="17" /> ตัวอย่างข้อมูล (แก้ไขได้)
        </div>
        <div class="card-desc">ตรวจสอบและแก้ไขข้อมูลก่อนส่งออก Excel — คอลัมน์สีเทาคำนวณอัตโนมัติ</div>

        <div class="table-wrap">
            <table class="data-table edit-table">
                <thead>
                    <tr>
                        <th>วันที่ขออนุมัติ</th>
                        <th>วันที่สั่งซื้อ</th>
                        <th>วันที่รับของ</th>
                        <th>รหัสบริษัท</th>
                        <th class="text-right">จำนวนเงินรวม</th>
                        <th class="text-center">รหัสลงรับยา</th>
                        <th class="text-center">เลขทะเบียนคุม</th>
                        <th class="text-center">ลำดับ</th>
                        <th>เลขที่ลงรับ</th>
                        <th class="text-center readonly-col">ขอซื้อ</th>
                        <th class="text-center readonly-col">รายงาน</th>
                        <th class="text-center readonly-col">ใบสั่งซื้อ</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(row, idx) in editableRows" :key="idx">
                        <td><input v-model="row.approval_date" class="cell-input" /></td>
                        <td><input v-model="row.po_date" class="cell-input" /></td>
                        <td class="readonly-cell">{{ row.receive_date }}</td>
                        <td><input v-model="row.company_code" class="cell-input" /></td>
                        <td class="text-right">
                            <input v-model.number="row.total_amount" type="number" step="0.01"
                                class="cell-input amount" />
                        </td>
                        <td class="text-center">
                            <input v-model.number="row.receiving_code" type="number" class="cell-input num-center" />
                        </td>
                        <td class="text-center readonly-cell">{{ row.reg_no }}</td>
                        <td class="text-center readonly-cell">{{ row.running_in_reg }}</td>
                        <td class="readonly-cell">{{ row.invoice_no }}</td>
                        <td class="text-center readonly-cell">{{ row.request_no }}</td>
                        <td class="text-center readonly-cell">{{ row.report_no }}</td>
                        <td class="text-center readonly-cell">{{ row.po_no }}</td>
                    </tr>
                </tbody>
                <tfoot>
                    <tr>
                        <td colspan="4" class="text-right">รวมทั้งสิ้น</td>
                        <td class="text-right total-cell">{{ formatMoney(exportedTotal) }}</td>
                        <td colspan="7"></td>
                    </tr>
                </tfoot>
            </table>
        </div>

        <!-- Export button -->
        <div class="actions">
            <button class="btn btn-success btn-lg" :disabled="!canExport || exportLoading" @click="exportExcel">
                <span v-if="exportLoading" class="spinner"></span>
                <FileSpreadsheet v-if="!exportLoading" :size="16" />
                {{ exportLoading ? "กำลังส่งออก Excel..." : "ส่งออก Excel" }}
            </button>
        </div>

        <div v-if="exportError" class="status-msg status-error status-stack">
            <XCircle :size="14" /> {{ exportError }}
        </div>
    </div>

    <!-- Export result -->
    <div v-if="exportedFile" class="card">
        <div class="card-title">
            <CheckCircle :size="17" /> ส่งออก Excel สำเร็จ
        </div>

        <div class="result-card">
            <div class="result-card-title">
                <FileSpreadsheet :size="15" /> ไฟล์ที่สร้าง
            </div>
            <ul class="file-list">
                <li>
                    <FileSpreadsheet :size="14" /> <code>{{ fileName(exportedFile) }}</code>
                    <span class="file-path">{{ exportedFile }}</span>
                </li>
            </ul>
            <div class="result-stats">
                <span class="stat-chip">
                    <Package :size="13" /> {{ editableRows.length }} รายการ
                </span>
                <span class="stat-chip money">
                    <Banknote :size="13" /> {{ formatMoney(exportedTotal) }} บาท
                </span>
            </div>
        </div>

        <!-- Carry-forward info -->
        <div v-if="carryForward" class="carry-box section-spaced">
            <div class="carry-box-title">
                <ArrowRight :size="15" /> ค่าสำหรับรอบถัดไป (Carry-Forward)
            </div>
            <div class="carry-grid">
                <div class="carry-item">
                    <span class="carry-label">เลขทะเบียนคุมถัดไป</span>
                    <span class="carry-val">{{ carryForward.next_reg_no }}</span>
                </div>
                <div class="carry-item">
                    <span class="carry-label">ลำดับถัดไปในสมุด</span>
                    <span class="carry-val">{{ carryForward.next_running }}</span>
                </div>
                <div class="carry-item">
                    <span class="carry-label">เลขขอซื้อ/รายงาน ถัดไป</span>
                    <span class="carry-val">{{ carryForward.next_po_no }}</span>
                </div>
                <div class="carry-item">
                    <span class="carry-label">เลขใบสั่งซื้อ ถัดไป</span>
                    <span class="carry-val">{{ carryForward.next_purchase_no }}</span>
                </div>
            </div>
        </div>

        <div class="save-actions">
            <button class="btn btn-success" @click="saveToHistory">
                <Save :size="15" /> บันทึกรอบนี้สู่ประวัติ
            </button>
        </div>
    </div>
</div>
</template>

<style scoped>
.date-picker-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.date-input-cal {
    flex: 1;
    padding: 11px 13px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    background: var(--c-surface-raised);
    box-shadow: var(--shadow-ring);
    color: var(--c-text);
    cursor: pointer;
    transition: box-shadow 0.15s, background 0.15s;
}

.date-input-cal:focus {
    outline: none;
    box-shadow:
        0 0 0 2px var(--c-primary),
        rgba(200, 16, 46, 0.12) 0px 0px 0px 4px;
}

.date-clear-btn {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--c-text-muted);
    cursor: pointer;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
}

.date-clear-btn:hover {
    background: var(--c-error-bg);
    color: var(--c-error);
}

.date-thai-preview {
    color: var(--c-primary);
    font-weight: 600;
}

.no-data {
    text-align: center;
    padding: 24px;
    color: var(--c-warn);
    font-size: 15px;
    background: var(--c-warn-bg);
    box-shadow: var(--shadow-ring);
    border-radius: var(--radius-lg);
}

.period {
    color: var(--c-text) !important;
}

.locked-table-wrap {
    margin-top: 14px;
}

/* Editable table */

.edit-table {
    min-width: 1100px;
}

.edit-table td {
    padding: 6px 8px;
    vertical-align: middle;
}

.cell-input {
    border: none;
    border-radius: 8px;
    padding: 7px 8px;
    font-size: 12px;
    width: 100%;
    min-width: 64px;
    background: var(--c-surface-raised);
    box-shadow: var(--shadow-ring);
    color: var(--c-text);
    font-family: inherit;
}

.cell-input:focus {
    outline: none;
    box-shadow:
        0 0 0 2px var(--c-primary),
        rgba(200, 16, 46, 0.12) 0px 0px 0px 4px;
    background: var(--c-surface);
}

.cell-input.amount {
    text-align: right;
    min-width: 88px;
}

.cell-input.num-center {
    text-align: center;
    min-width: 52px;
}

.readonly-cell {
    color: var(--c-text-muted);
    font-size: 12px;
    background: rgba(255, 248, 243, 0.72);
    white-space: nowrap;
}

.readonly-col {
    color: var(--c-text-muted);
    font-size: 11px;
}

.total-cell {
    font-weight: 700;
    color: var(--c-primary);
}
</style>
