<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { fileName, formatMoney, toThaiFullDate, THAI_MONTHS, THAI_MONTHS_SHORT } from "../lib/format";
import type {
    DbConfig,
    GenerateResult,
    PreviewData,
    RoundHistoryEntry,
    TabId,
} from "../lib/types";
import {
    AlertTriangle,
    ArrowRight,
    Banknote,
    Calculator,
    CalendarDays,
    CheckCircle,
    Database,
    FileOutput,
    Package,
    Save,
    Wallet,
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
    budgetTotal: number;
    previousBalance: number;
    approvalDate: string;
    r2Carry: { next_reg_no: string; next_running: number; next_po_no: number } | null;
}>();

const emit = defineEmits<{
    (e: "update:year", v: number): void;
    (e: "update:budgetTotal", v: number): void;
    (e: "update:previousBalance", v: number): void;
    (e: "update:approvalDate", v: string): void;
    (e: "saveHistory", entry: RoundHistoryEntry): void;
    (e: "navigate", tab: TabId): void;
    (e: "generated"): void;
}>();

function onYearInput(e: Event) {
    emit("update:year", parseInt((e.target as HTMLInputElement).value, 10) || 0);
}

const toast = useToast();

const loading = ref(false);
const error = ref("");
const result = ref<GenerateResult | null>(null);

watch(
    () => [props.year, props.month, props.round, props.dateFrom, props.dateTo, props.budgetTotal, props.previousBalance, props.approvalDate],
    () => {
        result.value = null;
        error.value = "";
    }
);

// Holds the native <input type="date"> value and converts it to the
// Thai full format expected by the cover letters.
const approvalDatePicker = ref("");

function onApprovalDatePick() {
    emit("update:approvalDate", toThaiFullDate(approvalDatePicker.value));
}

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
        props.budgetTotal > 0
);

const previewBalance = computed(() => {
    if (!props.previewData || props.previewData.row_count === 0) return null;
    const firstAmount = props.previewData.invoices[0]?.total_cost ?? 0;
    return props.previousBalance - firstAmount;
});

async function generate() {
    if (!canGenerate.value) return;
    loading.value = true;
    error.value = "";
    result.value = null;

    try {
        const res = await invoke<GenerateResult>("generate_cover_letters", {
            params: {
                db_config: { ...props.dbConfig },
                date_from: props.dateFrom,
                date_to: props.dateTo,
                year: props.year,
                month: props.month,
                round: props.round,
                budget_total: props.budgetTotal,
                previous_balance: props.previousBalance,
                approval_date: props.approvalDate.trim() || null,
                output_dir: props.outputDir,
            },
        });
        result.value = res;
        emit("generated");
        toast.success(
            "สร้าง PDF สำเร็จ",
            `สร้าง ${res.total_rows} หน้า ยอดรวม ${formatMoney(res.total_amount)} บาท`
        );
    } catch (e) {
        error.value = String(e);
        toast.error("สร้าง PDF ล้มเหลว", String(e));
    } finally {
        loading.value = false;
    }
}

function saveToHistory() {
    if (!result.value) return;
    const now = new Date().toISOString();
    const monthShort = THAI_MONTHS_SHORT[props.month - 1] ?? "";
    // Use carry-forward from Report 2 for reg/po numbers (more accurate),
    // fall back to Report 3's carry-forward if Report 2 wasn't run yet.
    const regNo = props.r2Carry?.next_reg_no || result.value.carry_forward.next_reg_no || "";
    const running = props.r2Carry?.next_running ?? result.value.carry_forward.next_running ?? 0;
    const poNo = props.r2Carry?.next_po_no ?? result.value.carry_forward.next_po_no ?? 0;
    const entry: RoundHistoryEntry = {
        id: now,
        label: `${monthShort} ${props.year} รอบ ${props.round}`,
        fiscal_year: props.year,
        month: props.month,
        round: props.round,
        date_from: props.dateFrom,
        date_to: props.dateTo,
        next_reg_no: regNo,
        next_running: running,
        next_po_no: poNo,
        next_purchase_no: props.r2Carry ? undefined : result.value.carry_forward.next_purchase_no,
        remaining_balance: result.value.carry_forward.remaining_balance,
        budget_total: props.budgetTotal,
        total_amount: result.value.total_amount,
        invoice_count: result.value.total_rows,
        source_tab: "เบิกยาปะหน้า",
        created_at: now,
    };
    emit("saveHistory", entry);
}
</script>

