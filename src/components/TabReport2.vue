<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface DbConfig {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
}

interface InvoiceRow {
    invoice_no: string;
    vendor_code: string;
    company_name: string;
    company_keyword: string;
    total_cost: number;
    receive_date: string;
    category: string;
}

interface PreviewData {
    invoices: InvoiceRow[];
    total_amount: number;
    row_count: number;
}

interface CarryForward {
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    remaining_balance: number;
}

interface GenerateResult {
    files: string[];
    total_rows: number;
    total_amount: number;
    carry_forward: CarryForward;
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
    remaining_balance: number;
    budget_total: number;
    total_amount: number;
    invoice_count: number;
    created_at: string;
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
    startRegNo: string;
    startRunning: number;
    approvalDate: string;
}>();

const emit = defineEmits<{
    (e: "update:startPoNo", v: number): void;
    (e: "update:startRegNo", v: string): void;
    (e: "update:startRunning", v: number): void;
    (e: "update:approvalDate", v: string): void;
    (e: "saveHistory", entry: RoundHistoryEntry): void;
    (e: "carryResult", carry: { next_reg_no: string; next_running: number; next_po_no: number }): void;
}>();

const loading = ref(false);
const error = ref("");
const result = ref<GenerateResult | null>(null);

const THAI_MONTHS = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];

const THAI_MONTHS_SHORT = [
    "ม.ค.", "ก.พ.", "มี.ค.", "เม.ย.", "พ.ค.", "มิ.ย.",
    "ก.ค.", "ส.ค.", "ก.ย.", "ต.ค.", "พ.ย.", "ธ.ค.",
];

const periodText = computed(() => {
    if (!props.year || !props.month) return "ยังไม่ได้เลือกช่วงวันที่";
    return `${THAI_MONTHS[props.month - 1]} ${props.year} รอบ ${props.round}`;
});

const canGenerate = computed(
    () =>
        props.previewData !== null &&
        props.previewData.row_count > 0 &&
        props.dateFrom !== "" &&
        props.dateTo !== "" &&
        props.startRegNo.trim() !== ""
);

function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function fileName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
}

async function generate() {
    if (!canGenerate.value) return;
    loading.value = true;
    error.value = "";
    result.value = null;

    try {
        const res = await invoke<GenerateResult>("generate_receiving_summary", {
            params: {
                db_config: { ...props.dbConfig },
                date_from: props.dateFrom,
                date_to: props.dateTo,
                year: props.year,
                month: props.month,
                round: props.round,
                start_po_no: props.startPoNo,
                start_reg_no: props.startRegNo,
                start_running: props.startRunning,
                approval_date: props.approvalDate.trim() || null,
                output_dir: props.outputDir,
            },
        });
        result.value = res;
        emit("carryResult", {
            next_reg_no: res.carry_forward.next_reg_no,
            next_running: res.carry_forward.next_running,
            next_po_no: res.carry_forward.next_po_no,
        });
    } catch (e) {
        error.value = String(e);
    } finally {
        loading.value = false;
    }
}

function saveToHistory() {
    if (!result.value) return;
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
        next_reg_no: result.value.carry_forward.next_reg_no,
        next_running: result.value.carry_forward.next_running,
        next_po_no: result.value.carry_forward.next_po_no,
        remaining_balance: result.value.carry_forward.remaining_balance,
        budget_total: 0,
        total_amount: result.value.total_amount,
        invoice_count: result.value.total_rows,
        created_at: now,
    };
    emit("saveHistory", entry);
}
</script>

