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

const props = defineProps<{
    dbConfig: DbConfig;
    startDateHtml: string;
    endDateHtml: string;
    outputDir: string;
    previewData: PreviewData | null;
    round: number;
}>();

const emit = defineEmits<{
    (e: "update:startDateHtml", v: string): void;
    (e: "update:endDateHtml", v: string): void;
    (e: "update:outputDir", v: string): void;
    (e: "update:previewData", v: PreviewData | null): void;
    (e: "update:round", v: number): void;
}>();

const loading = ref(false);
const error = ref("");

const THAI_MONTHS = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];

const THAI_MONTHS_SHORT = [
    "ม.ค.", "ก.พ.", "มี.ค.", "เม.ย.", "พ.ค.", "มิ.ย.",
    "ก.ค.", "ส.ค.", "ก.ย.", "ต.ค.", "พ.ย.", "ธ.ค.",
];

function toApiDate(html: string): string {
    return html.replace(/-/g, "");
}

function buddhistYear(html: string): number {
    return parseInt(html.substring(0, 4)) + 543;
}

function getMonth(html: string): number {
    return parseInt(html.substring(5, 7));
}

function getDay(html: string): number {
    return parseInt(html.substring(8, 10));
}

const periodLabel = computed(() => {
    if (!props.startDateHtml || !props.endDateHtml) return "กรุณาเลือกช่วงวันที่";
    const sy = buddhistYear(props.startDateHtml);
    const sm = getMonth(props.startDateHtml);
    const sd = getDay(props.startDateHtml);
    const ey = buddhistYear(props.endDateHtml);
    const em = getMonth(props.endDateHtml);
    const ed = getDay(props.endDateHtml);
    if (sm === em && sy === ey) {
        return `${sd}–${ed} ${THAI_MONTHS[sm - 1]} ${sy}`;
    }
    return `${sd} ${THAI_MONTHS_SHORT[sm - 1]} – ${ed} ${THAI_MONTHS_SHORT[em - 1]} ${ey}`;
});

const isDbReady = computed(
    () => props.dbConfig.host.trim() !== "" && props.dbConfig.username.trim() !== ""
);
const canFetch = computed(
    () => isDbReady.value && props.startDateHtml !== "" && props.endDateHtml !== ""
);

const drugCount = computed(() =>
    props.previewData?.invoices.filter((i) => i.category === "ยา").length ?? 0
);
const supplyCount = computed(() =>
    props.previewData?.invoices.filter((i) => i.category === "วัสดุเภสัชกรรม").length ?? 0
);

function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function setQuickRange(type: "month" | "1-10" | "11-20" | "21-end") {
    const today = new Date();
    const y = today.getFullYear();
    const m = today.getMonth(); // 0-based
    const pad = (n: number) => String(n).padStart(2, "0");

    if (type === "month") {
        const lastDay = new Date(y, m + 1, 0).getDate();
        emit("update:startDateHtml", `${y}-${pad(m + 1)}-01`);
        emit("update:endDateHtml", `${y}-${pad(m + 1)}-${pad(lastDay)}`);
    } else if (type === "1-10") {
        emit("update:startDateHtml", `${y}-${pad(m + 1)}-01`);
        emit("update:endDateHtml", `${y}-${pad(m + 1)}-10`);
    } else if (type === "11-20") {
        emit("update:startDateHtml", `${y}-${pad(m + 1)}-11`);
        emit("update:endDateHtml", `${y}-${pad(m + 1)}-20`);
    } else {
        const lastDay = new Date(y, m + 1, 0).getDate();
        emit("update:startDateHtml", `${y}-${pad(m + 1)}-21`);
        emit("update:endDateHtml", `${y}-${pad(m + 1)}-${pad(lastDay)}`);
    }
}

async function fetchData() {
    if (!canFetch.value) return;
    loading.value = true;
    error.value = "";
    emit("update:previewData", null);
    try {
        const data = await invoke<PreviewData>("fetch_preview", {
            config: { ...props.dbConfig },
            dateFrom: toApiDate(props.startDateHtml),
            dateTo: toApiDate(props.endDateHtml),
        });
        emit("update:previewData", data);
    } catch (e) {
        error.value = String(e);
    } finally {
        loading.value = false;
    }
}
</script>