<template>
<div class="report-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">เบิกยาปะหน้า</h2>
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
            <span>หน้าทั้งหมด <strong>{{ previewData.row_count }} หน้า</strong></span>
            <span class="context-sep"></span>
            <span>ยอดเบิกจ่ายรอบนี้ <strong>{{ formatMoney(previewData.total_amount) }} บาท</strong></span>
            <span class="context-right">
                <span class="badge badge-success">ข้อมูลพร้อม</span>
            </span>
        </div>

        <!-- Budget params -->
        <div class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><Wallet :size="16" /> ตั้งค่างบประมาณ</div>
                </div>
            </div>

            <div class="section-label"><CalendarDays :size="13" /> ข้อมูลงวด</div>
            <div class="form-grid">
                <div class="form-group">
                    <label for="r3-year">ปีงบประมาณ</label>
                    <input id="r3-year" type="number" min="2500" max="2700"
                        :value="year > 0 ? year : ''" placeholder="เช่น 2569" @input="onYearInput" />
                </div>
                <div class="form-group">
                    <label for="r3-month">เดือน</label>
                    <input id="r3-month" type="text" :value="month > 0 ? THAI_MONTHS[month - 1] : '-'" readonly />
                </div>
                <div class="form-group">
                    <label for="r3-round">รอบที่</label>
                    <input id="r3-round" type="text" :value="round" readonly />
                </div>
            </div>

            <div class="section-label section-spaced"><Banknote :size="13" /> งบประมาณและการอนุมัติ</div>
            <div class="form-grid">
                <div class="form-group">
                    <label for="r3-budget">ยอดเงินจัดสรรทั้งปี (บาท)</label>
                    <input id="r3-budget" type="number" step="0.01" min="0" :value="budgetTotal"
                        @input="emit('update:budgetTotal', parseFloat(($event.target as HTMLInputElement).value) || 0)"
                        placeholder="5843812.60" />
                </div>
                <div class="form-group">
                    <label for="r3-balance">ยอดคงเหลือก่อนรอบนี้ (บาท)</label>
                    <input id="r3-balance" type="number" step="0.01" min="0" :value="previousBalance"
                        @input="emit('update:previousBalance', parseFloat(($event.target as HTMLInputElement).value) || 0)"
                        placeholder="ยอดที่เหลือจากรอบที่แล้ว" />
                </div>
                <div class="form-group">
                    <label for="r3-approval">วันที่ขออนุมัติ (แสดงบนเอกสาร)</label>
                    <div class="input-group">
                        <input id="r3-approval" type="date" v-model="approvalDatePicker"
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

            <div v-if="previewData.row_count > 0" class="budget-preview">
                <div class="budget-preview-title"><Calculator :size="13" /> ตัวอย่างการคำนวณ (บิลแรก)</div>
                <div class="budget-row">
                    <span class="budget-label">ยอดเงินจัดสรร</span>
                    <span class="budget-val">{{ formatMoney(budgetTotal) }}</span>
                </div>
                <div class="budget-row">
                    <span class="budget-label">ยอดคงเหลือก่อนรอบนี้</span>
                    <span class="budget-val">{{ formatMoney(previousBalance) }}</span>
                </div>
                <div class="budget-row highlight">
                    <span class="budget-label">เบิกจ่ายครั้งนี้ (บิลแรก)</span>
                    <span class="budget-val debit">- {{ formatMoney(previewData.invoices[0]?.total_cost ?? 0) }}</span>
                </div>
                <div class="budget-row total">
                    <span class="budget-label">ยอดคงเหลือหลังบิลแรก</span>
                    <span class="budget-val" :class="(previewBalance ?? 0) < 0 ? 'negative' : 'positive'">
                        {{ formatMoney(previewBalance ?? 0) }}
                    </span>
                </div>
            </div>

            <div class="actions actions-row">
                <button class="btn btn-primary btn-lg" :disabled="!canGenerate || loading" @click="generate">
                    <span v-if="loading" class="spinner"></span>
                    <FileOutput v-else :size="16" />
                    {{ loading ? "กำลังสร้าง PDF..." : "สร้าง PDF เบิกยาปะหน้า" }}
                </button>
            </div>

            <div v-if="error" class="status-msg status-error status-stack">
                <XCircle :size="15" /> {{ error }}
            </div>
        </div>

        <!-- Result -->
        <div v-if="result" class="card">
            <div class="card-head">
                <div>
                    <div class="card-title"><CheckCircle :size="16" /> สร้าง PDF สำเร็จ</div>
                </div>
            </div>

            <div class="result-card">
                <div class="result-card-title"><FileOutput :size="15" /> ไฟล์ที่สร้าง</div>
                <ul class="file-list">
                    <li v-for="f in result.files" :key="f">
                        <FileOutput :size="14" /> <code>{{ fileName(f) }}</code>
                        <span class="file-path">{{ f }}</span>
                    </li>
                </ul>
                <div class="result-stats">
                    <span class="stat-chip"><Package :size="13" /> {{ result.total_rows }} หน้า</span>
                    <span class="stat-chip money"><Banknote :size="13" /> {{ formatMoney(result.total_amount) }} บาท</span>
                </div>
            </div>

            <div class="carry-box section-spaced">
                <div class="carry-box-title"><ArrowRight :size="15" /> ค่าสำหรับรอบถัดไป</div>
                <div class="carry-grid">
                    <div class="carry-item">
                        <span class="carry-label">ยอดงบประมาณที่จัดสรร</span>
                        <span class="carry-val">{{ formatMoney(budgetTotal) }}</span>
                    </div>
                    <div class="carry-item">
                        <span class="carry-label">ยอดคงเหลือหลังรอบนี้</span>
                        <span class="carry-val">{{ formatMoney(result.carry_forward.remaining_balance) }}</span>
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

<style scoped>
.approval-preview {
    color: var(--c-primary);
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 5px;
}
</style>