<template>
<div class="report-wrap">
    <!-- ── Info banner ── -->
    <div class="card info-banner">
        <div class="banner-title">📊 สรุปรับยา (Receiving Summary)</div>
        <div class="banner-desc">
            สร้างสรุปยอดยาประจำเดือน — A4 Landscape PDF (หลายหน้าอัตโนมัติ 10 แถว/หน้า)
        </div>
    </div>

    <!-- ── Data summary ── -->
    <div class="card">
        <div class="card-title">📊 ข้อมูลที่จะใช้สร้างรายงาน</div>

        <div v-if="!previewData" class="no-data">
            ⚠️ ยังไม่มีข้อมูล — กรุณาไปที่แท็บ 🔍 ดึงข้อมูล ก่อน
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

    <!-- ── Report params ── -->
    <div class="card">
        <div class="card-title">🔢 ตั้งค่าเลขที่เอกสาร</div>
        <div class="card-desc">ค่าเหล่านี้ต่อเนื่องจากรอบก่อน — สามารถโหลดจากประวัติรอบได้</div>

        <div class="section-label">📋 ข้อมูลงวด</div>
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

        <div class="section-label" style="margin-top:16px">🔢 เลขที่เอกสาร (ต่อเนื่องจากรอบก่อน)</div>
        <div class="form-grid">
            <div class="form-group">
                <label>เลขขอซื้อ / PO เริ่มต้น</label>
                <input type="number" min="1" :value="startPoNo"
                    @input="emit('update:startPoNo', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                <span class="field-hint">
                    request_no และ report_no เพิ่มทีละ 2, po_no เพิ่มทีละ 1
                </span>
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
                <input type="text" :value="approvalDate"
                    @input="emit('update:approvalDate', ($event.target as HTMLInputElement).value)"
                    placeholder="เช่น 6 พ.ย. 68" />
                <span class="field-hint">ปล่อยว่าง = ใช้วันที่รับของจากบิลแรก</span>
            </div>
        </div>

        <div class="info-box">
            💡 เลขขอซื้อ (request_no) และเลขรายงาน (report_no) จะเพิ่มทีละ 2 ต่อแถว
            เลขใบสั่งซื้อ (po_no) จะเพิ่มทีละ 1 ต่อแถว
        </div>

        <!-- Generate button -->
        <div class="actions">
            <button class="btn btn-primary btn-lg" :disabled="!canGenerate || loading" @click="generate">
                <span v-if="loading" class="spinner"></span>
                {{ loading ? "กำลังสร้าง PDF..." : "📊 สร้าง PDF สรุปรับยา" }}
            </button>
        </div>

        <div v-if="error" class="status-msg status-error" style="margin-top:12px">
            ❌ {{ error }}
        </div>
    </div>

    <!-- ── Result ── -->
    <div v-if="result" class="card">
        <div class="card-title">✅ สร้าง PDF สำเร็จ</div>

        <div class="result-card">
            <div class="result-card-title">📄 ไฟล์ที่สร้าง</div>
            <ul class="file-list">
                <li v-for="f in result.files" :key="f">
                    📄 <code>{{ fileName(f) }}</code>
                    <span class="file-path">{{ f }}</span>
                </li>
            </ul>
            <div class="result-stats">
                <span class="stat-chip">📦 {{ result.total_rows }} รายการ</span>
                <span class="stat-chip money">💰 {{ formatMoney(result.total_amount) }} บาท</span>
            </div>
        </div>

        <!-- Carry-forward info -->
        <div class="carry-box" style="margin-top:16px">
            <div class="carry-box-title">➡️ ค่าสำหรับรอบถัดไป (Carry-Forward)</div>
            <div class="carry-grid">
                <div class="carry-item">
                    <span class="carry-label">เลขทะเบียนคุมถัดไป</span>
                    <span class="carry-val">{{ result.carry_forward.next_reg_no }}</span>
                </div>
                <div class="carry-item">
                    <span class="carry-label">ลำดับถัดไปในสมุด</span>
                    <span class="carry-val">{{ result.carry_forward.next_running }}</span>
                </div>
                <div class="carry-item">
                    <span class="carry-label">เลขขอซื้อ/PO ถัดไป</span>
                    <span class="carry-val">{{ result.carry_forward.next_po_no }}</span>
                </div>
            </div>
        </div>

        <div class="save-actions" style="margin-top:16px">
            <button class="btn btn-success" @click="saveToHistory">
                💾 บันทึกรอบนี้สู่ประวัติ
            </button>
        </div>
    </div>
</div>
</template>

<style scoped>
.report-wrap {
    max-width: 800px;
}

.info-banner {
    background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
    border-color: #bbf7d0;
}

.banner-title {
    font-size: 15px;
    font-weight: 700;
    color: #065f46;
    margin-bottom: 4px;
}

.banner-desc {
    font-size: 13px;
    color: #059669;
}

.no-data {
    text-align: center;
    padding: 20px;
    color: var(--c-warn);
    font-size: 14px;
    background: var(--c-warn-bg);
    border-radius: var(--radius);
}

.period {
    color: var(--c-primary) !important;
}

.actions {
    margin-top: 20px;
}

.file-path {
    font-size: 10px;
    color: var(--c-text-muted);
    word-break: break-all;
    display: block;
    margin-top: 2px;
    padding-left: 20px;
}

.result-stats {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    flex-wrap: wrap;
}

.stat-chip {
    background: #d1fae5;
    color: #065f46;
    border-radius: 999px;
    padding: 3px 12px;
    font-size: 12px;
    font-weight: 600;
}

.stat-chip.money {
    background: #dbeafe;
    color: #1e40af;
}

.save-actions {
    display: flex;
    gap: 8px;
}
</style>
