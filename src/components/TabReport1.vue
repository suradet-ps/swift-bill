<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { fileName, formatMoney, THAI_MONTHS, THAI_MONTHS_SHORT } from "../lib/format";
import type {
    CarryForward,
    DbConfig,
    GenerateResult,
    InvoiceSubmissionPreview,
    InvoiceSubmissionRow,
    PreviewData,
    RoundHistoryEntry,
    TabId,
} from "../lib/types";
import {
    AlertTriangle,
    ArrowRight,
    Banknote,
    CheckCircle,
    Database,
    Eye,
    FileSpreadsheet,
    FileText,
    Hash,
    Info,
    Package,
    Pencil,
    Save,
    XCircle,
} from "lucide-vue-next";

const props = defineProps<{
    dbConfig: DbConfig;
    dateFrom: string;
    dateTo: string;
    year: number;
    month: number;
    round: number;
    outputDir: string;
    previewData: PreviewData | null;
    startRegNo: string;
    startRunning: number;
}>();

const emit = defineEmits<{
    (e: "update:year", v: number): void;
    (e: "update:startRegNo", v: string): void;
    (e: "update:startRunning", v: number): void;
    (e: "saveHistory", entry: RoundHistoryEntry): void;
    (e: "navigate", tab: TabId): void;
    (e: "generated"): void;
}>();

function onYearInput(e: Event) {
    emit("update:year", parseInt((e.target as HTMLInputElement).value, 10) || 0);
}

const toast = useToast();

const previewLoading = ref(false);
const exportLoading = ref(false);
const previewError = ref("");
const exportError = ref("");

const editableRows = ref<InvoiceSubmissionRow[]>([]);
const carryForward = ref<CarryForward | null>(null);
const exportedFile = ref<string | null>(null);
const pdfLoading = ref(false);
const exportedPdfFile = ref<string | null>(null);

watch(
    () => [props.year, props.month, props.round, props.dateFrom, props.dateTo, props.startRegNo, props.startRunning],
    () => {
        editableRows.value = [];
        carryForward.value = null;
        exportedFile.value = null;
        exportedPdfFile.value = null;
        exportError.value = "";
    }
);

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
        props.startRegNo.trim() !== ""
);

const canExport = computed(() => editableRows.value.length > 0 && !previewLoading.value);

const exportedTotal = computed(() =>
    editableRows.value.reduce((s, r) => s + r.total_amount, 0)
);

async function previewReport() {
    if (!canPreview.value) return;
    previewLoading.value = true;
    previewError.value = "";
    editableRows.value = [];
    exportedFile.value = null;
    exportedPdfFile.value = null;
    exportError.value = "";
    carryForward.value = null;

    try {
        const preview = await invoke<InvoiceSubmissionPreview>("preview_invoice_submission", {
            params: {
                db_config: { ...props.dbConfig },
                date_from: props.dateFrom,
                date_to: props.dateTo,
                year: props.year,
                month: props.month,
                round: props.round,
                start_reg_no: props.startRegNo,
                start_running: props.startRunning,
                output_dir: props.outputDir,
            },
        });
        editableRows.value = preview.rows.map((r) => ({ ...r }));
        carryForward.value = preview.carry_forward;
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
        const res = await invoke<GenerateResult>("export_invoice_submission_excel", {
            params: {
                rows: editableRows.value,
                year: props.year,
                month: props.month,
                round: props.round,
                start_reg_no: props.startRegNo,
                start_running: props.startRunning,
                output_dir: props.outputDir,
            },
        });
        exportedFile.value = res.files[0];
        carryForward.value = res.carry_forward;
        emit("generated");
        toast.success("ส่งออก Excel สำเร็จ", "บันทึกไฟล์เรียบร้อยแล้ว");
    } catch (e) {
        exportError.value = String(e);
        toast.error("ส่งออก Excel ล้มเหลว", String(e));
    } finally {
        exportLoading.value = false;
    }
}

async function exportPdf() {
    if (!canExport.value) return;
    pdfLoading.value = true;
    exportError.value = "";
    exportedPdfFile.value = null;

    try {
        const res = await invoke<GenerateResult>("export_invoice_submission_pdf", {
            params: {
                rows: editableRows.value,
                year: props.year,
                month: props.month,
                round: props.round,
                start_reg_no: props.startRegNo,
                start_running: props.startRunning,
                output_dir: props.outputDir,
            },
        });
        exportedPdfFile.value = res.files[0];
        carryForward.value = res.carry_forward;
        emit("generated");
        toast.success("บันทึก PDF สำเร็จ", "บันทึกไฟล์เรียบร้อยแล้ว");
    } catch (e) {
        exportError.value = String(e);
        toast.error("บันทึก PDF ล้มเหลว", String(e));
    } finally {
        pdfLoading.value = false;
    }
}