<template>
<div class="query-wrap">
    <!-- ── Date range card ── -->
    <div class="card">
        <div class="card-title">🔍 เลือกช่วงวันที่และดึงข้อมูล</div>
        <div class="card-desc">เลือกช่วงวันที่รับยา (RECEIVE_DATE) แล้วกด "ดึงข้อมูล" เพื่อโหลดรายการบิลจาก INVS</div>

        <!-- Quick range buttons -->
        <div class="quick-range">
            <span class="quick-label">เดือนนี้ :</span>
            <button class="btn btn-secondary" @click="setQuickRange('1-10')">งวด 1 (1–10)</button>
            <button class="btn btn-secondary" @click="setQuickRange('11-20')">งวด 2 (11–20)</button>
            <button class="btn btn-secondary" @click="setQuickRange('21-end')">งวด 3 (21–สิ้นเดือน)</button>
            <button class="btn btn-secondary" @click="setQuickRange('month')">ทั้งเดือน</button>
        </div>

        <div class="form-grid-2" style="margin-top:12px">
            <div class="form-group">
                <label>วันที่เริ่มต้น</label>
                <input type="date" :value="startDateHtml"
                    @input="emit('update:startDateHtml', ($event.target as HTMLInputElement).value)" />
            </div>
            <div class="form-group">
                <label>วันที่สิ้นสุด</label>
                <input type="date" :value="endDateHtml"
                    @input="emit('update:endDateHtml', ($event.target as HTMLInputElement).value)" />
            </div>
            <div class="form-group">
                <label>รอบที่ (สำหรับรายงานทั้ง 3 ระบบ)</label>
                <input type="number" min="1" max="99" :value="round"
                    @input="emit('update:round', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                <span class="field-hint">รอบภายในช่วงเวลาเดียวกัน เช่น รอบ 1, 2, 3…</span>
            </div>
            <div class="form-group full">
                <label>โฟลเดอร์บันทึก PDF</label>
                <input type="text" :value="outputDir"
                    @input="emit('update:outputDir', ($event.target as HTMLInputElement).value)"
                    placeholder="เช่น C:\Reports หรือ /Users/me/Documents (ปล่อยว่าง = โฟลเดอร์ปัจจุบัน)" />
                <span class="field-hint">ระบบจะสร้างโฟลเดอร์ย่อย output/ ภายในโฟลเดอร์ที่ระบุ</span>
            </div>
        </div>

        <div v-if="startDateHtml && endDateHtml" class="period-badge">
            📅 {{ periodLabel }}
        </div>

        <div v-if="!isDbReady" class="status-msg status-warn" style="margin-top:12px">
            ⚠️ กรุณาตั้งค่าการเชื่อมต่อฐานข้อมูลก่อน (แท็บ ⚙️ ฐานข้อมูล)
        </div>

        <div class="actions">
            <button class="btn btn-primary btn-lg" :disabled="!canFetch || loading" @click="fetchData">
                <span v-if="loading" class="spinner"></span>
                {{ loading ? "กำลังโหลดข้อมูล..." : "🔍 ดึงข้อมูล" }}
            </button>
        </div>

        <div v-if="error" class="status-msg status-error" style="margin-top:12px">
            ❌ {{ error }}
        </div>
    </div>

    <!-- ── Preview results ── -->
    <div v-if="previewData" class="card">
        <div class="card-title">📊 ผลการดึงข้อมูล</div>

        <!-- Summary stats -->
        <div class="preview-summary">
            <div class="summary-stat">
                <span class="summary-stat-label">จำนวนรายการ</span>
                <span class="summary-stat-value">{{ previewData.row_count }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">ยอดรวมทั้งหมด</span>
                <span class="summary-stat-value money">{{ formatMoney(previewData.total_amount) }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">💊 ยา</span>
                <span class="summary-stat-value">{{ drugCount }} ใบ</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">🧴 วัสดุเภสัชกรรม</span>
                <span class="summary-stat-value">{{ supplyCount }} ใบ</span>
            </div>
        </div>

        <div v-if="previewData.row_count === 0" class="empty-result">
            ⚠️ ไม่พบข้อมูลในช่วงวันที่นี้ กรุณาเลือกช่วงวันที่ใหม่
        </div>

        <div v-else class="table-wrap">
            <table class="data-table">
                <thead>
                    <tr>
                        <th class="text-center">#</th>
                        <th>วันที่รับของ</th>
                        <th>เลขที่เอกสาร</th>
                        <th>รหัสบริษัท</th>
                        <th>ชื่อบริษัท</th>
                        <th class="text-center">ประเภท</th>
                        <th class="text-right">จำนวนเงิน (บาท)</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(inv, idx) in previewData.invoices" :key="idx">
                        <td class="text-center num">{{ idx + 1 }}</td>
                        <td>{{ inv.receive_date }}</td>
                        <td><code>{{ inv.invoice_no }}</code></td>
                        <td><code>{{ inv.vendor_code }}</code></td>
                        <td>{{ inv.company_name }}</td>
                        <td class="text-center">
                            <span :class="['cat-badge', inv.category === 'ยา' ? 'cat-drug' : 'cat-supply']">
                                {{ inv.category }}
                            </span>
                        </td>
                        <td class="text-right money-cell">{{ formatMoney(inv.total_cost) }}</td>
                    </tr>
                </tbody>
                <tfoot>
                    <tr>
                        <td colspan="6" class="text-right">รวมทั้งสิ้น</td>
                        <td class="text-right">{{ formatMoney(previewData.total_amount) }}</td>
                    </tr>
                </tfoot>
            </table>
        </div>
    </div>

    <div v-else-if="!loading" class="card empty-card">
        <div class="empty-icon">🔍</div>
        <div class="empty-text">เลือกช่วงวันที่แล้วกด "ดึงข้อมูล" เพื่อดูรายการบิล</div>
        <div class="empty-hint">ข้อมูลที่ดึงมาจะถูกใช้สำหรับสร้างรายงานทั้ง 3 ระบบ</div>
    </div>
</div>
</template>

<style scoped>
.query-wrap {
    max-width: 1000px;
}

.actions {
    margin-top: 16px;
}

.quick-range {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.quick-label {
    font-size: 12px;
    color: var(--c-text-muted);
    font-weight: 600;
    white-space: nowrap;
}

.quick-range .btn {
    padding: 5px 12px;
    font-size: 12px;
}

.empty-result {
    text-align: center;
    padding: 20px;
    color: var(--c-warn);
    font-size: 14px;
}

.empty-card {
    text-align: center;
    padding: 48px 20px !important;
}

.empty-icon {
    font-size: 40px;
    margin-bottom: 12px;
}

.empty-text {
    font-size: 15px;
    color: var(--c-text-muted);
    margin-bottom: 6px;
}

.empty-hint {
    font-size: 13px;
    color: var(--c-text-light);
}

.num {
    color: var(--c-text-light);
    font-size: 12px;
}

.money-cell {
    font-variant-numeric: tabular-nums;
    font-size: 13px;
}

code {
    font-family: monospace;
    font-size: 12px;
    background: #f3f4f6;
    padding: 1px 5px;
    border-radius: 3px;
}
</style>
