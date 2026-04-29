<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, CalendarDays, Package, Banknote, Clock, ArrowRight, Download, Check, X, Trash2, Lightbulb } from 'lucide-vue-next'
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
    source_tab?: string;
    created_at: string;
}

defineProps<{
    entries: RoundHistoryEntry[];
}>();

const emit = defineEmits<{
    (e: "loadEntry", entry: RoundHistoryEntry): void;
    (e: "deleteEntry", id: string): void;
}>();

const confirmingId = ref<string | null>(null);

const THAI_MONTHS = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];

function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function formatDate(yyyymmdd: string): string {
    if (!yyyymmdd || yyyymmdd.length < 8) return yyyymmdd;
    const y = parseInt(yyyymmdd.substring(0, 4)) + 543;
    const m = parseInt(yyyymmdd.substring(4, 6));
    const d = parseInt(yyyymmdd.substring(6, 8));
    return `${d} ${THAI_MONTHS[m - 1] ?? ""} ${y}`;
}

function formatCreatedAt(iso: string): string {
    try {
        const d = new Date(iso);
        return d.toLocaleString("th-TH", {
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    } catch {
        return iso;
    }
}

function doDelete(id: string) {
    emit("deleteEntry", id);
    confirmingId.value = null;
}
</script>

<template>
<div class="history-wrap">
    <div class="page-header">
        <h2 class="page-title">ประวัติรอบ</h2>
        <p class="page-desc">ค่า carry-forward แต่ละรอบ — กด "โหลด" เพื่อนำค่าไปใช้ในรอบถัดไป</p>
    </div>

    <!-- Empty state -->
    <div v-if="entries.length === 0" class="card empty-card">
        <div class="empty-icon">
            <FolderOpen :size="52" stroke-width="1.5" />
        </div>
        <div class="empty-title">ยังไม่มีประวัติรอบ</div>
        <div class="empty-desc">
            หลังจากสร้าง PDF แต่ละรายงานสำเร็จ กด "บันทึกรอบนี้สู่ประวัติ"
            เพื่อบันทึก carry-forward ไว้ใช้งานรอบถัดไป
        </div>
    </div>

    <!-- Entry list -->
    <div v-else class="entries">
        <div v-for="entry in entries" :key="entry.id" class="entry-card">
            <!-- Entry header -->
            <div class="entry-header">
                <div class="entry-title-row">
                    <span class="entry-label">{{ entry.label }}</span>
                    <span class="entry-round-badge">รอบ {{ entry.round }}</span>
                    <span v-if="entry.source_tab" class="source-tab-badge">{{ entry.source_tab }}</span>
                </div>
                <div class="entry-meta">
                    <span class="meta-chip">
                        <CalendarDays :size="12" /> {{ formatDate(entry.date_from) }} – {{ formatDate(entry.date_to) }}
                    </span>
                    <span class="meta-chip">
                        <Package :size="12" /> {{ entry.invoice_count }} บิล
                    </span>
                    <span class="meta-chip money">
                        <Banknote :size="12" /> {{ formatMoney(entry.total_amount) }} บาท
                    </span>
                    <span class="meta-chip muted">
                        <Clock :size="12" /> {{ formatCreatedAt(entry.created_at) }}
                    </span>
                </div>
            </div>

            <!-- Carry-forward values -->
            <div class="carry-section">
                <div class="carry-title">
                    <ArrowRight :size="14" /> ค่า Carry-Forward สำหรับรอบถัดไป (รอบ {{ entry.round + 1 }})
                </div>
                <div class="carry-values">
                    <div class="cv-item">
                        <span class="cv-label">เลขทะเบียนคุม</span>
                        <span class="cv-val reg">{{ entry.next_reg_no || "—" }}</span>
                    </div>
                    <div class="cv-item">
                        <span class="cv-label">ลำดับในสมุด</span>
                        <span class="cv-val">{{ entry.next_running }}</span>
                    </div>
                    <div class="cv-item">
                        <span class="cv-label">เลขขอซื้อ/PO</span>
                        <span class="cv-val">{{ entry.next_po_no || "—" }}</span>
                    </div>
                    <div class="cv-item">
                        <span class="cv-label">ยอดงบคงเหลือ</span>
                        <span class="cv-val money">{{ entry.remaining_balance > 0 ? formatMoney(entry.remaining_balance)
                            : "—" }}</span>
                    </div>
                    <div v-if="entry.budget_total > 0" class="cv-item">
                        <span class="cv-label">งบประมาณรวม</span>
                        <span class="cv-val">{{ formatMoney(entry.budget_total) }}</span>
                    </div>
                </div>
            </div>

            <!-- Actions -->
            <div class="entry-actions">
                <button class="btn btn-primary" @click="emit('loadEntry', entry)">
                    <Download :size="15" /> โหลดค่านี้ไปใช้รอบถัดไป
                </button>
                <template v-if="confirmingId === entry.id">
                    <span class="confirm-text">ยืนยันลบรอบนี้?</span>
                    <button class="btn btn-danger" @click="doDelete(entry.id)">
                        <Check :size="15" /> ยืนยันลบ
                    </button>
                    <button class="btn btn-secondary" @click="confirmingId = null">
                        <X :size="15" /> ยกเลิก
                    </button>
                </template>
                <button v-else class="btn btn-danger" @click="confirmingId = entry.id">
                    <Trash2 :size="15" /> ลบ
                </button>
            </div>
        </div>
    </div>

    <!-- How to use -->
    <div class="card tip-card">
        <div class="card-title">
            <Lightbulb :size="17" /> วิธีใช้งานประวัติรอบ
        </div>
        <ol class="tip-list">
            <li>สร้าง PDF แต่ละรายงานสำเร็จแล้ว → กด <strong>บันทึกรอบนี้สู่ประวัติ</strong></li>
            <li>เมื่อเริ่มรอบใหม่ → กด <strong>โหลดค่านี้ไปใช้รอบถัดไป</strong></li>
            <li>ระบบจะ pre-fill ค่า carry-forward ให้ทุกแท็บอัตโนมัติ และสลับไปแท็บ ดึงข้อมูล</li>
            <li>เลือกช่วงวันที่ใหม่ → ดึงข้อมูล → สร้างรายงานได้เลย</li>
        </ol>
    </div>
</div>
</template>

<style scoped>
/* Empty state */
.empty-card {
    text-align: center;
    padding: 64px 28px !important;
}

.empty-icon {
    color: var(--c-text-light);
    margin-bottom: 20px;
    display: flex;
    justify-content: center;
}

.empty-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--c-text);
    margin-bottom: 10px;
    letter-spacing: -0.3px;
}

