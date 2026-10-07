<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { fileName, formatMoney, toThaiShortDate, THAI_MONTHS, THAI_MONTHS_SHORT } from "../lib/format";
import type {
    CarryForward,
    DbConfig,
    PreviewData,
    ReceivingNumberingInfo,
    ReceivingSummaryGenerateResult,
    ReceivingSummaryPreview,
    ReceivingSummaryRow,
    RoundHistoryEntry,
    TabId,
} from "../lib/types";
import {
    AlertTriangle,
    ArrowRight,
    Banknote,
    CalendarDays,
    CheckCircle,
    Database,
    Eye,
    FileSpreadsheet,
    FileText,
    Hash,
    Package,
    Pencil,
    Save,
    X,
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
    startPoNo: number;
    startPurchaseNo: number;
    startRegNo: string;
    startRunning: number;
    approvalDate: string;
}>();

const emit = defineEmits<{
    (e: "update:year", v: number): void;
    (e: "update:startPoNo", v: number): void;
    (e: "update:startPurchaseNo", v: number): void;
    (e: "update:startRegNo", v: string): void;
    (e: "update:startRunning", v: number): void;
    (e: "update:approvalDate", v: string): void;
    (e: "saveHistory", entry: RoundHistoryEntry): void;
    (e: "carryResult", carry: { next_reg_no: string; next_running: number; next_po_no: number; next_purchase_no: number }): void;
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
const editableRows = ref<ReceivingSummaryRow[]>([]);
const carryForward = ref<CarryForward | null>(null);
const numberingInfo = ref<ReceivingNumberingInfo | null>(null);
const exportedFile = ref<string | null>(null);
const pdfLoading = ref(false);
const exportedPdfFile = ref<string | null>(null);

watch(
    () => [props.year, props.month, props.round, props.dateFrom, props.dateTo, props.startPoNo, props.startPurchaseNo, props.startRegNo, props.startRunning, props.approvalDate],
    () => {
        editableRows.value = [];
        carryForward.value = null;
        numberingInfo.value = null;
        exportedFile.value = null;
        exportedPdfFile.value = null;
        exportError.value = "";
    }
);

// Holds the native <input type="date"> value (YYYY-MM-DD) and converts
// it to the Thai short format expected by the PDF.
const approvalDatePicker = ref("");

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

async function previewReport() {
    if (!canPreview.value) return;
    previewLoading.value = true;
    previewError.value = "";
    editableRows.value = [];
    exportedFile.value = null;
    exportedPdfFile.value = null;
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
        const res = await invoke<ReceivingSummaryGenerateResult>("export_receiving_summary_pdf", {
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
        exportedPdfFile.value = res.files[0];
        carryForward.value = res.carry_forward;
        numberingInfo.value = res.numbering_info;
        emit("carryResult", {
            next_reg_no: res.carry_forward.next_reg_no,
            next_running: res.carry_forward.next_running,
            next_po_no: res.carry_forward.next_po_no,
            next_purchase_no: res.carry_forward.next_purchase_no,
        });
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
        next_purchase_no: carryForward.value.next_purchase_no,
        remaining_balance: carryForward.value.remaining_balance,
        budget_total: 0,
        total_amount: exportedTotal.value,
        invoice_count: editableRows.value.length,
        source_tab: "สรุปรับยา",
        created_at: now,
    };
    emit("saveHistory", entry);
}
</script>

<template>
<div class="report-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">สรุปรับยา</h2>
            <p class="page-desc">
                สรุปยอดรับยาประจำรอบ พร้อมจัดสรรเลขขอซื้อ รายงาน และใบสั่งซื้อต่อเนื่องจากรอบก่อน
            </p>
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
            <p class="empty-desc">
                เริ่มจากขั้นตอนที่ 1 เลือกช่วงวันที่และดึงรายการบิลจาก INVS
                จากนั้นกลับมาที่หน้านี้เพื่อสร้างรายงาน
            </p>
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
                    <div class="card-title"><Hash :size="16" /> ตั้งค่าเลขที่เอกสาร</div>
                    <div class="card-desc">
                        ค่าต่อเนื่องจากรอบก่อน สามารถโหลดจากประวัติรอบได้ที่เมนูประวัติรอบ
                    </div>
                </div>
            </div>

            <div class="section-label"><CalendarDays :size="13" /> ข้อมูลงวด</div>
            <div class="form-grid">
                <div class="form-group">
                    <label for="r2-year">ปีงบประมาณ</label>
                    <input id="r2-year" type="number" min="2500" max="2700"
                        :value="year > 0 ? year : ''" placeholder="เช่น 2569" @input="onYearInput" />
                    <span class="field-hint">ค่าเริ่มต้นมาจากช่วงวันที่ เลือกแก้ไขได้</span>
                </div>
                <div class="form-group">
                    <label for="r2-month">เดือน</label>
                    <input id="r2-month" type="text" :value="month > 0 ? THAI_MONTHS[month - 1] : '-'" readonly />
                </div>
                <div class="form-group">
                    <label for="r2-round">รอบที่</label>
                    <input id="r2-round" type="text" :value="round" readonly />
                    <span class="field-hint">กำหนดที่หน้าดึงข้อมูล</span>
                </div>
            </div>

            <div class="section-label section-spaced"><Hash :size="13" /> เลขที่เอกสารต่อเนื่องจากรอบก่อน</div>
            <div class="form-grid">
                <div class="form-group">
                    <label for="r2-po">เลขขอซื้อ / รายงาน เริ่มต้น</label>
                    <input id="r2-po" type="number" min="1" :value="startPoNo"
                        @input="emit('update:startPoNo', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                    <span class="field-hint">ขอซื้อใช้ค่านี้ รายงานจะบวก 1 ให้อัตโนมัติ</span>
                </div>
                <div class="form-group">
                    <label for="r2-purchase">เลขใบสั่งซื้อ เริ่มต้น</label>
                    <input id="r2-purchase" type="number" min="1" :value="startPurchaseNo"
                        @input="emit('update:startPurchaseNo', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                    <span class="field-hint">นับอิสระจากเลขขอซื้อ</span>
                </div>
                <div class="form-group">
                    <label for="r2-reg">เลขทะเบียนคุมเริ่มต้น</label>
                    <input id="r2-reg" type="text" :value="startRegNo"
                        @input="emit('update:startRegNo', ($event.target as HTMLInputElement).value)"
                        placeholder="เช่น 69ภ12" />
                </div>
                <div class="form-group">
                    <label for="r2-running">ลำดับเริ่มต้นในสมุด (0-9)</label>
                    <input id="r2-running" type="number" min="0" max="9" :value="startRunning"
                        @input="emit('update:startRunning', parseInt(($event.target as HTMLInputElement).value) || 0)" />
                </div>
                <div class="form-group">
                    <label for="r2-approval">วันที่ขออนุมัติ (แสดงบนเอกสาร)</label>
                    <div class="input-group">
                        <input id="r2-approval" type="date" v-model="approvalDatePicker"
                            @change="onApprovalDatePick" />
                        <button v-if="approvalDate || approvalDatePicker" type="button"
                            class="btn btn-secondary btn-icon" aria-label="ล้างวันที่"
                            @click="approvalDatePicker = ''; emit('update:approvalDate', '')">
                            <X :size="15" />
                        </button>
                    </div>
                    <span v-if="approvalDate" class="field-hint approval-preview">
                        <CalendarDays :size="12" /> {{ approvalDate }}
                    </span>
                    <span v-else class="field-hint">ปล่อยว่าง = ใช้วันที่รับของจากบิลแรก</span>
                </div>
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

        <!-- Skipped locked numbers -->
        <div v-if="numberingInfo && numberingInfo.skipped_locked_sets.length > 0" class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><AlertTriangle :size="16" /> เลขล็อกที่ระบบข้ามให้อัตโนมัติ</div>
                    <div class="card-desc">ระบบตรวจเลขล็อกก่อนจัดสรรเลขจริงทุกครั้ง</div>
                </div>
            </div>
            <div class="callout callout-warn">
                <AlertTriangle :size="15" />
                <div class="callout-body">
                    <span class="callout-title">
                        เริ่มใช้เลขจริงที่ ขอซื้อ/รายงาน {{ numberingInfo.start_po_no }}
                        และใบสั่งซื้อ {{ numberingInfo.start_purchase_no }}
                    </span>
                    <span class="callout-desc">ข้ามเลขล็อกไป {{ numberingInfo.skipped_locked_sets.length }} ชุด</span>
                </div>
            </div>
            <div class="table-wrap section-spaced">
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
                            <td>{{ locked.note || "-" }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Editable preview -->
        <div v-if="editableRows.length > 0" class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><Pencil :size="16" /> ตัวอย่างข้อมูล (แก้ไขได้)</div>
                    <div class="card-desc">ตรวจสอบและแก้ไขข้อมูลก่อนส่งออก คอลัมน์สีเทาคำนวณอัตโนมัติ</div>
                </div>
            </div>

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
                            <th class="text-center">ขอซื้อ</th>
                            <th class="text-center">รายงาน</th>
                            <th class="text-center">ใบสั่งซื้อ</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="(row, idx) in editableRows" :key="idx">
                            <td><input v-model="row.approval_date" class="cell-input" :aria-label="`วันที่ขออนุมัติ ลำดับ ${idx + 1}`" /></td>
                            <td><input v-model="row.po_date" class="cell-input" :aria-label="`วันที่สั่งซื้อ ลำดับ ${idx + 1}`" /></td>
                            <td class="readonly-cell">{{ row.receive_date }}</td>
                            <td><input v-model="row.company_code" class="cell-input" :aria-label="`รหัสบริษัท ลำดับ ${idx + 1}`" /></td>
                            <td class="text-right">
                                <input v-model.number="row.total_amount" type="number" step="0.01"
                                    class="cell-input amount" :aria-label="`จำนวนเงิน ลำดับ ${idx + 1}`" />
                            </td>
                            <td class="text-center">
                                <input v-model.number="row.receiving_code" type="number"
                                    class="cell-input num-center" :aria-label="`รหัสลงรับยา ลำดับ ${idx + 1}`" />
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
                    <div class="card-desc">เปิดไฟล์จากโฟลเดอร์ที่กำหนด แล้วบันทึกรอบนี้เพื่อใช้ต่อในรอบถัดไป</div>
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
            <span class="callout-desc">ลองเลือกช่วงวันที่ใหม่ที่หน้าดึงข้อมูล</span>
        </div>
        <button class="btn btn-secondary btn-sm" @click="emit('navigate', 'query')">ดึงข้อมูลใหม่</button>
    </div>
</div>
</template>

<style scoped>
.approval-preview {
    color: var(--c-primary);
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 5px;
}

.edit-table {
    min-width: 1080px;
}
</style>
