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
    startRegNo: string;
    startRunning: number;
}>();

const emit = defineEmits<{
    (e: "update:startRegNo", v: string): void;
    (e: "update:startRunning", v: number): void;
    (e: "saveHistory", entry: RoundHistoryEntry): void;
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
        const res = await invoke<GenerateResult>("generate_invoice_submission", {
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
        result.value = res;
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
        <div class="banner-title">📋 ส่งหนี้เบิกยา (Invoice Submission List)</div>
        <div class="banner-desc">
            สร้างรายการส่งหนี้สินและเอกสารเบิกเงิน — A4 Landscape PDF (หลายหน้าอัตโนมัติ 7 แถว/หน้า)
        </div>
    </div>

    <!-- ── Data summary from query ── -->
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
        <div class="card-title">🔢 ตั้งค่าเลขทะเบียนคุม</div>
        <div class="card-desc">ค่าเหล่านี้ต่อเนื่องจากรอบก่อน — สามารถโหลดจากประวัติรอบได้</div>

        <div class="form-grid">
            <div class="form-group">
                <label>ปีงบประมาณ (แสดงบน PDF)</label>
                <input type="text" :value="year > 0 ? String(year) : '—'" readonly />
            </div>
            <div class="form-group">
                <label>เดือน</label>
                <input type="text" :value="month > 0 ? THAI_MONTHS[month - 1] : '—'" readonly />
            </div>
            <div class="form-group">
                <label>รอบที่</label>
                <input type="text" :value="round" readonly />
                <span class="field-hint">กำหนดที่แท็บ 🔍 ดึงข้อมูล</span>
            </div>
            <div class="form-group">
                <label>เลขทะเบียนคุมเริ่มต้น</label>
                <input type="text" :value="startRegNo"
                    @input="emit('update:startRegNo', ($event.target as HTMLInputElement).value)"
                    placeholder="เช่น 69ภ12" />
                <span class="field-hint">เลขที่ทะเบียนเล่มแรกของรอบนี้</span>
            </div>
            <div class="form-group">
                <label>ลำดับเริ่มต้นในสมุด (0–9)</label>
                <input type="number" min="0" max="9" :value="startRunning"
                    @input="emit('update:startRunning', parseInt(($event.target as HTMLInputElement).value) || 0)" />
                <span class="field-hint">ลำดับแรกในเล่มทะเบียน (เล่มใหม่ใส่ 0)</span>
            </div>
        </div>

        <div class="info-box">
            💡 แต่ละสมุดทะเบียนมี 10 ลำดับ (0–9) เมื่อครบจะขึ้นเล่มใหม่โดยอัตโนมัติ
            เช่น 69ภ12 ลำดับ 8 → 69ภ12(8), 69ภ12(9), 69ภ13(0), ...
        </div>

        <!-- Generate button -->
        <div class="actions">
            <button class="btn btn-primary btn-lg" :disabled="!canGenerate || loading" @click="generate">
                <span v-if="loading" class="spinner"></span>
                {{ loading ? "กำลังสร้าง PDF..." : "📋 สร้าง PDF ส่งหนี้เบิกยา" }}
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
    width: 100%;
}

.info-banner {
    background: linear-gradient(135deg, var(--c-primary-light) 0%, #FFD8C8 100%);
    border-color: #F0C4B8;
}

.banner-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--c-primary);
    margin-bottom: 5px;
}

.banner-desc {
    font-size: 14px;
    color: var(--c-primary-mid);
}

.no-data {
    text-align: center;
    padding: 20px;
    color: var(--c-warn);
    font-size: 15px;
    background: var(--c-warn-bg);
    border-radius: var(--radius);
}

.period {
    color: var(--c-primary) !important;
}

.actions {
    margin-top: 22px;
}

.file-path {
    font-size: 11px;
    color: var(--c-text-muted);
    word-break: break-all;
    display: block;
    margin-top: 2px;
    padding-left: 20px;
}

.result-stats {
    display: flex;
    gap: 8px;
    margin-top: 14px;
    flex-wrap: wrap;
}

/* #166534 on #dcfce7 → 6.8:1 ✓ */
.stat-chip {
    background: #dcfce7;
    color: var(--c-success);
    border: 1px solid #86efac;
    border-radius: 999px;
    padding: 4px 14px;
    font-size: 13px;
    font-weight: 600;
}

/* #C8102E on #FFF0EC → ~7.8:1 ✓ */
.stat-chip.money {
    background: var(--c-primary-light);
    color: var(--c-primary);
    border-color: #F0C4B8;
}

.save-actions {
    display: flex;
    gap: 10px;
}

@media (prefers-color-scheme: dark) {
    .info-banner {
        background: linear-gradient(135deg, #2A0808 0%, #350A0A 100%);
        border-color: #501515;
    }

    .banner-desc {
        color: var(--c-primary-mid);
    }

    .stat-chip {
        background: #052e16;
        color: #4ade80;
        border-color: #166534;
    }

    .stat-chip.money {
        background: #2A0808;
        color: var(--c-primary);
        border-color: #501515;
    }
}
</style>