.empty-desc {
    font-size: 14px;
    color: var(--c-text-light);
    max-width: 420px;
    margin: 0 auto;
    line-height: 1.65;
}

/* Entry cards */
.entries {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.entry-card {
    background: var(--c-surface);
    box-shadow: var(--shadow-card);
    border-radius: var(--radius-lg);
    padding: 24px 26px;
    transition: box-shadow 0.15s, transform 0.15s;
}

.entry-card:hover {
    box-shadow: var(--shadow-card);
    transform: translateY(-1px);
}

.entry-header {
    margin-bottom: 18px;
}

.entry-title-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
    flex-wrap: wrap;
}

.entry-label {
    font-size: 18px;
    font-weight: 600;
    color: var(--c-text);
    letter-spacing: -0.36px;
}

/* #C8102E on #FFF0EC → ~7.8:1 ✓ */
.entry-round-badge {
    background: rgba(255, 240, 236, 0.8);
    color: var(--c-primary);
    box-shadow: rgba(200, 16, 46, 0.10) 0px 0px 0px 1px;
    border-radius: 999px;
    padding: 5px 12px;
    font-size: 13px;
    font-weight: 600;
}

.source-tab-badge {
    background: rgba(255, 255, 255, 0.9);
    color: var(--c-text-muted);
    box-shadow: var(--shadow-ring);
    border-radius: 999px;
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 500;
}

.entry-meta {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.meta-chip {
    font-size: 13px;
    color: var(--c-text-muted);
    background: rgba(255, 255, 255, 0.76);
    box-shadow: var(--shadow-ring);
    border-radius: 999px;
    padding: 6px 12px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
}

/* #C8102E on #FFF0EC → ~7.8:1 ✓ */
.meta-chip.money {
    color: var(--c-primary);
    background: rgba(255, 240, 236, 0.84);
    box-shadow: rgba(200, 16, 46, 0.10) 0px 0px 0px 1px;
}

.meta-chip.muted {
    color: var(--c-text-light);
    font-size: 12px;
}

/* Carry values */
.carry-section {
    background: var(--c-primary-light);
    box-shadow: var(--shadow-ring);
    border-radius: var(--radius-lg);
    padding: 18px 20px;
    margin-bottom: 16px;
}

/* #C8102E on #FFF0EC → ~7.8:1 ✓ */
.carry-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--c-primary);
    margin-bottom: 14px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
}

.carry-values {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
}

.cv-item {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 150px;
    flex: 1 1 150px;
}

/* #5C2C1E on #FFF0EC → ~11:1 ✓ */
.cv-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--c-text-muted);
    font-weight: 600;
}

.cv-val {
    font-size: 16px;
    font-weight: 600;
    color: var(--c-text);
}

/* #C8102E on #FFF0EC → ~7.8:1 ✓ */
.cv-val.reg {
    color: var(--c-primary);
    font-family: "Consolas", "Fira Code", monospace;
}

.cv-val.money {
    color: var(--c-primary);
}

/* Entry actions */
.entry-actions {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
}

.confirm-text {
    font-size: 13px;
    font-weight: 600;
    color: var(--c-error);
    padding: 0 6px;
    white-space: nowrap;
}

/* Tip card — warm cream tint */
.tip-card {
    background: var(--c-primary-light);
}

.tip-list {
    padding-left: 20px;
    line-height: 1.95;
    font-size: 14px;
    color: var(--c-text-muted);
}

.tip-list li {
    margin-bottom: 4px;
}

.tip-list strong {
    color: var(--c-text);
}
</style>