function saveToHistory() {
    if (!carryForward.value || (!exportedFile.value && !exportedPdfFile.value)) return;
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
        remaining_balance: carryForward.value.remaining_balance,
        budget_total: 0,
        total_amount: exportedTotal.value,
        invoice_count: editableRows.value.length,
        source_tab: "ส่งหนี้เบิกยา",
        created_at: now,
    };
    emit("saveHistory", entry);
}
</script>

<template>
<div class="report-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">ส่งหนี้เบิกยา</h2>
        </div>
        <div class="page-actions">
            <span v-if="previewData" class="badge badge-brand">{{ periodText }}</span>
            <button class="btn btn-ghost btn-sm" @click="emit('navigate', 'query')">
                <Database :size="14" /> ดึงข้อมูลใหม่
            </button>
        </div>
    </div>

    <!-- No data -->
    <div v-if="!previewData" class="card">
        <div class="empty-state">
            <div class="empty-icon"><Database :size="40" stroke-width="1.5" /></div>
            <div class="empty-title">ยังไม่มีข้อมูลสำหรับสร้างรายงาน</div>
            <p class="empty-desc">ดึงข้อมูลก่อนสร้างรายงาน</p>
            <div class="empty-actions">
                <button class="btn btn-primary" @click="emit('navigate', 'query')">
                    ไปที่ดึงข้อมูล
                </button>
            </div>
        </div>
    </div>

    <template v-else>
        <!-- Data context -->
        <div class="context-bar">
            <span>ช่วงเวลา <strong>{{ periodText }}</strong></span>
            <span class="context-sep"></span>
            <span>บิลในรอบนี้ <strong>{{ previewData.row_count }} รายการ</strong></span>
            <span class="context-sep"></span>
            <span>ยอดรวม <strong>{{ formatMoney(previewData.total_amount) }} บาท</strong></span>
            <span class="context-right">
                <span class="badge badge-success">ข้อมูลพร้อม</span>
            </span>
        </div>

        <!-- Report params -->
        <div class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><Hash :size="16" /> ตั้งค่าเลขทะเบียนคุม</div>
                </div>
            </div>

            <div class="form-grid">
                <div class="form-group">
                    <label for="r1-year">ปีงบประมาณ</label>
                    <input id="r1-year" type="number" min="2500" max="2700"
                        :value="year > 0 ? year : ''" placeholder="เช่น 2569" @input="onYearInput" />
                </div>
                <div class="form-group">
                    <label for="r1-month">เดือน</label>
                    <input id="r1-month" type="text" :value="month > 0 ? THAI_MONTHS[month - 1] : '-'" readonly />
                </div>
                <div class="form-group">
                    <label for="r1-round">รอบที่</label>
                    <input id="r1-round" type="text" :value="round" readonly />
                </div>
                <div class="form-group">
                    <label for="r1-reg">เลขทะเบียนคุมเริ่มต้น</label>
                    <input id="r1-reg" type="text" :value="startRegNo"
                        @input="emit('update:startRegNo', ($event.target as HTMLInputElement).value)"
                        placeholder="เช่น 69ภ12" />
                </div>
                <div class="form-group">
                    <label for="r1-running">ลำดับเริ่มต้นในสมุด (0-9)</label>
                    <input id="r1-running" type="number" min="0" max="9" :value="startRunning"
                        @input="emit('update:startRunning', parseInt(($event.target as HTMLInputElement).value) || 0)" />
                    <span class="field-hint">เล่มใหม่ใส่ 0</span>
                </div>
            </div>

            <div class="info-box section-spaced">
                <Info :size="15" />
                <span>สมุดทะเบียนละ 10 ลำดับ (0-9) เมื่อครบระบบจะขึ้นเล่มใหม่ให้อัตโนมัติ</span>
            </div>

            <div class="actions actions-row">
                <button class="btn btn-primary btn-lg" :disabled="!canPreview || previewLoading" @click="previewReport">
                    <span v-if="previewLoading" class="spinner"></span>
                    <Eye v-else :size="16" />
                    {{ previewLoading ? "กำลังโหลดตัวอย่าง..." : "แสดงตัวอย่าง" }}
                </button>
            </div>

            <div v-if="previewError" class="status-msg status-error status-stack">
                <XCircle :size="15" /> {{ previewError }}
            </div>
        </div>

        <!-- Editable preview -->
        <div v-if="editableRows.length > 0" class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><Pencil :size="16" /> ตัวอย่างข้อมูล (แก้ไขได้)</div>
                </div>
            </div>

            <div class="table-wrap">
                <table class="data-table edit-table">
                    <thead>
                        <tr>
                            <th class="text-center">#</th>
                            <th>วันที่รับของ</th>
                            <th>เลขที่เอกสาร</th>
                            <th class="text-center">เลขทะเบียนคุม</th>
                            <th class="text-center">ลำดับ</th>
                            <th>วัน/เดือน/ปีใบส่งของ</th>
                            <th>รหัสบริษัท</th>
                            <th>ค่าใช้จ่ายเรื่อง</th>
                            <th class="text-right">จำนวนเงินรวม</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="row in editableRows" :key="row.seq">
                            <td class="text-center num">{{ row.seq }}</td>
                            <td><input v-model="row.receive_date" class="cell-input" :aria-label="`วันที่รับของ ลำดับ ${row.seq}`" /></td>
                            <td><input v-model="row.invoice_no" class="cell-input" :aria-label="`เลขที่เอกสาร ลำดับ ${row.seq}`" /></td>
                            <td class="text-center readonly-cell">{{ row.reg_no }}</td>
                            <td class="text-center readonly-cell">{{ row.running_in_reg }}</td>
                            <td><input v-model="row.invoice_date" class="cell-input" :aria-label="`วันที่ใบส่งของ ลำดับ ${row.seq}`" /></td>
                            <td><input v-model="row.company_name" class="cell-input wide" :aria-label="`ชื่อบริษัท ลำดับ ${row.seq}`" /></td>
                            <td>
                                <select v-model="row.category" class="cell-select" :aria-label="`ประเภท ลำดับ ${row.seq}`">
                                    <option>ยา</option>
                                    <option>วัสดุเภสัชกรรม</option>
                                </select>
                            </td>
                            <td class="text-right">
                                <input v-model.number="row.total_amount" type="number" step="0.01"
                                    class="cell-input amount" :aria-label="`จำนวนเงิน ลำดับ ${row.seq}`" />
                            </td>
                        </tr>
                    </tbody>
                    <tfoot>
                        <tr>
                            <td colspan="8" class="text-right">รวมทั้งสิ้น</td>
                            <td class="text-right total-cell">{{ formatMoney(exportedTotal) }}</td>
                        </tr>
                    </tfoot>
                </table>
            </div>

            <div class="actions actions-row">
                <button class="btn btn-primary btn-lg" :disabled="!canExport || pdfLoading" @click="exportPdf">
                    <span v-if="pdfLoading" class="spinner"></span>
                    <FileText v-else :size="16" />
                    {{ pdfLoading ? "กำลังบันทึก PDF..." : "บันทึก PDF" }}
                </button>
                <button class="btn btn-success btn-lg" :disabled="!canExport || exportLoading" @click="exportExcel">
                    <span v-if="exportLoading" class="spinner"></span>
                    <FileSpreadsheet v-else :size="16" />
                    {{ exportLoading ? "กำลังส่งออก Excel..." : "ส่งออก Excel" }}
                </button>
            </div>

            <div v-if="exportError" class="status-msg status-error status-stack">
                <XCircle :size="15" /> {{ exportError }}
            </div>
        </div>

        <!-- Export result -->
        <div v-if="exportedFile || exportedPdfFile" class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><CheckCircle :size="16" /> ส่งออกสำเร็จ</div>
                </div>
            </div>

            <div class="result-card">
                <div class="result-card-title"><FileSpreadsheet :size="15" /> ไฟล์ที่สร้าง</div>
                <ul class="file-list">
                    <li v-if="exportedFile">
                        <FileSpreadsheet :size="14" /> <code>{{ fileName(exportedFile) }}</code>
                        <span class="file-path">{{ exportedFile }}</span>
                    </li>
                    <li v-if="exportedPdfFile">
                        <FileText :size="14" /> <code>{{ fileName(exportedPdfFile) }}</code>
                        <span class="file-path">{{ exportedPdfFile }}</span>
                    </li>
                </ul>
                <div class="result-stats">
                    <span class="stat-chip"><Package :size="13" /> {{ editableRows.length }} รายการ</span>
                    <span class="stat-chip money"><Banknote :size="13" /> {{ formatMoney(exportedTotal) }} บาท</span>
                </div>
            </div>

            <div v-if="carryForward" class="carry-box section-spaced">
                <div class="carry-box-title"><ArrowRight :size="15" /> ค่าสำหรับรอบถัดไป</div>
                <div class="carry-grid">
                    <div class="carry-item">
                        <span class="carry-label">เลขทะเบียนคุมถัดไป</span>
                        <span class="carry-val">{{ carryForward.next_reg_no }}</span>
                    </div>
                    <div class="carry-item">
                        <span class="carry-label">ลำดับถัดไปในสมุด</span>
                        <span class="carry-val">{{ carryForward.next_running }}</span>
                    </div>
                </div>
            </div>

            <div class="save-actions">
                <button class="btn btn-secondary" @click="saveToHistory">
                    <Save :size="15" /> บันทึกรอบนี้สู่ประวัติ
                </button>
            </div>
        </div>
    </template>

    <div v-if="previewData && previewData.row_count === 0" class="callout callout-warn">
        <AlertTriangle :size="15" />
        <div class="callout-body">
            <span class="callout-title">ช่วงวันที่นี้ยังไม่มีรายการบิล</span>
        </div>
        <button class="btn btn-secondary btn-sm" @click="emit('navigate', 'query')">ดึงข้อมูลใหม่</button>
    </div>
</div>
</template>
